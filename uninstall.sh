#!/usr/bin/env bash
# Удаление Joycode из пользовательского окружения.
set -euo pipefail

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor"

rm -f "$BIN_DIR/joycode"
rm -f "$APP_DIR/joycode.desktop"
rm -f "$ICON_DIR/scalable/apps/joycode.svg"
for s in 48 64 128 256; do
    rm -f "$ICON_DIR/${s}x${s}/apps/joycode.png"
done

update-desktop-database "$APP_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true

echo "✓ Joycode удалён."
echo "  Конфиг $BIN_DIR/bindings.toml оставлен — удали вручную при желании."
echo "  udev-правило (если ставил): sudo rm /etc/udev/rules.d/99-joycode-uinput.rules"
