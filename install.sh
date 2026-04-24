#!/bin/bash
set -euo pipefail

INSTALL_DIR="${1:-$HOME/.local/bin}"
BINARY="newt"

if ! command -v cargo &>/dev/null; then
    # rustup installs to ~/.cargo/env but may not be in the current shell's PATH
    if [ -f "$HOME/.cargo/env" ]; then
        # shellcheck source=/dev/null
        source "$HOME/.cargo/env"
    else
        echo "error: cargo not found — install Rust first: https://rustup.rs"
        exit 1
    fi
fi

echo "Building $BINARY (release)..."
cargo build --release

mkdir -p "$INSTALL_DIR"
cp "target/release/$BINARY" "$INSTALL_DIR/$BINARY"
chmod +x "$INSTALL_DIR/$BINARY"

echo "Installed: $INSTALL_DIR/$BINARY"

if ! echo "$PATH" | tr ':' '\n' | grep -qx "$INSTALL_DIR"; then
    echo ""
    echo "Note: $INSTALL_DIR is not in your PATH."
    echo "Add this to your shell rc file:"
    echo "  export PATH=\"$INSTALL_DIR:\$PATH\""
fi
