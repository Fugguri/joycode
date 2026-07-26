//! Загрузка моноширинного программистского шрифта (Hack) с фолбэками.
//! Нужен и для вида, и функционально: дефолтный шрифт egui не покрывает кириллицу.
use std::fs;

/// Кандидаты моно-шрифтов с кириллицей, по приоритету.
const CANDIDATES: &[&str] = &[
    "/usr/share/fonts/TTF/Hack-Regular.ttf",
    "/usr/share/fonts/hack/Hack-Regular.ttf",
    "/usr/share/fonts/noto/NotoSansMono-Regular.ttf",
    "/usr/share/fonts/liberation/LiberationMono-Regular.ttf",
    "/usr/share/fonts/dejavu/DejaVuSansMono.ttf",
    "/usr/share/fonts/TTF/DejaVuSansMono.ttf",
];

/// Ставит найденный моно-шрифт первым в обе семьи (Proportional и Monospace),
/// сохраняя дефолты egui как фолбэк для недостающих глифов.
pub fn install(ctx: &egui::Context) {
    let Some((path, bytes)) = CANDIDATES.iter().find_map(|p| {
        fs::read(p).ok().map(|b| (*p, b))
    }) else {
        log::warn!("моно-шрифт не найден среди кандидатов, останется дефолт egui");
        return;
    };

    let mut fonts = egui::FontDefinitions::default();
    fonts
        .font_data
        .insert("ui".to_owned(), egui::FontData::from_owned(bytes).into());
    for family in [egui::FontFamily::Proportional, egui::FontFamily::Monospace] {
        fonts
            .families
            .entry(family)
            .or_default()
            .insert(0, "ui".to_owned());
    }
    ctx.set_fonts(fonts);
    log::info!("UI-шрифт загружен: {path}");
}
