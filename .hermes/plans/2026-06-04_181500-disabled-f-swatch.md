# Plan: Disabled-style "F" swatch for frame-only elements

**Goal:** When a pane element (frame component) has only FG editable (no BG toggle), the F swatch in the field selector should visually indicate "no toggle" — dimmer, no letter label, just a color block.

## Current behavior

For frame elements (PaneSelected, PaneUnselected, PaneHighlight), the field selector currently shows:
```
▏F  Pane (Selected)▕
```
The F is a colored block with contrasting "F" letter — same as non-frame elements even though there's no BG attribute to toggle to.

## Proposed change

For frame elements, show the FG color as a plain colored block — no "F" character, just a space with the FG background color. This makes it visually clear this is a single-attribute element (border only).

Before: `▏F  Pane (Selected)▕`  
After:  `▏█  Pane (Selected)▕`  (plain color block, no F)

For non-frame elements (which have both FG and BG), keep the current `F B` swatches as-is.

## Implementation

In `render_field_selector` in `src/ui/render.rs`:

When `!f.is_frame()` — current behavior (F + B swatches with labels).  
When `f.is_frame()` — render the FG color as a `Span::styled(" ", Style::new().bg(fg))` instead of `Span::styled("F", Style::new().fg(fg_ct).bg(fg))`.

The check `has_bg` already gates the B swatch. The same condition can control whether the F swatch shows a labeled "F" or a plain color block.

## Files

- `src/ui/render.rs` — `render_field_selector` method, ~line 380-396

## Validation

- `cargo build` — must compile clean
- `cargo test` — all 15 tests must pass
