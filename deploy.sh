#!/usr/bin/env bash
set -e

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
WHITE='\033[1;37m'
DIM='\033[2m'
BOLD='\033[1m'
RESET='\033[0m'

# ── Helpers ───────────────────────────────────────────────────────────────────
header() {
    echo ""
    echo -e "${CYAN}╔══════════════════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}║${RESET}  ${MAGENTA}${BOLD}⚡ zellij-bar-theme-config  deploy${RESET}               ${CYAN}║${RESET}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════╝${RESET}"
    echo ""
}

step() { echo -e "${BLUE}▶${RESET}  ${BOLD}$1${RESET}"; }
ok()   { echo -e "${GREEN}✓${RESET}  $1"; }
warn() { echo -e "${YELLOW}⚠${RESET}  $1"; }
die()  { echo -e "${RED}✗${RESET}  ${BOLD}$1${RESET}"; exit 1; }
dim()  { echo -e "${DIM}$1${RESET}"; }

# ── Bump version helper ───────────────────────────────────────────────────────
latest_tag() {
    git tag --sort=-version:refname | grep -E '^v[0-9]+\.[0-9]+\.[0-9]+$' | head -1
}

bump_version() {
    local tag="$1" part="$2"
    local ver="${tag#v}"
    local major minor patch
    IFS='.' read -r major minor patch <<< "$ver"
    case "$part" in
        major) echo "v$((major+1)).0.0" ;;
        minor) echo "v${major}.$((minor+1)).0" ;;
        patch) echo "v${major}.${minor}.$((patch+1))" ;;
    esac
}

# ── Main ──────────────────────────────────────────────────────────────────────
header

# Check we're in the right place
[ -f "zellij-tab-config/Cargo.toml" ] || die "Run this from the repo root"

# ── Git status ────────────────────────────────────────────────────────────────
step "Checking working tree"
if git diff --quiet && git diff --cached --quiet; then
    UNCOMMITTED=false
    warn "Nothing staged or modified — skipping commit"
else
    UNCOMMITTED=true
    echo ""
    git status --short | sed "s/^/   ${DIM}/"
    echo -e "${RESET}"
fi

# ── Commit ────────────────────────────────────────────────────────────────────
if [ "$UNCOMMITTED" = true ]; then
    echo -e "${WHITE}Commit message:${RESET} "
    read -r MSG
    [ -z "$MSG" ] && die "Commit message cannot be empty"

    git add -A
    git commit -m "$MSG"
    ok "Committed: ${DIM}${MSG}${RESET}"
fi

# ── Push ──────────────────────────────────────────────────────────────────────
step "Pushing to origin"
git push
ok "Pushed"

# ── Release? ──────────────────────────────────────────────────────────────────
echo ""
echo -e "${WHITE}Create a release?${RESET} ${DIM}[y/N]${RESET} "
read -r DO_RELEASE
[[ "$DO_RELEASE" =~ ^[Yy]$ ]] || { echo -e "${DIM}Skipping release.${RESET}"; echo ""; exit 0; }

# ── Version ───────────────────────────────────────────────────────────────────
CURRENT="$(latest_tag)"
if [ -z "$CURRENT" ]; then
    CURRENT="v0.0.0"
fi

echo ""
echo -e "  Current tag: ${YELLOW}${CURRENT}${RESET}"
echo ""
echo -e "  ${BOLD}[1]${RESET} patch  →  ${GREEN}$(bump_version "$CURRENT" patch)${RESET}"
echo -e "  ${BOLD}[2]${RESET} minor  →  ${GREEN}$(bump_version "$CURRENT" minor)${RESET}"
echo -e "  ${BOLD}[3]${RESET} major  →  ${GREEN}$(bump_version "$CURRENT" major)${RESET}"
echo -e "  ${BOLD}[4]${RESET} custom"
echo ""
printf "  Choice [1]: "
read -r CHOICE
CHOICE="${CHOICE:-1}"

case "$CHOICE" in
    1) NEW_TAG="$(bump_version "$CURRENT" patch)" ;;
    2) NEW_TAG="$(bump_version "$CURRENT" minor)" ;;
    3) NEW_TAG="$(bump_version "$CURRENT" major)" ;;
    4)
        printf "  Tag (e.g. v1.2.3): "
        read -r NEW_TAG
        [[ "$NEW_TAG" =~ ^v[0-9]+\.[0-9]+\.[0-9]+$ ]] || die "Invalid tag format"
        ;;
    *) die "Invalid choice" ;;
esac

# ── Tag & push ────────────────────────────────────────────────────────────────
echo ""
step "Tagging ${NEW_TAG}"
git tag "$NEW_TAG"
git push origin "$NEW_TAG"
ok "Tagged and pushed ${YELLOW}${NEW_TAG}${RESET}"

# ── CI link ───────────────────────────────────────────────────────────────────
echo ""
REPO="$(git remote get-url origin | sed 's/.*github.com[:/]//' | sed 's/\.git$//')"
echo -e "${CYAN}╔══════════════════════════════════════════════════╗${RESET}"
echo -e "${CYAN}║${RESET}  ${GREEN}${BOLD}Release triggered!${RESET}                               ${CYAN}║${RESET}"
echo -e "${CYAN}║${RESET}  ${DIM}https://github.com/${REPO}/actions${RESET}"
echo -e "${CYAN}╚══════════════════════════════════════════════════╝${RESET}"
echo ""
