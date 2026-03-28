use bevy::{color::Color, ui::BorderColor};

const DEBUG_BORDER_COLOR: Color = Color::srgba(0.0, 1.0, 1.0, 0.0003);
pub const DEBUG_BCOLOR: BorderColor = BorderColor {
    top: DEBUG_BORDER_COLOR,
    right: DEBUG_BORDER_COLOR,
    bottom: DEBUG_BORDER_COLOR,
    left: DEBUG_BORDER_COLOR,
};
pub const PANEL_BGCOLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.5);
pub const INVENTORY_STATS_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.9);
pub const WALKIE_TALKIE_COLOR: Color = Color::srgba(1.0, 1.0, 0.5, 0.9);
pub const DIALOG_TEXT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.7);
