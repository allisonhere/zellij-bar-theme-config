# Full project copy: repo root → ~/projects/zellit

## Current structure

The `.git` is at the parent level, not inside the Rust crate. The real repo root is:

```
~/Projects/zellij/zellij-tab-config/       ← git repo root
├── deploy.sh                              ← bump version, tag, push, CI trigger
├── install.sh                             ← interactive installer (prebuilt or source)
├── install-binary.sh                      ← download prebuilt binary
├── install-source.sh                      ← git clone + cargo build + install
├── README.md, LICENSE, PROJECT_STATUS.md
├── loader.png, picker.png, pic.png, screen.png
├── .github/                               ← CI workflows
├── .git/
└── zellij-tab-config/                     ← the Rust crate (Cargo.toml, src/)
```

The scripts reference:
- Crate path: `zellij-tab-config/Cargo.toml` (deploy.sh line 52, 121)
- Binary name: `zellij-tab-config` (all install scripts)
- GitHub URL: `github.com/allisonhere/zellij-bar-theme-config` (install.sh, install-source.sh)
- Title: "zellij-bar-theme-config" (deploy.sh line 20)

## Plan

### Step 1: Copy the entire repo root (not just the crate)

```bash
cp -a ~/Projects/zellij/zellij-tab-config ~/projects/zellit
```

This moves everything — scripts, README, CI, git history, images, and the nested crate.

Then delete stale `target/`:

```bash
rm -rf ~/projects/zellit/zellij-tab-config/target
```

### Step 2: Rename the crate subdirectory

```bash
mv ~/projects/zellit/zellij-tab-config ~/projects/zellit/zellit
```

### Step 3: Update script paths

**`deploy.sh`** — crate path checks and Cargo.toml path:
- Line 52: `[ -f "zellij-tab-config/Cargo.toml" ]` → `[ -f "zellit/Cargo.toml" ]`
- Line 121: `CARGO_TOML="zellij-tab-config/Cargo.toml"` → `CARGO_TOML="zellit/Cargo.toml"`
- Line 20: title text "zellij-bar-theme-config" → "zellit"

**`install.sh`** and **`install-source.sh`** — manifest path in git clone:
- Line 74/26: `$TMP/repo/zellij-tab-config/Cargo.toml` → `$TMP/repo/zellit/Cargo.toml`

### Step 4: Update binary name (if desired)

All install scripts use `BIN="zellij-tab-config"`. Optionally change to `BIN="zellit"` and update `Cargo.toml`'s `[[bin]] name`.

### Step 5: Update GitHub URLs

All install scripts reference `github.com/allisonhere/zellij-bar-theme-config`. If you rename the GitHub repo to `zellit`, the redirect handles old URLs — but the scripts reference the old URL directly, so they won't benefit from the redirect.

Update to `github.com/allisonhere/zellit`.

### Step 6: Rename GitHub repo

```bash
cd ~/projects/zellit
gh repo rename zellit
```

Or via GitHub Settings → rename. Old URLs redirect automatically.

### Step 7: Update remote

```bash
cd ~/projects/zellit
git remote set-url origin git@github.com:allisonhere/zellit.git
```

## Files that change

| File | What changes |
|---|---|
| `zellit/Cargo.toml` | package name, binary name (optional) |
| `zellit/src/update.rs` | asset name, User-Agent (optional) |
| `deploy.sh` | crate path checks, Cargo.toml path, title |
| `install.sh` | manifest path, GitHub URL, binary name |
| `install-binary.sh` | binary name |
| `install-source.sh` | manifest path, GitHub URL, binary name |

## Verification

```bash
cd ~/projects/zellit
cargo build  # in the crate subdirectory (or in root if Cargo workspace)
cargo test
```

## Risks

- **Copy the parent, not the child.** If you only copy `zellij-tab-config/`, you lose scripts, CI, README, and git history.
- Scripts hardcode paths to the crate subdirectory — those need updating.
- GitHub rename redirect works for browsers and git clones, but the install scripts hardcode the old URL — update those manually regardless.
