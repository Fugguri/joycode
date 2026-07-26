#!/usr/bin/env bash
# Удаление Joycode из пользовательского окружения.
# Использование:
#   ./uninstall.sh          — снести бинарь, ярлык, иконки (конфиг сохранить)
#   ./uninstall.sh --purge  — снести всё, включая конфиг и udev-правило
#   ./uninstall.sh -y        — без подтверждения
set -euo pipefail

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor"
CONFIG="$BIN_DIR/bindings.toml"
UDEV_RULE="/etc/udev/rules.d/99-joycode-uinput.rules"

PURGE=0
ASSUME_YES=0
for arg in "$@"; do
    case "$arg" in
        --purge) PURGE=1 ;;
        -y|--yes) ASSUME_YES=1 ;;
        *) echo "неизвестный флаг: $arg"; exit 1 ;;
    esac
done

echo "Будет удалено:"
echo "  • бинарь   $BIN_DIR/joycode"
echo "  • ярлык    $APP_DIR/joycode.desktop"
echo "  • иконки   $ICON_DIR/{scalable,48,64,128,256}/apps/joycode.*"
if [[ $PURGE -eq 1 ]]; then
    echo "  • конфиг   $CONFIG"
    echo "  • udev     $UDEV_RULE (нужен sudo)"
fi
if [[ $ASSUME_YES -eq 0 ]]; then
    read -rp "Продолжить? [y/N] " ans
    [[ "$ans" =~ ^[Yy]$ ]] || { echo "отменено."; exit 0; }
fi

# Остановить запущенный экземпляр, иначе бинарь «занят».
if pgrep -x joycode >/dev/null 2>&1; then
    echo "→ Останавливаю запущенный joycode…"
    pkill -x joycode 2>/dev/null || true
    sleep 0.5
fi

echo "→ Удаляю бинарь, ярлык, иконки"
rm -f "$BIN_DIR/joycode"
rm -f "$APP_DIR/joycode.desktop"
rm -f "$ICON_DIR/scalable/apps/joycode.svg"
for s in 48 64 128 256; do
    rm -f "$ICON_DIR/${s}x${s}/apps/joycode.png"
done

echo "→ Обновляю кэши"
update-desktop-database "$APP_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true

if [[ $PURGE -eq 1 ]]; then
    echo "→ Удаляю конфиг"
    rm -f "$CONFIG"
    if [[ -f "$UDEV_RULE" ]]; then
        echo "→ Удаляю udev-правило (sudo)"
        sudo rm -f "$UDEV_RULE" && sudo udevadm control --reload-rules || \
            echo "  не удалось удалить udev-правило — снеси вручную: sudo rm $UDEV_RULE"
    fi
    echo
    echo "✓ Joycode полностью удалён."
else
    echo
    echo "✓ Joycode удалён."
    echo "  Конфиг сохранён: $CONFIG"
    echo "  Полное удаление (с конфигом и udev): ./uninstall.sh --purge"
fi
