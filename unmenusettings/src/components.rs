use bevy::{prelude::*, utils::HashMap};

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
    pub idx: usize,
    pub on_activate: MenuEvent,
}

impl MenuItem {
    pub fn new(idx: usize, on_activate: MenuEvent) -> Self {
        MenuItem { idx, on_activate }
    }
}

#[derive(Event, Debug, Clone, Copy, Default)]
pub enum MenuEvent {
    Back(MenuEvBack),
    #[default]
    None,
}

impl MenuEvent {
    pub fn is_none(&self) -> bool {
        matches!(self, MenuEvent::None)
    }
}

#[derive(Event, Debug, Clone, Copy)]
pub struct MenuEvBack;
