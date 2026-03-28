use bevy::{
    color::{Color, palettes::css},
    ui::BorderColor,
};

const DEBUG_BORDER_COLOR: Color = Color::srgba(0.0, 1.0, 1.0, 0.0003);
pub const DEBUG_BCOLOR: BorderColor = BorderColor {
    top: DEBUG_BORDER_COLOR,
    right: DEBUG_BORDER_COLOR,
    bottom: DEBUG_BORDER_COLOR,
    left: DEBUG_BORDER_COLOR,
};
pub const MENU_ITEM_COLOR_ON: Color = Color::Srgba(css::ORANGE_RED);
pub const MENU_ITEM_COLOR_OFF: Color = Color::Srgba(css::GRAY);
pub const MENU_DESC_TEXT_COLOR: Color = Color::srgba(0.8, 0.94, 0.98, 1.0);

pub const DIALOG_TEXT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.7);
pub const DIALOG_BOLD_TEXT_COLOR: Color = Color::srgba(0.0, 0.8, 1.0, 0.9);
pub const PANEL_BGCOLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.5);
