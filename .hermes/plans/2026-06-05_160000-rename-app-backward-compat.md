# Rename app: binary, package, repo — with backward compat

## Goal

Rename the application from `zellij-tab-config` / `zellij-bar-theme-config` to `zellit`:
- Cargo package name: `zellij_tab_config` → `zellit`
- Binary name: `zellij-tab-config` → `zellit`
- GitHub org/repo in code: `allisonhere/zellij-bar-theme-config` → `allisonhere/zellit`
- README explains the rename
- **Old versions (v0.6.0 and below) can still self-update**

## How self-update works (critical for backward compat)

Old binary (v0.6.0) does two things:
1. **Version check**: GET `api.github.com/repos/allisonhere/zellij-bar-theme-config/releases/latest`
2. **Download**: GET `github.com/allisonhere/zellij-bar-theme-config/releases/download/{tag}/zellij-tab-config-linux-x86_64`

Step 1 is handled by GitHub's 301 redirect on repo rename — already works.
Step 2 requires the **old asset name** to still exist in new releases.

## The backward compat strategy

Publish **both** asset names in each release:

| Asset name | Who downloads it |
|---|---|
| `zellit-linux-x86_64` | New versions (post-rename) |
| `zellij-tab-config-linux-x86_64` | Old versions (v0.6.0 and below) |

Both are byte-identical copies of the same binary. Old users find the old asset name via the old download URL (redirected), get the new binary, and now they're on the new version that knows the new name.

## Step-by-step

### Step 1: Update Cargo.toml

```toml
# Before
name = "zellij_tab_config"
[[bin]]
name = "zellij-tab-config"

# After
name = "zellit"
[[bin]]
name = "zellit"
```

### Step 2: Update src/update.rs

- Line 4: `REPO` → `"allisonhere/zellit"`
- Line 25: asset name → `"zellit-linux-x86_64"`
- Line 48, 73: User-Agent → `"zellit"`
- Test at line 114: expected asset name → `"zellit-linux-x86_64"`

### Step 3: Update CI workflow (.github/workflows/release.yml)

Line 29: build output is now `zellit` (from Cargo.toml rename)
Line 34: upload **both** files:

```yaml
      - name: Rename binaries
        run: |
          cp target/x86_64-unknown-linux-musl/release/zellit zellit-linux-x86_64
          cp target/x86_64-unknown-linux-musl/release/zellit zellij-tab-config-linux-x86_64   # backward compat

      - name: Upload release binaries
        uses: softprops/action-gh-release@v2
        with:
          files: |
            zellit-linux-x86_64
            zellij-tab-config-linux-x86_64
```

The `zellij-tab-config-linux-x86_64` file is a byte-identical copy of `zellit-linux-x86_64` — old versions find it by the name they expect and update successfully.

### Step 4: Update about screen links (src/ui/render.rs)

Line 1961, 1968: `zellij-bar-theme-config` → `zellit`

### Step 5: Update install scripts

- `install.sh`, `install-source.sh`, `install-binary.sh`: change `REPO` URLs to `allisonhere/zellit`
- `install.sh`, `install-source.sh`: change `BIN` to `zellit`
- `deploy.sh` line 20: title text → `zellit`

### Step 6: Update README.md

Explain the rename — "Previously known as zellij-bar-theme-config / zellij-tab-config. Renamed to zellit with v0.7.0."

### Step 7: Verify

```bash
cargo test        # 15 tests, update test now expects new asset name
cargo build       # binary is now `target/debug/zellit`
cargo run         # runs as zellit
```

## Files that change

| File | Changes |
|---|---|
| `Cargo.toml` | package name, binary name |
| `src/update.rs` | REPO, asset name, User-Agent, test assertion |
| `src/ui/render.rs` | About screen GitHub + issues URLs |
| `.github/workflows/release.yml` | binary source path, upload both asset names |
| `install.sh` | REPO, LATEST_RELEASE URLs, BIN name |
| `install-source.sh` | REPO URL, BIN name |
| `install-binary.sh` | LATEST_RELEASE URL, BIN name |
| `deploy.sh` | title text |
| `README.md` | explain rename |

## Backward compat verification

Simulate the old binary's update path:

```
# Old binary version check (redirected by GitHub)
curl -s https://api.github.com/repos/allisonhere/zellij-bar-theme-config/releases/latest
# → returns zellit's latest release ✓

# Old binary download (redirected by GitHub, asset name still exists)
curl -sL https://github.com/allisonhere/zellij-bar-theme-config/releases/download/v0.7.0/zellij-tab-config-linux-x86_64
# → returns the zellit binary ✓
```

## Risks

- **The backward compat asset MUST be published in every release until you're comfortable dropping it.** If a release goes out with only `zellit-linux-x86_64` and no `zellij-tab-config-linux-x86_64`, old versions cannot update.
- Old users who never update will always use the redirects — that's fine, they still work.
- Once all users are on > v0.6.0, the `zellij-tab-config-linux-x86_64` asset can be dropped from future releases.
