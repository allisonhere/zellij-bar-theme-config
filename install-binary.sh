#!/usr/bin/env sh
set -e

BIN="zellij-tab-config"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
LATEST_RELEASE="https://github.com/allisonhere/zellij-bar-theme-config/releases/latest/download"

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
