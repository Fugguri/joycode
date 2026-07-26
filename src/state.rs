//! Общее состояние между фоновым движком и UI.
use parking_lot::Mutex;
use std::collections::{HashSet, VecDeque};
use std::sync::Arc;

const LOG_CAP: usize = 40;

#[derive(Default)]
pub struct AppState {
    pub connected: bool,
    pub gamepad_name: String,
    /// Кнопки, зажатые прямо сейчас (имена gilrs Debug).
    pub pressed: HashSet<String>,
    /// Armed — впрыск клавиш включён. Disarmed — движок читает геймпад, но ничего не шлёт.
    pub armed: bool,
    /// Ошибка инициализации движка/инжектора для показа в UI.
    pub error: Option<String>,
    /// Последние действия для лога в UI.
    pub log: VecDeque<String>,
}

impl AppState {
    pub fn push_log(&mut self, line: impl Into<String>) {
        self.log.push_front(line.into());
        while self.log.len() > LOG_CAP {
            self.log.pop_back();
        }
    }
}

pub type SharedState = Arc<Mutex<AppState>>;

pub fn new_state() -> SharedState {
    Arc::new(Mutex::new(AppState::default()))
}
