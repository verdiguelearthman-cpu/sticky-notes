# StickyNotes

A lightweight desktop sticky notes app for Windows. Each note is an independent always-on-top window that floats on your screen — like real sticky notes.

## Features

- **Multi-window** — each note is a separate OS window, freely draggable
- **Always on top** — notes stay visible above other apps (toggleable per note)
- **Transparency linked to focus** — blurred notes fade to 30% opacity, hover to restore
- **6 color themes** — yellow, pink, blue, green, purple, orange
- **Rich text editing** — with auto-save and HTML sanitization (DOMPurify)
- **Timed reminders** — set date/time reminders with repeat (daily, weekly, weekday)
- **System tray** — right-click: New Note, Show All, Quit; double-click: create note
- **Local persistence** — notes, positions, and sizes saved as JSON, restored on startup
- **Minimal footprint** — notes skip the taskbar, app lives in the system tray

## Tech Stack

| Layer | Technology |
|---|---|
| Framework | Tauri 2.x (Rust backend) |
| Frontend | React 18 + TypeScript |
| State | Zustand |
| Build | Vite |
| Notifications | tauri-plugin-notification |

## Prerequisites

- [Node.js](https://nodejs.org/) >= 18
- [Rust](https://rustup.rs/) stable toolchain
- Windows: MSVC build tools (via Visual Studio Build Tools)

## Getting Started

```bash
# Clone
git clone https://github.com/verdiguelearthman-cpu/sticky-notes.git
cd sticky-notes

# Install dependencies
npm install

# Run in development mode
npm run tauri dev

# Build for production (outputs MSI/NSIS installer)
npm run tauri build
```

## Project Structure

```
src/                    # React frontend
  components/
    StickyNote.tsx      # Core note component (drag, edit, opacity)
    ColorPicker.tsx     # Color selection dropdown
    ReminderModal.tsx   # Reminder configuration UI
    ErrorBoundary.tsx   # Global error boundary
  store/
    noteStore.ts        # Zustand state + Tauri IPC
  types/
    index.ts            # TypeScript types + color config

src-tauri/              # Rust backend
  src/
    lib.rs              # Data model, persistence, window mgmt, tray, reminder scheduler
    main.rs             # Binary entry point
  capabilities/
    default.json        # Tauri permissions
  Cargo.toml            # Rust dependencies
  tauri.conf.json       # Tauri configuration
```

## Architecture

Each sticky note runs as an independent Tauri webview window:

- **Manager window** (hidden) — manages system tray and app lifecycle
- **Note windows** — borderless, transparent, always-on-top, skip taskbar
- **Reminder scheduler** — background thread polls every 30s, fires OS notifications
- **Data** — stored as `notes.json` in the OS app data directory

## License

MIT
