# Repositionable Color Picker with Shift+Arrows

**Date**: 2026-06-04
**Goal**: Let the user move the color picker overlay around with Shift+Arrow keys so they can see whichever part of the preview matters most while editing.

## Current State

The color picker is a centered 76x24 overlay. It renders via `Clear` widget, completely wiping the preview underneath. `picker_layout()` in `color_picker.rs` calls `centered_rect()` which anchors it to the middle of the screen:

```rust
// color_picker.rs line 528-530
let overlay_w = 76u16.min(area.width.saturating_sub(4));
let overlay_h = 24u16.min(area.height.saturating_sub(4));
let overlay = super::render::centered_rect(area, overlay_w, overlay_h);
```

## Proposed Approach: Shift+Arrow Repositioning

Instead of one fixed position, track an anchor offset in `App` state. Shift+Arrow keys nudge the overlay by a grid step (e.g. 4 columns / 2 rows) in the Preview input mode (not ColorPicker mode — the user repositions before or during picking). The overlay snaps to a few discrete positions so it never lands at a confusing half-overlap.

### Positions (4-cardinal + center)

```
┌──────────────────────────────────────────────────────────┐
│ sidebar │              preview area                      │
│         │  ┌───────────────────────────────────────┐     │
│ TOP-LEFT │  │ tab bar                              │     │   TOP-RIGHT
│         │  │ ┌── left pane ──┬── right pane ──┐    │     │
│  ┌──────┼──┼─┼──────────────┼────────────────┼────┼─────┤
│  │picker│  │ │              │                │    │picker│
│  │      │  │ │  preview     │   preview      │    │     │
│  └──────┼──┼─┼──────────────┼────────────────┼────┼─────┤
│         │  │ │              │                │    │     │
│ BOT-LEFT│  │ └──────────────┴────────────────┘    │     │ BOT-RIGHT
│  ┌──────┼──┼─┐                                  ┌─┼─────┤
│  │picker│  │ │   status bar                      │ │picker│
│  └──────┼──┼─┘                                  └─┼─────┤
│         │  │                                       │     │
│         │  └───────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────┘
```

The overlay snaps to **corner** positions so part of the preview is always visible. Center position is also available (current behavior).

### State changes (`src/ui/state.rs`)

Add to `App`:
```rust
pub struct App {
    // ... existing fields ...
    /// Overlay anchor position for color picker and other overlays.
    pub overlay_anchor: OverlayAnchor,
}
```

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OverlayAnchor {
    Center,       // default, current behavior
    TopLeft,
    TopRight,
    BottomLeft,
    BottomRight,
}
```

Add methods:
```rust
impl OverlayAnchor {
    pub fn all() -> &[Self; 5] { &[Center, TopLeft, TopRight, BottomLeft, BottomRight] }
    pub fn next(self) -> Self { /* cycle through */ }
    pub fn prev(self) -> Self { /* cycle backward */ }
}

impl App {
    pub fn nudge_overlay_left(&mut self)  { self.overlay_anchor = self.overlay_anchor.prev(); }
    pub fn nudge_overlay_right(&mut self) { self.overlay_anchor = self.overlay_anchor.next(); }
}
```

### Event changes (`src/ui/events.rs`)

In Preview mode, add Shift+Arrow bindings:
```rust
InputMode::Preview => {
    match key.code {
        // ... existing ...
        KeyCode::Left if has_shift  => { app.nudge_overlay_left();  app.message = None; }
        KeyCode::Right if has_shift => { app.nudge_overlay_right(); app.message = None; }
        // Alternatively, just arrow through all 5 positions with Shift+Left/Right
    }
}
```

Also in ColorPicker mode — allow repositioning while the picker is open:
```rust
InputMode::ColorPicker => {
    match key.code {
        // ... existing ...
        KeyCode::Left if has_shift  => { app.nudge_overlay_left(); }
        KeyCode::Right if has_shift => { app.nudge_overlay_right(); }
    }
}
```

The picker closes on Escape, so the user can reposition it freely without leaving color edit mode.

### Layout changes (`src/ui/color_picker.rs`)

Modify `picker_layout()`:
```rust
pub fn picker_layout(area: Rect, mode: ColorPickerMode, anchor: OverlayAnchor) -> PickerRects {
    let overlay = overlay_rect(area, anchor);  // replaces centered_rect
    // ... rest unchanged ...
}

fn overlay_rect(area: Rect, anchor: OverlayAnchor) -> Rect {
    let w = OVERLAY_W.min(area.width.saturating_sub(4));
    let h = OVERLAY_H.min(area.height.saturating_sub(4));
    let sidebar_w = 24; // sidebar is always present
    let right_area = Rect { x: sidebar_w, y: 0, width: area.width - sidebar_w, height: area.height };

    match anchor {
        OverlayAnchor::Center     => centered_rect(right_area, w, h),
        OverlayAnchor::TopLeft    => Rect { x: sidebar_w, y: 0, width: w, height: h },
        OverlayAnchor::TopRight   => Rect { x: area.width - w, y: 0, width: w, height: h },
        OverlayAnchor::BottomLeft => Rect { x: sidebar_w, y: area.height - h, width: w, height: h },
        OverlayAnchor::BottomRight=> Rect { x: area.width - w, y: area.height - h, width: w, height: h },
    }
}
```

Key detail: the overlay is positioned relative to the **right panel area** (after the 24-char sidebar). The overlay never covers the sidebar — the sidebar is always visible.

### Render changes (`src/ui/render.rs`)

Pass `self.overlay_anchor` to `picker_layout()`:
```rust
fn render_color_picker_overlay(&self, frame: &mut Frame) {
    let rects = picker_layout(frame.area(), self.color_editor.mode, self.overlay_anchor);
    // ... rest unchanged ...
}
```

Remove the `Clear` widget? Actually, `Clear` is still needed because the overlay renders on top of the preview. The difference is just WHERE the clear+overlay goes.

### Status bar indicator

Add a hint in the status bar when in preview mode:
```
↑↓ ELEMENT  c COLOR  ←→ SHIFT MOVE  / FIND  ...  q QUIT
```

## Behavior Summary

| Key           | Mode    | Action                              |
|---------------|---------|-------------------------------------|
| Shift+←       | Preview | Cycle overlay anchor left           |
| Shift+→       | Preview | Cycle overlay anchor right          |
| Shift+←       | Picker  | Nudge overlay anchor left (live)    |
| Shift+→       | Picker  | Nudge overlay anchor right (live)   |
| c             | Preview | Open picker at current anchor       |
| Esc           | Picker  | Close picker, return to preview     |

The overlay anchor persists across sessions — it's in `App` state and could be saved to memory.

## Step-by-step Plan

1. **State** (`src/ui/state.rs`):
   - Add `OverlayAnchor` enum with 5 variants
   - Add `overlay_anchor: OverlayAnchor` field to `App` (default: `Center`)
   - Add `nudge_overlay_left()`, `nudge_overlay_right()` methods
   - Add `OverlayAnchor::prev()` / `next()` cycle helpers

2. **Events** (`src/ui/events.rs`):
   - In Preview mode: map Shift+Left → `nudge_overlay_left()`, Shift+Right → `nudge_overlay_right()`
   - In ColorPicker mode: same bindings, so user can reposition without closing picker

3. **Layout** (`src/ui/color_picker.rs`):
   - Change `picker_layout()` signature: add `anchor: OverlayAnchor` parameter
   - Replace `centered_rect()` with `overlay_rect()` helper that positions based on anchor
   - Overlay positioned within right panel area (after sidebar)

4. **Render** (`src/ui/render.rs`):
   - Pass `self.overlay_anchor` to `picker_layout()` call
   - Update status bar keybinding hint to show "SHIFT MOVE"
   - Add `Shift+←→` to help screen keybinding list

5. **Test**:
   - `cargo build` — compile with new enum/field
   - `cargo test` — all 15 existing tests pass (they use centered, no behavior change needed since default is Center)
   - Verify picker appears at correct positions for each anchor

## Files Changed

| File                    | Change                          | Lines   |
|-------------------------|---------------------------------|---------|
| `src/ui/state.rs`       | +OverlayAnchor enum, +field, +methods | ~30     |
| `src/ui/events.rs`      | +Shift+Arrow bindings (preview + picker) | ~8      |
| `src/ui/color_picker.rs`| Modify `picker_layout()` sig, add `overlay_rect()` | ~20     |
| `src/ui/render.rs`      | Pass anchor to layout, update keybinding hints | ~10     |

## Risks & Edge Cases

- **Overlay clips at edge**: If terminal is narrower than sidebar_w + overlay_w, the right-anchored overlay would overflow. Need to clamp: `overlay_rect()` should use `area.width.saturating_sub(w)` for right edge.
- **Clear widget**: Still needed since overlay renders on top of preview. The `Clear` operates on the overlay rect, which now moves.
- **Shift key detection**: Ratatui's crossterm backend sends `KeyCode::Left` with `KeyModifiers::SHIFT`. Need to check if current event handling supports modifier detection. If not, fall back to a different key (e.g. `[`/`]` or `<`/`>` or Tab).
- **Persistence**: Anchor position is ephemeral (in `App`). Could save to memory if wanted, but not necessary for MVP.

## Open Question

Does the current event loop detect Shift as a modifier on arrow keys? If not, alternatives:
- **`<` / `>` keys** (no shift needed) — `KeyCode::Char('<')` / `KeyCode::Char('>')`
- **`[` / `]` keys** — already used for color channel decrement in color picker mode, so conflict
- **`Tab` / `Shift+Tab`** — Tab is already used in color picker for focus switching
- **Alt+←→** — same modifier detection issue
- **`H` / `L`** — vim-style, but H is already `HexField` focus in color picker

Best fallback: `<` and `>` in Preview mode. They're unbound and intuitive for "slide left/right".
