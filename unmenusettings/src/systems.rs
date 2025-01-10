use crate::components::{MenuItem, SettingsMenu, SettingsState};
use crate::menus::MenuSettingsLevel1;
use bevy::prelude::*;
use strum::IntoEnumIterator;
use uncore::colors::{MENU_ITEM_COLOR_OFF, MENU_ITEM_COLOR_ON};
use uncore::states::AppState;

pub fn handle_input(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut q: Query<&mut SettingsMenu>,
    mut next_state: ResMut<NextState<SettingsState>>,
    mut app_next_state: ResMut<NextState<AppState>>,
) {
    let mut menu = q.single_mut();

    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        if menu.selected_item_idx == 0 {
            menu.selected_item_idx = MenuSettingsLevel1::iter().count() - 1;
        } else {
            menu.selected_item_idx -= 1;
        }
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        menu.selected_item_idx = (menu.selected_item_idx + 1) % MenuSettingsLevel1::iter().count();
    } else if keyboard_input.just_pressed(KeyCode::Enter) {
        // TODO: Implement Enter
    } else if keyboard_input.just_pressed(KeyCode::Escape) {
        app_next_state.set(AppState::MainMenu);
        next_state.set(SettingsState::default());
    }
}

pub fn item_highlight_system(
    menu: Query<&SettingsMenu>,
    mut menu_items: Query<(&mut MenuItem, &mut TextColor)>,
) {
    let menu = menu.single(); // Assuming you have only one Menu component
    for (mut item, mut text_color) in &mut menu_items {
        let is_selected = MenuSettingsLevel1::iter()
            .position(|x| x == item.identifier)
            .unwrap_or_default()
            == menu.selected_item_idx;
        if item.highlighted != is_selected {
            item.highlighted = is_selected;
            text_color.0 = if item.highlighted {
                MENU_ITEM_COLOR_ON
            } else {
                MENU_ITEM_COLOR_OFF
            };
        }
    }
}
