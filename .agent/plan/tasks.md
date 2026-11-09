# ProcrastiTracker 1:1 Visual & Structural Parity Plan

## Objective
Achieve 1:1 visual, structural, and behavioral parity with canonical ProcrastiTracker (photo1 `media_1794157041471_0e9b3140.png`), completely eliminating deviations observed in photo2 (`media_1794157111474_b0b273b6.png`).

---

### Phase 1: Canonical 15-Tag Palette, Seed Data, and Legacy Ingestion
- [x] **Task 1.1: Fix Tag Color Values & Reconcile Canonical 15 Tags in DB**
  - **Files**: `src/db/schema.rs`, `src/model/tag.rs`
  - **Action**: Fix Win32 `COLORREF` (BGR) to RGB conversion in default tags. Canonical colors:
    - `UNTAGGED`: `#D0D0D0`
    - `work`: `#60FF60`
    - `games`: `#FF6060` (Win32 `0x6060FF` -> Red 255, Green 96, Blue 96)
    - `surfing`: `#FFFF60` (Win32 `0x60FFFF` -> Red 255, Green 255, Blue 96)
    - `entertainment`: `#FF60FF`
    - `communication`: `#60FFFF` (Win32 `0xFFFF60` -> Red 96, Green 255, Blue 255)
    - `organization`: `#6060FF` (Win32 `0xFF6060` -> Red 96, Green 96, Blue 255)
    - `project 1`: `#FF0000`
    - `project 2`: `#00FF00`
    - `project 3`: `#0000FF`
    - `project 4`: `#60B0B0`
    - `project 5`: `#B060B0`
    - `project 6`: `#B0B060`
    - `project 7`: `#606060`
    - `project 8`: `#B0B0B0`
  - **Deduplication**: Prevent runaway duplicate default rule insertions (`SELECT EXISTS`).
  - **Legacy Migration**: Ingest user's 33,685 legacy sessions from `actlog.sqlite` into `actlog.db` so the UI has full historical activity data spanning months.
  - **Verify**: `cargo test --test db_test`

---

### Phase 2: Unicode & Windows Font Fallback (Tofu Elimination)
- [x] **Task 2.1: Load Windows System Fonts in Theme Configuration**
  - **Files**: `src/ui/theme.rs`
  - **Action**: In `configure_win32_theme(ctx)`, inspect and load `C:\Windows\Fonts\segoeui.ttf`, `C:\Windows\Fonts\Nirmala.ttc` (Bengali / Indic fallback), and `C:\Windows\Fonts\msyh.ttc` into `egui::FontDefinitions`. Ensure proportional font family prioritizes Segoe UI, Nirmala, and default fallbacks.
  - **Verify**: `cargo check`

---

### Phase 3: Vertical Day Minimap Graph (`src/ui/views/day_graph.rs`)
- [x] **Task 3.1: Implement 1:1 `renderdaystat` with Stacked Daily Bars and Hover Hit-Testing**
  - **Files**: `src/ui/views/day_graph.rs`, `src/ui/views/mod.rs`
  - **Action**: Implement vertical minimap:
    - Width ~80-90px on right side of window.
    - White canvas (`#FFFFFF`) with 1px border (`#7A7A7A`).
    - Aggregation per active day across entire database date range (`start_date` to `end_date`).
    - Stacked horizontal colored tag bars drawn per day from top to bottom.
    - Hit-testing on mouse move to calculate hovered day and format `YYYY-MM-DD -> HH:MM:SS`.
    - Provide callback or state update for left sidebar bottom status label.
  - **Verify**: `cargo check`

---

### Phase 4: Left Sidebar 1:1 Alignment & Bottom Status Label
- [x] **Task 4.1: Refactor Left Sidebar Controls to Match photo1**
  - **Files**: `src/ui/views/statistics.rs`
  - **Action**:
    - Expanded Level: slider with tick marks, no extraneous numeric edit box (`show_value(false)`).
    - Filter < Minutes: text box (default 0).
    - Filter on sub-strings: text box without trailing colon (`Filter on sub-strings`).
    - Date Range GroupBox:
      - Start date and end date displayed as `M/D/YYYY` with `[📅]` calendar icon button.
      - Interactive calendar grid popup for `[📅]` allowing direct date picking and quick "Today" selection.
      - Quick Set combobox defaulting to `All Time` when loading full historical date range.
    - Tagging GroupBox:
      - `[ ] Filter on tag` checkbox.
      - `Apply Tag to Node` button (`min_size` matching dialog button).
      - `Rename Tag` button.
      - 15 Canonical Tags ListView with 12x12 square swatches and Win32 selection highlight.
    - Bottom status label: `"Hover over graph..."` when not hovering minimap, or active day time string when hovering.
  - **Verify**: `cargo check`

---

### Phase 5: Center TreeView with Root Node, Guidelines, and In-Row Bars
- [x] **Task 5.1: Synthetic Root Node & Math Parity**
  - **Files**: `src/ui/views/statistics.rs`
  - **Action**:
    - Insert root node `[-] <total_time> - (root)` at the top of the tree.
    - Root node has no colored bar.
    - Default selection set to `(root)` with native Win32 light-blue selection rectangle.
    - Child nodes calculate bar width proportional to `max_top_level_seconds` (top child app).
  - **Verify**: `cargo check`

- [x] **Task 5.2: Full-Height In-Row Colored Bars Behind Text, Dotted Guidelines, and Formatting**
  - **Files**: `src/ui/views/statistics.rs`, `src/ui/common.rs`
  - **Action**:
    - Draw dotted tree guidelines connecting parent to children with multi-level ancestor tracking (`ancestor_has_more`).
    - Extend horizontal guideline all the way to text for leaf nodes (`!has_children`).
    - Draw square 9x9 `[+]` / `[-]` toggles with manual toggle persistence (`expanded_overrides`).
    - 1:1 single-child chain collapse (`node.h:372`): apps with 1 window title are collapsed into `app - window_title` leaf nodes.
    - Convert app names to lowercase (`Code.exe` -> `code`).
    - Format all times as `H:MM:SS` (hours without leading zero, minutes/seconds 2-digit zero-padded).
    - For collapsed / leaf nodes: paint the tag-colored rectangle for full row height starting at text indentation and extending across `total_bar_w`.
    - Render the node text `"<time> - <name>"` in crisp black directly ON TOP of the colored bar.
  - **Verify**: `cargo test`

---

### Phase 6: Single-Window 3-Panel Layout & Removal of Unsolicited Chrome
- [x] **Task 6.1: Clean Removal of Top Menu Bar and Tab Strip**
  - **Files**: `src/ui/app.rs`, `src/main.rs`, `src/ui/views/mod.rs`
  - **Action**:
    - Delete `win32_menu_bar` TopBottomPanel.
    - Delete `[ Statistics ] [ Time View ]` tab strip.
    - Clean up dead code (`time_view.rs`).
    - Set window size to `[1024.0, 560.0]` matching photo1.
    - Wire Left Sidebar, Center TreeView, and Right Day Minimap into authentic 3-panel layout.
  - **Verify**: `cargo test`, `cargo build`

---

### Phase 7: Automated Screenshot Verification Loop & Visual Parity Testing
- [x] **Task 7.1: Build Screenshot Capture Script & Comparison Harness**
  - **Files**: `tests/take_screenshot.ps1`, `tests/statistics_parity_test.rs`
  - **Action**: Launch compiled `actlog.exe`, capture framebuffer screenshot to `target/screenshot_current.png`, compare visually against canonical photo1.
  - **Verify**: 10/10 tests passing (0 failures), screenshot matches photo1 layout, guidelines, tags, and day minimap.
