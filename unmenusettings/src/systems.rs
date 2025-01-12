use crate::components::{MenuEvBack, MenuEvent, MenuItem, SettingsMenu, SettingsState};
use bevy::prelude::*;
use uncore::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::states::AppState;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut SettingsMenu>,
    menu_items: Query<&MenuItem>,
    mut ev_menu: EventWriter<MenuEvent>,
) {
    let mut menu = q.single_mut();
    let max_menu_idx = menu_items.iter().count();
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if menu.selected_item_idx == 0 {
            menu.selected_item_idx = max_menu_idx - 1;
        } else {
            menu.selected_item_idx -= 1;
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_item_idx = (menu.selected_item_idx + 1) % max_menu_idx;
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        if let Some(menu_item) = menu_items
            .iter()
            .find(|item| item.idx == menu.selected_item_idx)
        {
            ev_menu.send(menu_item.on_activate);
        }
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        ev_menu.send(MenuEvent::Back(MenuEvBack));
    }
}

pub fn item_highlight_system(
    menu: Query<&SettingsMenu>,
    mut menu_items: Query<(&MenuItem, &mut TextColor)>,
) {
    let menu = menu.single(); // Assuming you have only one Menu component
    for (item, mut text_color) in &mut menu_items {
        let is_selected = item.idx == menu.selected_item_idx;
        let color = if is_selected {
            MENU_ITEM_COLOR_ON
        } else {
            MENU_ITEM_COLOR_OFF
        };
        let alpha = if item.on_activate.is_none() { 0.2 } else { 1.0 };
        text_color.0 = color.with_alpha(alpha);
    }
}

pub fn menu_routing_system(
    mut ev_menu: EventReader<MenuEvent>,
    mut ev_back: EventWriter<MenuEvBack>,
) {
    for ev in ev_menu.read() {
        match ev {
            MenuEvent::Back(menu_back) => {
                ev_back.send(menu_back.to_owned());
            }
            MenuEvent::None => {}
        }
    }
}

pub fn menu_back_event(
    mut events: EventReader<MenuEvBack>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    for _ev in events.read() {
        app_next_state.set(AppState::MainMenu);
        next_state.set(SettingsState::default());
    }
}
