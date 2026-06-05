# About Screen / Modal Ideas

**Date**: 2026-06-04
**Goal**: Design a cool, polished about screen/modal for zellij-tab-config.

## Context

The app is a Zellij theme editor TUI with a dark midnight-blue aesthetic, magenta accents, sidebar tree nav, and color picker overlay. The about screen should feel at home in this world — not a generic dialog.

Currently there is no about screen. `?` opens the help overlay. The about screen would be triggered by a new keybinding (e.g. `A` or from the help screen).

## Approaches

### Option A: Terminal "Fetch" Style (neofetch-like)

```
┌─────────────────────────────────────────────────┐
│  ╭──────────────────╮  zellij-tab-config v0.5.0 │
│  │ ╔══╗╔══╗ ╔══╗╔══╗│  ─────────────────────── │
│  │ ╚══╗║  ║ ║  ║╚══╗│  Theme editor for Zellij  │
│  │ ╚══╝╚══╝ ╚══╝╚══╝│                            │
│  │   tab   config    │  Terminal   Alacritty     │
│  ╰──────────────────╯  Shell      zsh 5.9       │
│                        OS         CachyOS       │
│                        Themes     12 loaded      │
│                        Config     ~/.config/...  │
│                                                    │
│  Built with ♥ using Ratatui + Rust                  │
│  github.com/nousresearch/hermes-agent               │
└─────────────────────────────────────────────────┘
```

ASCII art "ZT" monogram + system info in fetch layout. Familiar pattern for terminal users.

**Pros**: Terminal-native, recognizable format, easy to implement with text.
**Cons**: Fetch format is overused, might feel generic.

### Option B: Animated Color Showcase

The about screen IS the theme — it cycles through the app's palette colors in a border or background animation, showing off what the editor can do. Pulses through: midnight blue → magenta → violet → teal.

```
┌─ zellij-tab-config ─────────────────────────────┐
│                                                    │
│          ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓           │
│          ▓  Theme Editor for Zellij  ▓              │
│          ▓  v0.5.0                  ▓              │
│          ▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓▓           │
│                                                    │
│   ████████████ palette colors scroll here ██████   │
│                                                    │
│  Press any key to close                            │
└────────────────────────────────────────────────────┘
```

**Pros**: Shows off the app's purpose (color editing), visually impressive.
**Cons**: More complex to implement (needs timer/animation integration), might be distracting.

### Option C: Minimal Dark Card

Clean, minimal, matches the app's own UI aesthetic perfectly. A single bordered card with a subtle glow effect.

```
┌─────────────────────────────────────────────────┐
│                                                    │
│               zellij-tab-config                    │
│                                                    │
│                  v0.5.0                            │
│           Theme editor for Zellij                  │
│                                                    │
│     ─────────────────────────────────────          │
│                                                    │
│     Author   Nous Research / Hermes Agent          │
│     License  MIT                                   │
│     Built    Rust + Ratatui                        │
│                                                    │
│     github.com/nousresearch/hermes-agent           │
│                                                    │
│                  press any key                     │
│                                                    │
└─────────────────────────────────────────────────┘
```

**Pros**: Fits the app perfectly, trivial to implement (~30 lines), consistent.
**Cons**: Not "cool" — just clean. Might be too minimal.

### Option D: ASCII Art Zellij Logo + Splash

Recreate a stylized Zellij logo in ASCII/box-drawing characters. The Zellij logo is a geometric "Z" made of blocks — very doable in terminal chars.

```
┌─────────────────────────────────────────────────┐
│                                                    │
│          ██████╗   ███████╗                       │
│          ╚════██╗  ╚══███╔╝                       │
│           █████╔╝    ███╔╝                        │
│           ╚═══██╗   ███╔╝                         │
│          ██████╔╝  ███████╗                       │
│          ╚═════╝   ╚══════╝                       │
│                                                    │
│          zellij-tab-config  v0.5.0                  │
│          "because default themes suck"              │
│                                                    │
│          ◇  TabBar   ◇  Panes                      │
│          ◇  Content  ◇  Status                      │
│              14 elements · 4 groups                  │
│                                                    │
│          github/nousresearch/hermes-agent            │
│                                                    │
└─────────────────────────────────────────────────┘
```

**Pros**: Recognizable Zellij branding, splash-screen feel, fun tagline.
**Cons**: ASCII logo takes effort to design, could look bad at small sizes.

### Option E: Interactive Credits Easter Egg

A playful twist: the "about" info reveals itself as a scrolling credits crawl (like movie credits), with each line fading in. Or a "matrix rain" of theme color names.

```
┌─ about ─────────────────────────────────────────┐
│                                                    │
│              ╔══════════════════╗                  │
│              ║  ZELLIJ TAB CONF ║                  │
│              ╚══════════════════╝                  │
│                                                    │
│         starring                                  │
│    ───────────────────                            │
│    Ratatui ......... rendering engine             │
│    Crossterm ....... terminal backend             │
│    Tokio ........... async runtime   (jk, no tokio)│
│    Rust ............ the language                 │
│                                                    │
│         directed by                               │
│    ───────────────────                            │
│    Nous Research                                 │
│                                                    │
│    no LLMs were harmed in the making              │
│    of this color picker                           │
│                                                    │
└────────────────────────────────────────────────────┘
```

**Pros**: Personality! Memorable, fun, shows app character. The "movie credits" format is unusual in a TUI.
**Cons**: Might be too playful for some, takes more code to animate.

## Recommendation

I'd combine **Option D (Zellij logo) + Option C (minimal card)** — a clean dark card with a small ASCII Zellij "Z" mark at the top, version, tagline, and links. No animation (keeps it simple), but the ASCII art gives it visual weight. Add one playful touch: the "because default themes suck" tagline or a cheeky credits line.

**~40 lines of code**, all in `render.rs` as a new `render_about_overlay` method, triggered by `A` in Preview mode.

## Implementation Sketch

1. Add `InputMode::About` variant
2. Add `A` key handler in Preview mode → `app.input_mode = InputMode::About`
3. Add `render_about_overlay()` — centered modal, dark card, ASCII logo, version, links
4. Esc / any key → back to Preview
5. Add `?` help entry: `A — About`
6. Build + test
