#!/usr/bin/env sh
set -e

REPO="https://github.com/allisonhere/zellij-bar-theme-config"
BIN="zellij-tab-config"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"
LATEST_RELEASE="https://github.com/allisonhere/zellij-bar-theme-config/releases/latest/download"

# ── Detect platform ──────────────────────────────────────────────────────────
OS="$(uname -s)"
ARCH="$(uname -m)"

case "${OS}_${ARCH}" in
    Linux_x86_64)  PREBUILT="${LATEST_RELEASE}/${BIN}-linux-x86_64" ;;
    *)             PREBUILT="" ;;
esac

echo "==> Installing $BIN"
echo ""

# ── Ask build vs prebuilt ────────────────────────────────────────────────────
if [ -n "$PREBUILT" ]; then
    printf "  [1] Download prebuilt binary (fast, Linux x86_64)\n"
    printf "  [2] Build from source      (requires cargo)\n"
    printf "\n  Choice [1]: "
    read -r choice </dev/tty
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
    cargo build --release --manifest-path "$TMP/repo/zellij-tab-config/Cargo.toml"

    cp "$TMP/repo/zellij-tab-config/target/release/$BIN" "$INSTALL_DIR/$BIN"
    chmod +x "$INSTALL_DIR/$BIN"
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
