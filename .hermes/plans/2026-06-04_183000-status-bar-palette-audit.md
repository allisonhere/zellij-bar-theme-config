# Plan: Tab bar + bottom status bar palette audit

**User question:** Does Zellij's tab bar and bottom menu-hint bar share the same palette, or have we missed incorporating a bottom bar component?

## Findings from Zellij theme docs

Zellij defines exactly 14 theme UI components (heading from `h4` elements on zellij.dev/documentation/themes.html):

1. `text_unselected`
2. `text_selected`
3. `ribbon_unselected`
4. `ribbon_selected`
5. `table_title`
6. `table_cell_unselected`
7. `table_cell_selected`
8. `list_unselected`
9. `list_selected`
10. `frame_unselected`
11. `frame_selected`
12. `frame_highlight`
13. `exit_code_success`
14. `exit_code_error`
15. `multiplayer_user_colors`

### What we have

Our `ThemeComponentType` enum matches items 1–14 exactly. ✓

### Missing

- **`multiplayer_user_colors`** — an array of colors (not a single component with base/background). Used for distinguishing different users in collaborative sessions. Not currently modeled.

### Bottom bar / status bar

Zellij's bottom status bar is a **plugin** (the `status-bar` plugin alias), not a separate theme component. It consumes the existing theme components — specifically `ribbon_selected` and `ribbon_unselected` for its pill indicators, and `text_unselected` for its background. This matches how our preview renders it — the bottom bar shares the tab bar palette.

**Verdict:** We have NOT missed a bottom bar component. Tab bar and bottom bar correctly share the ribbon palette. The 14 component types we model match Zellij's spec exactly (minus `multiplayer_user_colors`).

## Recommendation

No code changes needed for the bottom bar. Optionally add `multiplayer_user_colors` as a future enhancement.

## Files

None — informational audit only.
