//! Linux-бэкенд: виртуальная клавиатура поверх /dev/uinput.
//! Работает на уровне ядра — клавиши приходят в сфокусированное окно (X11/Wayland).
use crate::injector::KeyInjector;
use crate::keys::{Chord, Key};
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};
use std::io;

pub struct UinputInjector {
    device: VirtualDevice,
}

impl UinputInjector {
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for k in all_keycodes() {
            keys.insert(k);
        }
        let device = VirtualDevice::builder()?
            .name("joycode virtual keyboard")
            .with_keys(&keys)?
            .build()?;
        log::info!(
            "uinput-клавиатура создана, зарегистрировано {} клавиш",
            all_keycodes().len()
        );
        Ok(Self { device })
    }

    fn emit(&mut self, key: KeyCode, value: i32) -> io::Result<()> {
        let ev = InputEvent::new(EventType::KEY.0, key.code(), value);
        self.device.emit(&[ev])
    }

    fn press_key(&mut self, code: KeyCode, shift: bool) -> io::Result<()> {
        if shift {
            self.emit(KeyCode::KEY_LEFTSHIFT, 1)?;
        }
        self.emit(code, 1)?;
        self.emit(code, 0)?;
        if shift {
            self.emit(KeyCode::KEY_LEFTSHIFT, 0)?;
        }
        Ok(())
    }
}

impl KeyInjector for UinputInjector {
    fn key_down(&mut self, key: Key) -> io::Result<()> {
        match key_to_code(key) {
            Some((code, _)) => self.emit(code, 1),
            None => unknown(key),
        }
    }

    fn key_up(&mut self, key: Key) -> io::Result<()> {
        match key_to_code(key) {
            Some((code, _)) => self.emit(code, 0),
            None => unknown(key),
        }
    }

    fn key_tap(&mut self, key: Key) -> io::Result<()> {
        match key_to_code(key) {
            Some((code, shift)) => self.press_key(code, shift),
            None => unknown(key),
        }
    }

    fn tap_chord(&mut self, chord: &Chord) -> io::Result<()> {
        let mut held = Vec::new();
        for m in &chord.mods {
            if let Some((code, _)) = key_to_code(*m) {
                self.emit(code, 1)?;
                held.push(code);
            }
        }
        if let Some((code, shift)) = key_to_code(chord.key) {
            self.press_key(code, shift)?;
        }
        for code in held.into_iter().rev() {
            self.emit(code, 0)?;
        }
        Ok(())
    }

    fn type_text(&mut self, text: &str) -> io::Result<()> {
        for c in text.chars() {
            match char_to_code(c) {
                Some((code, shift)) => self.press_key(code, shift)?,
                None => log::warn!("type_text: символ {c:?} не маппится в keycode, пропущен"),
            }
        }
        Ok(())
    }
}

fn unknown(key: Key) -> io::Result<()> {
    log::warn!("uinput: клавиша {key:?} не маппится в keycode");
    Ok(())
}

/// Нейтральный Key → (evdev-код, нужен ли Shift).
fn key_to_code(key: Key) -> Option<(KeyCode, bool)> {
    let code = match key {
        Key::Space => KeyCode::KEY_SPACE,
        Key::Enter => KeyCode::KEY_ENTER,
        Key::Esc => KeyCode::KEY_ESC,
        Key::Tab => KeyCode::KEY_TAB,
        Key::Backspace => KeyCode::KEY_BACKSPACE,
        Key::Delete => KeyCode::KEY_DELETE,
        Key::Insert => KeyCode::KEY_INSERT,
        Key::Home => KeyCode::KEY_HOME,
        Key::End => KeyCode::KEY_END,
        Key::Up => KeyCode::KEY_UP,
        Key::Down => KeyCode::KEY_DOWN,
        Key::Left => KeyCode::KEY_LEFT,
        Key::Right => KeyCode::KEY_RIGHT,
        Key::PageUp => KeyCode::KEY_PAGEUP,
        Key::PageDown => KeyCode::KEY_PAGEDOWN,
        Key::Slash => KeyCode::KEY_SLASH,
        Key::Ctrl => KeyCode::KEY_LEFTCTRL,
        Key::Alt => KeyCode::KEY_LEFTALT,
        Key::Shift => KeyCode::KEY_LEFTSHIFT,
        Key::Super => KeyCode::KEY_LEFTMETA,
        Key::Char(c) => return char_to_code(c),
    };
    Some((code, false))
}

/// ASCII-символ → (evdev-код, нужен ли Shift). Скан-коды US-раскладки.
fn char_to_code(c: char) -> Option<(KeyCode, bool)> {
    let lower = c.to_ascii_lowercase();
    let shift = c.is_ascii_uppercase();
    let key = match lower {
        'a' => KeyCode::KEY_A,
        'b' => KeyCode::KEY_B,
        'c' => KeyCode::KEY_C,
        'd' => KeyCode::KEY_D,
        'e' => KeyCode::KEY_E,
        'f' => KeyCode::KEY_F,
        'g' => KeyCode::KEY_G,
        'h' => KeyCode::KEY_H,
        'i' => KeyCode::KEY_I,
        'j' => KeyCode::KEY_J,
        'k' => KeyCode::KEY_K,
        'l' => KeyCode::KEY_L,
        'm' => KeyCode::KEY_M,
        'n' => KeyCode::KEY_N,
        'o' => KeyCode::KEY_O,
        'p' => KeyCode::KEY_P,
        'q' => KeyCode::KEY_Q,
        'r' => KeyCode::KEY_R,
        's' => KeyCode::KEY_S,
        't' => KeyCode::KEY_T,
        'u' => KeyCode::KEY_U,
        'v' => KeyCode::KEY_V,
        'w' => KeyCode::KEY_W,
        'x' => KeyCode::KEY_X,
        'y' => KeyCode::KEY_Y,
        'z' => KeyCode::KEY_Z,
        '0' => return Some((KeyCode::KEY_0, false)),
        '1' => return Some((KeyCode::KEY_1, false)),
        '2' => return Some((KeyCode::KEY_2, false)),
        '3' => return Some((KeyCode::KEY_3, false)),
        '4' => return Some((KeyCode::KEY_4, false)),
        '5' => return Some((KeyCode::KEY_5, false)),
        '6' => return Some((KeyCode::KEY_6, false)),
        '7' => return Some((KeyCode::KEY_7, false)),
        '8' => return Some((KeyCode::KEY_8, false)),
        '9' => return Some((KeyCode::KEY_9, false)),
        ' ' => return Some((KeyCode::KEY_SPACE, false)),
        '/' => return Some((KeyCode::KEY_SLASH, false)),
        '-' => return Some((KeyCode::KEY_MINUS, false)),
        '.' => return Some((KeyCode::KEY_DOT, false)),
        ',' => return Some((KeyCode::KEY_COMMA, false)),
        '_' => return Some((KeyCode::KEY_MINUS, true)),
        ':' => return Some((KeyCode::KEY_SEMICOLON, true)),
        ';' => return Some((KeyCode::KEY_SEMICOLON, false)),
        _ => return None,
    };
    Some((key, shift))
}

/// Все коды, которые устройство должно уметь эмитить (регистрируются при создании).
fn all_keycodes() -> Vec<KeyCode> {
    let mut v = vec![
        KeyCode::KEY_SPACE,
        KeyCode::KEY_ENTER,
        KeyCode::KEY_ESC,
        KeyCode::KEY_TAB,
        KeyCode::KEY_BACKSPACE,
        KeyCode::KEY_UP,
        KeyCode::KEY_DOWN,
        KeyCode::KEY_LEFT,
        KeyCode::KEY_RIGHT,
        KeyCode::KEY_PAGEUP,
        KeyCode::KEY_PAGEDOWN,
        KeyCode::KEY_LEFTSHIFT,
        KeyCode::KEY_LEFTCTRL,
        KeyCode::KEY_LEFTALT,
        KeyCode::KEY_LEFTMETA,
        KeyCode::KEY_DELETE,
        KeyCode::KEY_INSERT,
        KeyCode::KEY_HOME,
        KeyCode::KEY_END,
        KeyCode::KEY_SLASH,
        KeyCode::KEY_MINUS,
        KeyCode::KEY_DOT,
        KeyCode::KEY_COMMA,
        KeyCode::KEY_SEMICOLON,
    ];
    for c in b'a'..=b'z' {
        if let Some((k, _)) = char_to_code(c as char) {
            v.push(k);
        }
    }
    for c in b'0'..=b'9' {
        if let Some((k, _)) = char_to_code(c as char) {
            v.push(k);
        }
    }
    v
}
