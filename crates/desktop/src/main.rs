#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod bases_view;
mod canvas_view;
mod file_tree;
mod graph_view;
mod markdown_view;
mod shell;
mod shortcuts;
mod state;
mod theme;
mod ui_kit;
mod vault_config;

use std::path::PathBuf;
use std::sync::Arc;

use eframe::egui;

use app::CottonMdApp;

const APP_ICON_RGBA: &[u8] = include_bytes!("../../../assets/AppIcon-64.rgba");
const APP_ICON_SIZE: u32 = 64;

fn main() -> eframe::Result<()> {
    let initial_vault = std::env::args()
        .nth(1)
        .filter(|argument| !argument.starts_with('-'))
        .map(PathBuf::from);

    let icon = egui::IconData {
        rgba: APP_ICON_RGBA.to_vec(),
        width: APP_ICON_SIZE,
        height: APP_ICON_SIZE,
    };

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_title("CottonMD")
            .with_icon(Arc::new(icon))
            .with_inner_size([1320.0, 840.0])
            .with_min_inner_size([760.0, 500.0]),
        ..Default::default()
    };

    eframe::run_native(
        "CottonMD",
        options,
        Box::new(move |creation_context| {
            theme::install_fonts(&creation_context.egui_ctx);
            Ok(Box::new(CottonMdApp::new(initial_vault)))
        }),
    )
}
