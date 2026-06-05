# Rename project: external links and repo strategy

## Current state

- GitHub remote: `git@github.com:allisonhere/zellij-bar-theme-config.git`
- Local directory: `zellij-tab-config`
- Desired local directory: `zellit`
- External links presumably point to `github.com/allisonhere/zellij-bar-theme-config/...`

## Key facts

**GitHub automatically handles renames.** When you rename a repo on GitHub (Settings → rename), all old URLs get a 301 permanent redirect to the new ones:

- `github.com/allisonhere/zellij-bar-theme-config` → `github.com/allisonhere/zellit`
- `github.com/allisonhere/zellij-bar-theme-config/issues/5` → `github.com/allisonhere/zellit/issues/5`
- Raw URLs, clone URLs, git remotes — all redirect transparently

This works for repos, not for organizations. If you were changing `allisonhere` to something else, you'd be screwed — but changing the repo name within the same account is seamless.

## Approaches

### A) Rename the GitHub repo, update local

GitHub repo: `zellij-bar-theme-config` → `zellit`
Local dir: `zellij-tab-config` → `zellit`

```bash
cd ~/Projects/zellij/zellij-tab-config
cp -a zellij-tab-config ~/projects/zellit

# Update remote URL
cd ~/projects/zellit
git remote set-url origin git@github.com:allisonhere/zellit.git

# Rename on GitHub
gh repo rename zellit  # or via Settings page
```

**Pros**: Single repo, history preserved, redirects work for all old links.
**Cons**: None — this is the intended GitHub workflow.

### B) Rename GitHub repo but don't touch the old one

Same as A, but you leave the old directory as-is and work from the new copy exclusively.

**Pros**: Clean break.
**Cons**: Same as A.

### C) Create a new repo, archive the old one

Create `allisonhere/zellit` as a fresh repo. Archive `zellij-bar-theme-config` with a README saying "moved to zellit".

**Pros**: Old links get a clear message about where to find the new project.
**Cons**: Old links don't redirect *to* the new repo — they show an archived page. Visitors have to click through.

### D) Do nothing on GitHub, just change locally

Keep the GitHub repo name as `zellij-bar-theme-config`. Only change your local directory name.

```bash
mv zellij-tab-config ~/projects/zellit
```

Remote stays `origin git@github.com:allisonhere/zellij-bar-theme-config.git`.

**Pros**: Zero impact on external links. No GitHub changes needed.
**Cons**: The directory name doesn't match the repo name (but it already doesn't — your repo is `zellij-bar-theme-config` and your dir is `zellij-tab-config`, so you're already in this boat).

## Recommendation

**Go with A.** GitHub renames are a one-click operation, redirects are automatic and permanent, and you keep all issues/PRs/stars/history. Old links Just Work. Then update the Cargo.toml and binary name if you want to fully commit to `zellit`.

## What to update in the project (if going with A)

| What | Old value | New value |
|---|---|---|
| Cargo.toml package name | `zellij_tab_config` | `zellit` |
| Cargo.toml binary name | `zellij-tab-config` | `zellit` |
| update.rs asset name | `zellij-tab-config-linux-x86_64` | `zellit-linux-x86_64` |
| update.rs User-Agent | `zellij-tab-config` | `zellit` |
| Local directory | `zellij-tab-config` | `zellit` |
| GitHub repo | `zellij-bar-theme-config` | `zellit` |

But the rename itself doesn't require any of this — you can change the directory and repo names independently of the binary name.
