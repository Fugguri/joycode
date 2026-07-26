# Joycode

Управление [Claude Code](https://claude.com/claude-code) с игрового геймпада на Linux.
Push-to-talk на удержание кнопки, подтверждения, навигация стиками, слэш-команды
и системные аккорды (`ctrl+u`, `ctrl+c`) — не отрывая рук от геймпада.

Написано на Rust: `gilrs` читает геймпад, `evdev`/`uinput` эмулирует клавиатуру на
уровне ядра (работает и под X11, и под Wayland), интерфейс на `egui`.

## Возможности

- **Push-to-talk** — держишь кнопку = зажат пробел (голосовой ввод Claude Code).
- **Подтверждения и навигация** — Enter/Esc/стрелки на кнопки и крестовину.
- **Стики** — левый как стрелки с авто-повтором, правый как PageUp/PageDown (скролл).
- **Системные аккорды** — в биндинг можно вписать `ctrl+u`, `ctrl+c`, `alt+left` и т.п.
- **Текстовые макросы** — впечатать строку (например `/` для меню команд).
- **Тумблер ARMED** — впрыск клавиш полностью выключается на время настройки.
- **Настройки в UI** — маппинг правится мышкой, хранится в `bindings.toml`.
- Тёмная и светлая темы, моноширинный интерфейс.

## Требования

- Linux, геймпад (тестировалось на Xbox 360 controller).
- Rust (для сборки).
- Доступ на запись к `/dev/uinput` (см. установку udev-правила ниже).

## Установка

### Из исходников

```sh
git clone https://github.com/Fugguri/joycode.git
cd joycode
./install.sh
```

Ставит бинарь в `~/.local/bin/joycode`, ярлык и иконки — в пользовательские папки.
Запуск: команда `joycode` или иконка «Joycode» в меню приложений.

### Arch / Manjaro (пакет)

Рецепт пакета — в [`packaging/aur/`](packaging/aur). Собрать и поставить локально:

```sh
cd packaging/aur
makepkg -si
```

После публикации в AUR — `paru -S joycode-git` (либо `yay -S joycode-git`,
`pamac install joycode-git`).

### Доступ к /dev/uinput

Впрыск клавиш требует записи в `/dev/uinput`. При установке из исходников, если доступа нет:

```sh
./install.sh --udev   # ставит udev-правило (нужен sudo), затем перелогинься
```

Пакет (`makepkg`/AUR) кладёт udev-правило с `uaccess` сам — после установки
перелогинься или перезагрузись.

## Управление (по умолчанию)

Всё настраивается на вкладке **Настройки** (правится мышкой, хранится в
`~/.config/joycode/bindings.toml`). Дефолтная раскладка: геймпад держит навигацию,
правку текста и управление окнами; текст печатается/диктуется отдельно.

| Кнопка                | Действие                                   |
| --------------------- | ------------------------------------------ |
| LB / RB               | `super+alt+←` / `super+alt+→` — окна        |
| LT (держать)          | Backspace — стирать назад                   |
| RT (держать)          | Delete — стирать вперёд                     |
| A / B                 | Enter / Esc                                 |
| X                     | Space                                       |
| Y                     | `/` — меню слэш-команд                       |
| Back                  | `super` — обзор GNOME                        |
| D-Pad ↑↓, левый стик  | стрелки (с авто-повтором)                    |
| правый стик ↑↓        | PageUp / PageDown — скролл вывода            |

`super+alt+←/→` и `super` — шорткаты GNOME; под другим окружением поставь свои.

<details><summary>Полный <code>bindings.toml</code></summary>

```toml
# Верхние кнопки — окна и правка текста
[map.LeftTrigger]        # LB
type = "key"
key = "super+alt+left"
[map.RightTrigger]       # RB
type = "key"
key = "super+alt+right"
[map.LeftTrigger2]       # LT — держать
type = "hold"
key = "backspace"
[map.RightTrigger2]      # RT — держать
type = "hold"
key = "delete"

# Лицевые кнопки
[map.South]              # A
type = "key"
key = "enter"
[map.East]               # B
type = "key"
key = "esc"
[map.West]               # X
type = "key"
key = "space"
[map.North]              # Y
type = "text"
text = "/"
enter = false
[map.Select]             # Back
type = "key"
key = "super"

# Навигация
[map.DPadUp]
type = "key"
key = "up"
[map.DPadDown]
type = "key"
key = "down"
[map.LeftStickUp]
type = "key"
key = "up"
[map.LeftStickDown]
type = "key"
key = "down"
[map.LeftStickLeft]
type = "key"
key = "left"
[map.LeftStickRight]
type = "key"
key = "right"
[map.RightStickUp]
type = "key"
key = "pageup"
[map.RightStickDown]
type = "key"
key = "pagedown"
```
</details>

### Горячие клавиши окна

`1` / `2` / `3` — вкладки · `Space` — вкл/выкл системы · `T` — сменить тему.

## Как работает впрыск

Виртуальная клавиатура (Linux — `uinput`, Windows/macOS — `enigo`) эмулирует
клавиши на уровне ОС — они приходят в **активное окно**. Держи фокус на терминале
с Claude Code. Push-to-talk — это действие «Удержание» с клавишей `space`: повесь
его в Настройках на любую кнопку, и удержание будет держать пробел (голосовой ввод).

## Ограничения

- Клавиши идут в сфокусированное окно — терминал с Claude Code должен быть активен.
- `space` / `enter` / `esc` / стрелки / аккорды раскладко-независимы.
- Текстовые макросы печатают скан-коды **US-раскладки** — для латиницы переключи
  системную раскладку на английскую.

## Планы

- **Визуальный редактор биндингов** — картинка геймпада, клик по кнопке →
  назначение действия прямо на схеме.
- **Сворачивание в системный трей** — держать в фоне, окно не мешает
  (на GNOME нужен AppIndicator-расширение).
- **Прогон на Windows / macOS** — код собирается под них в CI, нужен живой тест.
- **Публикация в AUR** — `paru -S joycode-git`.

## Удаление

```sh
./uninstall.sh
```

## Лицензия

MIT — см. [LICENSE](LICENSE).
