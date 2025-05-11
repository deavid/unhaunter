use bevy::{
    color::{Color, palettes::css},
    ui::BorderColor,
};

pub const DEBUG_BCOLOR: BorderColor = BorderColor(Color::srgba(0.0, 1.0, 1.0, 0.0003));
pub const INVENTORY_STATS_COLOR: Color = Color::srgba(0.7, 0.7, 0.7, 0.9);
pub const WALKIE_TALKIE_COLOR: Color = Color::srgba(1.0, 1.0, 0.5, 0.9);
pub const PANEL_BGCOLOR: Color = Color::srgba(0.1, 0.1, 0.1, 0.5);
pub const TRUCKUI_BGCOLOR: Color = Color::srgba(0.082, 0.094, 0.118, 0.6);
pub const TRUCKUI_PANEL_BGCOLOR: Color = Color::srgba(0.106, 0.129, 0.157, 0.8);
pub const TRUCKUI_ACCENT_COLOR: Color = Color::srgba(0.290, 0.596, 0.706, 1.0);
pub const TRUCKUI_ACCENT2_COLOR: Color = Color::srgba(0.290, 0.596, 0.706, 0.2);
pub const TRUCKUI_ACCENT3_COLOR: Color = Color::srgba(0.650, 0.80, 0.856, 1.0);
pub const TRUCKUI_TEXT_COLOR: Color = Color::srgba(0.7, 0.82, 0.85, 1.0);
pub const BUTTON_EXIT_TRUCK_BGCOLOR: Color = Color::srgba(0.129, 0.165, 0.122, 1.0);
pub const BUTTON_EXIT_TRUCK_FGCOLOR: Color = Color::srgba(0.196, 0.275, 0.169, 1.0);
pub const BUTTON_EXIT_TRUCK_TXTCOLOR: Color = Color::srgba(0.416, 0.667, 0.271, 1.0);
pub const BUTTON_END_MISSION_BGCOLOR: Color = Color::srgba(0.224, 0.129, 0.122, 1.0);
pub const BUTTON_END_MISSION_FGCOLOR: Color = Color::srgba(0.388, 0.200, 0.169, 1.0);
pub const BUTTON_END_MISSION_TXTCOLOR: Color = Color::srgba(0.851, 0.522, 0.275, 1.0);
pub const DIALOG_TEXT_COLOR: Color = Color::srgba(1.0, 1.0, 1.0, 0.7);
pub const DIALOG_BOLD_TEXT_COLOR: Color = Color::srgba(0.0, 0.8, 1.0, 0.9);
pub const MENU_ITEM_COLOR_ON: Color = Color::Srgba(css::ORANGE_RED);
pub const MENU_ITEM_COLOR_OFF: Color = Color::Srgba(css::GRAY);
pub const MENU_DESC_TEXT_COLOR: Color = Color::srgba(0.8, 0.94, 0.98, 1.0);
