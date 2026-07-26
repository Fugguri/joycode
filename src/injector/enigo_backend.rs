//! Кроссплатформенный бэкенд впрыска через `enigo`.
//! Windows — SendInput, macOS — CGEvent (нужно разрешение Accessibility), BSD — X11.
//! Linux использует нативный uinput (см. uinput.rs), сюда не попадает.
use crate::injector::KeyInjector;
use crate::keys::{Chord, Key};
use enigo::{Direction, Enigo, Key as EKey, Keyboard, Settings};
use std::io;

pub struct EnigoInjector {
    enigo: Enigo,
}

impl EnigoInjector {
    pub fn new() -> io::Result<Self> {
        let enigo = Enigo::new(&Settings::default()).map_err(to_io)?;
        log::info!("enigo-инжектор создан");
        Ok(Self { enigo })
    }

    fn send(&mut self, key: Key, dir: Direction) -> io::Result<()> {
        match to_ekey(key) {
            Some(k) => self.enigo.key(k, dir).map_err(to_io),
            None => {
                log::warn!("enigo: клавиша {key:?} недоступна на этой платформе, пропущена");
                Ok(())
            }
        }
    }
}

impl KeyInjector for EnigoInjector {
    fn key_down(&mut self, key: Key) -> io::Result<()> {
        self.send(key, Direction::Press)
    }

    fn key_up(&mut self, key: Key) -> io::Result<()> {
        self.send(key, Direction::Release)
    }

    fn key_tap(&mut self, key: Key) -> io::Result<()> {
        self.send(key, Direction::Click)
    }

    fn tap_chord(&mut self, chord: &Chord) -> io::Result<()> {
        for m in &chord.mods {
            self.send(*m, Direction::Press)?;
        }
        let r = self.send(chord.key, Direction::Click);
        // Модификаторы отпускаем в любом случае, чтобы не залипли.
        for m in chord.mods.iter().rev() {
            let _ = self.send(*m, Direction::Release);
        }
        r
    }

    fn type_text(&mut self, text: &str) -> io::Result<()> {
        self.enigo.text(text).map_err(to_io)
    }
}

fn to_io<E: std::fmt::Display>(e: E) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e.to_string())
}

/// Нейтральный Key → enigo Key. None — если клавиши нет на этой платформе.
fn to_ekey(key: Key) -> Option<EKey> {
    Some(match key {
        Key::Space => EKey::Space,
        Key::Enter => EKey::Return,
        Key::Esc => EKey::Escape,
        Key::Tab => EKey::Tab,
        Key::Backspace => EKey::Backspace,
        Key::Delete => EKey::Delete,
        Key::Insert => return insert_ekey(), // есть только под Windows
        Key::Home => EKey::Home,
        Key::End => EKey::End,
        Key::Up => EKey::UpArrow,
        Key::Down => EKey::DownArrow,
        Key::Left => EKey::LeftArrow,
        Key::Right => EKey::RightArrow,
        Key::PageUp => EKey::PageUp,
        Key::PageDown => EKey::PageDown,
        Key::Slash => EKey::Unicode('/'),
        Key::Ctrl => EKey::Control,
        Key::Alt => EKey::Alt,
        Key::Shift => EKey::Shift,
        Key::Super => EKey::Meta,
        Key::Char(c) => EKey::Unicode(c),
    })
}

#[cfg(target_os = "windows")]
fn insert_ekey() -> Option<EKey> {
    Some(EKey::Insert)
}

#[cfg(not(target_os = "windows"))]
fn insert_ekey() -> Option<EKey> {
    None
}
