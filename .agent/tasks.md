# Implementation Tasks: ActLog (`tasks.md`)

Step-by-step atomic execution checklist. Every task takes 2-5 minutes and has an automated verification command.

---

### Phase 1: Project Scaffold & Data Layer (`src/db`)

- [x] **Task 1.1: Initialize Cargo workspace and dependencies**
  - **Files**: `Cargo.toml`, `src/lib.rs`
  - **Action**: Defined dependencies (`windows`, `rusqlite`, `serde`, `serde_json`, `tiny_http`, `chrono`, `eframe`, `egui`, `regex`, `crossbeam-channel`). Configured release profile (`opt-level = "z"`, `lto = true`).
  - **Verify**: `cargo check` (PASSED)

- [x] **Task 1.2: Define domain models and entities**
  - **Files**: `src/db/models.rs`, `src/db/mod.rs`
  - **Action**: Created `ActivitySession`, `Tag`, `Rule`, `MatchField`, `RawTick`, `DaySummary` structs with serde derives.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 1.3: Implement SQLite schema migrations and connection pool**
  - **Files**: `src/db/schema.rs`, `tests/db_test.rs`
  - **Action**: Wrote schema creation script (`activities`, `tags`, `rules`, indexes) and WAL pragma configuration.
  - **Verify**: `cargo test --test db_test test_schema_creation` (PASSED)

- [x] **Task 1.4: Implement session aggregation and repository operations**
  - **Files**: `src/db/repository.rs`, `tests/db_test.rs`
  - **Action**: Implemented `upsert_session`, `flush_active_session`, `get_day_activities`, `get_daily_summary`.
  - **Verify**: `cargo test --test db_test test_session_aggregation_and_flush` (PASSED)

> **Phase 1 Verification Gate**: PASSED (2 passed; 0 failed)

---

### Phase 2: Tracker Core & Win32 Polling (`src/tracker`)

- [x] **Task 2.1: Implement Win32 window title and executable path resolver**
  - **Files**: `src/tracker/window.rs`, `src/tracker/mod.rs`
  - **Action**: Call `GetForegroundWindow`, query process image path via `QueryFullProcessImageNameW`, extract base executable name, read window title via `GetWindowTextW`. Handles UWP ApplicationFrameHost wrappers.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 2.2: Implement idle detection and XInput gamepad activity check**
  - **Files**: `src/tracker/idle.rs`, `src/tracker/gamepad.rs`
  - **Action**: Read user inactivity duration via `GetLastInputInfo`. Call `XInputGetState` across slots 0-3 with thumbstick and trigger deadzone checks to detect controller gameplay.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 2.3: Implement Rule Engine for categorization**
  - **Files**: `src/tracker/rules.rs`, `tests/rules_test.rs`
  - **Action**: Implement pure rule matcher matching `app_name`, `window_title`, or `domain` against regex/substring patterns to assign `tag_id`.
  - **Verify**: `cargo test --test rules_test` (PASSED)

- [x] **Task 2.4: Build tracker daemon loop and accumulator**
  - **Files**: `src/tracker/engine.rs`, `tests/tracker_test.rs`
  - **Action**: Assembled 1-second ticker thread. Accumulates consecutive ticks in memory. Flushes to DB repository upon window/tag change, AFK transition, or 60s timeout.
  - **Verify**: `cargo check --lib` (PASSED)

> **Phase 2 Verification Gate**: PASSED (`rules_test` and `db_test` green)

---

### Phase 3: Ingestion Server & Integration API (`src/server`)

- [x] **Task 3.1: Implement local HTTP server and routing**
  - **Files**: `src/server/http.rs`, `src/server/mod.rs`
  - **Action**: Spun up `tiny_http` on `127.0.0.1:5566`. Parsed routes `/api/heartbeat/browser`, `/api/activities/current`, `/api/stats/summary`, and `/dashboard`.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 3.2: Implement browser heartbeat ingestion handler**
  - **Files**: `src/server/handlers.rs`, `tests/server_test.rs`
  - **Action**: Parsed incoming JSON `{ url, domain, title }`. Feeds into tracker state cache so browser window ticks correlate with active tab URL.
  - **Verify**: `cargo test --test server_test test_browser_heartbeat_ingest` (PASSED)

- [x] **Task 3.3: Expose live activity and summary endpoints for third-party integrations**
  - **Files**: `src/server/handlers.rs`, `tests/server_test.rs`
  - **Action**: Returns live JSON for AniList / Game tracker scrobblers. Serves interactive web dashboard for browser viewing.
  - **Verify**: `cargo test --test server_test test_live_activity_api` (PASSED)

> **Phase 3 Verification Gate**: PASSED (2 passed; 0 failed)

---

### Phase 4: Desktop Window UI (`src/ui`)

- [x] **Task 4.1: Scaffold `eframe` window and state architecture**
  - **Files**: `src/ui/app.rs`, `src/ui/mod.rs`
  - **Action**: Set up `ActLogApp` implementing `eframe::App`. Top status banner with live pulse, date bar, metric cards, timeline, and activity table.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 4.2: Build Date Navigator and Metric Cards**
  - **Files**: `src/ui/views/summary.rs`
  - **Action**: Render active time, idle time, productivity score, and date switcher (`< [Date] >`).
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 4.3: Build Activity Table and Category breakdown**
  - **Files**: `src/ui/views/table.rs`, `src/ui/views/timeline.rs`
  - **Action**: Render tabular list of apps/domains with percentage bars, duration formatting, and color-coded tag pills.
  - **Verify**: `cargo check --lib` (PASSED)

- [x] **Task 4.4: Add Windows System Tray integration & App launcher**
  - **Files**: `src/ui/tray.rs`, `src/main.rs`
  - **Action**: Wire `eframe::run_native`, start daemon loop and HTTP server concurrently.
  - **Verify**: `cargo build` (PASSED)

> **Phase 4 Verification Gate**: PASSED

---

### Phase 5: Browser Companion Extension (`extension/`)

- [x] **Task 5.1: Create Chrome MV3 Extension manifest and background script**
  - **Files**: `extension/manifest.json`, `extension/background.js`
  - **Action**: Listens for tab activate and URL change. Debounces 500ms. Sends `POST http://127.0.0.1:5566/api/heartbeat/browser`.
  - **Verify**: Validated manifest JSON format and JS syntax.

- [x] **Task 5.2: Create Extension Popup (View browser time in-browser)**
  - **Files**: `extension/popup.html`, `extension/popup.js`
  - **Action**: Fetches `http://127.0.0.1:5566/api/stats/summary` to show today's active time and current tab status directly inside browser popup, with link to full web view.
  - **Verify**: Validated popup markup and script.

---

### Phase 6: End-to-End Verification

- [x] **Task 6.1: Run full unit & integration test suite**
  - **Action**: Executed `cargo test --all`.
  - **Verify**: 100% test pass (5 passed, 0 failed).

- [x] **Task 6.2: Build optimized release binary**
  - **Action**: Executed `cargo build --release`.
  - **Verify**: Standalone binary produced: `target/release/actlog.exe` (5.1 MB).
