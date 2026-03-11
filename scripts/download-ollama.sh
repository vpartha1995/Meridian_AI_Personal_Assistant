#!/usr/bin/env bash
# ──────────────────────────────────────────────────────────────────────────────
# Download the Ollama binary for bundling as a Tauri sidecar (macOS).
#
# Run ONCE before `npm run tauri:build`:
#   bash scripts/download-ollama.sh
#
# Tauri sidecar naming convention:
#   binaries/ollama-aarch64-apple-darwin      (Apple Silicon)
#   binaries/ollama-x86_64-apple-darwin       (Intel)
# ──────────────────────────────────────────────────────────────────────────────
set -euo pipefail

OLLAMA_VERSION="v0.3.12"
BASE_URL="https://github.com/ollama/ollama/releases/download/${OLLAMA_VERSION}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BIN_DIR="${SCRIPT_DIR}/../src-tauri/binaries"
mkdir -p "${BIN_DIR}"

download_binary() {
    local arch="$1"          # arm64 | amd64
    local triple="$2"        # aarch64-apple-darwin | x86_64-apple-darwin
    local src_name="ollama-darwin-${arch}"
    local dst_file="${BIN_DIR}/ollama-${triple}"

    if [ -f "${dst_file}" ]; then
        local size_mb
        size_mb=$(du -m "${dst_file}" | cut -f1)
        echo "[SKIP] ollama-${triple} already exists (${size_mb} MB)"
        return 0
    fi

    local url="${BASE_URL}/${src_name}"
    echo "Downloading Ollama ${OLLAMA_VERSION} (${arch})..."
    echo "  From: ${url}"
    echo "  To:   ${dst_file}"

    if command -v curl &>/dev/null; then
        curl -fL --progress-bar "${url}" -o "${dst_file}"
    else
        wget -q --show-progress "${url}" -O "${dst_file}"
    fi

    chmod +x "${dst_file}"
    local size_mb
    size_mb=$(du -m "${dst_file}" | cut -f1)
    echo "[OK] ${size_mb} MB -> ${dst_file}"
}

# Download for both architectures (needed for universal binary builds).
download_binary "arm64"  "aarch64-apple-darwin"
download_binary "amd64"  "x86_64-apple-darwin"

echo ""
echo "Both Ollama binaries ready in src-tauri/binaries/"
echo "Next step: npm run tauri:build -- --target universal-apple-darwin"
