# Fix About Screen Branding

**Date**: 2026-06-04  
**Context**: About screen ASCII art spells "ZELLIJ" but repo is `zellij-tab-config` (a Zellij plugin). Links point to `hermes-agent`, author is `-allie`.

## The Issue

The about screen currently shows:

```
███████╗ ███████╗ ██╗      ██╗      ██╗ ██╗ ██╗
╚══███╔╝ ██╔════╝ ██║      ██║      ██║ ██║ ██║
  ███╔╝  █████╗   ██║      ██║      ██║ ██║ ██║
 ███╔╝   ██╔══╝   ██║      ██║      ██║ ██║ ██║
███████╗ ███████╗ ███████╗ ███████╗ ██║ ██║ ██║
╚══════╝ ╚══════╝ ╚══════╝ ╚══════╝ ╚═╝ ╚═╝ ╚═╝
```

Tagline: "because default themes suck  —allie"  
Links: `hermes-agent` repo (wrong project — this is `zellij-tab-config`)

This is wrong on two levels:
1. ASCII art is for the letter "Z" as in "Zellij" — but the Z logo was copy-pasted from `hermes-agent` about screen concepts. It makes sense as a tribute to Zellij but doesn't represent `zellij-tab-config` specifically.
2. The links pointer to `hermes-agent` repo — should be the actual project repo.

## Options

### Option A: Keep the Z, fix the links

Keep the ZELLIJ ASCII art as a tribute to the Zellij terminal multiplexer (which this tool configures). Replace links with actual project repo.

- `repo: github.com/allie/zellij-tab-config` (or wherever it lives)  
- `issues: github.com/allie/zellij-tab-config/issues`  
- `docs: zellij.dev` (Zellij docs, since this is a Zellij tool)

**Pros**: Z logo is visually distinctive and fun, honors Zellij. Minimal changes.  
**Cons**: People might think the app IS Zellij itself, not a configurator for it.

### Option B: Replace Z with a mini-ASCII logo for the app

Create a small ASCII representation — maybe just "ztc" in block letters or a terminal-window ASCII art frame. Replace all links with actual project repo.

**Pros**: Honest branding, no confusion.  
**Cons**: Less visually striking than the Z block letters. More work to design.

### Option C: Keep Z but add subtitle context

Keep the Z logo but add a subtitle line explaining "Zellij Theme Configurator" above or below the tagline. Fix links.

**Pros**: Retains the cool Z, adds clarity.  
**Cons**: One extra line of text.

### Option D: Full branding overhaul

Redesign the about screen to prominently show "zellij-tab-config" with proper ASCII art, tagline, author credit, and all correct links. Treat it as its own brand rather than borrowing ZELLIJ.

**Pros**: Clean, unambiguous, own identity.  
**Cons**: Most work.

## Recommendation

**Option C** — keep the Z as tribute, add a "Zellij Theme Configurator" subtitle, fix all links to point to the actual `zellij-tab-config` repo. This preserves the visual impact while being honest about what the tool is.

If the repo URL is `github.com/nousresearch/hermes-agent` (the agent running this), discover the actual project repo from `git remote -v`.

## Files

- `src/ui/render.rs` — `render_about_mode()` function (~lines 1857-1990): ASCII logo, tagline, links section

## Verification

- `cargo build` — zero warnings
- `cargo test` — 15/15 green
- Manual: press `A` in preview mode, verify branding makes sense

## Open Question

What is the actual repo URL for zellij-tab-config? Need `git remote -v` to find it.
