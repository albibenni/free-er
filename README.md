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

## Open at Startup (Hyprland / systemd)

Run the install script once from the repo root:

```bash
./scripts/install-startup.sh
```

This will:

1. Build release binaries (`cargo build --release`)
2. Install `free-er` and `free-er-ui` to `~/.local/bin/`
3. Create and enable a systemd user service for the daemon (`~/.config/systemd/user/free-er.service`)
4. Add `exec-once = uwsm-app -- free-er-ui` to `~/.config/hypr/autostart.conf`

After running, the daemon starts immediately and on every login. The UI launches automatically with the Hyprland session.

Re-running the script is safe and handles new builds: it restarts the daemon service and relaunches `free-er-ui` if it is already running, so the new binaries take effect immediately without requiring a logout.

**Useful commands:**

```bash
# Check daemon status
systemctl --user status free-er.service

# View daemon logs
journalctl --user -u free-er.service -f

# Stop and disable
systemctl --user disable --now free-er.service
```

To fully uninstall startup behaviour, also remove the `exec-once` line from `~/.config/hypr/autostart.conf`.

## Coverage Setup

Install the coverage tool once:

```bash
cargo install cargo-llvm-cov
```

Then run:

```bash
make coverage
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
| `make coverage` | Print per-file + total coverage table in terminal |
| `make clean` | Remove build artifacts and extension/dist |
| `make help` | Show all available targets |

## Browser Extension

First, build it:

```bash
make extension
```

### Chrome / Chromium / Brave

1. Go to `chrome://extensions`
2. Enable **Developer mode** (top-right toggle)
3. Click **Load unpacked**
4. Select the `extension/` folder (the root folder containing `manifest.json`, not `extension/dist/`)
5. The **free-er** extension appears in your toolbar

### Firefox

1. Go to `about:debugging`
2. Click **This Firefox**
3. Click **Load Temporary Add-on…**
4. Select `extension/manifest.json`
5. The extension stays loaded until Firefox is restarted (temporary install)

For a permanent Firefox install you'd need to sign it via Mozilla — for personal use the temporary method is fine.

### How it works

1. Make sure the daemon is running (`make daemon` or `make run`)
2. The extension polls `http://127.0.0.1:10000/api/status` every 2 seconds
3. Start a focus session from the UI
4. Any tab navigating to a non-allowed URL is redirected to the block page at `http://127.0.0.1:10000`
5. Click the extension icon in the toolbar to see current status and active allowed URLs

### Quick test without the UI

```bash
# Start daemon
make daemon

# Trigger a focus session manually
echo '{"cmd":"StartFocus","rule_set_id":"00000000-0000-0000-0000-000000000000"}' | nc -U /tmp/free-er.sock

# Check status
echo '{"cmd":"GetStatus"}' | nc -U /tmp/free-er.sock
```

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

### Google Calendar integration

Google Calendar sync uses OAuth2. Because OAuth2 requires an app registration with Google, you need to create credentials once as the developer — after that, any user just clicks **Connect** in Settings and gets a browser login popup with no further setup.

**One-time developer setup:**

1. Go to [console.cloud.google.com](https://console.cloud.google.com) and create a project
2. Enable the **Google Calendar API**: APIs & Services → Enable APIs → search "Google Calendar API"
3. Create credentials: APIs & Services → Credentials → Create Credentials → **OAuth client ID**
   - Application type: **Web application**
   - Authorized redirect URI: `http://127.0.0.1:10000/oauth/google/callback`
4. Copy the client ID and secret, then create `~/.config/free-er/google_client.json`:

```json
{
  "client_id": "YOUR_CLIENT_ID.apps.googleusercontent.com",
  "client_secret": "YOUR_CLIENT_SECRET"
}
```

Once the file is in place, restart the daemon and click **Connect** in the Settings tab. A browser window will open for Google login. After authorizing, the daemon stores the tokens and syncs your calendar every 15 minutes.

**Error 403: access_denied?**

Unverified apps can only be used by approved test users. To add yourself:

1. Go to your project → **Google Auth Platform** → **Audience** (in the left sidebar)
2. Scroll to **Test users** → **Add users**
3. Add your Google account email and save

You can add up to 100 test users without going through Google's full verification process (only needed if you publish the app publicly).

All Google Calendar events in the next 30 days are imported and shown in the Schedule tab.

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
