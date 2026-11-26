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

const APP_ICON_PNG: &[u8] = include_bytes!("../../../assets/AppIcon-256.png");

#[cfg(any(target_os = "macos", test))]
fn is_macos_app_bundle_exe(exe_path: &std::path::Path) -> bool {
    let Some(macos_dir) = exe_path.parent() else {
        return false;
    };
    let Some(contents_dir) = macos_dir.parent() else {
        return false;
    };
    let Some(bundle_dir) = contents_dir.parent() else {
        return false;
    };

    macos_dir.file_name().is_some_and(|name| name == "MacOS")
        && contents_dir
            .file_name()
            .is_some_and(|name| name == "Contents")
        && bundle_dir.extension().is_some_and(|ext| ext == "app")
        && contents_dir.join("Info.plist").is_file()
        && contents_dir.join("Resources").join("AppIcon.icns").is_file()
}

fn load_runtime_icon() -> egui::IconData {
    #[cfg(target_os = "macos")]
    if std::env::current_exe()
        .ok()
        .is_some_and(|exe_path| is_macos_app_bundle_exe(&exe_path))
    {
        return egui::IconData::default();
    }

    eframe::icon_data::from_png_bytes(APP_ICON_PNG)
        .expect("embedded AppIcon-256.png should be a valid PNG")
}

fn main() -> eframe::Result<()> {
    let initial_vault = std::env::args()
        .nth(1)
        .filter(|argument| !argument.starts_with('-'))
        .map(PathBuf::from);

    let icon = load_runtime_icon();

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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn embedded_runtime_icon_is_high_resolution() {
        let icon = eframe::icon_data::from_png_bytes(APP_ICON_PNG).expect("valid PNG icon");
        assert_eq!(icon.width, 256);
        assert_eq!(icon.height, 256);
        assert_eq!(icon.rgba.len(), 256 * 256 * 4);
    }

    #[test]
    fn detects_macos_app_bundle_only_when_icns_exists() {
        let unique = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system clock")
            .as_nanos();
        let root = std::env::temp_dir().join(format!("cottonmd-bundle-test-{unique}"));
        let contents = root.join("CottonMD.app").join("Contents");
        let exe = contents.join("MacOS").join("CottonMD");
        let resources = contents.join("Resources");
        fs::create_dir_all(exe.parent().expect("MacOS dir")).expect("create MacOS");
        fs::create_dir_all(&resources).expect("create Resources");
        fs::write(&exe, b"").expect("create exe");
        fs::write(contents.join("Info.plist"), b"").expect("create Info.plist");
        let icns = resources.join("AppIcon.icns");
        fs::write(&icns, b"").expect("create AppIcon.icns");

        assert!(is_macos_app_bundle_exe(&exe));

        fs::remove_file(&icns).expect("remove icns");
        assert!(!is_macos_app_bundle_exe(&exe));

        let standalone = root.join("target").join("release").join("CottonMD");
        assert!(!is_macos_app_bundle_exe(&standalone));

        let _ = fs::remove_dir_all(&root);
    }
}
