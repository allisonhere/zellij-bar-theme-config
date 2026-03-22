# Zellij Tab Config TUI

A terminal UI application for configuring Zellij tab bar themes.

## Project Location

```
/home/allie/Projects/zellij/zellij-tab-config/zellij-tab-config/
```

## Files

- `src/main.rs` - Entry point
- `src/theme/mod.rs` - Theme data structures (RgbColor, ThemeComponent, Theme, ThemeComponentType)
- `src/config/mod.rs` - ConfigManager for KDL file parsing/saving
- `src/ui/mod.rs` - Main UI logic (preview, color picker)

## Dependencies

```toml
ratatui = { version = "0.30", features = ["all-widgets"] }
crossterm = "0.29"
kdl = "4"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
dirs = "5"
thiserror = "2"
```

## Features

### Navigation
- **Arrow keys** - Navigate between UI elements in the preview
- **Tab** - Toggle between foreground/background color editing
- **c** - Open RGB color picker
- **Enter** - Apply color changes
- **Esc** - Cancel/close picker
- **s** - Open save-as prompt for a named theme
- **l** - Load an existing saved theme
- **q** - Quit

### Color Picker (pik-style)
- **Up/Down** - Select RGB channel to edit
- **Left/Right** - Adjust selected channel value (by 5)
- **Tab** - Switch between fg/bg (remembers state)
- Visual sliders with `█` and `░` characters

### Editable Elements

| Element | Description |
|---------|-------------|
| Tab (Selected) | Selected tab in tab bar |
| Tab (Unselected) | Unselected tabs in tab bar |
| Status Bar | Status bar including "Locked" indicator |
| Pane (Selected) | Selected pane frame |
| Pane (Highlight) | Highlighted pane frame |
| Pane (Unselected) | Unselected pane frame |
| List Item (Selected) | Selected list item |
| List Item (Unselected) | Unselected list item |
| Exit Code (Success) | Success exit code display |
| Exit Code (Error) | Error exit code display |

## Theme Components

Each element has:
- **Foreground (base)** - Primary text color
- **Background** - Background color

Zellij theme format supports additional emphasis levels (0-3) but these are not currently editable in this UI.

## Color Preview

When editing, the preview panel shows:
- Current element name and selected attribute
- Hex color value
- Yellow `◄` indicator on selected element

## Color Picker Preview

Shows styled text samples with:
- Tab name
- Status text
- List item
- Exit code

Uses the other attribute's color as context (e.g., editing FG shows new color on current BG).

## Running

```bash
cd /home/allie/Projects/zellij/zellij-tab-config/zellij-tab-config
cargo run
```

Themes are saved to `~/.config/zellij/themes/` as named `.kdl` files.

## KDL Format

```kdl
themes {
    theme_name {
        ribbon_selected {
            base 255 255 255
            background 80 80 80
            emphasis_0 255 255 255
            emphasis_1 200 200 200
            emphasis_2 150 150 150
            emphasis_3 100 100 100
        }
        // ... more components
    }
}
```

## TODO / Known Issues

- [ ] KDL parsing functions exist but not fully wired up for loading existing themes
- [ ] Could add more color adjustment (shift+arrow for finer control)
- [ ] Could add theme reset functionality
- [ ] Unused code warnings for `ColorAttribute`, some config methods

## Inspiration

- Color picker UI inspired by [pik](https://github.com/immanelg/pik)
- pik uses horizontal sliders with visual progress bars
- Keyboard-driven navigation similar to vim-style editors
