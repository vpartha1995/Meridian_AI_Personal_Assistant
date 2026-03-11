# Meridian — Build Guide

## Prerequisites

| Tool | Version | Install |
|------|---------|---------|
| Node.js | ≥ 18.x | https://nodejs.org |
| Rust | ≥ 1.77 (stable) | https://rustup.rs |
| Tauri CLI | ≥ 2.0 | `cargo install tauri-cli --version "^2"` |
| **Windows only** | Visual C++ Build Tools | [VS Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) |
| **macOS only** | Xcode Command Line Tools | `xcode-select --install` |

## Quick Start

```bash
# 1. Clone / enter project
cd meridian

# 2. Install JS dependencies
npm install

# 3. Generate app icons (Windows: use PowerShell script)
# Windows:
powershell -ExecutionPolicy Bypass -File scripts\create-icons.ps1
# macOS/Linux:
# npm install -g @tauri-apps/cli && npx tauri icon public/meridian-icon.svg

# 4. Start development server
npm run tauri:dev
```

## Building Installers

The Ollama AI engine is **automatically downloaded and bundled** by the build script.
No manual download steps — one command does everything.

### Windows `.exe` (NSIS installer — fully self-contained)
```powershell
# Single command: downloads Ollama, compiles, packages the installer
npm run tauri:build
# Output: src-tauri\target\release\bundle\nsis\Meridian_1.0.0_x64-setup.exe
```

### macOS `.dmg` (Universal Binary — Intel + Apple Silicon)
```bash
# One-time Rust target setup
rustup target add x86_64-apple-darwin aarch64-apple-darwin

# Single command: downloads both Ollama arches, compiles, packages the DMG
npm run tauri:build:macos
# Output: src-tauri/target/universal-apple-darwin/release/bundle/dmg/Meridian_1.0.0_universal.dmg
```

> The first build downloads ~100 MB (Ollama binary).
> Subsequent builds skip the download — cached in `src-tauri/binaries/`.

## Setting Up AI (Local LLM)

Meridian bundles the **Ollama** daemon inside the installer — no separate installation
needed. On first launch a setup screen lets users download the **phi3:mini** model
(2.3 GB, one-time) directly from within the app.

```
First launch → Onboarding → "Local AI Setup" → click "Download Phi-3 Mini Now"
```

The model download streams real-time progress. The app works without AI (basic
summaries only) until the model download completes.

**Optional: larger model** (requires 8+ GB RAM):
```bash
# After installing Meridian, open a terminal:
ollama pull mistral:7b
# Then change the model in Settings → AI
```

## Connecting Integrations

Each integration requires an OAuth app registration:

| Service | Where to create app | Scopes needed |
|---------|---------------------|---------------|
| Gmail | [Google Cloud Console](https://console.cloud.google.com) → APIs & Services → Credentials | `gmail.readonly`, `userinfo.email` |
| Slack | [Slack API](https://api.slack.com/apps) → Create App | `channels:history`, `search:read` |
| Jira | [Atlassian Developer](https://developer.atlassian.com/console/myapps/) | `read:jira-work`, `read:jira-user` |
| Outlook | [Azure Portal](https://portal.azure.com) → App registrations | `Mail.Read`, `User.Read` |
| Zoom | [Zoom Marketplace](https://marketplace.zoom.us) → Build App | `meeting:read` |
| Google Chat | [Google Cloud Console](https://console.cloud.google.com) | `chat.messages.readonly` |

Set OAuth client IDs via environment variables before building:
```bash
export GMAIL_CLIENT_ID="xxx.apps.googleusercontent.com"
export SLACK_CLIENT_ID="xxx"
export SLACK_CLIENT_SECRET="xxx"
# ... etc.
```

Or configure them in Settings → Integrations within the app.

## Environment Variables

```bash
# Required for each integration (set before building or at runtime)
GMAIL_CLIENT_ID=
SLACK_CLIENT_ID=
SLACK_CLIENT_SECRET=
JIRA_CLIENT_ID=
JIRA_CLIENT_SECRET=
OUTLOOK_CLIENT_ID=
ZOOM_CLIENT_ID=
ZOOM_CLIENT_SECRET=
GCHAT_CLIENT_ID=

# For CI signing (Windows)
TAURI_SIGNING_PRIVATE_KEY=
TAURI_SIGNING_PRIVATE_KEY_PASSWORD=

# For macOS notarization
APPLE_CERTIFICATE=
APPLE_CERTIFICATE_PASSWORD=
APPLE_SIGNING_IDENTITY=
APPLE_ID=
APPLE_PASSWORD=
APPLE_TEAM_ID=
```

## Architecture Overview

```
meridian/
├── src/                    React 18 + TypeScript frontend
│   ├── components/         UI components (Dashboard, Overlay, Settings, Tasks)
│   ├── store/              Zustand state management
│   └── lib/                Tauri IPC wrappers and utilities
└── src-tauri/              Rust backend
    └── src/
        ├── ai/             Ollama integration + AI summarization
        ├── integrations/   OAuth + API clients for all platforms
        ├── storage/        SQLite (encrypted) + OS keychain
        ├── tasks/          Task management + reminder engine
        ├── notifications/  Native OS notifications
        └── commands/       Tauri IPC command handlers
```

## Data & Privacy

- All data is stored **locally** in your OS app data directory
- Tokens stored in **OS keychain** (Windows Credential Manager / macOS Keychain)
- Database uses **AES-256-GCM** field-level encryption
- AI runs **100% locally** via Ollama — no data sent to any server
- No telemetry, no analytics, no cloud sync

## Development Notes

```bash
# Run tests
cargo test --manifest-path src-tauri/Cargo.toml

# Check Rust lints
cargo clippy --manifest-path src-tauri/Cargo.toml

# TypeScript type check
npm run build
```
