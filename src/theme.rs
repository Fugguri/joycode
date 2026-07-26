//! Палитра и применение темы. Идентичность «IDE/терминал»: графит + янтарный акцент.
use egui::Color32;

pub struct Palette {
    pub bg: Color32,       // фон приложения
    pub card: Color32,     // панели/карточки
    pub card2: Color32,    // вторичная поверхность (неактивные виджеты)
    pub card_hover: Color32,
    pub inset: Color32,    // «утопленный» фон: лог, поля ввода
    pub border: Color32,
    pub text: Color32,
    pub text_dim: Color32,
    pub accent: Color32, // янтарный — единственный акцент
    pub err: Color32,
}

pub fn palette(dark: bool) -> Palette {
    if dark {
        Palette {
            bg: Color32::from_rgb(0x14, 0x16, 0x1B),
            card: Color32::from_rgb(0x1B, 0x1E, 0x25),
            card2: Color32::from_rgb(0x23, 0x27, 0x30),
            card_hover: Color32::from_rgb(0x2A, 0x2F, 0x3A),
            inset: Color32::from_rgb(0x0E, 0x10, 0x14),
            border: Color32::from_rgb(0x2C, 0x31, 0x3B),
            text: Color32::from_rgb(0xD6, 0xDA, 0xE0),
            text_dim: Color32::from_rgb(0x7C, 0x84, 0x8F),
            accent: Color32::from_rgb(0xE8, 0xA3, 0x3C),
            err: Color32::from_rgb(0xE5, 0x67, 0x5B),
        }
    } else {
        Palette {
            bg: Color32::from_rgb(0xEC, 0xEE, 0xF1),
            card: Color32::from_rgb(0xFF, 0xFF, 0xFF),
            card2: Color32::from_rgb(0xF1, 0xF3, 0xF6),
            card_hover: Color32::from_rgb(0xE9, 0xEC, 0xF0),
            inset: Color32::from_rgb(0xF6, 0xF7, 0xF9),
            border: Color32::from_rgb(0xD8, 0xDC, 0xE2),
            text: Color32::from_rgb(0x1B, 0x1F, 0x26),
            text_dim: Color32::from_rgb(0x6A, 0x72, 0x7C),
            accent: Color32::from_rgb(0xC5, 0x7A, 0x16),
            err: Color32::from_rgb(0xC0, 0x39, 0x2B),
        }
    }
}

/// Применяет визуалы и метрики к контексту egui.
pub fn apply(ctx: &egui::Context, dark: bool) {
    let p = palette(dark);
    let mut v = if dark {
        egui::Visuals::dark()
    } else {
        egui::Visuals::light()
    };
    v.dark_mode = dark;
    v.panel_fill = p.bg;
    v.window_fill = p.card;
    v.extreme_bg_color = p.inset;
    v.faint_bg_color = p.card2;
    v.code_bg_color = p.inset;
    v.override_text_color = Some(p.text);
    v.hyperlink_color = p.accent;
    v.selection.bg_fill = p.accent.gamma_multiply(0.35);
    v.selection.stroke = egui::Stroke::new(1.0, p.accent);

    let rounding = egui::Rounding::same(8.0);
    v.window_rounding = egui::Rounding::same(12.0);

    let stroke_dim = egui::Stroke::new(1.0, p.text_dim);
    let stroke_text = egui::Stroke::new(1.0, p.text);
    let stroke_border = egui::Stroke::new(1.0, p.border);

    // Неинтерактивные (label-фон панелей)
    v.widgets.noninteractive.bg_fill = p.card;
    v.widgets.noninteractive.weak_bg_fill = p.card;
    v.widgets.noninteractive.bg_stroke = stroke_border;
    v.widgets.noninteractive.fg_stroke = stroke_dim;
    v.widgets.noninteractive.rounding = rounding;

    // Неактивные кнопки/комбобоксы
    v.widgets.inactive.bg_fill = p.card2;
    v.widgets.inactive.weak_bg_fill = p.card2;
    v.widgets.inactive.bg_stroke = stroke_border;
    v.widgets.inactive.fg_stroke = stroke_text;
    v.widgets.inactive.rounding = rounding;

    // Наведение
    v.widgets.hovered.bg_fill = p.card_hover;
    v.widgets.hovered.weak_bg_fill = p.card_hover;
    v.widgets.hovered.bg_stroke = egui::Stroke::new(1.0, p.accent);
    v.widgets.hovered.fg_stroke = stroke_text;
    v.widgets.hovered.rounding = rounding;

    // Нажатие/активно
    v.widgets.active.bg_fill = p.accent;
    v.widgets.active.weak_bg_fill = p.accent;
    v.widgets.active.bg_stroke = egui::Stroke::new(1.0, p.accent);
    v.widgets.active.fg_stroke = egui::Stroke::new(1.0, p.bg);
    v.widgets.active.rounding = rounding;

    // Открытый комбобокс
    v.widgets.open.bg_fill = p.card2;
    v.widgets.open.weak_bg_fill = p.card2;
    v.widgets.open.bg_stroke = egui::Stroke::new(1.0, p.accent);
    v.widgets.open.fg_stroke = stroke_text;
    v.widgets.open.rounding = rounding;

    ctx.set_visuals(v);

    let mut style = (*ctx.style()).clone();
    style.spacing.item_spacing = egui::vec2(10.0, 8.0);
    style.spacing.button_padding = egui::vec2(12.0, 7.0);
    style.spacing.window_margin = egui::Margin::same(16.0);
    style.spacing.menu_margin = egui::Margin::same(8.0);
    style.spacing.interact_size.y = 30.0;

    use egui::{FontFamily, FontId, TextStyle};
    style.text_styles = [
        (TextStyle::Heading, FontId::new(21.0, FontFamily::Proportional)),
        (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Monospace, FontId::new(13.0, FontFamily::Monospace)),
        (TextStyle::Button, FontId::new(14.0, FontFamily::Proportional)),
        (TextStyle::Small, FontId::new(11.0, FontFamily::Proportional)),
    ]
    .into();
    ctx.set_style(style);
}
