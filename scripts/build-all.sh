#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# Meridian — Multi-platform Release Builder
# Builds Windows .exe installer and macOS .dmg
# Run on a macOS CI machine with Rust cross-compilation targets.
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

BOLD='\033[1m'
GREEN='\033[0;32m'
YELLOW='\033[0;33m'
NC='\033[0m'

header() { echo -e "\n${BOLD}==> $1${NC}"; }
ok()     { echo -e "${GREEN}✓ $1${NC}"; }

header "Building Meridian for all platforms"

# Ensure node deps
header "Installing dependencies"
npm ci
ok "npm dependencies ready"

# ── macOS Universal Binary (Intel + Apple Silicon) ────────────────────────────
if [ "$(uname -s)" == "Darwin" ]; then
  header "Building macOS Universal Binary"

  # Add both targets
  rustup target add x86_64-apple-darwin
  rustup target add aarch64-apple-darwin

  npm run tauri:build -- --target universal-apple-darwin
  ok "macOS build complete → src-tauri/target/universal-apple-darwin/release/bundle/"
fi

# ── Windows (cross-compile from macOS/Linux, or run on Windows) ──────────────
if [ "${BUILD_WINDOWS:-false}" == "true" ] || [ "$(uname -s)" == "MINGW"* ]; then
  header "Building Windows Installer"

  rustup target add x86_64-pc-windows-msvc 2>/dev/null || true
  npm run tauri:build -- --target x86_64-pc-windows-msvc
  ok "Windows build complete → src-tauri/target/x86_64-pc-windows-msvc/release/bundle/"
fi

header "Build artifacts"
find src-tauri/target -name "*.exe" -o -name "*.dmg" -o -name "*.msi" 2>/dev/null | while read f; do
  size=$(du -sh "$f" 2>/dev/null | cut -f1)
  echo "  [$size] $f"
done

ok "All builds complete!"
