# free-er — Linux/Hyprland Port Plan

> Rust full-stack port of [free](../free/) (macOS focus blocker) for Arch Linux + Hyprland (Wayland).

---

## Goal

Replicate the core feature set of `free` on Linux:

- Website blocking during focus sessions via allowlists
- Weekly schedules + one-off sessions
- Pomodoro timer
- Strict / unblockable mode
- Calendar import (CalDAV / `.ics`)
- Waybar integration
- Autostart via systemd user service

---

## Architecture Overview

```
┌─────────────────────────────────────────────────────────┐
│                    free-er daemon                        │
│  (Rust, systemd user service, Unix socket IPC)          │
│                                                          │
│  AppState ── BlockingCoordinator ── RuleMatcher         │
│      │              │                                    │
│  Schedules       /etc/hosts OR local DNS redirect       │
│  Pomodoro        LocalServer (block page :10000)        │
│  CalDAV sync                                            │
└──────────────┬──────────────────────────────────────────┘
               │ Unix socket (~/.local/share/free-er/free-er.sock)
       ┌───────┴────────┐──────────────────┐
       │                │                  │
  GTK4/Relm4 UI   Waybar module      Browser Extension
  (settings,       (status script,    (WebExtension,
   schedules,       reads socket)      reports URLs,
   allowlists,                         shows block page)
   pomodoro)
```

---

## Tech Stack

| Layer | Crate / Tool |
|---|---|
| Core daemon | `tokio` (async runtime) |
| IPC | Unix socket + JSON (or `tarpc`) |
| UI | `relm4` (GTK4 bindings, Elm-style) |
| Block page server | `axum` |
| CalDAV / iCal | `ical` crate + `reqwest` |
| Persistence | `serde` + JSON files (`~/.config/free-er/`) |
| Waybar module | Shell script reading the socket |
| Autostart | systemd user service |
| Browser integration | WebExtension (JS, Chrome + Firefox) |

---

## Project Structure

```
free-er/
├── Cargo.toml                  # workspace
├── crates/
│   ├── daemon/                 # core state + blocking engine
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app_state.rs        # AppState (focus, pomodoro, schedules)
│   │   │   ├── blocking.rs         # BlockingCoordinator (/etc/hosts writer)
│   │   │   ├── rule_matcher.rs     # RuleMatcher (wildcard URL matching)
│   │   │   ├── schedule.rs         # Schedule model + isActive()
│   │   │   ├── rule_set.rs         # RuleSet (allowed list) model
│   │   │   ├── pomodoro.rs         # Pomodoro timer state machine
│   │   │   ├── calendar.rs         # CalDAV / .ics import
│   │   │   ├── local_server.rs     # axum block page server (:10000)
│   │   │   ├── ipc.rs              # Unix socket server (commands + status)
│   │   │   └── persistence.rs      # load/save JSON config
│   │   └── Cargo.toml
│   │
│   ├── ui/                     # GTK4 / Relm4 settings app
│   │   ├── src/
│   │   │   ├── main.rs
│   │   │   ├── app.rs              # root Relm4 component
│   │   │   ├── sections/
│   │   │   │   ├── focus.rs        # focus toggle + status
│   │   │   │   ├── schedules.rs    # weekly calendar view
│   │   │   │   ├── allowed_lists.rs
│   │   │   │   ├── pomodoro.rs
│   │   │   │   ├── calendar.rs     # CalDAV import settings
│   │   │   │   └── settings.rs     # strict mode, theme, etc.
│   │   │   └── ipc_client.rs       # talks to daemon socket
│   │   └── Cargo.toml
│   │
│   └── shared/                 # shared types (models, IPC protocol)
│       ├── src/
│       │   ├── lib.rs
│       │   ├── models.rs           # RuleSet, Schedule, PomodoroConfig, etc.
│       │   └── ipc.rs              # IPC command/response types (serde)
│       └── Cargo.toml
│
├── extension/                  # Browser WebExtension
│   ├── manifest.json           # MV3, Chrome + Firefox compatible
│   ├── background.js           # monitors active tab URL, sends to daemon
│   ├── content.js              # shows block page overlay
│   └── popup/                  # optional popup UI
│
├── waybar/
│   └── free-er.sh              # script: query socket, output JSON for waybar
│
├── systemd/
│   └── free-er.service         # ~/.config/systemd/user/free-er.service
│
└── block-page/
    └── index.html              # served by local_server on :10000
```

---

## Phases

### Phase 1 — Core Daemon (MVP blocking)

- [ ] Workspace setup (`Cargo.toml`, `shared` crate with models)
- [ ] Port `RuleMatcher` (wildcard URL matching) from Swift → Rust
- [ ] Port `Schedule` model + `is_active()` logic
- [ ] Port `RuleSet` (allowed list) model
- [ ] `AppState`: focus on/off, active rule set, schedule evaluation loop
- [ ] `BlockingCoordinator`: write blocked domains to `/etc/hosts` (managed section)
- [ ] `LocalServer` (axum): serve block page on `localhost:10000`
- [ ] IPC: Unix socket accepting JSON commands (`StartFocus`, `StopFocus`, `GetStatus`, etc.)
- [ ] Persistence: load/save config from `~/.config/free-er/config.json`

### Phase 2 — Browser Extension

- [ ] WebExtension manifest (Chrome MV3 + Firefox compatible)
- [ ] Background script: poll active tab URL every second, POST to daemon socket via native messaging or HTTP
- [ ] Content script: if current tab is `localhost:10000`, show styled block page
- [ ] Native messaging host (Rust binary) OR simple HTTP endpoint on daemon

> **Note**: Native messaging is more robust than HTTP for URL reporting because it doesn't require CORS and works even with HTTPS pages.

### Phase 3 — Pomodoro

- [ ] `PomodoroState` machine: `Idle → Focus → Break → Focus → …`
- [ ] Configurable focus/break durations
- [ ] Preset timers (25/5, 50/10, 90/20)
- [ ] Strict mode: breaks cannot be taken manually
- [ ] IPC commands: `StartPomodoro`, `EndBreak`, `SkipBreak`, `GetPomodoroStatus`

### Phase 4 — Calendar Import

- [ ] CalDAV client (`reqwest` + `ical` crate)
- [ ] Parse `.ics` events into `Schedule` entries
- [ ] Title-based import rules (e.g. events containing "work" → focus session)
- [ ] Configurable per-import allowed list assignment
- [ ] Auto-remove past imported schedules

### Phase 5 — GTK4 / Relm4 UI

- [ ] Main window with sidebar navigation (Focus, Schedules, Allowed Lists, Pomodoro, Calendar, Settings)
- [ ] Focus section: toggle button, active list indicator, break button
- [ ] Schedules section: weekly grid view with drag-to-create, drag-to-move, drag-to-resize (snap to 15m)
- [ ] Allowed lists section: list management, wildcard URL input, import from open tabs (via extension)
- [ ] Pomodoro section: timer display, preset selector, config
- [ ] Settings section: strict mode toggle, CalDAV credentials, theme/accent
- [ ] Tray icon (via `libappindicator` or `ksni`)

### Phase 6 — Waybar + Autostart

- [ ] `waybar/free-er.sh`: query daemon socket, output Waybar JSON (text, tooltip, class)
- [ ] Waybar config snippet
- [ ] `systemd/free-er.service` user service
- [ ] Install script: copies service, enables it, installs extension

---

## IPC Protocol (Unix Socket)

Commands sent as newline-delimited JSON:

```json
// Commands (client → daemon)
{ "cmd": "StartFocus", "rule_set_id": "uuid" }
{ "cmd": "StopFocus" }
{ "cmd": "TakeBreak", "duration_secs": 300 }
{ "cmd": "StartPomodoro", "focus_secs": 1500, "break_secs": 300 }
{ "cmd": "GetStatus" }
{ "cmd": "AddRuleSet", "name": "Work", "urls": ["github.com", "*.rust-lang.org"] }
{ "cmd": "AddSchedule", ... }

// Response (daemon → client)
{
  "focus_active": true,
  "mode": "Pomodoro",
  "pomodoro_phase": "Focus",
  "seconds_remaining": 843,
  "active_rule_set": "Work",
  "strict": false,
  "next_schedule": { "name": "Work Hours", "starts_in_secs": 3600 }
}
```

---

## Blocking Strategy

### Option A — `/etc/hosts` (default, no root required with sudoers)

- Write a managed block (`# free-er start` … `# free-er end`) into `/etc/hosts`
- Requires one sudoers line: `username ALL=(ALL) NOPASSWD: /usr/bin/free-er-hosts-helper`
- Fast, works system-wide

### Option B — Browser Extension only (zero root)

- Extension intercepts navigation and redirects blocked domains to `localhost:10000`
- No system changes needed
- Only blocks in the monitored browser

**Default**: Option B (extension), with Option A available as opt-in for stricter enforcement.

---

## macOS → Rust Component Map

| macOS (Swift) | Linux (Rust) |
|---|---|
| `RuleMatcher.swift` | `crates/daemon/src/rule_matcher.rs` |
| `Schedule.swift` | `crates/shared/src/models.rs` (`Schedule`) |
| `RuleSet.swift` | `crates/shared/src/models.rs` (`RuleSet`) |
| `AppState.swift` | `crates/daemon/src/app_state.rs` |
| `AppStateBlockingCoordinator` | `crates/daemon/src/blocking.rs` |
| `BrowserMonitor.swift` | `extension/background.js` + `crates/daemon/src/ipc.rs` |
| `DefaultBrowserAutomator` | Browser extension (no Accessibility API on Linux) |
| `LocalServer.swift` | `crates/daemon/src/local_server.rs` (axum) |
| `CalendarManager.swift` | `crates/daemon/src/calendar.rs` |
| `LaunchAtLoginManager` | `systemd/free-er.service` |
| AppKit menu bar + popover | Waybar module + tray icon |
| AppKit settings window | `crates/ui/` (Relm4) |

---

## Key Dependencies

```toml
# daemon
tokio = { version = "1", features = ["full"] }
axum = "0.7"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
ical = "0.10"
reqwest = { version = "0.12", features = ["json"] }
chrono = "0.4"
uuid = { version = "1", features = ["v4", "serde"] }

# ui
relm4 = "0.9"
relm4-components = "0.9"
gtk4 = "0.9"
```

---

## Out of Scope (for now)

- App blocking (block entire processes, not just URLs)
- Light/dark theme customization (use system GTK theme)
- Multi-monitor tray
