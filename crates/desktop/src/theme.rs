use std::sync::Arc;

use eframe::egui::{self, Color32, CornerRadius, FontFamily, FontId, Stroke, Style, Visuals};

pub const RADIUS_CARD: u8 = 12;
pub const RADIUS_CONTROL: u8 = 8;
pub const RADIUS_BADGE: u8 = 5;
pub const FONT_SEMIBOLD: &str = "inter-semibold";

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub canvas: Color32,
    pub bg_editor: Color32,
    pub bg_sidebar: Color32,
    pub bg_elevated: Color32,
    pub bg_code: Color32,
    pub border: Color32,
    pub border_strong: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub badge_bg: Color32,
    pub dot_grid: Color32,
    pub shadow: Color32,
    pub unsaved: Color32,
    pub done: Color32,
    pub unresolved: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            canvas: Color32::from_rgb(16, 17, 20),
            bg_editor: Color32::from_rgb(24, 25, 29),
            bg_sidebar: Color32::from_rgb(24, 25, 29),
            bg_elevated: Color32::from_rgb(34, 36, 42),
            bg_code: Color32::from_rgb(30, 32, 37),
            border: Color32::from_rgb(42, 44, 51),
            border_strong: Color32::from_rgb(64, 67, 76),
            text_primary: Color32::from_rgb(232, 233, 237),
            text_muted: Color32::from_rgb(130, 135, 146),
            accent: Color32::from_rgb(124, 146, 240),
            accent_soft: Color32::from_rgb(34, 40, 68),
            badge_bg: Color32::from_rgb(42, 44, 51),
            dot_grid: Color32::from_rgb(44, 46, 54),
            shadow: Color32::from_black_alpha(90),
            unsaved: Color32::from_rgb(224, 150, 116),
            done: Color32::from_rgb(120, 196, 140),
            unresolved: Color32::from_rgb(214, 120, 120),
        }
    }

    pub fn light() -> Self {
        Self {
            canvas: Color32::from_rgb(238, 239, 242),
            bg_editor: Color32::from_rgb(255, 255, 255),
            bg_sidebar: Color32::from_rgb(255, 255, 255),
            bg_elevated: Color32::from_rgb(243, 244, 246),
            bg_code: Color32::from_rgb(243, 244, 246),
            border: Color32::from_rgb(227, 229, 234),
            border_strong: Color32::from_rgb(203, 207, 214),
            text_primary: Color32::from_rgb(17, 19, 24),
            text_muted: Color32::from_rgb(138, 143, 154),
            accent: Color32::from_rgb(31, 63, 191),
            accent_soft: Color32::from_rgb(230, 234, 251),
            badge_bg: Color32::from_rgb(236, 237, 240),
            dot_grid: Color32::from_rgb(213, 216, 222),
            shadow: Color32::from_black_alpha(28),
            unsaved: Color32::from_rgb(196, 98, 52),
            done: Color32::from_rgb(40, 140, 84),
            unresolved: Color32::from_rgb(190, 64, 64),
        }
    }
}

pub fn install_fonts(ctx: &egui::Context) {
    let mut fonts = egui::FontDefinitions::default();
    fonts.font_data.insert(
        "inter-regular".into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Inter-Regular.ttf"
        ))),
    );
    fonts.font_data.insert(
        FONT_SEMIBOLD.into(),
        Arc::new(egui::FontData::from_static(include_bytes!(
            "../../../assets/fonts/Inter-SemiBold.ttf"
        ))),
    );
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter-regular".into());
    fonts
        .families
        .entry(FontFamily::Name(FONT_SEMIBOLD.into()))
        .or_default()
        .extend([FONT_SEMIBOLD.to_owned(), "inter-regular".to_owned()]);
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    ctx.set_fonts(fonts);
}

pub fn semibold(size: f32) -> FontId {
    FontId::new(size, FontFamily::Name(FONT_SEMIBOLD.into()))
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) -> Palette {
    let pal = if dark { Palette::dark() } else { Palette::light() };
    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };

    visuals.panel_fill = pal.canvas;
    visuals.window_fill = pal.bg_editor;
    visuals.extreme_bg_color = pal.bg_code;
    visuals.faint_bg_color = pal.bg_elevated;
    visuals.hyperlink_color = pal.accent;
    visuals.selection.bg_fill = pal.accent_soft;
    visuals.selection.stroke = Stroke::new(1.0, pal.accent);

    visuals.widgets.noninteractive.bg_fill = pal.bg_editor;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    visuals.widgets.inactive.bg_fill = pal.bg_elevated;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    visuals.widgets.active.bg_fill = pal.accent;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, pal.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.hovered.bg_fill = pal.bg_elevated;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, pal.border_strong);
    visuals.popup_shadow = egui::Shadow { offset: [0, 8], blur: 24, spread: 0, color: pal.shadow };
    visuals.window_shadow = visuals.popup_shadow;
    visuals.widgets.active.corner_radius = CornerRadius::same(RADIUS_CONTROL);

    visuals.window_corner_radius = CornerRadius::same(RADIUS_CARD);
    visuals.window_stroke = Stroke::new(1.0, pal.border);

    let mut style = Style::default();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);

    pal
}
