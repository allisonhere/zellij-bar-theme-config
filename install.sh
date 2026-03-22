#!/usr/bin/env sh
set -e

REPO="https://github.com/allisonhere/zellij-bar-theme-config"
BIN="zellij-tab-config"
INSTALL_DIR="${INSTALL_DIR:-$HOME/.local/bin}"

echo "==> Installing $BIN"

# Check dependencies
if ! command -v cargo >/dev/null 2>&1; then
    echo "Error: cargo not found. Install Rust from https://rustup.rs" >&2
    exit 1
fi

# Clone to a temp dir, build, install
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT

echo "==> Cloning $REPO"
git clone --depth 1 "$REPO" "$TMP/repo"

echo "==> Building (release)"
cargo build --release --manifest-path "$TMP/repo/zellij-tab-config/Cargo.toml"

echo "==> Installing to $INSTALL_DIR"
mkdir -p "$INSTALL_DIR"
cp "$TMP/repo/zellij-tab-config/target/release/$BIN" "$INSTALL_DIR/$BIN"
chmod +x "$INSTALL_DIR/$BIN"

echo ""
echo "  Installed: $INSTALL_DIR/$BIN"

# Warn if install dir is not in PATH
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
