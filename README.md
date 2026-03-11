# Meridian — AI Personal Productivity Assistant

> Runs 100% locally. No cloud. No subscriptions. No data leaves your machine.

[![Build Meridian](https://github.com/vpartha1995/Meridian_AI_Personal_Assistant/actions/workflows/build.yml/badge.svg)](https://github.com/vpartha1995/Meridian_AI_Personal_Assistant/actions/workflows/build.yml)

---

## Download

| Platform | Installer | Requirements |
|----------|-----------|--------------|
| **Windows 10/11** | [Meridian_1.0.0_x64-setup.exe](https://github.com/vpartha1995/Meridian_AI_Personal_Assistant/releases/latest/download/Meridian_1.0.0_x64-setup.exe) | 64-bit, 4 GB RAM |
| **macOS 13+** | [Meridian_1.0.0_universal.dmg](https://github.com/vpartha1995/Meridian_AI_Personal_Assistant/releases/latest/download/Meridian_1.0.0_universal.dmg) | Apple Silicon or Intel |

**Latest release:** https://github.com/vpartha1995/Meridian_AI_Personal_Assistant/releases/latest

### Install in 3 steps

**Windows**
1. Download `Meridian_1.0.0_x64-setup.exe`
2. Double-click and follow the installer (no admin rights needed)
3. Launch Meridian from the Start menu

**macOS**
1. Download `Meridian_1.0.0_universal.dmg`
2. Open the DMG → drag **Meridian** to **Applications**
3. Launch from Launchpad or Spotlight

> **First launch:** Meridian will guide you through a one-time AI model download (~2.3 GB).
> No other software needs to be installed — Ollama is bundled inside the installer.

---

## What is Meridian?

Meridian is a desktop productivity assistant that:

- **Summarises your day** — pulls tasks, emails, meetings into a single morning briefing
- **Manages tasks** — create, prioritise, and track tasks with due dates and reminders
- **Connects your tools** — Gmail, Slack, Jira, Outlook, Zoom, Google Chat
- **AI everywhere** — ask questions, draft replies, summarise threads — all via local AI
- **Quick-capture overlay** — press a hotkey anywhere to jot a task or ask AI without leaving your current app

Everything runs locally using **Ollama + Phi-3 Mini**. No API keys, no monthly fees, no data sent anywhere.

---

## Features

| Feature | Details |
|---------|---------|
| Dashboard | Daily briefing — top tasks, upcoming meetings, priority emails |
| Task Manager | Full CRUD, priorities, due dates, reminders, AI tagging |
| AI Chat | Streaming chat with local LLM, context-aware |
| Quick Capture | Global hotkey overlay — tasks + AI assistant |
| Integrations | Gmail, Slack, Jira, Outlook, Zoom, Google Chat |
| Privacy | AES-256-GCM encrypted local DB, tokens in OS keychain |
| Offline | Works 100% offline after model download |

---

## Architecture

```
meridian/
├── src/                    React 18 + TypeScript + Tailwind frontend
│   ├── components/         Dashboard, Tasks, Overlay, Settings, Onboarding
│   ├── store/              Zustand state management
│   └── lib/                Tauri IPC wrappers
└── src-tauri/              Rust backend (Tauri 2.0)
    ├── src/ai/             Ollama sidecar management + streaming model pull
    ├── src/integrations/   OAuth + API clients (Gmail, Slack, Jira, ...)
    ├── src/storage/        SQLite (sqlx) + AES-256-GCM + OS keychain
    ├── src/tasks/          Task engine + reminder scheduler
    └── src/commands/       Tauri IPC command handlers
```

**Tech stack:** Tauri 2.0 · React 18 · TypeScript · Rust · SQLite · Ollama · Tailwind CSS · Framer Motion · Zustand

---

## Building from Source

See [BUILD.md](BUILD.md) for full instructions.

Quick start:
```bash
# Install deps
npm install

# Dev mode
npm run tauri:dev

# Production build (auto-downloads Ollama, compiles, packages installer)
npm run tauri:build
```

---

## Contributing

Issues and PRs welcome. Please open an issue before submitting large changes.

---

## Privacy

- All data stored locally in your OS app data directory
- OAuth tokens stored in Windows Credential Manager / macOS Keychain
- AI model runs on your hardware — no inference sent to any server
- No telemetry, no analytics, no cloud sync of any kind

---

## License

MIT
