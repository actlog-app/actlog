# Implementation Plan: ActLog (Rust Modern Procrastitracker)

## Goal Description
Build a modern Windows time tracking application in pure Rust, replacing legacy Procrastitracker with:
1. Native Windows daemon capturing active window title, process executable, and idle state.
2. XInput gamepad polling to ensure controller gaming is accurately tracked as active.
3. Local HTTP ingestion server for browser extensions (capturing active URL/domain) and third-party integrations (AniList, game time profiles).
4. SQLite WAL persistence with smart aggregation (no 1-row-per-second DB spam).
5. Clean, lightweight native desktop window UI built with `egui` / `eframe` and Windows system tray support.

---

## User Review Required

> [!IMPORTANT]
> **Desktop UI Engine**: We are using `egui` (`eframe`), giving a single statically linked binary with ~15-20MB RAM usage and zero webview/node dependencies.
> **Database**: We are using `rusqlite` with Write-Ahead Logging (WAL) mode, replacing Procrastitracker's `.pt` binary format so external Python/Rust/PowerShell scripts can query it directly.

> [!NOTE]
> **Browser Tracking**: Chrome and modern browsers do not allow desktop apps to read tab URLs via Win32 DDE. Browser tab tracking will be served via a lightweight local loopback HTTP endpoint (`127.0.0.1:5566`) receiving heartbeat beacons from a companion browser extension.

---

## Open Questions

- **Question 1**: Do you prefer the desktop window to auto-start on Windows boot (via Windows Registry Run key), or manual launch? (Default: configurable in Settings UI, off by default).
- **Question 2**: For the browser extension, do you want Chrome (Manifest V3) or Firefox first? (Default: Chrome MV3).

---

## Proposed Changes

### Component 1: Core Domain & Data Layer (`src/db/`)
Handles SQLite migrations, session aggregation, tag classification, and analytical queries.

#### [NEW] `Cargo.toml`
- Dependencies: `windows`, `rusqlite`, `tiny_http`, `serde`, `serde_json`, `eframe`, `egui`, `chrono`, `crossbeam-channel`, `regex`.

#### [NEW] `src/db/schema.rs`
- Initializes tables: `activities`, `tags`, `rules`.
- Executes WAL mode pragma and performance indexes.

#### [NEW] `src/db/models.rs`
- Rust structs: `ActivitySession`, `Tag`, `Rule`, `DaySummary`.

#### [NEW] `src/db/repository.rs`
- CRUD operations for activities and tags.
- Heartbeat aggregation logic: merges current tick with prior activity if `app_name`, `window_title`, and `url` match within merge threshold.

---

### Component 2: Windows System Poller (`src/tracker/`)
Replaces C++ `procrastitracker.cpp`, `IdleTracker.cpp`, and `xinputactivity.h`.

#### [NEW] `src/tracker/window.rs`
- Calls Windows API `GetForegroundWindow`, `GetWindowThreadProcessId`, `QueryFullProcessImageNameW`, and `GetWindowTextW`.
- Extracts clean executable name (e.g. `code.exe`) and window title.

#### [NEW] `src/tracker/idle.rs`
- Calls `GetLastInputInfo` to measure user keyboard/mouse inactivity in milliseconds.

#### [NEW] `src/tracker/gamepad.rs`
- Uses Windows `xinput1_4.dll` (`XInputGetState`) to detect gamepad button or joystick motion, preventing false AFK while playing games with a controller.

#### [NEW] `src/tracker/engine.rs`
- Background tracker thread running every 1000ms.
- Classifies window against `Rule` engine (tag assignment).
- Merges consecutive active seconds into `ActivitySession`.

---

### Component 3: Local Ingestion & Integration API (`src/server/`)
Accepts browser extension payloads and exposes stats for third-party tools.

#### [NEW] `src/server/http.rs`
- Lightweight `tiny_http` server on `127.0.0.1:5566`.
- `POST /api/heartbeat/browser`: Ingests `{ url, domain, title }` from browser extension and stores the latest active tab.
- `GET /api/activities/current`: Returns live tracking state for AniList / game time scripts.
- `GET /api/stats/summary`: Returns daily summary in JSON.

---

### Component 4: Desktop Window UI (`src/ui/`)
Native desktop UI replacing C++ `statview.h`.

#### [NEW] `src/ui/app.rs`
- `eframe::App` implementation.
- Date navigator (`< [Today] >`).
- Summary metric cards (Active Time, AFK Time, Productivity Score).
- Stacked hourly timeline visualizer.
- Process / Browser activity data table with category badges.

#### [NEW] `src/ui/tray.rs`
- Windows system tray icon with "Open Dashboard", "Pause Tracking", "Exit".

#### [NEW] `src/main.rs`
- CLI / Daemon entry point.
- Initializes database, starts tracker loop, spawns HTTP server, and mounts desktop UI window.

---

### Component 5: Browser Companion Extension (`extension/`)

#### [NEW] `extension/manifest.json`
- Manifest V3 extension configuration for Chrome / Edge / Brave.

#### [NEW] `extension/background.js`
- Listens to `chrome.tabs.onActivated` and `chrome.tabs.onUpdated`.
- Debounces active tab URL and posts to `http://127.0.0.1:5566/api/heartbeat/browser`.

---

## Verification Plan

### Automated Tests
1. **DB Aggregation Tests**:
   - `cargo test --lib db::tests::test_heartbeat_merge`
   - Verifies sequential ticks within 10s merge into single activity span; tick with title change creates new span.
2. **Rule Engine Matching Tests**:
   - `cargo test --lib tracker::tests::test_rule_matching`
   - Verifies regex and substring rule matching for app names and URL domains.
3. **HTTP Server API Tests**:
   - `cargo test --lib server::tests::test_browser_heartbeat_endpoint`
   - Posts synthetic browser payload and verifies retrieval in DB.

### Manual Verification
1. Launch app with `cargo run`.
2. Confirm tray icon appears and desktop window opens showing active foreground window.
3. Plug in gamepad / move sticks -> verify idle timer resets in debug overlay.
4. Open Chrome with test extension loaded -> verify URL domain shows up alongside `chrome.exe` in desktop timeline.
5. Query `http://127.0.0.1:5566/api/activities/current` in PowerShell -> verify JSON returns active window.

