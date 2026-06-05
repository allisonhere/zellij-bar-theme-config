#!/usr/bin/env sh
set -e

REPO="https://github.com/allisonhere/zellit"
BIN="zellit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

# ── Colors ────────────────────────────────────────────────────────────────────
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
DIM='\033[2m'
BOLD='\033[1m'
RESET='\033[0m'

echo ""
echo "  ${CYAN}╔══════════════════════════════════════════╗${RESET}"
echo "  ${CYAN}║${RESET}  ${MAGENTA}${BOLD}zellij-tab-config${RESET} is now called ${GREEN}${BOLD}zellit${RESET}  ${CYAN}║${RESET}"
echo "  ${CYAN}║${RESET}  ${DIM}github.com/allisonhere/zellit${RESET}             ${CYAN}║${RESET}"
echo "  ${CYAN}╚══════════════════════════════════════════╝${RESET}"
echo ""
if [ -t 0 ]; then
    printf "  Press Enter to install ${GREEN}zellit${RESET} "
    read -r _
    echo ""
fi

echo "==> Installing $BIN (from source)"

if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found. Install Rust from https://rustup.rs" >&2
    exit 1
fi
if ! command -v git >/dev/null 2>&1; then
    echo "Error: git not found." >&2
    exit 1
fi

TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> Cloning $REPO"
git clone --depth 1 "$REPO" "$TMP/repo"

echo "==> Building (release)"
cargo build --release --manifest-path "$TMP/repo/Cargo.toml"

mkdir -p "$INSTALL_DIR"
cp "$TMP/repo/target/release/$BIN" "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

# ── Old binary cleanup ────────────────────────────────────────────────────────
OLD="$INSTALL_DIR/zellij-tab-config"
if [ -f "$OLD" ]; then
    echo ""
    printf "  ${YELLOW}Old binary found:${RESET} %s\n" "$OLD"
    printf "  ${DIM}Remove it to keep your system clean? [y/N]${RESET} "
    read -r answer
    [ "$answer" = "y" ] || [ "$answer" = "Y" ] && rm -f "$OLD" && echo "  ${GREEN}Removed.${RESET}"
fi

echo ""
echo "  Installed: $INSTALL_DIR/$BIN"

case ":${PATH}:" in
    *":$INSTALL_DIR:"*) ;;
    *)
        echo ""
        echo "  Warning: $INSTALL_DIR is not in your PATH."
        echo "  Add this to your shell config:"
        echo "    export PATH=\"\$PATH:$INSTALL_DIR\""
        ;;
esac

echo ""
echo "  Run: $BIN"
