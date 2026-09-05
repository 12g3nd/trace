# Trace

A personal TODO.txt replacement for Windows 11.

I built Trace to replace the plain `TODO.txt` scratchpad I kept open in Notepad. It keeps the instant, zero-friction feel of typing a line of text into a text file, but adds local SQLite storage, keyboard reordering, priority tags, and a global summon hotkey.

It is tailored for my personal machine and the Orbit Noir desktop theme (matching Lacquer and my terminal setup).

Examples:
<img width="301" height="165" alt="sidecar" src="https://github.com/user-attachments/assets/67a9c91a-ba23-4e4d-80f9-291594467e40" />
<img width="198.5" height="302.5" alt="todo" src="https://github.com/user-attachments/assets/3904349e-af9a-43c9-ae61-94ecfe1caeba" />

---

## Orbit Sidecar

Trace also runs a compact **Orbit Sidecar** in the bottom-left taskbar plane of the primary monitor. It remains available when the main task window is hidden and when Trace starts through autostart with `--minimized`, without adding a taskbar, Alt+Tab, or system-tray entry. It follows the normal taskbar z-order, so it does not overlay full-screen apps. The full Trace window is a single-instance, always-on-top utility palette by default, so summoning it remains immediate and does not add another taskbar or Alt+Tab presence; its pin control can temporarily release or restore topmost behavior.

The Sidecar provides:

- Local launch/focus controls for **LocalSend**, **Codex**, and **Claude** using their installed Windows app identities. Open Codex and Claude windows are removed from the Windows taskbar; their Sidecar buttons receive a blue active marker instead.
- A quiet **Trace** doorway that summons the main window and focuses its normal capture workflow.
- A switchable **Media** bay for the current Windows system media session, with album art, title, play/pause, and next when supported. Hovering it intentionally opens a larger, non-activating controller with artist metadata and previous/play-pause/next controls; clicking either player's non-control surface focuses the source app.
- A minimal **Load** bay showing native Windows memory use and CPU load; click it to open Task Manager.

The selected Trace/Media/Load bay persists locally. Use the quiet chevrons or the mouse wheel over the bay to cycle it. Media is read through one shared Windows GSMTC refresh path, while Load uses lightweight native system APIs and only polls while visible. The Sidecar and media popover intentionally never query or display TODO contents, task metadata, or task counts.

Right-click Sidecar for **Open Trace**, **Re-anchor Sidecar**, **Start with Windows**, and **Quit Trace**. Trace has no tray icon and does not modify or reserve the Windows taskbar.

---

## How I use it

Press **`Win+Shift+T`** (or **`Alt+Shift+T`**) anywhere on Windows. The window pops up with the cursor already focused. Type a task, press Enter, and press Escape to dismiss.

```text
Message X about RC tomorrow ~ rc **
```

- **Task text**: Everything before `~`
- **Due date**: Parsed naturally (`today`, `tomorrow`, `in 3 days`, `friday`, `due:2026-09-01`)
- **Context tag**: After `~` (normalized to lowercase)
- **Priority**: Trailing asterisks `*` through `*****` (1 to 5)

Plain text with no special syntax also works.

Double-click a task (or select it and press Enter) to edit its text and optional web link. Inline edits autosave 650ms after the latest input and flush before blur, hide, close, switching tasks, or quitting, so there is no separate save ritual. Linked tasks show one quiet vector link control that opens through the system default application; links are included in JSON and CSV exports while TODO.txt remains standard.

---

## Keybindings

| Key | Action |
|---|---|
| `Win+Shift+T` / `Alt+Shift+T` | Summon or hide Trace from anywhere in Windows |
| `Ctrl+K` | Command Palette (export, clear completed, settings) |
| `↑` / `↓` or `j` / `k` | Move selection |
| `Space` | Toggle task completion |
| `Enter` | Edit selected task inline |
| `Tab` | Jump to capture input |
| `/` | Filter / search tasks |
| `Alt+↑` / `Alt+↓` | Reorder task within its section |
| `Ctrl+1` / `Ctrl+2` / `Ctrl+3` | Move task to Now / Later / Someday |
| `Delete` / `Backspace` | Delete task (6-second undo toast) |
| `Ctrl+Z` | Undo last action |
| `Escape` | Close dialog, finish an inline edit, or dismiss window |

---

## CLI

A small PowerShell script is included in `cli/` for adding tasks straight from the terminal:

```powershell
.\cli\trace.ps1 add "Review RC ~ rc **"
.\cli\trace.ps1 list
```

---

## Exporting Data

Press `Ctrl+K` to export anytime:
- **`TODO.txt`** — standard plain text with `(A)` priority notation and `~context`
- **`JSON`** — full schema dump with timestamps
- **`CSV`** — spreadsheet format

---

## Database Location

All data is stored locally in SQLite:

```text
%APPDATA%/com.orbitnoir.trace/trace.db
```

---

## Development

```powershell
pnpm install
pnpm tauri dev
```

Run tests and typechecks:

```powershell
pnpm exec vitest run
pnpm check
cd src-tauri
cargo check
cargo test
```
