# Joycode

**English** · [Русский](README.md)

Control [Claude Code](https://claude.com/claude-code) with a game controller on Linux.
Push-to-talk on a held button, confirmations, stick navigation, slash commands and
system chords (`ctrl+u`, `ctrl+c`) — without taking your hands off the gamepad.

Written in Rust: `gilrs` reads the gamepad, `evdev`/`uinput` emulates a keyboard at
the kernel level (works on both X11 and Wayland), UI on `egui`.

## Features

- **Push-to-talk** — hold a button = Space held down (Claude Code voice input).
- **Confirmations and navigation** — Enter/Esc/arrows on buttons and the D-pad.
- **Sticks** — left stick as arrows with auto-repeat, right stick as PageUp/PageDown (scroll).
- **System chords** — a binding can be `ctrl+u`, `ctrl+c`, `alt+left`, etc.
- **Text macros** — type a string (e.g. `/` for the command menu).
- **ARMED toggle** — key injection is fully disabled while you configure.
- **In-app settings** — mappings edited with the mouse, stored in `bindings.toml`.
- Dark and light themes, monospaced UI.

## Requirements

- Linux, a game controller (tested on an Xbox 360 controller).
- Rust (to build).
- Write access to `/dev/uinput` (see the udev rule below).

## Installation

### From source

```sh
git clone https://github.com/Fugguri/joycode.git
cd joycode
./install.sh
```

Installs the binary to `~/.local/bin/joycode`, the launcher and icons into your user
directories. Run via the `joycode` command or the “Joycode” icon in your app menu.

### Arch / Manjaro (package)

The package recipe is in [`packaging/aur/`](packaging/aur). Build and install locally:

```sh
cd packaging/aur
makepkg -si
```

Once published to the AUR — `paru -S joycode-git` (or `yay -S joycode-git`,
`pamac install joycode-git`).

### /dev/uinput access

Key injection needs write access to `/dev/uinput`. When installing from source, if
you don't have it:

```sh
./install.sh --udev   # installs a udev rule (needs sudo), then re-login
```

The package (`makepkg`/AUR) ships a `uaccess` udev rule itself — after install,
re-login or reboot.

## Controls (default)

Everything is configurable on the **Settings** tab (edited with the mouse, stored in
`~/.config/joycode/bindings.toml`). The default layout keeps navigation, text editing
and window management on the gamepad; text is typed/dictated separately.

| Button                 | Action                                     |
| ---------------------- | ------------------------------------------ |
| LB / RB                | `super+alt+←` / `super+alt+→` — windows     |
| LT (hold)              | Backspace — delete backward                 |
| RT (hold)              | Delete — delete forward                     |
| A / B                  | Enter / Esc                                 |
| X                      | Space                                       |
| Y                      | `/` — slash-command menu                     |
| Back                   | `super` — GNOME overview                     |
| D-Pad ↑↓, left stick   | arrows (with auto-repeat)                    |
| right stick ↑↓         | PageUp / PageDown — scroll output            |

`super+alt+←/→` and `super` are GNOME shortcuts; set your own on other desktops.

<details><summary>Full <code>bindings.toml</code></summary>

```toml
# Top buttons — windows and text editing
[map.LeftTrigger]        # LB
type = "key"
key = "super+alt+left"
[map.RightTrigger]       # RB
type = "key"
key = "super+alt+right"
[map.LeftTrigger2]       # LT — hold
type = "hold"
key = "backspace"
[map.RightTrigger2]      # RT — hold
type = "hold"
key = "delete"

# Face buttons
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

# Navigation
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

### Window hotkeys

`1` / `2` / `3` — tabs · `Space` — toggle the system · `T` — switch theme.

## How injection works

A virtual keyboard (Linux — `uinput`, Windows/macOS — `enigo`) emulates keys at the
OS level — they arrive in the **focused window**. Keep the terminal with Claude Code
focused. Push-to-talk is a “Hold” action with the `space` key: bind it in Settings to
any button, and holding it holds Space down (voice input).

## Limitations

- Keys go to the focused window — the terminal with Claude Code must be active.
- `space` / `enter` / `esc` / arrows / chords are layout-independent.
- Text macros type **US-layout** scancodes — for Latin, switch your system layout to English.

## Roadmap

- **Visual binding editor** — a gamepad image, click a button → assign its action on the map.
- **Minimize to system tray** — keep it running in the background out of the way
  (on GNOME needs the AppIndicator extension).
- **Windows / macOS run** — the code builds for them in CI, needs a live test.
- **AUR publish** — `paru -S joycode-git`.

## Uninstall

```sh
./uninstall.sh
```

## License

MIT — see [LICENSE](LICENSE).
