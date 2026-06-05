# zellit

> **Previously known as `zellij-bar-theme-config` / `zellij-tab-config`.** Renamed to `zellit` starting with v0.7.0. The old names redirect to this repo. I got tired of remembering and typing zellij-tab-config ;)

A terminal UI theme editor for [Zellij](https://zellij.dev). Browse, edit, save, and apply Zellij themes with live preview.

## Features

- **Live preview** — see your theme applied to a simulated Zellij session as you edit
- **Sidebar tree** — all 14 theme elements organized by group (TabBar, Panes, Content, Status) with color swatches
- **Fuzzy search** — press `/` to find and jump to any element by name
- **Color picker** — RGB sliders or HSL color field with hex input, copy/paste, and undo
- **Auto-positioning** — picker opens opposite the selected preview element so you always see your changes
- **Theme loader** — browse built-in themes, filter by type, search, rename/delete saved themes
- **Self-update** — press `U` to download the latest release on Linux x86_64

## Keybindings

### Preview

| Key          | Action                                      |
|-------------|---------------------------------------------|
| `↑/j  ↓/k`  | Navigate all preview elements               |
| `1`–`4`     | Jump to group (TabBar / Panes / Content / Status) |
| `/`         | Fuzzy search and jump to any element        |
| `Tab`       | Toggle FG / BG (pane borders: FG only)      |
| `c`         | Open color picker for selected color        |
| `y`         | Yank (copy) current color                   |
| `p`         | Paste yanked color                          |
| `u`         | Undo last color change                      |
| `s`         | Save theme as… (prompts for name)           |
| `l`         | Open theme loader                           |
| `a`         | Apply current theme to Zellij config        |
| `U`         | Install latest release (Linux x86_64)       |
| `?`         | Toggle help screen                          |
| `q / Esc`   | Quit                                        |

### Color Picker

| Key             | Action                                    |
|-----------------|-------------------------------------------|
| `Tab / Shift+Tab`| Move focus between controls               |
| `m`             | Switch RGB sliders / HSL field            |
| `f`             | Toggle FG / BG (non-frame elements only)  |
| `← → ↑ ↓`       | Nudge the focused control                 |
| `Shift / Alt`   | Coarse / fine nudging                     |
| `Enter`         | Edit focused value field or confirm       |
| `#`             | Jump to hex field editing                 |
| `y`             | Yank (copy) current color                 |
| `Mouse drag`    | Drag in HSL field or lightness slider     |
| `Esc`           | Cancel                                    |

### Theme Loader

| Key          | Action                                    |
|-------------|-------------------------------------------|
| `type`       | Search and filter themes                  |
| `↑ ↓`        | Navigate themes                           |
| `Enter`      | Load selected theme into editor           |
| `b`          | Filter built-in themes                    |
| `s`          | Filter saved themes                       |
| `a`          | Apply selected theme to Zellij            |
| `r`          | Rename selected saved theme               |
| `d`          | Delete selected saved theme               |
| `Esc`        | Clear search or cancel                    |

## Build

```bash
cargo build --release
```

The binary is `target/release/zellit`.

## License

MIT
