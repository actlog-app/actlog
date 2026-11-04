# UI/UX Specification & Plan: Modern Focus Dashboard (Option 1)

**Surface Mode**: Operate (High glanceability, scanability, responsive interaction).  
**Aesthetic World**: Modern Linear / Rize slate. Deep slate background (`#0D0F12`), high-contrast tabular typography, subtle elevation borders, clean 24-hour chronological timeline ribbon.

---

## 1. Visual Architecture & Layout Wireframe

```
+---------------------------------------------------------------------------------------------------+
|  [◀ Today, Nov 4 ▶]  [Today] [Yesterday] [7D] [30D]          [● Tracking Active: code.exe (45m)] [⏸] |
+---------------------------------------------------------------------------------------------------+
|  HEADLINE METRICS                                                                                 |
|  +--------------------------------+  +------------------------+  +-----------------------------+  |
|  |  78% Focus Score               |  |  5h 42m Total Active   |  |  38m Idle / Away            |  |
|  |  [==== Focus ====][=Neutral=]  |  |  Top: VS Code (3h 15m) |  |  Semi-Idle: 12m             |  |
|  +--------------------------------+  +------------------------+  +-----------------------------+  |
+---------------------------------------------------------------------------------------------------+
|  DAY FLOW (24h Interactive Chronological Ribbon)                                                  |
|  [00:00        06:00 ======= Dev ====== 12:00 =Lunch= 13:00 ==Browser== 18:00 ==Game==   24:00]   |
+---------------------------------------------------------------------------------------------------+
|  SPLIT EXPLORATION PANE                                                                           |
|  CATEGORIES (Left 240px)                | APPLICATIONS & DRILL-DOWN (Right Flex)                  |
|  [All Categories]              5h 42m   | [🔍 Search apps or documents...                       ] |
|  - Development (Work)   3h 15m (57%)    |                                                         |
|  - Communication        1h 10m (20%)    | ▾ code.exe (VS Code)                  3h 15m [========] |
|  - Entertainment          45m  (13%)    |     • main.rs - actlog                  1h 45m (54%)      |
|  - Gaming                 32m  (10%)    |     • app.rs - actlog                   1h 30m (46%)      |
|                                         | ▸ chrome.exe (Google Chrome)          1h 10m [====    ] |
|                                         | ▸ spotify.exe                           45m  [===     ] |
+---------------------------------------------------------------------------------------------------+
```

---

## 2. Core Impeccable Design Directives

1. **Hierarchy & Glanceability**:
   - Primary metric: Focus score + total active time legible from 3 feet away.
   - Secondary layer: Chronological timeline ribbon visually explaining the day's progression.
   - Tertiary layer: Ranked app list with interactive disclosure triangles (`▾` / `▸`) for detailed window titles.
2. **Eliminate 2008 Cognitive Friction**:
   - No "Expanded Level" sliders (apps expand on single click).
   - No manual "Filter < Minutes" edit boxes (micro-events under 60s automatically group into "Brief Tasks").
   - Quick date chips (`Today`, `Yesterday`, `7D`, `30D`) for instant 1-click navigation.
3. **Tabular Precision**:
   - Monospace font strictly applied to durations (`03:15:22`) and percentages (`57.0%`) so decimal points and colons align vertically across rows.
4. **Interactive Hover & Tooltip**:
   - Hovering over timeline segments reveals precise start/end time and window title.

