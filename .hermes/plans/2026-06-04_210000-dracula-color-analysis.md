# Dracula Theme Color Analysis

**Date**: 2026-06-04  
**Context**: User asks why Dracula theme preview shows green as dominant and light purple as secondary.

## The Dracula Palette (Official)

Dracula (the famous theme) uses these colors:

| Role       | Hex     | RGB              | Note         |
|-----------|---------|------------------|--------------|
| Background | #282A36 | (40, 42, 54)     | dark purple-gray |
| Green     | #50FA7B | (80, 250, 123)   | accent / selection |
| Pink      | #FF79C6 | (255, 121, 198)  | emphasis / highlights |
| Purple    | #BD93F9 | (189, 147, 249)  | the "real" purple |
| Orange    | #FFB86C | (255, 184, 108)  | warm accent |
| Cyan      | #8BE9FD | (139, 233, 253)  | cool accent |

## How Zellij Maps It

The bundled `dracula.kdl` maps Dracula colors to Zellij components:

- `frame_selected.base` = **#50FA7B (green)** ← the green you see on the selected pane border
- `frame_highlight.base` = **#FFB86C (orange)** ← highlight border
- `frame_unselected.base` = (63, 63, 63) ← dark gray, not from Dracula palette
- `ribbon_selected.background` = **#50FA7B (green)** ← the green on the selected tab
- `text_selected.background` = (40, 42, 54) ← Dracula background

The "light purple" you're seeing is `emphasis_3 = #FF79C6 (pink)` — used across all components as the tertiary emphasis color.

## Where's the Real Purple?

Dracula's signature **#BD93F9 (purple)** is **completely absent** from the zellij theme. The theme uses:
- Green (#50FA7B) for selection accents
- Pink (#FF79C6) for emphasis
- Orange (#FFB86C) for highlights
- Cyan (#8BE9FD) for cooler accents

None of the 14 components use purple (#BD93F9) at all.

## Options

### Option A: Educate — this IS Dracula

Dracula green (#50FA7B) is the canonical Dracula accent color. The theme is correct. No code change needed — just explain that Dracula's "dominant" color has always been green for active/selected elements.

**Pros**: Zero work, factually correct.  
**Cons**: Doesn't address user's visual preference. They may want purple instead.

### Option B: Swap green → purple

Replace `frame_selected.base` and `ribbon_selected.background` from green (#50FA7B) to purple (#BD93F9). This makes the selected pane border and selected tab purple instead of green.

| Component | Current | Proposed |
|-----------|---------|----------|
| frame_selected.base | 80 250 123 (green) | 189 147 249 (purple) |
| ribbon_selected.background | 80 250 123 (green) | 189 147 249 (purple) |
| table_title.base | 80 250 123 (green) | 189 147 249 (purple) |
| exit_code_success.base | 80 250 123 (green) | 189 147 249 (purple) |

**Pros**: Purple-dominant Dracula, visually closer to what the user expects.  
**Cons**: Devates from canonical Dracula palette. Green for success/selection is the Dracula standard.

### Option C: Create a "Dracula Purple" variant

Keep dracula.kdl as-is, create a new `dracula-purple.kdl` bundled theme with the purple swap. User can switch between them.

**Pros**: Both palettes available, no breakage.  
**Cons**: Yet another bundled theme to maintain.

## Recommendation

Option A — this is just how Dracula works. The green is the official Dracula accent color (#50FA7B). The "light purple" is pink (#FF79C6), not true purple (#BD93F9). If the user wants a purple-dominant variant, Option B is a 4-line KDL edit.

## Files

- `src/bundled_themes/dracula.kdl` — the 14-component theme definition
- `src/config/mod.rs` — KDL parser and color resolution
- `src/ui/render.rs` — preview rendering (no bug, just showing what's defined)
