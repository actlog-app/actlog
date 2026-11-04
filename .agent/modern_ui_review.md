# Plan Review: Modern Focus Dashboard (`review-plan`)

**Target Plan**: `.agent/modern_ui_plan.md`  
**Review Method**: 4-Axis Adversarial Review

---

## 1. Architecture Review

### Findings & Tradeoffs
1. **Immediate Mode Rendering vs State Aggregation**:
   - *Issue*: Rebuilding tree hierarchies and timeline segments on every frame in `egui` (60fps) would waste CPU cycles on pure math.
   - *Recommendation*: Cache precomputed `ProcessedDayView` in `ActLogApp`. Only recompute when:
     - Date selection changes.
     - New database records arrive (via 3-5s debounce timer).
     - User types in the search box.
2. **Category Filter State Isolation**:
   - *Issue*: Filtering by category should not reload SQLite data from disk.
   - *Recommendation*: Filter in memory from the cached `Vec<ActivitySession>`. Instant zero-latency UI response.

---

## 2. Code Quality Review

### Findings & Tradeoffs
1. **Component Modularity**:
   - *Issue*: Putting 1,000 lines into `app.rs` makes maintenance fragile.
   - *Recommendation*: Cleanly split UI into targeted views under `src/ui/views/`:
     - `header.rs`: Date picker, quick date chips, live pulse status.
     - `metrics.rs`: Focus score hero card, active time, idle time.
     - `ribbon.rs`: 24-hour chronological visual ribbon with time ticks and hover tooltips.
     - `apps.rs`: Split pane with category list (left) and expandable app/title tree (right).

---

## 3. Tests Review

### Findings & Tradeoffs
1. **Data Transformation & Metric Calculation Testing**:
   - *Ensure*: Focus score computation (`(active_work / total_time) * 100`) must have unit tests covering edge cases:
     - 0 active seconds (divide-by-zero protection).
     - Entirely idle day.
     - Mixed productivity scores (-2 to +2).
2. **Timeline Ribbon Segment Merging**:
   - *Ensure*: Test that sequential activities merge into cohesive ribbon blocks so the timeline doesn't render thousands of 1px slivers.

---

## 4. Performance Review

### Findings & Tradeoffs
1. **Timeline Ribbon Drawing Overhead**:
   - *Issue*: If 1,000 micro-switches occur during the day, drawing 1,000 distinct `Rect` primitives every frame can introduce draw-call overhead.
   - *Recommendation*: Merge adjacent activities sharing the same tag within a 60-second window before rendering the ribbon. Caps visual segments at under 200 per day.
2. **Scroll Area Memory**:
   - *Technique*: Use `egui::ScrollArea` with row culling so off-screen window titles are never painted.

