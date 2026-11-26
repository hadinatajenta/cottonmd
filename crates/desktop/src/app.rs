use std::path::PathBuf;

use eframe::egui;

use crate::shell::center::CenterViews;
use crate::shell::{center, left_sidebar, modals, right_sidebar, status_bar, top_bar};
use crate::shortcuts::handle_shortcuts;
use crate::state::AppState;
use crate::theme::{apply_theme, Palette};

#[derive(Clone, Copy)]
struct AppliedTheme {
    is_dark: bool,
    palette: Palette,
}

pub struct CottonMdApp {
    state: AppState,
    views: CenterViews,
    applied_theme: Option<AppliedTheme>,
}

impl CottonMdApp {
    pub fn new(initial_vault: Option<PathBuf>) -> Self {
        Self {
            state: AppState::new(initial_vault),
            views: CenterViews::default(),
            applied_theme: None,
        }
    }

    fn resolve_palette(&mut self, ctx: &egui::Context) -> Palette {
        let is_dark = self.state.dark_mode;
        if let Some(applied) = self.applied_theme.filter(|applied| applied.is_dark == is_dark) {
            return applied.palette;
        }
        let palette = apply_theme(ctx, is_dark);
        self.applied_theme = Some(AppliedTheme { is_dark, palette });
        palette
    }
}

impl eframe::App for CottonMdApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let pal = self.resolve_palette(ctx);
        handle_shortcuts(ctx, &mut self.state);
        top_bar::show(ctx, &mut self.state, pal);
        status_bar::show(ctx, &self.state, pal);
        left_sidebar::show(ctx, &mut self.state, pal);
        right_sidebar::show(ctx, &mut self.state, pal);
        center::show(ctx, &mut self.state, &mut self.views, pal);
        modals::show(ctx, &mut self.state, pal);
    }
}
