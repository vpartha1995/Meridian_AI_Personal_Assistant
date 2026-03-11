#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# Meridian — Developer Setup Script
# Run once to install all prerequisites.
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
RED='\033[0;31m'
NC='\033[0m' # No Color

header()  { echo -e "\n${BOLD}==> $1${NC}"; }
ok()      { echo -e "${GREEN}✓ $1${NC}";    }
warn()    { echo -e "${YELLOW}⚠ $1${NC}";  }
err()     { echo -e "${RED}✗ $1${NC}"; exit 1; }

header "Meridian Setup"
echo "This script installs all build prerequisites for Meridian."

# ── 1. Node.js (>= 18) ───────────────────────────────────────────────────────
header "Checking Node.js"
if command -v node &>/dev/null; then
  NODE_VER=$(node --version | cut -d. -f1 | tr -d v)
  if [ "$NODE_VER" -ge 18 ]; then
    ok "Node.js $(node --version)"
  else
    warn "Node.js is too old ($(node --version)). Please upgrade to v18+."
  fi
else
  err "Node.js not found. Install from https://nodejs.org"
fi

# ── 2. Rust ───────────────────────────────────────────────────────────────────
header "Checking Rust"
if command -v rustc &>/dev/null; then
  ok "Rust $(rustc --version)"
else
  warn "Rust not found. Installing via rustup…"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
  source "$HOME/.cargo/env"
  ok "Rust installed: $(rustc --version)"
fi

# ── 3. Tauri CLI ──────────────────────────────────────────────────────────────
header "Checking Tauri CLI"
if command -v cargo-tauri &>/dev/null || cargo tauri --version &>/dev/null 2>&1; then
  ok "Tauri CLI available"
else
  warn "Installing Tauri CLI…"
  cargo install tauri-cli --version "^2.0"
  ok "Tauri CLI installed"
fi

# ── 4. OS-specific deps ───────────────────────────────────────────────────────
header "Checking OS dependencies"
OS=$(uname -s)
if [ "$OS" == "Linux" ]; then
  sudo apt-get install -y \
    libwebkit2gtk-4.1-dev libappindicator3-dev librsvg2-dev patchelf \
    libgtk-3-dev libayatana-appindicator3-dev
  ok "Linux dependencies installed"
elif [ "$OS" == "Darwin" ]; then
  ok "macOS — no extra system deps needed (uses system WebKit)"
fi

# ── 5. npm install ────────────────────────────────────────────────────────────
header "Installing npm packages"
npm install
ok "npm packages installed"

# ── 6. Generate icons ────────────────────────────────────────────────────────
header "Generating app icons"
node scripts/generate-icons.mjs || warn "Icon generation skipped — install sharp first: npm install -g sharp-cli"

# ── 7. Ollama check ───────────────────────────────────────────────────────────
header "Checking Ollama (optional, for AI features)"
if command -v ollama &>/dev/null; then
  ok "Ollama found: $(ollama --version 2>&1 | head -1)"
  echo "   Pull the default model with: ollama pull phi3:mini"
else
  warn "Ollama not found. AI summarization will be disabled."
  echo "   Install from: https://ollama.ai"
  echo "   Then run: ollama pull phi3:mini"
fi

header "Setup complete!"
echo "Start development with:  npm run tauri:dev"
echo "Build release with:      npm run tauri:build"
