pub mod center;
pub mod file_tree_view;
pub mod left_sidebar;
pub mod modals;
pub mod right_sidebar;
pub mod status_bar;
pub mod top_bar;
pub mod widgets;

pub const MOD_KEY: &str = if cfg!(target_os = "macos") { "Cmd" } else { "Ctrl" };
pub const SIDEBAR_DEFAULT_WIDTH: f32 = 260.0;
pub const SIDEBAR_MIN_WIDTH: f32 = 180.0;
pub const SIDEBAR_MAX_WIDTH: f32 = 420.0;
