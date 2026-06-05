#!/usr/bin/env sh
set -e

REPO="https://github.com/allisonhere/zellit"
BIN="zellit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
LATEST_RELEASE="https://github.com/allisonhere/zellit/releases/latest/download"

# ── Detect platform ──────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}_${ARCH}" in
    Linux_x86_64)  PREBUILT="${LATEST_RELEASE}/${BIN}-linux-x86_64" ;;
    *)             PREBUILT="" ;;
esac

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

echo "==> Installing $BIN"
echo ""

# ── Ask build vs prebuilt ────────────────────────────────────────────────────
if [ -n "$PREBUILT" ]; then
    printf "  [1] Download prebuilt binary (fast, Linux x86_64)\n"
    printf "  [2] Build from source      (requires cargo)\n"
    printf "\n  Choice [1]: "
    if [ -t 0 ]; then
        read -r choice
    else
        read -r choice </dev/tty 2>/dev/null || choice="1"
    fi
    choice="${choice:-1}"
else
    echo "  No prebuilt binary available for ${OS} ${ARCH}."
    echo "  Building from source (requires cargo)."
    choice="2"
fi

echo ""

# ── Install ──────────────────────────────────────────────────────────────────
mkdir -p "$INSTALL_DIR"

if [ "$choice" = "1" ] && [ -n "$PREBUILT" ]; then
    # Download prebuilt
    if command -v curl >/dev/null 2>&1; then
        echo "==> Downloading binary"
        curl -fsSL "$PREBUILT" -o "$INSTALL_DIR/$BIN"
    elif command -v wget >/dev/null 2>&1; then
        echo "==> Downloading binary"
        wget -qO "$INSTALL_DIR/$BIN" "$PREBUILT"
    else
        echo "Error: curl or wget is required to download the binary." >&2
        exit 1
    fi
    chmod +x "$INSTALL_DIR/$BIN"
else
    # Build from source
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

    cp "$TMP/repo/target/release/$BIN" "$INSTALL_DIR/$BIN"
    chmod +x "$INSTALL_DIR/$BIN"
fi

# ── Old binary cleanup ────────────────────────────────────────────────────────
OLD="$INSTALL_DIR/zellij-tab-config"
if [ -f "$OLD" ]; then
    echo ""
    printf "  ${YELLOW}Old binary found:${RESET} %s\n" "$OLD"
    printf "  ${DIM}Remove it to keep your system clean? [y/N]${RESET} "
    read -r answer
    [ "$answer" = "y" ] || [ "$answer" = "Y" ] && rm -f "$OLD" && echo "  ${GREEN}Removed.${RESET}"
fi

# ── Done ─────────────────────────────────────────────────────────────────────
echo ""
echo "  Installed: $INSTALL_DIR/$BIN"

case ":$PATH:" in
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
