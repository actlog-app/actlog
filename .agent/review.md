# Plan Review: ActLog Architecture & Implementation

**Review Date**: 2026-11-04  
**Target Plan**: `.agent/plan.md`  
**Review Method**: 4-Axis Adversarial Review (`review-plan`)

---

## 1. Architecture Review

### Findings & Tradeoffs
1. **Desktop UI Thread vs Background Poller**:
   - *Risk*: If `eframe` GUI loop blocks or if window is closed/minimized, the activity poller and HTTP server must continue uninterrupted.
   - *Recommendation*: Dedicated `std::thread` for Windows foreground/idle polling + separate thread for `tiny_http` server. Communicate with GUI thread via `crossbeam_channel::unbounded` or an `Arc<Mutex<SharedState>>` cache. Never run polling on the GUI render tick.
2. **Local HTTP Ingestion Security**:
   - *Risk*: A local web page in Chrome could attempt DNS rebinding or CSRF against `127.0.0.1:5566` to read activity data.
   - *Recommendation*: Restrict CORS headers to `chrome-extension://*` or localhost, and require a simple local bearer token or origin check on mutating endpoints (`POST /api/heartbeat/browser`).
3. **Database Concurrency (GUI Reads vs Daemon Writes)**:
   - *Risk*: SQLite `database is locked` error if both the 1-second aggregator and the desktop window queries access the database simultaneously.
   - *Recommendation*: Enable WAL mode (`PRAGMA journal_mode = WAL;`) and set a busy timeout (`PRAGMA busy_timeout = 5000;`). Keep all writes strictly inside a single writer loop on the tracker thread, exposing read-only connections to the UI.

---

## 2. Code Quality Review

### Findings & Tradeoffs
1. **Win32 String & Handle Safety**:
   - *Risk*: Direct Win32 API calls (`GetWindowTextW`, `QueryFullProcessImageNameW`, `OpenProcess`) involve raw pointers, buffers, and system handles. Failing to close `HANDLE` leads to handle leaks.
   - *Recommendation*: Wrap all Windows handles in RAII types or use `windows::Win32::Foundation::CloseHandle`. Sanitize null-terminated UTF-16 conversions into safe Rust `String` instances at the boundary before leaving `src/tracker/window.rs`.
2. **Domain/Rule Matching Modularity**:
   - *Risk*: Hardcoding matching logic inside the polling loop makes it impossible to unit test without running on Windows.
   - *Recommendation*: Isolate rule evaluation into pure domain functions `fn match_rule(session: &RawTick, rules: &[Rule]) -> Option<TagId>`. This allows 100% unit test coverage on Linux/CI without Win32 dependencies.

---

## 3. Tests Review

### Findings & Tradeoffs
1. **Mockability of Windows System Calls**:
   - *Risk*: CI runners and headless test environments will fail if tests directly call `GetForegroundWindow` or `XInputGetState`.
   - *Recommendation*: Implement a trait `WindowPoller` and `InputIdlePoller` with a `RealWin32Poller` (cfg Windows) and a `MockPoller` (for tests). All aggregation, tick merging, and rule classification tests will run deterministically against synthetic tick streams.
2. **Missing Edge Cases in Test Matrix**:
   - *Add*: Test midnight day boundary rollover (session starting at `23:59:50` and ending at `00:00:10`).
   - *Add*: Test title change within the same process (e.g. switching tabs in browser or switching files in IDE).
   - *Add*: Test rapid 100ms window switching (debounce / jitter suppression).

---

## 4. Performance Review

### Findings & Tradeoffs
1. **Database Write Amplification**:
   - *Risk*: Writing an update to SQLite every 1000ms would cause continuous disk I/O, wear SSDs, and drain laptop batteries.
   - *Recommendation*: In-memory session accumulator. Keep active session in memory. Flush to SQLite only when:
     - The active window or browser tab changes.
     - User goes idle / AFK.
     - Every 60 seconds as a safety checkpoint.
     - Application shutdown.
     This reduces disk writes from 86,400 per day down to <1,500 per day.
2. **Memory Footprint**:
   - *Target*: `<20MB` working set.
   - *Technique*: Avoid storing unbounded activity history in memory. UI only queries the selected date range (`LIMIT / OFFSET` or daily bucket aggregation in SQL).

