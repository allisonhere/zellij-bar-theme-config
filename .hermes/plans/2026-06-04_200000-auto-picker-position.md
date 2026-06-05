# Auto-Position Picker Based on Selected Element

**Date**: 2026-06-04
**Goal**: Remove manual picker repositioning keys. Instead, when the user presses `c` to open the color picker, it automatically opens in a position that leaves the selected preview element visible — opposite side of the screen.

## Problem

Manual repositioning with `<` / `>` is an extra cognitive step. The user edits an element and wants to see the preview — they shouldn't have to think about where to place the picker. The picker should just know.

## Approach: Derive Anchor from Selected Element

Each `PreviewElement` knows which pane it renders in. The picker should open on the opposite side:

- If editing a **left-pane** element → picker opens **bottom-right** (right pane visible, picker covers bottom-right of right panel → left pane fully visible)
- If editing a **right-pane** element → picker opens **bottom-left** (left pane covered, right pane visible)
- If editing a **neutral** element (tab bar, status bar) → picker opens **bottom-right** (most preview detail is in the left pane)

The preview area layout:
```
┌─────────────────────────────────────────────────────┐
│ sidebar │ tab bar                                   │
│ (24)    │ ┌─── left pane (60%) ──┬── right pane ──┐ │
│         │ │  PaneSelected       │  PaneUnselected │ │
│         │ │  TextSelected       │  PaneHighlight  │ │
│         │ │  TextUnselected     │                 │ │
│         │ │  TableTitle         │                 │ │
│         │ │  TableCellSelected  │                 │ │
│         │ │  TableCellUnselected│                 │ │
│         │ │  ListSelected       │                 │ │
│         │ │  ListUnselected     │                 │ │
│         │ └─────────────────────┴─────────────────┘ │
│         │ status bar                                │
└─────────────────────────────────────────────────────┘
```

### Element → Pane Mapping

| Element              | Pane      | Picker Position |
|----------------------|-----------|-----------------|
| Tab Selected         | Tab bar   | BottomRight     |
| Tab Unselected       | Tab bar   | BottomRight     |
| Pane Selected        | Left      | BottomRight     |
| Text Selected        | Left      | BottomRight     |
| Text Unselected      | Left      | BottomRight     |
| Pane Unselected      | Right     | BottomLeft      |
| Pane Highlight       | Right     | BottomLeft      |
| Table Title          | Left      | BottomRight     |
| Table Cell Selected  | Left      | BottomRight     |
| Table Cell Uns       | Left      | BottomRight     |
| List Selected        | Left      | BottomRight     |
| List Unselected      | Left      | BottomRight     |
| Exit Success         | Status    | BottomRight     |
| Exit Error           | Status    | BottomRight     |
| Status Bar           | Status    | BottomRight     |

Only two right-pane elements: PaneUnselected, PaneHighlight. Everything else → BottomRight.

## Implementation

### 1. Add `preferred_picker_anchor()` to `PreviewElement` (`src/ui/state.rs`)

```rust
impl PreviewElement {
    pub fn preferred_picker_anchor(&self) -> OverlayAnchor {
        match self {
            Self::PaneUnselected | Self::PaneHighlight => OverlayAnchor::BottomLeft,
            _ => OverlayAnchor::BottomRight,
        }
    }
}
```

### 2. Remove manual repositioning

Remove from `src/ui/state.rs`:
- `nudge_overlay_left()` method
- `nudge_overlay_right()` method
- `overlay_anchor` field from `App` struct (and Default)

Remove from `src/ui/events.rs`:
- `<` and `>` key handlers for overlay repositioning

### 3. Derive anchor on-the-fly in render (`src/ui/render.rs`)

In `render_color_picker_overlay`:
```rust
// Old:
let rects = picker_layout(frame.area(), self.color_editor.mode, self.overlay_anchor);
// New:
let anchor = self.selected_element.preferred_picker_anchor();
let rects = picker_layout(frame.area(), self.color_editor.mode, anchor);
```

### 4. Update mouse handler in events.rs

The mouse handler calls `picker_layout` with `app.overlay_anchor` — change to derive from `app.selected_element`:
```rust
let anchor = app.selected_element.preferred_picker_anchor();
let rects = picker_layout(ratatui::layout::Rect::new(0, 0, w, h), app.color_editor.mode, anchor);
```

### 5. Remove `overlay_anchor` from state

- Remove field from `App` struct
- Remove default value `OverlayAnchor::Center`
- Remove `nudge_overlay_left()` / `nudge_overlay_right()` methods
- Remove `<` / `>` keybindings from events

## Files Changed

| File                | Change                              | Lines |
|---------------------|-------------------------------------|-------|
| `src/ui/state.rs`   | +`preferred_picker_anchor()` method, -`overlay_anchor` field, -`nudge_*` methods | +8 / -18 |
| `src/ui/events.rs`  | -`<` `>` handlers, update mouse handler to derive anchor | -8 / +3 |
| `src/ui/render.rs`  | Derive anchor from selected_element instead of overlay_anchor field | +1 / -1 |

Net: ~-15 lines. Simpler than the manual approach.

## Behavior

1. User navigates to any element (↑↓)
2. Presses `c` to open color picker
3. Picker automatically opens in the corner opposite the selected element:
   - Right-pane elements (Pane Unselected, Pane Highlight) → picker bottom-left → right pane stays visible
   - Everything else → picker bottom-right → left pane stays visible
4. No manual repositioning needed — it's always right

## Verification

- `cargo build` — clean
- `cargo test` — all 15 pass (no test expects overlay_anchor field anywhere since it's never tested by existing tests)
- Manual: pick PaneSelected → press `c` → picker opens bottom-right, left pane visible. Pick PaneUnselected → press `c` → picker opens bottom-left, right pane visible.
