#!/usr/bin/env bash
# install-startup.sh
# Sets up free-er to launch automatically at login:
#   - daemon: systemd user service (free-er.service)
#   - UI:     Hyprland exec-once in ~/.config/hypr/autostart.conf
#
# Run from the repo root: ./scripts/install-startup.sh
# Re-running is safe — it will overwrite existing files.

set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN_DIR="$HOME/.local/bin"
SYSTEMD_DIR="$HOME/.config/systemd/user"
HYPR_AUTOSTART="$HOME/.config/hypr/autostart.conf"

# ── 1. Build release binaries ─────────────────────────────────────────────────
echo "==> Building release binaries..."
cargo build --release --manifest-path "$REPO_ROOT/Cargo.toml"

# ── 2. Install binaries ───────────────────────────────────────────────────────
echo "==> Installing binaries to $BIN_DIR..."
mkdir -p "$BIN_DIR"
cp "$REPO_ROOT/target/release/free-er"    "$BIN_DIR/free-er"
cp "$REPO_ROOT/target/release/free-er-ui" "$BIN_DIR/free-er-ui"
chmod +x "$BIN_DIR/free-er" "$BIN_DIR/free-er-ui"

# ── 3. Create systemd user service for the daemon ────────────────────────────
echo "==> Creating systemd user service..."
mkdir -p "$SYSTEMD_DIR"
cat > "$SYSTEMD_DIR/free-er.service" <<EOF
[Unit]
Description=free-er daemon
After=network.target

[Service]
ExecStart=$BIN_DIR/free-er
Restart=on-failure
RestartSec=3

[Install]
WantedBy=default.target
EOF

# ── 4. Enable and (re)start the daemon service ───────────────────────────────
echo "==> Enabling and (re)starting free-er.service..."
systemctl --user daemon-reload
systemctl --user enable free-er.service
systemctl --user restart free-er.service

# ── 5. Add UI to Hyprland autostart (idempotent) ─────────────────────────────
EXEC_LINE="exec-once = uwsm-app -- free-er-ui"
if grep -qF "$EXEC_LINE" "$HYPR_AUTOSTART" 2>/dev/null; then
    echo "==> Hyprland autostart already configured, skipping."
else
    echo "==> Adding free-er-ui to $HYPR_AUTOSTART..."
    echo "" >> "$HYPR_AUTOSTART"
    echo "# free-er UI" >> "$HYPR_AUTOSTART"
    echo "$EXEC_LINE" >> "$HYPR_AUTOSTART"
fi

# ── 6. Relaunch UI if already running ────────────────────────────────────────
if pgrep -x free-er-ui > /dev/null; then
    echo "==> Relaunching free-er-ui..."
    pkill -x free-er-ui
    sleep 0.5
    uwsm-app -- free-er-ui &
else
    echo "==> free-er-ui not running, will start on next Hyprland login."
fi

# ── Done ──────────────────────────────────────────────────────────────────────
echo ""
echo "Done! Summary:"
echo "  Daemon:  systemctl --user status free-er.service"
echo "  UI:      will launch automatically on next Hyprland login"
echo ""
echo "To stop/disable:"
echo "  systemctl --user disable --now free-er.service"
echo "  # and remove the exec-once line from $HYPR_AUTOSTART"
