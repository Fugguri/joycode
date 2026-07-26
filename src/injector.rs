//! Абстракция впрыска клавиш. Трейт `KeyInjector` реализуется под каждую ОС;
//! `new()` возвращает бэкенд для текущей платформы.
use crate::keys::{Chord, Key};
use std::io;

pub trait KeyInjector {
    fn key_down(&mut self, key: Key) -> io::Result<()>;
    fn key_up(&mut self, key: Key) -> io::Result<()>;
    fn key_tap(&mut self, key: Key) -> io::Result<()>;
    /// Тап аккорда: зажать модификаторы → тап клавиши → отпустить.
    fn tap_chord(&mut self, chord: &Chord) -> io::Result<()>;
    /// Впечатать строку посимвольно.
    fn type_text(&mut self, text: &str) -> io::Result<()>;
}

#[cfg(target_os = "linux")]
mod uinput;

#[cfg(not(target_os = "linux"))]
mod enigo_backend;

/// Создаёт инжектор под текущую ОС.
/// Linux — нативный uinput (работает под Wayland); остальные — enigo.
#[cfg(target_os = "linux")]
pub fn new() -> io::Result<Box<dyn KeyInjector>> {
    Ok(Box::new(uinput::UinputInjector::new()?))
}

#[cfg(not(target_os = "linux"))]
pub fn new() -> io::Result<Box<dyn KeyInjector>> {
    Ok(Box::new(enigo_backend::EnigoInjector::new()?))
}
