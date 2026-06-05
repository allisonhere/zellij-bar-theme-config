#!/usr/bin/env sh
set -e

BIN="zellit"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
LATEST_RELEASE="https://github.com/allisonhere/zellit/releases/latest/download"

# ── Colors ────────────────────────────────────────────────────────────────────
# Use printf so the variables hold real ESC bytes; POSIX `sh` echo does not
# interpret backslash escapes, so embedding them as literals would print garbage.
if [ -t 1 ]; then
    RED=$(printf '\033[0;31m')
    GREEN=$(printf '\033[0;32m')
    YELLOW=$(printf '\033[1;33m')
    CYAN=$(printf '\033[0;36m')
    MAGENTA=$(printf '\033[0;35m')
    DIM=$(printf '\033[2m')
    BOLD=$(printf '\033[1m')
    RESET=$(printf '\033[0m')
else
    RED='' GREEN='' YELLOW='' CYAN='' MAGENTA='' DIM='' BOLD='' RESET=''
fi

OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}_${ARCH}" in
    Linux_x86_64) PREBUILT="${LATEST_RELEASE}/${BIN}-linux-x86_64" ;;
    *)
        echo "Error: no prebuilt binary for ${OS} ${ARCH}." >&2
        echo "Use install-source.sh to build from source." >&2
        exit 1
        ;;
esac

echo ""
echo "  ${CYAN}╔══════════════════════════════════════════╗${RESET}"
echo "  ${CYAN}║${RESET}  ${MAGENTA}${BOLD}zellij-tab-config${RESET} is now called ${GREEN}${BOLD}zellit${RESET}  ${CYAN}║${RESET}"
echo "  ${CYAN}║${RESET}  ${DIM}github.com/allisonhere/zellit${RESET}           ${CYAN}║${RESET}"
echo "  ${CYAN}╚══════════════════════════════════════════╝${RESET}"
echo ""
if [ -t 0 ]; then
    printf "  Press Enter to install ${GREEN}zellit${RESET} "
    read -r _
    echo ""
fi

echo "==> Installing $BIN (prebuilt)"
mkdir -p "$INSTALL_DIR"

if command -v curl >/dev/null 2>&1; then
    curl -fsSL "$PREBUILT" -o "$INSTALL_DIR/$BIN"
elif command -v wget >/dev/null 2>&1; then
    wget -qO "$INSTALL_DIR/$BIN" "$PREBUILT"
else
    echo "Error: curl or wget is required." >&2
    exit 1
fi

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
