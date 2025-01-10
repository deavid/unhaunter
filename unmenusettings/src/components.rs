use bevy::{prelude::*, utils::HashMap};

use crate::menus::MenuSettingsLevel1;

#[derive(Component, Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum MenuType {
    MainCategories,
    CategorySettingList,
    SettingEdit,
}

// Marker component for the main settings menu UI
#[derive(Component)]
pub struct SettingsMenu {
    pub menu_type: MenuType,
    pub selected_item_idx: usize,
    pub last_selected: HashMap<MenuType, usize>,
    pub settings_entity: Option<Entity>,
}

#[derive(Component)]
pub struct SCamera;

#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, States, Default)]
pub enum SettingsState {
    #[default]
    Main,
    Category,
    Setting,
}

#[derive(Component)]
pub struct MenuItem {
    pub identifier: MenuSettingsLevel1,
    pub highlighted: bool,
}

impl MenuItem {
    pub fn new(identifier: MenuSettingsLevel1) -> Self {
        MenuItem {
            identifier,
            highlighted: false,
        }
    }
}
