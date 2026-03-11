# ──────────────────────────────────────────────────────────────────────────────
# Download the Ollama binary for bundling as a Tauri sidecar (Windows).
#
# Run ONCE before `npm run tauri:build`:
#   powershell -ExecutionPolicy Bypass -File scripts\download-ollama.ps1
#
# Tauri sidecar naming convention:
#   src-tauri/binaries/ollama-x86_64-pc-windows-msvc.exe
# ──────────────────────────────────────────────────────────────────────────────

$ErrorActionPreference = "Stop"
$ProgressPreference    = "SilentlyContinue"

# Force TLS 1.2 (required for GitHub on older PS versions)
[Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12

$ScriptDir  = Split-Path -Parent $MyInvocation.MyCommand.Path
$BinDir     = Join-Path $ScriptDir "..\src-tauri\binaries"
$DestFile   = Join-Path $BinDir "ollama-x86_64-pc-windows-msvc.exe"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# ── Already downloaded? ────────────────────────────────────────────────────────
if (Test-Path $DestFile) {
    $SizeMB = [math]::Round((Get-Item $DestFile).Length / 1MB, 1)
    Write-Host "[SKIP] ollama-x86_64-pc-windows-msvc.exe already present (${SizeMB} MB)"
    Write-Host "       Delete it and re-run to refresh."
    exit 0
}

# ── Helper: download with curl.exe (Win10 1803+) or WebClient fallback ─────────
function Download-File {
    param([string]$Url, [string]$Out)
    Write-Host "  Downloading: $Url"
    $curlExe = "$env:SystemRoot\System32\curl.exe"
    if (Test-Path $curlExe) {
        & $curlExe -fsSL --retry 3 --retry-delay 2 -o $Out $Url
        if ($LASTEXITCODE -ne 0) { throw "curl.exe failed (exit $LASTEXITCODE)" }
    } else {
        # Fallback: .NET WebClient (slower progress but reliable)
        (New-Object System.Net.WebClient).DownloadFile($Url, $Out)
    }
}

# ── 1. Try to reuse a locally installed ollama.exe ─────────────────────────────
$LocalPaths = @(
    "$env:LOCALAPPDATA\Programs\Ollama\ollama.exe",
    "$env:ProgramFiles\Ollama\ollama.exe",
    (Get-Command ollama -ErrorAction SilentlyContinue)?.Source
) | Where-Object { $_ -and (Test-Path $_) }

if ($LocalPaths.Count -gt 0) {
    $Src    = $LocalPaths[0]
    $SizeMB = [math]::Round((Get-Item $Src).Length / 1MB, 1)
    Write-Host "[LOCAL] Found Ollama at: $Src (${SizeMB} MB)"
    Write-Host "  Copying to sidecar directory..."
    Copy-Item $Src $DestFile -Force
    Write-Host "[OK] Copied -> $DestFile"
    Write-Host ""
    Write-Host "Next step: npm run tauri:build"
    exit 0
}

# ── 2. Query GitHub API for the latest Ollama release ─────────────────────────
Write-Host "Querying GitHub for the latest Ollama release..."
$ApiHeaders = @{ "User-Agent" = "MeridianBuild/1.0"; "Accept" = "application/vnd.github.v3+json" }
try {
    $Release = Invoke-RestMethod -Uri "https://api.github.com/repos/ollama/ollama/releases/latest" `
                                 -Headers $ApiHeaders
} catch {
    throw "GitHub API request failed: $_`nCheck your internet connection and try again."
}

$Version = $Release.tag_name
Write-Host "  Latest Ollama: $Version"

# ── 3. Find the right Windows asset ───────────────────────────────────────────
# Prefer a standalone zip/exe over the full installer.
$Asset = $Release.assets |
    Where-Object { $_.name -match "windows" -and $_.name -match "amd64" } |
    Sort-Object { if ($_.name -match "\.zip$") { 0 } else { 1 } } |
    Select-Object -First 1

if (-not $Asset) {
    # Fall back to OllamaSetup.exe
    $Asset = $Release.assets | Where-Object { $_.name -eq "OllamaSetup.exe" } | Select-Object -First 1
}

if (-not $Asset) {
    Write-Host ""
    Write-Host "Available release assets:"
    $Release.assets | ForEach-Object { Write-Host "  - $($_.name)" }
    throw "Could not find a Windows binary in Ollama $Version release."
}

Write-Host "  Asset: $($Asset.name)"
$TempFile = Join-Path $env:TEMP $Asset.name

# ── 4. Download the asset ─────────────────────────────────────────────────────
Download-File -Url $Asset.browser_download_url -Out $TempFile

# ── 5. Extract or copy to destination ────────────────────────────────────────
if ($Asset.name -match "\.zip$") {
    Write-Host "  Extracting ollama.exe from ZIP..."
    $ExtractDir = Join-Path $env:TEMP "ollama_extract_$([guid]::NewGuid().ToString('N').Substring(0,8))"
    Expand-Archive -Path $TempFile -DestinationPath $ExtractDir -Force

    $InnerExe = Get-ChildItem -Recurse -Filter "ollama.exe" $ExtractDir | Select-Object -First 1
    if (-not $InnerExe) { throw "ollama.exe not found inside the ZIP archive." }

    Copy-Item $InnerExe.FullName $DestFile -Force
    Remove-Item $ExtractDir -Recurse -Force
} else {
    # It's a standalone .exe (or the installer — copy as-is for the sidecar)
    Copy-Item $TempFile $DestFile -Force
}

Remove-Item $TempFile -Force -ErrorAction SilentlyContinue

$SizeMB = [math]::Round((Get-Item $DestFile).Length / 1MB, 1)
Write-Host "[OK] ollama-x86_64-pc-windows-msvc.exe (${SizeMB} MB) -> $DestFile"
Write-Host ""
Write-Host "Next step: npm run tauri:build"
