//! Маппинг кнопок геймпада → действия. Хранится в bindings.toml рядом с бинарём.
use crate::actions::Action;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Список кнопок геймпада: (имя gilrs::Button через Debug, человекочитаемый ярлык Xbox).
/// Имя слева — то, что приходит из `format!("{:?}", button)` в движке.
pub const BUTTONS: &[(&str, &str)] = &[
    ("LeftTrigger2", "LT (L2)"),
    ("RightTrigger2", "RT (R2)"),
    ("LeftTrigger", "LB (L1)"),
    ("RightTrigger", "RB (R1)"),
    ("South", "A"),
    ("East", "B"),
    ("West", "X"),
    ("North", "Y"),
    ("Select", "Back"),
    ("Start", "Start"),
    ("Mode", "Guide"),
    ("DPadUp", "D-Pad ↑"),
    ("DPadDown", "D-Pad ↓"),
    ("DPadLeft", "D-Pad ←"),
    ("DPadRight", "D-Pad →"),
    ("LeftThumb", "L3 (стик)"),
    ("RightThumb", "R3 (стик)"),
    // Виртуальные кнопки стиков — отклонение оси за порог трактуется как нажатие.
    ("LeftStickUp", "L-стик ↑"),
    ("LeftStickDown", "L-стик ↓"),
    ("LeftStickLeft", "L-стик ←"),
    ("LeftStickRight", "L-стик →"),
    ("RightStickUp", "R-стик ↑"),
    ("RightStickDown", "R-стик ↓"),
    ("RightStickLeft", "R-стик ←"),
    ("RightStickRight", "R-стик →"),
];

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct Bindings {
    /// имя кнопки (gilrs Debug) → действие
    pub map: HashMap<String, Action>,
}

impl Bindings {
    /// Дефолтный маппинг: push-to-talk + подтверждения + навигация.
    pub fn default_map() -> Self {
        let mut map = HashMap::new();
        // Верхние кнопки — управление окнами и правка текста.
        map.insert("LeftTrigger".into(), Action::Key { key: "super+alt+left".into() });
        map.insert("RightTrigger".into(), Action::Key { key: "super+alt+right".into() });
        map.insert("LeftTrigger2".into(), Action::Hold { key: "backspace".into() });
        map.insert("RightTrigger2".into(), Action::Hold { key: "delete".into() });
        // Лицевые кнопки.
        map.insert("South".into(), Action::Key { key: "enter".into() });
        map.insert("East".into(), Action::Key { key: "esc".into() });
        map.insert("West".into(), Action::Key { key: "space".into() });
        map.insert("North".into(), Action::Text { text: "/".into(), enter: false });
        map.insert("Select".into(), Action::Key { key: "super".into() });
        // Навигация: крестовина + левый стик — стрелки, правый стик — скролл.
        map.insert("DPadUp".into(), Action::Key { key: "up".into() });
        map.insert("DPadDown".into(), Action::Key { key: "down".into() });
        map.insert("LeftStickUp".into(), Action::Key { key: "up".into() });
        map.insert("LeftStickDown".into(), Action::Key { key: "down".into() });
        map.insert("LeftStickLeft".into(), Action::Key { key: "left".into() });
        map.insert("LeftStickRight".into(), Action::Key { key: "right".into() });
        map.insert("RightStickUp".into(), Action::Key { key: "pageup".into() });
        map.insert("RightStickDown".into(), Action::Key { key: "pagedown".into() });
        Bindings { map }
    }

    pub fn get(&self, button: &str) -> Action {
        self.map.get(button).cloned().unwrap_or(Action::None)
    }

    pub fn set(&mut self, button: &str, action: Action) {
        self.map.insert(button.to_string(), action);
    }

    /// Путь к конфигу рядом с исполняемым файлом (fallback — текущая папка).
    /// Папка конфига по XDG: $XDG_CONFIG_HOME/joycode или ~/.config/joycode.
    pub fn config_dir() -> PathBuf {
        let base = std::env::var_os("XDG_CONFIG_HOME")
            .map(PathBuf::from)
            .filter(|p| p.is_absolute())
            .unwrap_or_else(|| {
                let home = std::env::var_os("HOME")
                    .map(PathBuf::from)
                    .unwrap_or_else(|| PathBuf::from("."));
                home.join(".config")
            });
        base.join("joycode")
    }

    pub fn default_path() -> PathBuf {
        Self::config_dir().join("bindings.toml")
    }

    /// Старое расположение конфига (рядом с бинарём) — для одноразовой миграции.
    fn legacy_path() -> Option<PathBuf> {
        std::env::current_exe()
            .ok()
            .and_then(|p| p.parent().map(|d| d.join("bindings.toml")))
    }

    /// Загружает конфиг. Если его нет — переносит старый (рядом с бинарём),
    /// а при отсутствии и его создаёт дефолтный. Папку XDG создаёт при нужде.
    pub fn load_or_create(path: &Path) -> anyhow::Result<Self> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        if path.exists() {
            let raw = std::fs::read_to_string(path)?;
            let b: Bindings = toml::from_str(&raw)?;
            log::info!("конфиг загружен: {}", path.display());
            return Ok(b);
        }

        // Миграция: старый конфиг рядом с бинарём → XDG.
        if let Some(legacy) = Self::legacy_path() {
            if legacy != path && legacy.exists() {
                let raw = std::fs::read_to_string(&legacy)?;
                let b: Bindings = toml::from_str(&raw)?;
                b.save(path)?;
                log::info!("конфиг перенесён из {} → {}", legacy.display(), path.display());
                return Ok(b);
            }
        }

        let b = Bindings::default_map();
        b.save(path)?;
        log::info!("конфиг не найден, создан дефолтный: {}", path.display());
        Ok(b)
    }

    pub fn save(&self, path: &Path) -> anyhow::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let raw = toml::to_string_pretty(self)?;
        std::fs::write(path, raw)?;
        log::info!("конфиг сохранён: {}", path.display());
        Ok(())
    }
}
