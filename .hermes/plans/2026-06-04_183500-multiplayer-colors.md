# Plan: Elaborate on `multiplayer_user_colors`

## What it is

`multiplayer_user_colors` is a Zellij theme section for collaborative (multiplayer) sessions. It assigns a distinct color to each user who attaches to a session, ensuring all participants see the same color assignments — ordered by attach order (player_1 gets first color, player_2 second, etc.).

It's the ONLY theme section that differs structurally from the other 14 components. Format:

```kdl
multiplayer_user_colors {
    player_1 255 0 255
    player_2 0 217 227
    player_3 0 0 0
    player_4 255 230 0
    player_5 0 229 229
    player_6 0 0 0
    player_7 255 53 94
    player_8 0 0 0
    player_9 0 0 0
    player_10 0 0 0
}
```

No `base`/`background`/`emphasis_0-3` — just 10 flat RGB values keyed by player number.

## How it's different from other components

| Feature | Standard component (e.g., ribbon_selected) | multiplayer_user_colors |
|---------|-------------------------------------------|------------------------|
| Structure | `base`, `background`, `emphasis_0-3` | `player_1`…`player_10` |
| Values | 6 x RgbColor | 10 x RgbColor |
| Editable | FG/BG toggle + all emphasis levels | 10 flat colors |
| Preview-relevant | Yes | No (no visual preview for collab mode) |
| Fits ThemeComponent struct | Yes | No |

## What it would take to support

### Data model

Option A: Store as `[RgbColor; 10]` on `Theme` struct (separate from `components: HashMap<ThemeComponentType, ThemeComponent>`).

Option B: Add `MultiplayerUserColors` to `ThemeComponentType`, make it a special case everywhere it's accessed. This is ugly because it doesn't have base/background — every accessor would need a special case.

Option A is cleaner. In `src/theme/mod.rs`:

```rust
pub struct Theme {
    pub name: String,
    pub components: HashMap<ThemeComponentType, ThemeComponent>,
    pub player_colors: [RgbColor; 10],  // NEW
}
```

### Parsing (src/config/mod.rs)

In `parse_theme_kdl()`, handle `"multiplayer_user_colors"` as a special child node:
- Parse each `player_N` entry as an RGB triplet
- Store in `Theme.player_colors`

### Serialization (src/config/mod.rs)

In `theme_to_kdl()`, emit the `multiplayer_user_colors` block after the standard components.

### ThemeComponentType

Leave `ThemeComponentType` unchanged — `multiplayer_user_colors` is NOT a component type, it's a separate struct field on `Theme`.

### Preview / UI

No preview changes — there's nothing to render visually for collab user colors. Could expose it as a read-only info section in the theme loader detail panel, or skip entirely.

### Tests

- Parse a theme KDL with `multiplayer_user_colors` → round-trip through save → verify colors survive
- Default theme should default to something reasonable (e.g., a preset palette matching Zellij's defaults)

## Priority

**Low.** This is a niche feature for collaborative Zellij sessions. Most users will never see or need it. It doesn't affect the theme preview or any currently editable component. Worth adding for completeness but not urgent.
