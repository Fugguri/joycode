//! Виртуальная клавиатура поверх /dev/uinput.
//! Работает на уровне ядра — впрыснутые клавиши приходят в сфокусированное
//! окно независимо от композитора (X11/Wayland).
use crate::actions;
use evdev::{uinput::VirtualDevice, AttributeSet, EventType, InputEvent, KeyCode};
use std::io;

pub struct Injector {
    device: VirtualDevice,
}

impl Injector {
    /// Создаёт uinput-устройство. Требует rw-доступа к /dev/uinput.
    pub fn new() -> io::Result<Self> {
        let mut keys = AttributeSet::<KeyCode>::new();
        for k in actions::all_keycodes() {
            keys.insert(k);
        }
        let device = VirtualDevice::builder()?
            .name("joycode virtual keyboard")
            .with_keys(&keys)?
            .build()?;
        log::info!("uinput-клавиатура создана, зарегистрировано {} клавиш", actions::all_keycodes().len());
        Ok(Self { device })
    }

    fn emit(&mut self, key: KeyCode, value: i32) -> io::Result<()> {
        let ev = InputEvent::new(EventType::KEY.0, key.code(), value);
        self.device.emit(&[ev])
    }

    pub fn key_down(&mut self, key: KeyCode) -> io::Result<()> {
        self.emit(key, 1)
    }

    pub fn key_up(&mut self, key: KeyCode) -> io::Result<()> {
        self.emit(key, 0)
    }

    pub fn key_tap(&mut self, key: KeyCode) -> io::Result<()> {
        self.emit(key, 1)?;
        self.emit(key, 0)
    }

    /// Тап аккорда: зажать модификаторы → тап основной клавиши → отпустить в обратном порядке.
    /// Пример: ctrl+u очищает строку ввода в терминале.
    pub fn tap_chord(&mut self, chord: &actions::Chord) -> io::Result<()> {
        for m in &chord.mods {
            self.emit(*m, 1)?;
        }
        self.emit(chord.key, 1)?;
        self.emit(chord.key, 0)?;
        for m in chord.mods.iter().rev() {
            self.emit(*m, 0)?;
        }
        Ok(())
    }

    /// Впечатывает строку посимвольно (US-раскладка). Символы без маппинга — пропуск с WARNING.
    pub fn type_text(&mut self, text: &str) -> io::Result<()> {
        for c in text.chars() {
            match actions::char_to_key(c) {
                Some((key, shift)) => {
                    if shift {
                        self.emit(KeyCode::KEY_LEFTSHIFT, 1)?;
                    }
                    self.emit(key, 1)?;
                    self.emit(key, 0)?;
                    if shift {
                        self.emit(KeyCode::KEY_LEFTSHIFT, 0)?;
                    }
                }
                None => log::warn!("type_text: символ {c:?} не маппится в keycode, пропущен"),
            }
        }
        Ok(())
    }
}
