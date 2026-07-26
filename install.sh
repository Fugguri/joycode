#!/usr/bin/env bash
# Установка Joycode в пользовательское окружение (без root).
# Использование:
#   ./install.sh          — собрать и установить (бинарь, ярлык, иконки)
#   ./install.sh --udev   — доустановить udev-правило для /dev/uinput (нужен sudo)
set -euo pipefail

cd "$(dirname "$0")"

BIN_DIR="$HOME/.local/bin"
APP_DIR="$HOME/.local/share/applications"
ICON_DIR="$HOME/.local/share/icons/hicolor"

install_udev() {
    echo "→ Установка udev-правила для /dev/uinput (нужен sudo)…"
    sudo tee /etc/udev/rules.d/99-joycode-uinput.rules >/dev/null <<'EOF'
# Joycode: доступ к uinput для группы input
KERNEL=="uinput", MODE="0660", GROUP="input", OPTIONS+="static_node=uinput"
EOF
    sudo udevadm control --reload-rules
    sudo udevadm trigger
    if ! id -nG "$USER" | grep -qw input; then
        echo "→ Добавляю $USER в группу input…"
        sudo usermod -aG input "$USER"
        echo "  ВНИМАНИЕ: перелогинься, чтобы членство в группе вступило в силу."
    fi
    echo "✓ udev-правило установлено."
}

if [[ "${1:-}" == "--udev" ]]; then
    install_udev
    exit 0
fi

echo "→ Сборка (release)…"
cargo build --release --bin joycode

echo "→ Установка бинаря → $BIN_DIR/joycode"
install -Dm755 target/release/joycode "$BIN_DIR/joycode"

echo "→ Установка иконок"
install -Dm644 assets/joycode.svg "$ICON_DIR/scalable/apps/joycode.svg"
if command -v rsvg-convert >/dev/null; then
    for s in 48 64 128 256; do
        tmp="$(mktemp)"
        rsvg-convert -w "$s" -h "$s" assets/joycode.svg -o "$tmp"
        install -Dm644 "$tmp" "$ICON_DIR/${s}x${s}/apps/joycode.png"
        rm -f "$tmp"
    done
fi

echo "→ Установка ярлыка → $APP_DIR/joycode.desktop"
mkdir -p "$APP_DIR"
sed "s|@EXEC@|$BIN_DIR/joycode|g" assets/joycode.desktop.in > "$APP_DIR/joycode.desktop"

echo "→ Обновление кэшей"
update-desktop-database "$APP_DIR" 2>/dev/null || true
gtk-update-icon-cache -f -t "$ICON_DIR" 2>/dev/null || true

echo
echo "✓ Установлено. Запуск: joycode  (или найди «Joycode» в меню приложений)"

if [[ ! -w /dev/uinput ]]; then
    echo
    echo "⚠  Нет доступа на запись к /dev/uinput — впрыск клавиш работать не будет."
    echo "   Установи udev-правило:  ./install.sh --udev"
fi
