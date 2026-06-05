# Plan: FG/BG toggle — what "BG" actually means for pane components

**User observation:** The "F" selector toggles between FG and BG, but the effect isn't visible —
"BG" seems to mean border background, not pane background, since pane fill is derived from terminal BG.

## Findings

### How the preview renders panes

In `render_pane_selected` (line 564) and `render_pane_unselected` (line 639):

```rust
let pane_bg  = get_bg(ThemeComponentType::TextUnselected, t);     // pane fill
let border_color = get_fg(ThemeComponentType::FrameSelected, t);  // border
```

- **Pane fill** = `TextUnselected.background` (the text_unselected component's background field)
- **Pane border** = `FrameSelected.base` / `FrameUnselected.base` (the frame component's base field)
- `FrameSelected.background` and `FrameUnselected.background` are editable but NEVER used in the preview

### What gets written to KDL

`theme_to_kdl` writes all 6 fields per component (base, background, emphasis_0–3). So
`frame_selected.background` DOES get saved/applied to the Zellij config — it just has zero
visible effect in the preview.

### What "BG" means in the color picker

The color picker toggles `PreviewAttribute::Base ↔ Background` for whichever component
the current PreviewElement maps to. For pane elements:
- PaneSelected → FrameSelected.base (FG = border ✓) / FrameSelected.background (BG = unused ✗)
- PaneUnselected → FrameUnselected.base (FG = border ✓) / FrameUnselected.background (BG = unused ✗)
- TextSelected → TextSelected.base (FG = text ✓) / TextSelected.background (BG = text bg ✓)

So the FG/BG toggle works correctly for TextSelected and most other elements, but for
PaneSelected and PaneUnselected, the "BG" side is a dead end in preview terms.

### What Zellij likely uses frame.background for

In real Zellij, the frame component's `background` likely controls the folded/session
ribbon background or pane separator styling — not the pane content area background.
The pane content background in Zellij derives from the terminal palette "bg" key
(which maps to `TextUnselected.background` in this app).

## Options

### Option 1: Make pane preview use Frame*.background as pane fill

Change `pane_bg` in the preview from `TextUnselected.background` to the current
frame component's `background`. PaneSelected would use `FrameSelected.background`,
PaneUnselected would use `FrameUnselected.background`.

**Pros:** Makes the BG toggle actually visible for panes. Each pane can have its own fill.
**Cons:** Deviates from actual Zellij behavior (pane fill = terminal bg). Could confuse
users who expect pane bg to be unified. Breaks analogy with real Zellij.

### Option 2: Remove BG toggle for frame components

Hide the "F" (FG/BG) toggle when editing FrameSelected/FramUnselected/FramHighlight
components. Only border color (base) is editable.

**Pros:** No dead toggle. Clean. Matches Zellij's mental model where pane fill is
terminal-derived.
**Cons:** Removes theoretical ability to edit frame background. If Zellij ever uses
that field for something visible, users can't edit it without an update.

### Option 3: Add a dedicated "Pane Background" preview element

Add a `PaneBackground` PreviewElement that maps to TextUnselected (since that's what
controls pane fill). This gives users a separate entry to change pane fill color,
and keeps frame components border-only.

**Pros:** Gives the user explicit control over pane fill (visually and conceptually
clear). Frame = border, Pane BG = fill. No confusion.
**Cons:** Adds another element. TextUnselected already exists as "Text (Unselected)"
in the Status group — would need a different component mapping. Actually, TextUnselected
controls both regular pane text AND pane fill, so it's not clean to separate.

### Option 4: Show a hint/label that clarifies what "BG" means per component

In the color picker status bar, show "frame border" / "frame background" / "pane fill"
instead of just "FG" / "BG" for the current attribute.

**Pros:** Clarifies without changing behavior. Low risk.
**Cons:** Doesn't fix the dead toggle — the BG side of frame components still wouldn't
affect the preview.

### Option 5: Use Frame*.background for canvas BG, keep TextUnselected for pane fill

In `render_zellij_panes`, the canvas background (area between panes) could use
FrameUnselected.background. Not a direct user-facing fix but gives the field a visible
purpose.

**Pros:** Gives the dead field a purpose without breaking pane fill semantics.
**Cons:** Subtle—users may not notice. Canvas bg is a thin strip between panes.

## Recommendation

I'd go with **Option 2** (remove BG toggle for frame components) as the immediate fix —
it's clean, honest, and removes the confusing dead toggle. Then separately consider
**Option 3+5 combined**: add a "Pane Fill" element that maps to a new/appropriate
component, and use Frame*.background for something visible in the preview.

But this is worth discussing — what's the user's mental model for what "pane background"
should be?
