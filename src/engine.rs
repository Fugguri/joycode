//! Фоновый поток: читает геймпад через gilrs, мапит кнопки И стики в действия,
//! впрыскивает клавиши через uinput. Уважает флаг armed.
use crate::actions::Action;
use crate::config::Bindings;
use crate::injector::{self, KeyInjector};
use crate::keys::{key_from_name, parse_chord, Key};
use crate::state::SharedState;
use gilrs::{Axis, Event, EventType, Gilrs};
use parking_lot::Mutex;
use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

pub type SharedConfig = Arc<Mutex<Bindings>>;

/// За этим порогом отклонение стика считается «нажатием» виртуальной кнопки.
const DEADZONE: f32 = 0.6;
/// Задержка перед первым авто-повтором удерживаемого стика.
const REPEAT_DELAY: Duration = Duration::from_millis(350);
/// Интервал авто-повтора.
const REPEAT_INTERVAL: Duration = Duration::from_millis(90);

/// Запускает движок в отдельном потоке. Ошибки инициализации кладёт в state.error.
pub fn spawn(state: SharedState, config: SharedConfig) {
    thread::spawn(move || {
        if let Err(e) = run(state.clone(), config) {
            log::error!("движок остановлен: {e:?}");
            state.lock().error = Some(format!("{e}"));
        }
    });
}

fn run(state: SharedState, config: SharedConfig) -> anyhow::Result<()> {
    let mut gilrs = Gilrs::new().map_err(|e| anyhow::anyhow!("gilrs init: {e}"))?;

    {
        let mut s = state.lock();
        for (_id, gp) in gilrs.gamepads() {
            s.connected = true;
            s.gamepad_name = gp.name().to_string();
        }
    }

    let injector = match injector::new() {
        Ok(i) => i,
        Err(e) => {
            let msg = format!("не удалось создать инжектор клавиш: {e}");
            log::error!("{msg}");
            state.lock().error = Some(msg);
            return Err(e.into());
        }
    };
    thread::sleep(Duration::from_millis(200));
    state.lock().push_log("движок запущен, uinput готов");

    let mut eng = Engine {
        injector,
        state,
        config,
        active_axis_dirs: HashSet::new(),
        held_hold: HashSet::new(),
        repeats: HashMap::new(),
        prev_armed: false,
    };

    loop {
        while let Some(Event { event, .. }) = gilrs.next_event() {
            eng.handle(event);
        }
        eng.tick();
        thread::sleep(Duration::from_millis(8));
    }
}

struct Engine {
    injector: Box<dyn KeyInjector>,
    state: SharedState,
    config: SharedConfig,
    /// Виртуальные кнопки стиков, активные прямо сейчас (для детекта фронта по оси).
    active_axis_dirs: HashSet<String>,
    /// Логические кнопки с активным Hold — чтобы снять клавишу при отпускании/дизарме.
    held_hold: HashSet<String>,
    /// Логические кнопки на авто-повторе (стики) → время следующего срабатывания.
    repeats: HashMap<String, Instant>,
    prev_armed: bool,
}

impl Engine {
    fn armed(&self) -> bool {
        self.state.lock().armed
    }

    fn action_for(&self, button: &str) -> Action {
        self.config.lock().get(button)
    }

    fn handle(&mut self, event: EventType) {
        match event {
            EventType::ButtonPressed(btn, _) => self.on_press(&format!("{btn:?}")),
            EventType::ButtonReleased(btn, _) => self.on_release(&format!("{btn:?}")),
            EventType::AxisChanged(axis, val, _) => self.on_axis(axis, val),
            EventType::Connected => {
                let mut s = self.state.lock();
                s.connected = true;
                s.push_log("геймпад подключён");
            }
            EventType::Disconnected => {
                let mut s = self.state.lock();
                s.connected = false;
                s.push_log("геймпад отключён");
            }
            _ => {}
        }
    }

    /// Раз в цикл: авто-повтор удерживаемых стиков + снятие клавиш при выключении ARMED.
    fn tick(&mut self) {
        let armed = self.armed();

        // Переход ARMED → DISARMED: снять всё, что держим, чтобы клавиши не залипли.
        if self.prev_armed && !armed {
            self.release_all_held();
        }
        self.prev_armed = armed;

        if !armed || self.repeats.is_empty() {
            return;
        }
        let now = Instant::now();
        let due: Vec<String> = self
            .repeats
            .iter()
            .filter(|(_, &next)| now >= next)
            .map(|(name, _)| name.clone())
            .collect();
        for name in due {
            let action = self.action_for(&name);
            if let Action::Key { key } = action {
                if let Some(chord) = parse_chord(&key) {
                    if let Err(e) = self.injector.tap_chord(&chord) {
                        log::error!("авто-повтор {name}: {e}");
                    }
                }
            }
            self.repeats.insert(name, now + REPEAT_INTERVAL);
        }
    }

    fn on_press(&mut self, name: &str) {
        self.state.lock().pressed.insert(name.to_string());
        if !self.armed() {
            return;
        }
        let action = self.action_for(name);
        self.apply_press(name, &action);
    }

    fn on_release(&mut self, name: &str) {
        self.state.lock().pressed.remove(name);
        self.repeats.remove(name);
        // Снятие Hold делаем всегда (даже если дизармили в процессе) — против залипания.
        if self.held_hold.remove(name) {
            let action = self.action_for(name);
            if let Action::Hold { key } = action {
                if let Some(k) = key_from_name(&key) {
                    self.state.lock().push_log(format!("HOLD ↑ {key}"));
                    if let Err(e) = self.injector.key_up(k) {
                        log::error!("отпускание {name}: {e}");
                    }
                }
            }
        }
    }

    fn apply_press(&mut self, name: &str, action: &Action) {
        let res = match action {
            Action::None => return,
            Action::Hold { key } => match key_from_name(key) {
                Some(k) => {
                    self.held_hold.insert(name.to_string());
                    self.state.lock().push_log(format!("HOLD ↓ {key}"));
                    self.injector.key_down(k)
                }
                None => {
                    log::warn!("Hold: неизвестная клавиша {key:?}");
                    return;
                }
            },
            Action::Key { key } => match parse_chord(key) {
                Some(chord) => {
                    self.state.lock().push_log(format!("KEY {key}"));
                    // Стики держатся → включаем авто-повтор.
                    if is_stick(name) {
                        self.repeats
                            .insert(name.to_string(), Instant::now() + REPEAT_DELAY);
                    }
                    self.injector.tap_chord(&chord)
                }
                None => {
                    log::warn!("Key: не разобрал «{key}»");
                    return;
                }
            },
            Action::Text { text, enter } => {
                self.state
                    .lock()
                    .push_log(format!("TEXT «{text}»{}", if *enter { " ⏎" } else { "" }));
                let r = self.injector.type_text(text);
                if r.is_ok() && *enter {
                    let _ = self.injector.key_tap(Key::Enter);
                }
                r
            }
        };
        if let Err(e) = res {
            log::error!("впрыск {name}: {e}");
            self.state.lock().push_log(format!("ОШИБКА впрыска: {e}"));
        }
    }

    /// Отклонение оси → набор активных виртуальных кнопок; фронты дают press/release.
    /// gilrs: +Y = вверх, +X = вправо.
    fn on_axis(&mut self, axis: Axis, val: f32) {
        for &name in axis_directions(axis) {
            let is_active = if name.ends_with("Up") || name.ends_with("Right") {
                val > DEADZONE
            } else {
                val < -DEADZONE
            };
            let was_active = self.active_axis_dirs.contains(name);
            if is_active && !was_active {
                self.active_axis_dirs.insert(name.to_string());
                self.on_press(name);
            } else if !is_active && was_active {
                self.active_axis_dirs.remove(name);
                self.on_release(name);
            }
        }
    }

    /// Снять все удерживаемые Hold-клавиши и сбросить авто-повторы.
    fn release_all_held(&mut self) {
        let held: Vec<String> = self.held_hold.iter().cloned().collect();
        for name in held {
            let action = self.action_for(&name);
            if let Action::Hold { key } = action {
                if let Some(k) = key_from_name(&key) {
                    let _ = self.injector.key_up(k);
                }
            }
        }
        self.held_hold.clear();
        self.repeats.clear();
        self.state.lock().push_log("DISARMED — все клавиши сняты");
    }
}

fn is_stick(name: &str) -> bool {
    name.starts_with("LeftStick") || name.starts_with("RightStick")
}

/// Имена виртуальных кнопок, которые может дать эта ось (знак проверяется в on_axis).
fn axis_directions(axis: Axis) -> &'static [&'static str] {
    match axis {
        Axis::LeftStickX => &["LeftStickRight", "LeftStickLeft"],
        Axis::LeftStickY => &["LeftStickUp", "LeftStickDown"],
        Axis::RightStickX => &["RightStickRight", "RightStickLeft"],
        Axis::RightStickY => &["RightStickUp", "RightStickDown"],
        _ => &[],
    }
}
