//! Действия, которые кнопка геймпада впрыскивает в активное окно.
//! Перевод клавиш в коды — в keys.rs (нейтрально) и бэкендах инжектора.
use serde::{Deserialize, Serialize};

/// Что делает кнопка при нажатии.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Action {
    /// Ничего не делать.
    None,
    /// Одиночное нажатие клавиши или аккорда (тап). Пример: enter, esc, ctrl+u.
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
