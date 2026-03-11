# ──────────────────────────────────────────────────────────────────────────────
# Meridian Windows Setup Script (PowerShell)
# Run as Administrator for system-wide install, or as user for local install.
# ──────────────────────────────────────────────────────────────────────────────

param(
    [switch]$InstallOllama,
    [switch]$PullModel
)

$ErrorActionPreference = "Stop"
$ProgressPreference    = "SilentlyContinue"

function Write-Header { param($msg) Write-Host "`n==> $msg" -ForegroundColor Cyan  }
function Write-Ok     { param($msg) Write-Host "✓ $msg"    -ForegroundColor Green  }
function Write-Warn   { param($msg) Write-Host "⚠ $msg"    -ForegroundColor Yellow }
function Write-Err    { param($msg) Write-Host "✗ $msg"    -ForegroundColor Red; exit 1 }

Write-Header "Meridian Windows Setup"

# ── 1. Check Node.js ──────────────────────────────────────────────────────────
Write-Header "Checking Node.js"
try {
    $nodeVer = node --version
    Write-Ok "Node.js $nodeVer"
} catch {
    Write-Err "Node.js not found. Install from https://nodejs.org"
}

# ── 2. Check Rust ─────────────────────────────────────────────────────────────
Write-Header "Checking Rust"
try {
    $rustVer = rustc --version
    Write-Ok "Rust: $rustVer"
} catch {
    Write-Warn "Rust not found. Installing rustup..."
    Invoke-WebRequest -Uri "https://win.rustup.rs/x86_64" -OutFile "$env:TEMP\rustup-init.exe"
    & "$env:TEMP\rustup-init.exe" -y --default-toolchain stable
    $env:PATH += ";$env:USERPROFILE\.cargo\bin"
    Write-Ok "Rust installed"
}

# ── 3. Visual C++ Build Tools ─────────────────────────────────────────────────
Write-Header "Checking Visual C++ Build Tools"
$vsWhere = "${env:ProgramFiles(x86)}\Microsoft Visual Studio\Installer\vswhere.exe"
if (Test-Path $vsWhere) {
    Write-Ok "Visual Studio / Build Tools found"
} else {
    Write-Warn "Visual C++ Build Tools not found."
    Write-Warn "Install from: https://visualstudio.microsoft.com/visual-cpp-build-tools/"
    Write-Warn "Select: Desktop development with C++"
}

# ── 4. WebView2 ───────────────────────────────────────────────────────────────
Write-Header "Checking WebView2"
$wv2Key = "HKLM:\SOFTWARE\WOW6432Node\Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}"
if (Test-Path $wv2Key) {
    Write-Ok "WebView2 Runtime found"
} else {
    Write-Warn "WebView2 not found — it will be embedded in the installer (adds ~2MB)"
}

# ── 5. npm install ────────────────────────────────────────────────────────────
Write-Header "Installing npm packages"
npm install
Write-Ok "npm packages installed"

# ── 6. Ollama (optional) ──────────────────────────────────────────────────────
Write-Header "Ollama (AI engine)"
try {
    $ollamaVer = ollama --version 2>&1
    Write-Ok "Ollama: $ollamaVer"
} catch {
    Write-Warn "Ollama not found"
    if ($InstallOllama) {
        Write-Host "Downloading Ollama..."
        $ollamaUrl = "https://ollama.ai/download/OllamaSetup.exe"
        Invoke-WebRequest -Uri $ollamaUrl -OutFile "$env:TEMP\OllamaSetup.exe"
        Start-Process -FilePath "$env:TEMP\OllamaSetup.exe" -Wait
        Write-Ok "Ollama installed"
    } else {
        Write-Warn "Run with -InstallOllama to auto-install, or get it from https://ollama.ai"
    }
}

if ($PullModel) {
    Write-Header "Pulling phi3:mini model (~2.3 GB)"
    ollama pull phi3:mini
    Write-Ok "phi3:mini model ready"
}

Write-Header "Setup complete!"
Write-Host "Start development: npm run tauri:dev" -ForegroundColor Green
Write-Host "Build release:     npm run tauri:build" -ForegroundColor Green
