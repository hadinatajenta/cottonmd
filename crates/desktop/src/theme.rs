use eframe::egui::{self, Color32, CornerRadius, Stroke, Style, Visuals};

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg_editor: Color32,
    pub bg_sidebar: Color32,
    pub bg_elevated: Color32,
    pub bg_code: Color32,
    pub border: Color32,
    pub text_primary: Color32,
    pub text_muted: Color32,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub unsaved: Color32,
    pub done: Color32,
    pub unresolved: Color32,
}

impl Palette {
    pub fn dark() -> Self {
        Self {
            bg_editor: Color32::from_rgb(26, 27, 30),
            bg_sidebar: Color32::from_rgb(20, 21, 23),
            bg_elevated: Color32::from_rgb(36, 38, 43),
            bg_code: Color32::from_rgb(20, 21, 24),
            border: Color32::from_rgb(44, 46, 51),
            text_primary: Color32::from_rgb(228, 229, 233),
            text_muted: Color32::from_rgb(144, 148, 158),
            accent: Color32::from_rgb(112, 152, 200),
            accent_soft: Color32::from_rgb(38, 52, 70),
            unsaved: Color32::from_rgb(208, 135, 112),
            done: Color32::from_rgb(143, 188, 143),
            unresolved: Color32::from_rgb(180, 120, 120),
        }
    }

    pub fn light() -> Self {
        Self {
            bg_editor: Color32::from_rgb(250, 249, 246),
            bg_sidebar: Color32::from_rgb(241, 239, 234),
            bg_elevated: Color32::from_rgb(229, 226, 218),
            bg_code: Color32::from_rgb(238, 235, 228),
            border: Color32::from_rgb(218, 214, 204),
            text_primary: Color32::from_rgb(31, 35, 40),
            text_muted: Color32::from_rgb(99, 106, 115),
            accent: Color32::from_rgb(53, 96, 145),
            accent_soft: Color32::from_rgb(218, 230, 244),
            unsaved: Color32::from_rgb(186, 92, 58),
            done: Color32::from_rgb(64, 132, 82),
            unresolved: Color32::from_rgb(168, 72, 72),
        }
    }
}

pub fn apply_theme(ctx: &egui::Context, dark: bool) -> Palette {
    let pal = if dark { Palette::dark() } else { Palette::light() };
    let mut visuals = if dark { Visuals::dark() } else { Visuals::light() };

    visuals.panel_fill = pal.bg_sidebar;
    visuals.window_fill = pal.bg_editor;
    visuals.extreme_bg_color = pal.bg_code;
    visuals.faint_bg_color = pal.bg_elevated;
    visuals.hyperlink_color = pal.accent;
    visuals.selection.bg_fill = pal.accent_soft;
    visuals.selection.stroke = Stroke::new(1.0, pal.accent);

    visuals.widgets.noninteractive.bg_fill = pal.bg_sidebar;
    visuals.widgets.noninteractive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.widgets.noninteractive.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.noninteractive.corner_radius = CornerRadius::same(4);

    visuals.widgets.inactive.bg_fill = pal.bg_elevated;
    visuals.widgets.inactive.bg_stroke = Stroke::new(1.0, pal.border);
    visuals.widgets.inactive.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.inactive.corner_radius = CornerRadius::same(4);

    visuals.widgets.hovered.bg_fill = pal.accent_soft;
    visuals.widgets.hovered.bg_stroke = Stroke::new(1.0, pal.accent);
    visuals.widgets.hovered.fg_stroke = Stroke::new(1.0, pal.text_primary);
    visuals.widgets.hovered.corner_radius = CornerRadius::same(4);

    visuals.widgets.active.bg_fill = pal.accent;
    visuals.widgets.active.bg_stroke = Stroke::new(1.0, pal.accent);
    visuals.widgets.active.fg_stroke = Stroke::new(1.0, Color32::WHITE);
    visuals.widgets.active.corner_radius = CornerRadius::same(4);

    visuals.window_corner_radius = CornerRadius::same(6);
    visuals.window_stroke = Stroke::new(1.0, pal.border);

    let mut style = Style::default();
    style.visuals = visuals;
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(8.0, 4.0);
    style.spacing.window_margin = egui::Margin::same(12);
    ctx.set_style(style);

    pal
}
