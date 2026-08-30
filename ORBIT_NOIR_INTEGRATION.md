# Trace — Orbit Noir Integration

How this TODO utility inherits and applies the Orbit Noir design system.

## Source of Truth

All Orbit Noir visual decisions derive from Lacquer's `DESIGN.md` (locked 2026-08-27), specifically:
- **§3** Colour — the palette, functional/emotional rule, distribution targets
- **§4** Materials — Instrument, Blueglass, Chrome
- **§5** Typography — four voices, each with one job
- **§6** Motion — three speed tiers, no entrance animations

## What This Utility Inherits (Global Orbit Noir)

These are cross-application constants. Changing them here means changing them everywhere.

| Token | Value | Notes |
|---|---|---|
| `--on-orbit` | `#0B1731` | Ground surface. Deepest layer. |
| `--on-atlantic` | `#102A4C` | Panels, raised surfaces. |
| `--on-ion` | `#4F7DFF` | Primary functional accent — focus, selection. |
| `--on-signal` | `#39D4D0` | Secondary functional — completion. |
| `--on-milkglass` | `#E8EFF5` | Primary text. |
| `--on-moondust` | `#A8B8CA` | Secondary text. |
| `--on-font-interface` | Inter | Workhorse: task text, inputs, labels. |
| `--on-font-graphic` | Space Grotesk | Structural labels: date, section headers. |
| `--on-font-mono` | IBM Plex Mono | Metadata: contexts, counters, search hints. |
| Hairline | `rgba(232, 239, 245, 0.14)` | Section separators. |
| Spacing | 4px base | Same grid across all Orbit Noir members. |
| Radius | 4px/6px | Restrained. No pills on structural elements. |
| Fast motion | 150ms | Hover, focus, small state changes. |
| Normal motion | 300ms | Panel open/close, surface transitions. |
| Easing | `cubic-bezier(0.4, 0, 0.2, 1)` | Standard deceleration curve. |

## What This Utility Restrains

Trace is the quietest member of the Orbit Noir family. The table below tracks what is deliberately dialed down relative to Lacquer:

| Aspect | Lacquer | Trace |
|---|---|---|
| Palette breadth | Full — all 9 tokens plus album-derived colour | 6 tokens active: Orbit, Atlantic, Ion, Signal, Milkglass, Moondust. Ultraviolet/Solar/Flare appear only as priority markers. |
| Album reactivity | Yes — Experience surface adapts to artwork | None. Static Orbit Noir only. |
| Typography voices | 4 — Editorial, Graphic, Interface, Instrument | 3 — Graphic (labels), Interface (body), Instrument (metadata). No serif. |
| Blueglass | Menus, FX rack, dialogs | Search overlay only. |
| Chrome linework | Moderate — rail separators, transport edge | Minimal — section separators, capture input border. |
| Motion tiers | 3 — fast, normal, atmosphere (800ms) | 2 — fast and normal. No slow atmospheric transitions. |
| Emotional colour | Ultraviolet + Solar as decorative/expressive | Near-zero. Priority markers are the only emotional colour. |

## Application-Specific Decisions

These belong to Trace specifically and would not propagate to other Orbit Noir members:

- **Priority bar**: 3px left-edge vertical bar, coloured from Moondust (low) through Ion (medium) to Solar/Flare (high). Replaces star decorations as the primary visual priority signal.
- **Capture inset**: The bottom input sits in `--on-surface-inset` (`#081328`), slightly darker than the ground. Creates a recessed instrument-panel feel.
- **Selection**: 2px Ion left border + `rgba(79,125,255,0.10)` wash. No outline ring.
- **Completion glyph**: Open circle → filled circle with checkmark, coloured Signal. The only place teal appears in the UI.
- **Section toggles**: Space Grotesk 11px uppercase, Moondust. Quiet instrument-panel labels with count badges in Instrument mono.
- **Search overlay**: Scrim (`rgba(11,23,49,0.7)`) + Atlantic panel with hairline border. Not Blueglass — no blur, no GPU cost.

## Orbit Sidecar

The Sidecar is Trace's smallest and quietest surface: a fixed 288×44 frameless shell island anchored 5 logical pixels from the left and 4 logical pixels from the bottom of the primary monitor's full bounds. Its restrained translucent Orbit ground, hairline border, 12px outer radius, and 150ms feedback place it in the same visual plane as the Orbit Rail without reserving or modifying Windows work area. Persistent 26px no-drag navigation hitboxes sit above a clipped, isolated bay transition layer so pointer targeting does not change with content.

- Three icon-only launchers use the supplied LocalSend, ChatGPT, and Claude vectors to open the locally installed applications. Signal indicates that LocalSend is running; Moondust indicates that it is not.
- The Trace bay is only a doorway into the main capture workflow. By design it has no database access and never displays task contents, dates, contexts, priorities, completion state, previews, or counts.
- The Media bay reads the current Windows GSMTC session and limits itself to compact artwork, an ellipsized title, play/pause, and next. Missing sessions, metadata, artwork, or transport support collapse quietly.
- The Load bay reads physical memory with `GlobalMemoryStatusEx` and CPU load from delta `GetSystemTimes` samples. It polls only while visible and opens Task Manager when clicked.
- The active Trace/Media/Load bay is the only Sidecar preference and is stored in webview `localStorage`.
- Both the Sidecar and main Trace utility carry `WS_EX_TOOLWINDOW` (and not `WS_EX_APPWINDOW`) and opt out of taskbar presence. The Sidecar explicitly restores native topmost z-order whenever shown or re-anchored. The single-instance main utility defaults to always-on-top, still accepts focus when summoned, and retains its pin control; closing it requests an edit flush, then hides it without stopping Sidecar.
- Trace has no system-tray icon. A native Sidecar context menu provides Open Trace, re-anchor, autostart, and full-process quit actions.

## Task Links

Each task can carry one nullable web link in the existing SQLite entity. Editing reveals a compact optional link field; blank removes it, bare domains normalize to HTTPS, and invalid values remain in a quiet validation state. A small inline SVG affordance opens stored links without selecting, editing, or completing the task. JSON and CSV include links; TODO.txt deliberately does not invent a nonstandard representation.

Task mutations are committed directly to SQLite before the UI refreshes. Active inline text/link drafts use a 650ms coalescing save queue, with immediate flushes on editor blur, task switch, main-window dismissal, and every full-process quit path. There is no periodic background save loop.

## Token Architecture

Tokens are declared in `src/styles/orbit-noir.css` with the `--on-` prefix (Orbit Noir). Every component references tokens by name, never raw hex. If a shared Orbit Noir token package is later extracted, this file becomes a thin re-export layer pointing at the shared source.

The intentional separation:
- **`orbit-noir.css`**: Palette, spacing, typography, radii, motion. Shared vocabulary.
- **`app.css`**: Font face imports, reset, scrollbar, global rules. Application-specific.
- **Component `<style>`**: Scoped styles that consume tokens. No raw values.

## What's Still Unresolved in the Broader System

These are things the Orbit Noir system hasn't settled across all members yet. Trace should be ready to adopt them when they stabilize:

- **Shared token package** — currently each app declares its own copy of the palette. An `@orbitnoir/tokens` package would let one change propagate everywhere.
- **Dark/light modes** — Orbit Noir is currently dark-only. If a secondary mode appears, Trace's token layer can swap values without touching components.
- **Icon system** — no standard Orbit Noir icon set exists yet. Trace uses inline SVGs.
- **Window chrome** — Lacquer builds its own titlebar. Trace retains native Windows decorations for the main utility while the deliberately tiny Sidecar is frameless.
- **Center Rail top gap** — the current Windhawk-managed center Orbit Rail exposes a narrow blue/background strip above its visible island compared with the left and right islands. Trace deliberately does not fill or modify that gap; Sidecar aligns to the intended common 48–50px band while the center Rail is tuned separately in Windhawk.
