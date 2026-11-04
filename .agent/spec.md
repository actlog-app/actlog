# Feature Specification: ActLog (Rust Modern Procrastitracker)

**Date**: 2026-11-04  
**Status**: DRAFT / PENDING REVIEW  
**Scope**: Windows Activity Tracker Daemon + Desktop Window UI + Browser Ingestion + SQLite DB + Integration Hooks

---

## 1.1 Goal & User Stories

### Problem
Legacy Procrastitracker is a 32-bit Win32 GDI app using dead DDE browser hooks, raw un-queryable `.pt` binary trees, and no integration APIs. Modern Windows users need an ultra-low-resource Rust daemon that:
1. Accurately tracks active applications, titles, and controller/gamepad state (no false AFK during gaming).
2. Captures exact browser URLs/domains via a lightweight browser extension.
3. Provides a clean desktop window UI to inspect daily/weekly usage, categories, and timelines.
4. Stores data in standard SQLite so external scripts/apps (AniList scrobblers, Steam/game time trackers) can query and sync.

### User Story
- *As a power user and gamer*, I want an always-on, low-overhead Windows tracker that accurately records desktop apps, games (including controller inputs), and browser tabs without draining CPU/RAM, so I can see where my time went and export or sync time to external services like AniList and game profiles.

### Non-Goals
- **No Cloud Dependency**: Local-first only. Zero mandatory telemetry, cloud accounts, or SaaS subscriptions.
- **No Heavy Runtimes**: No Electron, no Chromium desktop shells. Native Rust binary (`<20MB` RAM).
- **No Kernel Drivers / Invasive Keylogging**: Only tracks foreground window title, process executable name, active browser tab domain/title from extension, and idle time via Windows `GetLastInputInfo` / XInput. No keystroke recording.

---

## 1.2 UI & UX Specification (Desktop Window)

### Desktop UI Framework
- **Engine**: Pure Rust native immediate-mode desktop window (`eframe` / `egui`) with system tray integration (`tray-icon`).
- **Tray Behavior**: Minimizes to tray on close (`WM_CLOSE` -> hide). Double-click or right-click tray icon opens desktop window.

### ASCII Wireframe: Desktop Window (`1024x640`)

```
+-------------------------------------------------------------------------------+
| ActLog - Activity Tracker                     [ Today: 2026-11-04 ] [ < ] [ > ]|
+-------------------------------------------------------------------------------+
| Summary: Active: 6h 42m | Idle: 45m | Top App: VS Code (3h 10m) | Score: +72% |
+-------------------------------------------------------------------------------+
| Timeline Bar:                                                                 |
| [08:00 ==== Coding ====][12:00 =Lunch=][13:00 ==Browser==][16:00 ===Elden Ring=]
+-------------------------------------------------------------------------------+
| [Categories / Tags]     | Applications & Browser Details                      |
| [x] Development  3h 15m | - code.exe (VS Code)                   2h 45m (41%) |
| [x] Gaming       2h 10m | - eldenring.exe (Gamepad Active)       2h 10m (32%) |
| [x] Browser      1h 12m | - chrome.exe - github.com/rust-lang       42m  (10%) |
| [x] Media          45m  | - chrome.exe - youtube.com/watch          30m   (7%) |
| [x] Uncategorized  15m  | - explorer.exe                            15m   (2%) |
+-------------------------------------------------------------------------------+
| [ Settings ] [ Rules ] [ Export SQLite ] [ Open Web View ] [ Minimize to Tray]|
+-------------------------------------------------------------------------------+
```

### 4 Essential UI States

1. **Loading State**:
   - Initial window opening loads day aggregations asynchronously from SQLite via `crossbeam-channel`.
   - UI displays subtle spinner and placeholder pulse bars for the timeline.
2. **Empty State**:
   - Fresh install or day with zero records: "No activity recorded for this date. App daemon is active and monitoring in background." Action button: `[ Refresh ]` and `[ Test Poller ]`.
3. **Success / Active State**:
   - Top summary cards (Active time, Semi-idle time, Productivity score, Top category).
   - Hourly stacked timeline bar with category color codes.
   - Filterable data table of processes, window titles, and captured browser URLs.
   - Tag editor dropdown to reclassify selected processes or URL domains.
4. **Error State**:
   - Database locked or write error: Persistent amber alert banner at bottom: "Database busy: Retrying in background...". Never crashes or modal-blocks the UI thread.

### Edge Cases
- **Multi-Monitor / High DPI**: Pure vector layout via `egui`, automatic DPI scaling.
- **Midnight Rollover**: Daemon detects day change (`00:00:00`), flushes active session, and cuts over to next day partition.
- **Gamepad / XInput Only**: Playing games with an Xbox/DirectInput controller produces no mouse/keyboard events; XInput poller prevents false AFK triggers.

---

## 1.3 Data & State Contract

### Database Schema (SQLite with WAL Mode)

```sql
CREATE TABLE IF NOT EXISTS activities (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    start_time INTEGER NOT NULL,      -- Unix timestamp (seconds)
    end_time INTEGER NOT NULL,        -- Unix timestamp (seconds)
    duration INTEGER NOT NULL,        -- Seconds active
    app_name TEXT NOT NULL,           -- e.g. "chrome.exe", "code.exe"
    exe_path TEXT,                    -- full path to exe
    window_title TEXT NOT NULL,       -- active window text
    url TEXT,                         -- browser URL (if sent from extension)
    domain TEXT,                      -- browser domain (e.g. "github.com")
    tag_id INTEGER,                   -- foreign key to tags.id
    is_afk INTEGER DEFAULT 0,         -- 1 if marked idle / away
    FOREIGN KEY(tag_id) REFERENCES tags(id)
);

CREATE TABLE IF NOT EXISTS tags (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,        -- e.g. "Work", "Gaming", "Procrastination"
    color TEXT NOT NULL,              -- Hex color: "#4CAF50"
    productivity_score INTEGER DEFAULT 0 -- -2 (heavy procrastinate) to +2 (high focus)
);

CREATE TABLE IF NOT EXISTS rules (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    pattern TEXT NOT NULL,            -- regex or substring pattern
    match_field TEXT NOT NULL,        -- 'app_name', 'window_title', 'domain'
    tag_id INTEGER NOT NULL,
    priority INTEGER DEFAULT 0,
    FOREIGN KEY(tag_id) REFERENCES tags(id)
);

CREATE INDEX IF NOT EXISTS idx_activities_time ON activities(start_time, end_time);
CREATE INDEX IF NOT EXISTS idx_activities_app ON activities(app_name);
```

### Local Ingestion API Contract (HTTP loopback `127.0.0.1:5566`)

1. **Browser Heartbeat Ingestion**:
   - **Route**: `POST /api/heartbeat/browser`
   - **Payload**:
     ```json
     {
       "url": "https://github.com/rust-lang/rust",
       "domain": "github.com",
       "title": "rust-lang/rust: Empowering everyone...",
       "tab_active": true
     }
     ```
   - **Response**: `200 OK` `{ "status": "recorded" }`
2. **Third-Party Query API (for AniList / Game integrations)**:
   - **Route**: `GET /api/activities/current`
   - **Response**:
     ```json
     {
       "app_name": "vlc.exe",
       "window_title": "[SubsPlease] Sousou no Frieren - 28.mkv",
       "active_duration_seconds": 1240,
       "domain": null,
       "gamepad_active": false
     }
     ```
   - **Route**: `GET /api/stats/summary?date=2026-11-04`
   - Returns aggregated seconds grouped by `app_name`, `domain`, and `tag`.

---

## 1.4 Acceptance Criteria

- **AC-1 (Windows Poller)**: Foreground window changes are detected within 1.0 second. Process executable name, window title, and timestamp are captured without crashing on elevated (admin) windows.
- **AC-2 (Idle & XInput AFK Guard)**: Keyboard/mouse idle past user threshold (default 180s) flags AFK *unless* XInput gamepad buttons/sticks are active.
- **AC-3 (Heartbeat Aggregation)**: Consecutive ticks with matching app and title are aggregated into a single continuous activity span; database does not write 1 row per second.
- **AC-4 (Browser Tab Correlation)**: When Chrome/Firefox is foreground, active URL and domain sent from the browser extension are matched with the current window and saved into SQLite.
- **AC-5 (Desktop UI Responsiveness)**: Desktop window opens in under 200ms, consumes `<25MB` RAM, and renders the day's timeline and application breakdown.
- **AC-6 (Integrations Access)**: External script can query `GET /api/activities/current` over localhost HTTP and receive current active media/game title.

