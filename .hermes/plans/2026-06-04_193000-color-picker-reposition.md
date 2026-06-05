# Move Color Picker Off-Center So Preview Stays Visible

**Date**: 2026-06-04
**Goal**: Reposition the color picker overlay so the Zellij preview (panes, tab bar, content) remains visible while editing colors.

## Current State

```
┌──────────────────────────────────────────────────────────────┐
│ sidebar │              preview area                          │
│ (24)    │  tab bar                                          │
│         │  ┌────── left pane ──────┬── right pane ──┐       │
│         │  │                      │                 │       │
│         │  │    ┌──────────────────·─────────────────│       │
│         │  │    │  color picker    │                 │       │
│         │  │    │  overlay (76x24) │                 │       │
│         │  │    │  centered        │                 │       │
│         │  │    └──────────────────·─────────────────│       │
│         │  │                      │                 │       │
│         │  └──────────────────────┴─────────────────┘       │
│         │  status bar                                       │
└──────────────────────────────────────────────────────────────┘
```

The picker overlay (`Clear` widget + bordered block) fully covers the center of the preview — the user can't see panes, tab bar text, or content while picking colors. The preview does render behind it (line 1093: `self.render_preview(frame)`), but the `Clear` widget at line 1126 wipes everything underneath.

## Key Files

- `src/ui/render.rs` lines 1092–1566 — `render_color_picker_mode`, `render_color_picker_overlay`
- `src/ui/color_picker.rs` lines 527–566 — `picker_layout()` decides overlay position and sub-rect layout
- `src/ui/render.rs` line 250 — `centered_rect()` for center positioning

## Approaches

### Option A: Right-panel Dock — anchor picker to right side of right panel

Move the picker from a centered overlay to a fixed-width panel docked to the right edge of the right panel area. The sidebar stays on the left. The preview renders in the remaining space.

```
┌────────────────────────────────────────────────────────────────────┐
│ sidebar │     preview (shrunk)         │    color picker (38 cols) │
│ (24)    │  tab bar                     │  ┌── mode ──────────────┐ │
│         │  ┌─ left pane ─┬─ right ─┐  │  │ RGB / HSV / Field    │ │
│         │  │             │          │  │  │ ████ swatch          │ │
│         │  │             │          │  │  │ R [=======●===] 255  │ │
│         │  │             │          │  │  │ G [====●=======] 128  │ │
│         │  │             │          │  │  │ B [========●===] 64   │ │
│         │  │             │          │  │  │ hex  #FF8040          │ │
│         │  │             │          │  │  │ hsl  24° 75% 50%      │ │
│         │  └─────────────┴──────────┘  │  │                        │ │
│         │  status bar                 │  └────────────────────────┘ │
└────────────────────────────────────────────────────────────────────┘
```

**Pros**:
- Preview stays visible alongside picker — real-time feedback
- No overlay/obscuring; everything is in a split layout
- Uses existing `PickerRects` layout with adjusted column proportions (62/38 → constrained properly)
- Single source of truth: picker is always in the same place relative to preview

**Cons**:
- Most invasive change — replaces the overlay pattern with a permanent panel
- Picker needs responsive layout for narrow terminals (currently 76 cols, right panel might get tight)
- The "preview" column inside the picker (the 5-row swatch area in `side_col` at line 549) becomes redundant with the live preview already visible — might want to remove it and use that space for fields/controls
- Need to handle entering/exiting picker mode — the layout shift might be jarring

**Files changed**:
- `src/ui/render.rs` — `render_color_picker_mode()`: replace `render_preview() + overlay` pattern with split layout; adjust `render_color_picker_overlay()` to render into a panel not a centered rect
- `src/ui/color_picker.rs` — `picker_layout()`: change from centered overlay to right-anchored panel; drop `overlay` rect in favor of a target area rect

### Option B: Bottom Dock — anchor picker to bottom of screen

Move the picker to a horizontal strip at the bottom, leaving the top portion for the preview.

```
┌────────────────────────────────────────────────────────────────────┐
│ sidebar │              preview area                                │
│ (24)    │  tab bar                                                │
│         │  ┌── left pane ────┬─── right pane ──┐                  │
│         │  │                 │                  │                  │
│         │  │                 │                  │                  │
│         │  └─────────────────┴──────────────────┘                  │
│         │  status bar                                              │
├─────────┴─────────────────────────────────────────────────────────┤
│ color picker (full width, ~6-8 rows)                              │
│ mode │ R [=====●====]  │ G [======●===] │ B [======●===] │ ████   │
│ hex #FF8040            │ hsl 24° / 75% / 50%                      │
└────────────────────────────────────────────────────────────────────┘
```

**Pros**:
- Preview fully visible above
- Picker gets full width — no horizontal space crunch
- Compact horizontal layout fits the controls naturally (sliders side by side)
- Less jarring shift than a full layout restructure

**Cons**:
- Picker must be compact (6-8 rows max) — the current HSL field mode (16+ rows with visual color field) won't fit horizontally
- Need to redesign the HSL field mode: horizontal color bar instead of 2D field, or switch to RGB-sliders-only in this mode
- Field editing (hex, RGB, HSL) needs horizontal layout instead of stacked columns

**Files changed**:
- `src/ui/render.rs` — split layout: top (sidebar + main) + bottom (picker)
- `src/ui/color_picker.rs` — `picker_layout()`: full-width horizontal layout; drop HSL visual field for a compact horizontal bar

### Option C: Offset Overlay — slide the existing overlay to the right

Keep the overlay pattern but position it to the right side instead of centered, so the left pane of the preview stays visible.

```
┌────────────────────────────────────────────────────────────────────┐
│ sidebar │              preview area                                │
│ (24)    │  tab bar                                                │
│         │  ┌── left pane ────┬─────────────────────────┐          │
│         │  │  visible        │  color picker overlay    │          │
│         │  │  (unobscured)   │  ┌── mode ───────────┐   │          │
│         │  │                 │  │ R [===●======] 128 │   │          │
│         │  │  user sees      │  │ G [======●===] 64  │   │          │
│         │  │  their changes  │  │ B [=====●====] 200 │   │          │
│         │  │  here           │  │ hex #8040FF        │   │          │
│         │  │                 │  └────────────────────┘   │          │
│         │  └─────────────────┴───────────────────────────┘          │
│         │  status bar                                               │
└────────────────────────────────────────────────────────────────────┘
```

**Pros**:
- Least invasive — keeps the existing overlay widget and layout code mostly intact
- Left 60% of preview visible (left pane always visible, right pane partially obscured)
- Quicker to implement — just change `centered_rect` to `right_aligned_rect`

**Cons**:
- Only shows one pane, not both
- Overlay still obscures some of the preview (right pane, status elements)
- Feels like a half-measure — you still lose part of the preview
- The `Clear` widget will wipe the right portion of the preview, making it draw inconsistently

**Files changed**:
- `src/ui/render.rs` — add `right_aligned_rect()` helper, use in `picker_layout` call
- `src/ui/color_picker.rs` — `picker_layout()`: position overlay right-aligned instead of centered

## Comparison

| Dimension          | Option A (Right Dock)        | Option B (Bottom Dock)       | Option C (Offset Overlay)    |
|---------------------|------------------------------|------------------------------|------------------------------|
| Preview visibility  | Full — both panes visible    | Full — everything above picker| Left pane only               |
| Implementation effort | High — restructure layout   | Medium — redesign to horizontal | Low — move overlay right     |
| Responsiveness      | Picker gets narrow at small terms | Full width, but short height | Same as current (76 cols)    |
| Code change         | ~100 lines                   | ~80 lines                     | ~20 lines                    |
| Long-term feel      | Feels intentional, polished  | Feels like a HUD strip       | Feels like a compromise      |

## Recommendation

**Option A (Right-panel dock)** if you want the best visual result and are willing to put in the work. The live preview adjacent to the sliders is the gold-standard UX for a theme editor.

**Option B (Bottom dock)** if you want the preview fully visible but prefer a less disruptive code change than a full layout restructure.

**Option C (Offset overlay)** if you want the quickest fix and don't mind only seeing the left pane.

## Open Questions

1. In Option A, the picker's internal "preview" swatch column (the 5-row color block at `side_col` in `picker_layout`) becomes redundant — should we remove it and expand the fields area?
2. In Option B, the HSL color field (interactive 2D hue/saturation picker) doesn't fit horizontally — do we replace it with a 1D hue bar + separate sliders, or keep RGB-sliders-only?
3. Should the picker position be togglable (overlay vs docked), or committed to one layout?
