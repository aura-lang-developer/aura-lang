#!/usr/bin/env bash
# ==============================================================================
# 🌟 Aura Language Official One-Line Installer
# Usage: curl -fsSL https://raw.githubusercontent.com/mrojasb2000/aura-lang/main/install.sh | bash
# ==============================================================================

set -e

RESET='\033[0m'
BOLD='\033[1m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[0;33m'

echo -e "${PURPLE}${BOLD}"
echo "╔══════════════════════════════════════════════════════════════╗"
echo "║                  🌟 Installing Aura Lang 🌟                  ║"
echo "║        Ergonomic Systems Language with Zero Runtime         ║"
echo "╚══════════════════════════════════════════════════════════════╝"
echo -e "${RESET}"

REPO="mrojasb2000/aura-lang"
AURA_DIR="${AURA_INSTALL_DIR:-$HOME/.aura}"
BIN_DIR="$AURA_DIR/bin"

# 1. Detect OS & Architecture
OS="$(uname -s)"
ARCH="$(uname -m)"

case "$OS" in
    Darwin)
        TARGET_OS="apple-darwin"
        ;;
    Linux)
        TARGET_OS="unknown-linux-gnu"
        ;;
    *)
        echo -e "${RED}Error: Unsupported operating system '$OS'.${RESET}"
        exit 1
        ;;
esac

case "$ARCH" in
    x86_64|amd64)
        TARGET_ARCH="x86_64"
        ;;
    arm64|aarch64)
        TARGET_ARCH="aarch64"
        ;;
    *)
        echo -e "${RED}Error: Unsupported architecture '$ARCH'.${RESET}"
        exit 1
        ;;
esac

TARGET_TRIPLE="${TARGET_ARCH}-${TARGET_OS}"
echo -e "${CYAN}➜ Detected platform:${RESET} ${TARGET_TRIPLE}"

mkdir -p "$BIN_DIR"

# 2. Download latest release or fallback to cargo build
RELEASE_URL="https://github.com/${REPO}/releases/latest/download/aura-${TARGET_TRIPLE}.tar.gz"

echo -e "${CYAN}➜ Fetching Aura toolchain...${RESET}"

if curl --output /dev/null --silent --head --fail "$RELEASE_URL"; then
    echo -e "${GREEN}✓ Downloading precompiled binary package...${RESET}"
    curl -fsSL "$RELEASE_URL" | tar -xz -C "$BIN_DIR"
elif command -v cargo >/dev/null 2>&1; then
    echo -e "${YELLOW}ℹ Prebuilt binary not found for ${TARGET_TRIPLE}. Compiling latest release with cargo...${RESET}"
    cargo install --git "https://github.com/${REPO}.git" --root "$AURA_DIR"
else
    echo -e "${RED}Error: No prebuilt binary found and 'cargo' is not installed.${RESET}"
    echo -e "Please install Rust (https://rustup.rs) or download binaries from:"
    echo -e "https://github.com/${REPO}/releases"
    exit 1
fi

chmod +x "$BIN_DIR"/* 2>/dev/null || true

# 3. Configure Shell PATH
SHELL_NAME="$(basename "$SHELL")"
PROFILE=""

if [ "$SHELL_NAME" = "zsh" ]; then
    PROFILE="$HOME/.zshrc"
elif [ "$SHELL_NAME" = "bash" ]; then
    if [ -f "$HOME/.bashrc" ]; then
        PROFILE="$HOME/.bashrc"
    else
        PROFILE="$HOME/.bash_profile"
    fi
fi

echo ""
echo -e "${GREEN}${BOLD}✓ Aura Language installed successfully!${RESET}"
echo -e "  Location: ${BIN_DIR}"
echo ""

if [[ ":$PATH:" != *":$BIN_DIR:"* ]]; then
    if [ -n "$PROFILE" ] && [ -f "$PROFILE" ]; then
        if ! grep -q "AURA_DIR" "$PROFILE" 2>/dev/null; then
            echo "export PATH=\"$BIN_DIR:\$PATH\"" >> "$PROFILE"
            echo -e "${CYAN}➜ Added '$BIN_DIR' to $PROFILE${RESET}"
        fi
        echo -e "${YELLOW}To start using Aura immediately in this terminal session, run:${RESET}"
        echo -e "${BOLD}  source $PROFILE${RESET}"
    else
        echo -e "${YELLOW}Please add the following to your shell profile:${RESET}"
        echo -e "${BOLD}  export PATH=\"$BIN_DIR:\$PATH\"${RESET}"
    fi
fi

echo ""
echo -e "Try your first command:"
echo -e "${BOLD}  aurac --version${RESET}"
echo -e "${BOLD}  aurac playground${RESET}"
echo ""
