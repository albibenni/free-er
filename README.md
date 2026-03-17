# free-er

A focus session enforcer for Linux. Blocks distracting websites during work sessions using a browser extension + background daemon.

## Features

- **Focus sessions** — activate an allowlist of URLs; everything else gets blocked
- **Pomodoro timer** — built-in 25/5, 50/10, 90/20 presets with auto phase advance
- **Schedules** — recurring weekly time blocks that trigger focus sessions automatically
- **CalDAV / .ics import** — sync calendar events into schedules by keyword matching
- **Strict mode** — prevent manually stopping focus while a schedule is active
- **GTK4 UI** — native Linux desktop settings window
- **Browser extension** — Chrome/Firefox MV3, intercepts navigation in real time

## Architecture

```text
┌─────────────────────────────────────────────┐
│  Browser Extension (TypeScript + esbuild)   │
│  Polls /api/status · Redirects blocked tabs │
└───────────────┬─────────────────────────────┘
                │ HTTP (localhost:10000)
┌───────────────▼─────────────────────────────┐
│  Daemon (Rust · free-er)                    │
│  ├── Unix socket IPC  (/tmp/free-er.sock)   │
│  ├── Axum HTTP server (block page + API)    │
│  ├── Pomodoro state machine                 │
│  ├── Schedule checker                       │
│  ├── CalDAV sync (every 15 min)             │
│  └── Config persistence (~/.config/free-er) │
└───────────────▲─────────────────────────────┘
                │ Unix socket
┌───────────────┴─────────────────────────────┐
│  GTK4 UI (Rust · Relm4 · free-er-ui)        │
│  Focus · Pomodoro · Allowed Lists · Settings │
└─────────────────────────────────────────────┘
```

## Requirements

- Rust (stable, ≥ 1.92)
- GTK4 development libraries
- Node.js + pnpm (for the browser extension)

### Install GTK4 (Arch)

```bash
sudo pacman -S gtk4 glib2
```

### Install GTK4 (Ubuntu/Debian)

```bash
sudo apt install libgtk-4-dev
```

## Getting Started

```bash
# Clone
git clone <repo-url>
cd free-er

# Build + run everything in one command
make run
```

## Make Targets

| Target | Description |
| ------ | ----------- |
| `make build` | Build all Rust crates (debug) |
| `make release` | Build all Rust crates (release) |
| `make daemon` | Run the daemon |
| `make ui` | Run the GTK4 UI |
| `make extension` | Build the browser extension |
| `make extension-watch` | Watch and rebuild extension on changes |
| `make dev` | Build + start daemon in background + launch UI |
| `make test` | Run all tests |
| `make clean` | Remove build artifacts and extension/dist |
| `make help` | Show all available targets |

## Browser Extension

Build output lands in `extension/dist/`. Load it as an unpacked extension:

- **Chrome**: `chrome://extensions` → Enable developer mode → Load unpacked → select `extension/`
- **Firefox**: `about:debugging` → This Firefox → Load Temporary Add-on → select `extension/manifest.json`

The extension polls `http://127.0.0.1:10000/api/status` every 2 seconds. When a focus session is active, any navigation to a non-allowed URL is redirected to the block page.

## Configuration

Config is stored at `~/.config/free-er/config.json` and written automatically by the daemon.

### URL patterns

Allowed URL patterns support wildcards:

| Pattern | Matches |
| ------- | ------- |
| `github.com` | Exact hostname |
| `*.rust-lang.org` | Any subdomain + the root |
| `*` | Everything (allow all) |

### CalDAV / .ics import

Add a CalDAV source in the Settings section of the UI. Events whose title contains a configured keyword are imported as focus schedules.

Example config snippet:

```json
{
  "caldav": {
    "url": "https://calendar.example.com/user/calendar.ics",
    "username": "user",
    "password": "secret",
    "import_rules": [
      { "keyword": "Deep Work", "rule_set_id": "<uuid>" }
    ]
  }
}
```

## Workspace Structure

```text
free-er/
├── crates/
│   ├── shared/       Shared models and IPC types
│   ├── daemon/       Background daemon binary (free-er)
│   └── ui/           GTK4 settings UI (free-er-ui)
├── extension/        Browser extension (TypeScript)
├── block-page/       HTML page served when a site is blocked
└── Makefile
```

## IPC Protocol

The daemon listens on `/tmp/free-er.sock`. Commands are newline-delimited JSON:

```json
{ "cmd": "StartFocus",    "rule_set_id": "<uuid>" }
{ "cmd": "StopFocus" }
{ "cmd": "StartPomodoro", "focus_secs": 1500, "break_secs": 300 }
{ "cmd": "StopPomodoro" }
{ "cmd": "SkipBreak" }
{ "cmd": "GetStatus" }
{ "cmd": "AddRuleSet",    "name": "Work", "allowed_urls": ["github.com"] }
{ "cmd": "RemoveRuleSet", "id": "<uuid>" }
```
