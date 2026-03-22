# zellij-bar-theme-config

A terminal UI for creating, editing, and applying [Zellij](https://zellij.dev) themes. Live preview of your changes across all UI components — tab bar, panes, status bar, tables, lists — before writing anything to disk.

## Features

- **Live preview** — see every color change reflected instantly across a full Zellij-layout mockup
- **Per-component theming** — control foreground, background, and emphasis colors for each UI element independently
- **RGB color picker** — slider-based editor with hex preview
- **Load existing themes** — reads `.kdl` files from `~/.config/zellij/themes/`, including standard Zellij palette themes (`fg`, `bg`, `black`, `red`, …)
- **Save themes** — writes to the Zellij themes directory in the correct KDL format
- **Apply to Zellij** — sets `theme "<name>"` in `~/.config/zellij/config.kdl` so Zellij picks it up on next launch
- **Help overlay** — press `?` for a full keybinding reference

## Installation

### One-line install

```sh
curl -fsSL https://raw.githubusercontent.com/allisonhere/zellij-bar-theme-config/main/install.sh | sh
```

Installs to `~/.local/bin` by default. Override with `INSTALL_DIR`:

```sh
curl -fsSL https://raw.githubusercontent.com/allisonhere/zellij-bar-theme-config/main/install.sh | INSTALL_DIR=/usr/local/bin sh
```

Requires `git` and `cargo` ([rustup.rs](https://rustup.rs)).

### Manual install

```sh
git clone https://github.com/allisonhere/zellij-bar-theme-config.git
cd zellij-bar-theme-config/zellij-tab-config
cargo build --release
cp target/release/zellij-tab-config ~/.local/bin/
```

## Usage

```
zellij-tab-config
```

The app opens a full-terminal preview of a Zellij layout. Use the keyboard to navigate and edit.

### Keybindings

| Key | Action |
|-----|--------|
| `↑ ↓ ← →` | Navigate between preview elements |
| `Tab` | Toggle foreground / background editing |
| `c` | Open color picker for the selected color |
| `s` | Save theme as… (prompts for a name) |
| `l` | Open theme loader |
| `a` | Apply current theme to `~/.config/zellij/config.kdl` |
| `?` | Toggle help overlay |
| `q` / `Esc` | Quit |

**Color picker:**

| Key | Action |
|-----|--------|
| `↑ ↓` | Select R / G / B channel |
| `← →` | Adjust value by 5 |
| `Shift + ← →` | Adjust value by 1 |
| `PgUp / PgDn` | Adjust value by 25 |
| `Enter` | Confirm |
| `Esc` | Cancel |

**Theme loader:**

| Key | Action |
|-----|--------|
| `↑ ↓` | Navigate themes |
| `Enter` / `a` | Load selected theme |
| `Esc` | Cancel |

## Theme format

Themes are saved to `~/.config/zellij/themes/<name>.kdl` in this tool's per-component format:

```kdl
themes {
    my-theme {
        ribbon_selected {
            base 30 30 46
            background 137 180 250
            emphasis_0 255 255 255
            emphasis_1 200 200 200
            emphasis_2 150 150 150
            emphasis_3 100 100 100
        }
        // text_unselected, ribbon_unselected, frame_selected, … etc.
    }
}
```

Standard Zellij palette themes (those using `fg`, `bg`, `black`, `red`, … keys) are also supported — the loader maps palette colors to components automatically.

## Themeable components

| Component | What it styles |
|-----------|---------------|
| `text_unselected` / `text_selected` | General text / status bar |
| `ribbon_unselected` / `ribbon_selected` | Tab bar tabs |
| `frame_unselected` / `frame_selected` / `frame_highlight` | Pane borders |
| `table_title` / `table_cell_unselected` / `table_cell_selected` | Table widgets |
| `list_unselected` / `list_selected` | List widgets |
| `exit_code_success` / `exit_code_error` | Exit status indicators |

## Requirements

- Rust 2021 edition (stable)
- A terminal with true color support
