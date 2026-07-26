//! egui-интерфейс: верхняя панель с брендом/вкладками/переключателем темы
//! и три вкладки — Статус, Настройки, Инструкция. Тема: theme.rs.
use crate::actions::Action;
use crate::config::{Bindings, BUTTONS};
use crate::engine::SharedConfig;
use crate::state::SharedState;
use crate::theme::{self, Palette};
use egui::{Align, Align2, Color32, FontId, Frame, Layout, Margin, RichText, Sense, Stroke, Vec2};
use std::path::PathBuf;
use std::time::Duration;

#[derive(PartialEq, Clone, Copy)]
enum Tab {
    Status,
    Bindings,
    Help,
}

pub struct App {
    state: SharedState,
    config: SharedConfig,
    config_path: PathBuf,
    tab: Tab,
    dark: bool,
    save_notice: Option<String>,
}

impl App {
    pub fn new(
        cc: &eframe::CreationContext<'_>,
        state: SharedState,
        config: SharedConfig,
        config_path: PathBuf,
    ) -> Self {
        theme::apply(&cc.egui_ctx, true);
        Self {
            state,
            config,
            config_path,
            tab: Tab::Status,
            dark: true,
            save_notice: None,
        }
    }
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        ctx.request_repaint_after(Duration::from_millis(50));
        self.handle_shortcuts(ctx);
        self.handle_close(ctx);
        let p = theme::palette(self.dark);

        egui::TopBottomPanel::top("bar")
            .frame(Frame::none().fill(p.card).inner_margin(Margin::symmetric(16.0, 11.0)))
            .show(ctx, |ui| self.ui_topbar(ui, &p));

        egui::CentralPanel::default()
            .frame(Frame::none().fill(p.bg).inner_margin(Margin::same(16.0)))
            .show(ctx, |ui| match self.tab {
                Tab::Status => self.ui_status(ui, &p),
                Tab::Bindings => self.ui_bindings(ui, &p),
                Tab::Help => self.ui_help(ui, &p),
            });
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        crate::single_instance::cleanup();
    }
}

impl App {
    /// Горячие клавиши окна: 1/2/3 — вкладки, Space — тумблер, T — тема.
    /// Игнорируются, когда фокус в текстовом поле.
    fn handle_shortcuts(&mut self, ctx: &egui::Context) {
        if ctx.wants_keyboard_input() {
            return;
        }
        let mut goto = None;
        let mut toggle_theme = false;
        let mut toggle_arm = false;
        ctx.input(|i| {
            if i.key_pressed(egui::Key::Num1) {
                goto = Some(Tab::Status);
            }
            if i.key_pressed(egui::Key::Num2) {
                goto = Some(Tab::Bindings);
            }
            if i.key_pressed(egui::Key::Num3) {
                goto = Some(Tab::Help);
            }
            if i.key_pressed(egui::Key::T) {
                toggle_theme = true;
            }
            if i.key_pressed(egui::Key::Space) {
                toggle_arm = true;
            }
        });
        if let Some(t) = goto {
            self.tab = t;
        }
        if toggle_theme {
            self.dark = !self.dark;
            theme::apply(ctx, self.dark);
        }
        if toggle_arm {
            let mut s = self.state.lock();
            s.armed = !s.armed;
        }
    }

    /// Реальный выход: чистим сокет и выходим напрямую (без eframe-механики).
    fn quit(&self) -> ! {
        crate::single_instance::cleanup();
        std::process::exit(0);
    }

    /// Ctrl+Q — выход; закрытие окна (крестик) — сворачивание в фон (движок жив).
    fn handle_close(&mut self, ctx: &egui::Context) {
        // Ctrl + физическая клавиша Q (раскладко-независимо).
        let ctrl_q = ctx.input(|i| {
            i.modifiers.ctrl
                && i.events.iter().any(|e| {
                    matches!(
                        e,
                        egui::Event::Key { physical_key: Some(egui::Key::Q), pressed: true, .. }
                    )
                })
        });
        if ctrl_q {
            self.quit();
        }
        if ctx.input(|i| i.viewport().close_requested()) {
            ctx.send_viewport_cmd(egui::ViewportCommand::CancelClose);
            // Wayland не умеет set_visible(false) — сворачиваем (minimize).
            ctx.send_viewport_cmd(egui::ViewportCommand::Minimized(true));
            self.state
                .lock()
                .push_log("свёрнуто в фон · запусти joycode снова, чтобы открыть · Ctrl+Q — выход");
        }
    }

    fn ui_topbar(&mut self, ui: &mut egui::Ui, p: &Palette) {
        ui.horizontal(|ui| {
            // Бренд
            ui.label(RichText::new("›_").monospace().strong().size(19.0).color(p.accent));
            ui.label(RichText::new("joycode").strong().size(18.0).color(p.text));
            ui.add_space(14.0);

            // Вкладки
            for (tab, label) in [
                (Tab::Status, "Статус"),
                (Tab::Bindings, "Настройки"),
                (Tab::Help, "Инструкция"),
            ] {
                let selected = self.tab == tab;
                let txt = if selected {
                    RichText::new(label).color(p.accent).strong()
                } else {
                    RichText::new(label).color(p.text_dim)
                };
                if ui.selectable_label(selected, txt).clicked() {
                    self.tab = tab;
                }
            }

            // Правый край: тема + компактный статус ARMED
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if ui
                    .button(RichText::new("⏻").size(14.0))
                    .on_hover_text("Выход (Ctrl+Q)")
                    .clicked()
                {
                    self.quit();
                }
                ui.add_space(6.0);
                let icon = if self.dark { "☀" } else { "☾" };
                if ui
                    .button(RichText::new(icon).size(15.0))
                    .on_hover_text(if self.dark { "Светлая тема" } else { "Тёмная тема" })
                    .clicked()
                {
                    self.dark = !self.dark;
                    theme::apply(ui.ctx(), self.dark);
                }
                ui.add_space(6.0);
                let armed = self.state.lock().armed;
                armed_pill(ui, p, armed);
            });
        });
    }

    fn ui_status(&mut self, ui: &mut egui::Ui, p: &Palette) {
        let (connected, name, armed, error, pressed, log) = {
            let s = self.state.lock();
            (
                s.connected,
                s.gamepad_name.clone(),
                s.armed,
                s.error.clone(),
                s.pressed.clone(),
                s.log.iter().cloned().collect::<Vec<_>>(),
            )
        };

        if let Some(err) = error {
            card(ui, p, p.err, |ui| {
                ui.label(RichText::new(format!("⚠  {err}")).color(p.err).strong());
            });
            ui.add_space(10.0);
        }

        // Подключение
        ui.horizontal(|ui| {
            let (c, t) = if connected {
                (p.accent, format!("{name}"))
            } else {
                (p.text_dim, "геймпад не найден".to_string())
            };
            dot(ui, c, 5.0);
            ui.add_space(2.0);
            ui.label(RichText::new(t).color(if connected { p.text } else { p.text_dim }));
        });
        ui.add_space(12.0);

        // Фирменный переключатель ARMED
        if let Some(new_armed) = armed_switch(ui, p, armed) {
            self.state.lock().armed = new_armed;
        }
        ui.add_space(16.0);

        // Зажатые кнопки
        ui.label(RichText::new("АКТИВНЫЕ КНОПКИ").size(11.0).color(p.text_dim).strong());
        ui.add_space(6.0);
        ui.horizontal_wrapped(|ui| {
            let mut any = false;
            for (btn, label) in BUTTONS {
                if pressed.contains(*btn) {
                    any = true;
                    chip(ui, p, label, true);
                }
            }
            if !any {
                ui.label(RichText::new("ничего не нажато").color(p.text_dim).italics());
            }
        });
        ui.add_space(16.0);

        // Лог
        ui.label(RichText::new("ЛОГ").size(11.0).color(p.text_dim).strong());
        ui.add_space(6.0);
        Frame::none()
            .fill(p.inset)
            .rounding(10.0)
            .inner_margin(Margin::same(12.0))
            .stroke(Stroke::new(1.0, p.border))
            .show(ui, |ui| {
                egui::ScrollArea::vertical().max_height(190.0).auto_shrink([false, false]).show(ui, |ui| {
                    if log.is_empty() {
                        ui.label(RichText::new("— пусто —").color(p.text_dim).monospace());
                    }
                    for line in &log {
                        ui.label(RichText::new(line).monospace().size(12.0).color(p.text));
                    }
                });
            });
    }

    fn ui_bindings(&mut self, ui: &mut egui::Ui, p: &Palette) {
        ui.label(
            RichText::new("Кнопка → действие. Клавиши и аккорды: space, enter, esc, backspace, up/down, ctrl+u, ctrl+c, alt+left.")
                .color(p.text_dim)
                .size(12.5),
        );
        ui.add_space(8.0);

        // Панель действий — всегда на виду (над скроллом со списком).
        ui.horizontal(|ui| {
            if accent_button(ui, p, "Сохранить").clicked() {
                match self.config.lock().save(&self.config_path) {
                    Ok(()) => {
                        self.save_notice = Some(format!("сохранено → {}", self.config_path.display()))
                    }
                    Err(e) => self.save_notice = Some(format!("ошибка: {e}")),
                }
            }
            if ui.button("Сбросить к дефолту").clicked() {
                *self.config.lock() = Bindings::default_map();
                self.save_notice = Some("сброшено к дефолтному маппингу".into());
            }
            if let Some(notice) = &self.save_notice {
                ui.label(RichText::new(notice).color(p.text_dim).size(12.0));
            }
        });
        ui.add_space(8.0);

        let mut cfg = self.config.lock().clone();
        let mut changed = false;

        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            Frame::none()
                .fill(p.card)
                .rounding(10.0)
                .inner_margin(Margin::same(14.0))
                .stroke(Stroke::new(1.0, p.border))
                .show(ui, |ui| {
                    egui::Grid::new("bindings_grid")
                        .num_columns(3)
                        .striped(true)
                        .spacing([14.0, 9.0])
                        .show(ui, |ui| {
                            for (name, label) in BUTTONS {
                                ui.label(RichText::new(*label).color(p.text).monospace());

                                let mut action = cfg.get(name);
                                let mut kind = action.kind().to_string();
                                egui::ComboBox::from_id_salt(*name)
                                    .selected_text(kind_label(&kind))
                                    .width(120.0)
                                    .show_ui(ui, |ui| {
                                        for k in ["none", "key", "hold", "text"] {
                                            ui.selectable_value(&mut kind, k.to_string(), kind_label(k));
                                        }
                                    });
                                if kind != action.kind() {
                                    action = default_for_kind(&kind, &action);
                                    changed = true;
                                }

                                match &mut action {
                                    Action::None => {
                                        ui.label(RichText::new("—").color(p.text_dim));
                                    }
                                    Action::Key { key } | Action::Hold { key } => {
                                        if ui.text_edit_singleline(key).changed() {
                                            changed = true;
                                        }
                                    }
                                    Action::Text { text, enter } => {
                                        ui.horizontal(|ui| {
                                            if ui.text_edit_singleline(text).changed() {
                                                changed = true;
                                            }
                                            if ui.checkbox(enter, "⏎").changed() {
                                                changed = true;
                                            }
                                        });
                                    }
                                }
                                ui.end_row();

                                cfg.set(name, action);
                            }
                        });
                });
        });

        if changed {
            *self.config.lock() = cfg;
        }
    }

    fn ui_help(&mut self, ui: &mut egui::Ui, p: &Palette) {
        egui::ScrollArea::vertical().auto_shrink([false, false]).show(ui, |ui| {
            help_section(ui, p, "Как пользоваться", &[
                "1. Подключи Xbox-геймпад (провод или Bluetooth).",
                "2. Поставь терминал с Claude Code в фокус.",
                "3. На вкладке «Статус» включи тумблер.",
                "4. Управляй диалогом кнопками, правь текст LT/RT, листай окна LB/RB.",
                "5. Push-to-talk (голос) — повесь сам: действие «Удержание» + space.",
            ]);
            help_section(ui, p, "Дефолтные кнопки", &[
                "LB / RB  — super+alt+←/→ (переключение окон)",
                "LT / RT  — держать = Backspace / Delete",
                "A / B    — Enter / Esc",
                "X        — Space",
                "Y        — «/» (меню команд)",
                "Back     — super (обзор GNOME)",
                "D-Pad, L-стик — стрелки (с авто-повтором)",
                "R-стик   — PageUp / PageDown (скролл)",
            ]);
            help_section(ui, p, "Системные клавиши (аккорды)", &[
                "В поле клавиши можно писать комбинации:",
                "ctrl+u   — очистить строку ввода",
                "ctrl+c   — прервать",
                "ctrl+l   — очистить экран",
                "alt+left — на слово влево",
            ]);
            help_section(ui, p, "Горячие клавиши окна", &[
                "1 / 2 / 3 — вкладки Статус / Настройки / Инструкция",
                "Space     — включить/выключить систему",
                "T         — сменить тему",
                "Ctrl+Q    — выход",
            ]);
            help_section(ui, p, "Фоновый режим", &[
                "Закрытие окна сворачивает приложение в фон —",
                "движок продолжает работать, геймпад активен.",
                "Открыть снова — запусти joycode ещё раз.",
                "Полный выход — Ctrl+Q.",
            ]);
            help_section(ui, p, "Ограничения", &[
                "• Клавиши идут в активное окно — держи фокус на терминале.",
                "• space/enter/esc/стрелки/аккорды раскладко-независимы.",
                "• Текст («/», макросы) печатается скан-кодами US — для латиницы",
                "  переключи системную раскладку на английскую.",
                "• Тумблер выкл — впрыск полностью заблокирован.",
            ]);
        });
    }
}

// ── Компоненты ─────────────────────────────────────────────────────────────

/// Фирменный тумблер ARMED. Возвращает новое значение, если кликнули.
fn armed_switch(ui: &mut egui::Ui, p: &Palette, armed: bool) -> Option<bool> {
    let size = Vec2::new(ui.available_width(), 78.0);
    let (rect, resp) = ui.allocate_exact_size(size, Sense::click());
    let hovered = resp.hovered();
    if hovered {
        ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
    }

    let painter = ui.painter();
    let bg = if armed {
        p.card_hover
    } else if hovered {
        p.card2
    } else {
        p.card
    };
    let border = if armed { p.accent } else { p.border };
    painter.rect_filled(rect, 12.0, bg);
    painter.rect_stroke(rect, 12.0, Stroke::new(if armed { 1.5 } else { 1.0 }, border));

    // LED
    let led = rect.left_center() + Vec2::new(36.0, 0.0);
    if armed {
        painter.circle_filled(led, 22.0, p.accent.gamma_multiply(0.22));
        painter.circle_filled(led, 14.0, p.accent.gamma_multiply(0.45));
        painter.circle_filled(led, 9.0, p.accent);
    } else {
        painter.circle_stroke(led, 9.0, Stroke::new(2.0, p.text_dim));
    }

    // Тексты
    let cy = rect.center().y;
    let title = if armed { "СИСТЕМА АКТИВНА" } else { "ВЫКЛЮЧЕНО" };
    painter.text(
        egui::pos2(rect.min.x + 66.0, cy - 9.0),
        Align2::LEFT_CENTER,
        title,
        FontId::proportional(16.0),
        if armed { p.accent } else { p.text },
    );
    let sub = if armed {
        "нажми, чтобы выключить впрыск"
    } else {
        "нажми, чтобы включить впрыск"
    };
    painter.text(
        egui::pos2(rect.min.x + 66.0, cy + 12.0),
        Align2::LEFT_CENTER,
        sub,
        FontId::proportional(11.5),
        p.text_dim,
    );
    // Правый бейдж
    painter.text(
        egui::pos2(rect.max.x - 18.0, cy),
        Align2::RIGHT_CENTER,
        if armed { "ARMED" } else { "OFF" },
        FontId::monospace(15.0),
        if armed { p.accent } else { p.text_dim },
    );

    resp.clicked().then_some(!armed)
}

/// Компактный статус в верхней панели.
fn armed_pill(ui: &mut egui::Ui, p: &Palette, armed: bool) {
    let (fill, fg, text) = if armed {
        (p.accent.gamma_multiply(0.18), p.accent, "ARMED")
    } else {
        (p.card2, p.text_dim, "OFF")
    };
    Frame::none()
        .fill(fill)
        .rounding(999.0)
        .inner_margin(Margin::symmetric(10.0, 4.0))
        .show(ui, |ui| {
            ui.horizontal(|ui| {
                dot(ui, fg, 4.0);
                ui.label(RichText::new(text).monospace().size(11.0).color(fg).strong());
            });
        });
}

/// Кружок-индикатор.
fn dot(ui: &mut egui::Ui, color: Color32, r: f32) {
    let (rect, _) = ui.allocate_exact_size(Vec2::splat(r * 2.0 + 2.0), Sense::hover());
    ui.painter().circle_filled(rect.center(), r, color);
}

/// Чип активной кнопки.
fn chip(ui: &mut egui::Ui, p: &Palette, label: &str, active: bool) {
    let (fill, fg, stroke) = if active {
        (p.accent.gamma_multiply(0.16), p.accent, p.accent)
    } else {
        (p.card2, p.text_dim, p.border)
    };
    Frame::none()
        .fill(fill)
        .rounding(7.0)
        .inner_margin(Margin::symmetric(9.0, 4.0))
        .stroke(Stroke::new(1.0, stroke))
        .show(ui, |ui| {
            ui.label(RichText::new(label).monospace().size(12.0).color(fg).strong());
        });
}

/// Карточка с цветной левой рамкой (для ошибок/акцентов).
fn card(ui: &mut egui::Ui, p: &Palette, accent: Color32, add: impl FnOnce(&mut egui::Ui)) {
    Frame::none()
        .fill(p.card)
        .rounding(10.0)
        .inner_margin(Margin::same(12.0))
        .stroke(Stroke::new(1.0, accent))
        .show(ui, add);
}

/// Акцентная (янтарная) кнопка.
fn accent_button(ui: &mut egui::Ui, p: &Palette, label: &str) -> egui::Response {
    let text = RichText::new(label).color(p.bg).strong();
    ui.add(egui::Button::new(text).fill(p.accent).rounding(8.0))
}

fn help_section(ui: &mut egui::Ui, p: &Palette, title: &str, lines: &[&str]) {
    ui.add_space(4.0);
    ui.label(RichText::new(title).size(15.0).strong().color(p.accent));
    ui.add_space(4.0);
    for line in lines {
        ui.label(RichText::new(*line).monospace().size(12.5).color(p.text));
    }
    ui.add_space(12.0);
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "none" => "— ничего —",
        "key" => "Клавиша",
        "hold" => "Удержание",
        "text" => "Текст",
        _ => "?",
    }
}

fn default_for_kind(kind: &str, prev: &Action) -> Action {
    let key = match prev {
        Action::Key { key } | Action::Hold { key } => key.clone(),
        _ => "space".to_string(),
    };
    match kind {
        "key" => Action::Key { key },
        "hold" => Action::Hold { key },
        "text" => Action::Text { text: String::new(), enter: false },
        _ => Action::None,
    }
}
