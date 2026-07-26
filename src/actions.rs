//! Действия, которые кнопка геймпада впрыскивает в активное окно,
//! и перевод человекочитаемых имён клавиш / символов в evdev KeyCode.
use evdev::KeyCode;
use serde::{Deserialize, Serialize};

/// Что делает кнопка при нажатии.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Ничего не делать.
    None,
    /// Одиночное нажатие клавиши (down+up). Пример: Enter, Esc, стрелки.
    Key { key: String },
    /// Удержание клавиши, пока держишь кнопку геймпада (push-to-talk).
    Hold { key: String },
    /// Впечатать строку и опционально нажать Enter (макрос слэш-команды).
    Text { text: String, enter: bool },
}

impl Default for Action {
    fn default() -> Self {
        Action::None
    }
}

impl Action {
    /// Короткое имя типа для UI-комбобокса.
    pub fn kind(&self) -> &'static str {
        match self {
            Action::None => "none",
            Action::Key { .. } => "key",
            Action::Hold { .. } => "hold",
            Action::Text { .. } => "text",
        }
    }
}

/// Перевод имени клавиши ("space", "enter", "esc", "up", "ctrl"...) в KeyCode.
/// Именованные клавиши и модификаторы раскладко-независимы.
pub fn keycode_from_name(name: &str) -> Option<KeyCode> {
    let n = name.trim().to_lowercase();
    Some(match n.as_str() {
        "space" | "пробел" => KeyCode::KEY_SPACE,
        "enter" | "return" => KeyCode::KEY_ENTER,
        "esc" | "escape" => KeyCode::KEY_ESC,
        "tab" => KeyCode::KEY_TAB,
        "backspace" => KeyCode::KEY_BACKSPACE,
        "delete" | "del" => KeyCode::KEY_DELETE,
        "insert" | "ins" => KeyCode::KEY_INSERT,
        "home" => KeyCode::KEY_HOME,
        "end" => KeyCode::KEY_END,
        "up" => KeyCode::KEY_UP,
        "down" => KeyCode::KEY_DOWN,
        "left" => KeyCode::KEY_LEFT,
        "right" => KeyCode::KEY_RIGHT,
        "pageup" => KeyCode::KEY_PAGEUP,
        "pagedown" => KeyCode::KEY_PAGEDOWN,
        "slash" => KeyCode::KEY_SLASH,
        // Модификаторы для аккордов (ctrl+u и т.п.)
        "ctrl" | "control" => KeyCode::KEY_LEFTCTRL,
        "alt" => KeyCode::KEY_LEFTALT,
        "shift" => KeyCode::KEY_LEFTSHIFT,
        "super" | "meta" | "win" => KeyCode::KEY_LEFTMETA,
        _ => return None,
    })
}

/// Разобранный аккорд: модификаторы (держатся) + основная клавиша (тап).
pub struct Chord {
    pub mods: Vec<KeyCode>,
    pub key: KeyCode,
}

/// Парсит "ctrl+u", "ctrl+shift+left", "enter", "a" → Chord.
/// Всё, кроме последнего токена — модификаторы; последний — основная клавиша
/// (ищется сначала по имени, затем как одиночный ASCII-символ).
pub fn parse_chord(spec: &str) -> Option<Chord> {
    let parts: Vec<&str> = spec
        .split('+')
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .collect();
    let (&key_str, mods_str) = parts.split_last()?;

    let key = keycode_from_name(key_str).or_else(|| {
        let mut ch = key_str.chars();
        match (ch.next(), ch.next()) {
            (Some(c), None) => char_to_key(c).map(|(k, _)| k),
            _ => None,
        }
    })?;

    let mut mods = Vec::new();
    for m in mods_str {
        match keycode_from_name(m) {
            Some(k) => mods.push(k),
            None => {
                log::warn!("аккорд «{spec}»: неизвестный модификатор {m:?}");
                return None;
            }
        }
    }
    Some(Chord { mods, key })
}

/// Перевод ASCII-символа в (KeyCode, нужен ли Shift) для впечатывания текста.
/// ВНИМАНИЕ: это скан-коды раскладки US. Для латинских слэш-команд активная
/// раскладка в системе должна быть US, иначе выйдут буквы другой раскладки.
pub fn char_to_key(c: char) -> Option<(KeyCode, bool)> {
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

/// Все KeyCode, которые виртуальная клавиатура обязана уметь эмитить.
/// Регистрируется один раз при создании uinput-устройства.
pub fn all_keycodes() -> Vec<KeyCode> {
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
    // Буквы a-z и цифры 0-9 — для текстовых макросов.
    for c in b'a'..=b'z' {
        if let Some((k, _)) = char_to_key(c as char) {
            v.push(k);
        }
    }
    for c in b'0'..=b'9' {
        if let Some((k, _)) = char_to_key(c as char) {
            v.push(k);
        }
    }
    v
}
