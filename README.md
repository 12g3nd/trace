# Drift

A TODO.txt replacement. Orbit Noir utility.

Fast capture. Local storage. Keyboard-first. Extremely small.

## What this is

A personal task capture tool that preserves the speed of writing into a text file while adding safe persistence, structure, and recovery. Not a productivity app.

## Stack

- **Tauri 2** — native Windows shell, ~5MB binary
- **Svelte 5** — reactive UI, compiles away
- **SQLite** — local database, human-inspectable
- **TypeScript** — type-safe frontend
- **Orbit Noir** — personal design system (shared with Lacquer, launcher, terminal)

## Capture syntax

```
Message Michael about RC ~ rc **
```

- Text before `~` → task description
- Text after `~` → context (lowercase tag)
- Trailing `*` → priority (1–5)

All parts are optional. Typing plain text and pressing Enter works.

## Keyboard shortcuts

| Key | Action |
|---|---|
| `↑` / `↓` or `j` / `k` | Move selection |
| `Space` | Complete / uncomplete task |
| `Enter` | Edit selected task |
| `Tab` | Focus capture input |
| `/` | Search |
| `Alt+↑` / `Alt+↓` | Reorder task within group |
| `Ctrl+1` / `2` / `3` | Move to Now / Later / Someday |
| `Delete` | Delete task (undoable) |
| `Ctrl+Z` | Undo last action |
| `Escape` | Close overlay / cancel edit / deselect |

## Development

```powershell
# Prerequisites: Rust, Node.js 20+, pnpm
pnpm install
pnpm tauri dev
```

## Build

```powershell
pnpm tauri build
```

Produces a `.msi` installer and standalone `.exe` in `src-tauri/target/release/bundle/`.

## Data location

SQLite database is stored in the Tauri app data directory:

```
%APPDATA%/com.orbitnoir.drift/drift.db
```

The database is a standard SQLite file and can be opened with any SQLite tool.

## Design system

See [ORBIT_NOIR_INTEGRATION.md](./ORBIT_NOIR_INTEGRATION.md) for how this utility inherits from the Orbit Noir design system.
