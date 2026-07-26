//! Платформо-нейтральное представление клавиш и аккордов.
//! Бэкенды инжектора (uinput на Linux, будущие — на win/mac) мапят Key в свои коды.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Space,
    Enter,
    Esc,
    Tab,
    Backspace,
    Delete,
    Insert,
    Home,
    End,
    Up,
    Down,
    Left,
    Right,
    PageUp,
    PageDown,
    Slash,
    // Модификаторы
    Ctrl,
    Alt,
    Shift,
    Super,
    /// Обычный печатный символ.
    Char(char),
}

/// Имя клавиши → нейтральный Key. Одиночный символ трактуется как Char.
pub fn key_from_name(name: &str) -> Option<Key> {
    let n = name.trim().to_lowercase();
    Some(match n.as_str() {
        "space" | "пробел" => Key::Space,
        "enter" | "return" => Key::Enter,
        "esc" | "escape" => Key::Esc,
        "tab" => Key::Tab,
        "backspace" => Key::Backspace,
        "delete" | "del" => Key::Delete,
        "insert" | "ins" => Key::Insert,
        "home" => Key::Home,
        "end" => Key::End,
        "up" => Key::Up,
        "down" => Key::Down,
        "left" => Key::Left,
        "right" => Key::Right,
        "pageup" => Key::PageUp,
        "pagedown" => Key::PageDown,
        "slash" => Key::Slash,
        "ctrl" | "control" => Key::Ctrl,
        "alt" => Key::Alt,
        "shift" => Key::Shift,
        "super" | "meta" | "win" => Key::Super,
        _ => {
            let mut ch = n.chars();
            match (ch.next(), ch.next()) {
                (Some(c), None) => Key::Char(c),
                _ => return None,
            }
        }
    })
}

/// Аккорд: модификаторы (держатся) + основная клавиша (тап).
pub struct Chord {
    pub mods: Vec<Key>,
    pub key: Key,
}

/// Парсит "ctrl+u", "ctrl+shift+left", "enter", "a" → Chord.
pub fn parse_chord(spec: &str) -> Option<Chord> {
    let parts: Vec<&str> = spec
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let (&key_str, mods_str) = parts.split_last()?;
    let key = key_from_name(key_str)?;
    let mut mods = Vec::new();
    for m in mods_str {
        match key_from_name(m) {
            Some(k) => mods.push(k),
            None => {
                log::warn!("аккорд «{spec}»: неизвестный модификатор {m:?}");
                return None;
            }
        }
    }
    Some(Chord { mods, key })
}
