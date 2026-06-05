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
if [ -e /dev/tty ]; then
    # Read from the terminal directly so the prompt pauses even when the
    # script is run as `curl -fsSL ... | sh` (stdin is the pipe, not the tty).
    printf "  Press Enter to install ${GREEN}zellit${RESET} "
    read -r _ < /dev/tty
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
    if [ -e /dev/tty ]; then
        # Read from the terminal directly so the prompt stops and waits even
        # when the script is run as `curl -fsSL ... | sh` (stdin is the pipe).
        printf "  ${DIM}Remove it to keep your system clean? [y/N]${RESET} "
        read -r answer < /dev/tty
        case "$answer" in
            [yY] | [yY][eE][sS])
                rm -f "$OLD"
                printf "  ${GREEN}Removed.${RESET}\n"
                ;;
            *)
                printf "  ${DIM}Kept old binary.${RESET}\n"
                ;;
        esac
    else
        printf "  ${DIM}No terminal available; left old binary in place.${RESET}\n"
    fi
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
