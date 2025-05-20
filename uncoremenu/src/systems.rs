use crate::components::{MenuItemInteractive, MenuMouseTracker, MenuRoot, PrincipalMenuText};
use crate::events::KeyboardNavigate;
use bevy::{input::mouse::MouseMotion, prelude::*};
use uncore::colors;
use uncore::states::AppState;

/// Event sent when a menu item is clicked
#[derive(Event, Debug, Clone, Copy)]
pub struct MenuItemClicked {
    pub state: AppState,
    pub pos: usize,
}

/// Event sent when keyboard navigation changes the selected item
#[derive(Event, Debug, Clone, Copy)]
pub struct MenuItemSelected(pub usize);

/// Event sent when ESC is pressed in a menu
#[derive(Event, Debug, Clone, Copy)]
pub struct MenuEscapeEvent;

/// Detects mouse movement to enable hover selection. Mouse movement is tracked to prevent
/// unwanted initial hover states when opening menus.
pub fn menu_mouse_movement_system(
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_tracker: Query<&mut MenuMouseTracker>,
) {
    // Only process if there was mouse movement
    if !mouse_motion_events.is_empty() {
        mouse_motion_events.clear();
        for mut tracker in mouse_tracker.iter_mut() {
            tracker.mouse_moved = true;
        }
    }
}

/// Handles mouse interaction with menu items, including hover and click states.
/// Only processes hover events after mouse movement is detected to prevent unwanted
/// initial hover states.
pub fn menu_interaction_system(
    mut menu_query: Query<&mut MenuRoot>,
    interaction_query: Query<(&Interaction, &MenuItemInteractive), Changed<Interaction>>,
    mouse_tracker: Query<&MenuMouseTracker>,
    mut click_events: EventWriter<MenuItemClicked>,
    mut selection_events: EventWriter<MenuItemSelected>,
    app_state: Res<State<AppState>>,
) {
    let mouse_moved = mouse_tracker
        .iter()
        .next()
        .is_some_and(|tracker| tracker.mouse_moved);

    for (interaction, menu_item) in interaction_query.iter() {
        match *interaction {
            Interaction::Hovered => {
                if mouse_moved {
                    for mut menu in menu_query.iter_mut() {
                        if menu.selected_item != menu_item.identifier {
                            menu.selected_item = menu_item.identifier;
                            selection_events.send(MenuItemSelected(menu_item.identifier));
                        }
                    }
                }
            }
            Interaction::Pressed => {
                click_events.send(MenuItemClicked {
                    state: **app_state,
                    pos: menu_item.identifier,
                });
            }
            Interaction::None => {}
        }
    }
}

/// Handles keyboard navigation for menu items, including up/down arrows,
/// enter for selection, and escape key events.
pub fn menu_keyboard_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut menu_query: Query<&mut MenuRoot>,
    menu_items: Query<&MenuItemInteractive>,
    mut selection_events: EventWriter<MenuItemSelected>,
    mut keyboard_nav_events: EventWriter<KeyboardNavigate>,
    mut click_events: EventWriter<MenuItemClicked>,
    mut escape_events: EventWriter<MenuEscapeEvent>,
    app_state: Res<State<AppState>>,
) {
    let Ok(mut menu) = menu_query.get_single_mut() else {
        return;
    };

    // Get the total number of menu items
    let item_count = menu_items.iter().count();
    if item_count == 0 {
        warn!("No menu items found!");
        return;
    };

    // Handle up/down navigation
    let mut new_selection = None;
    if keyboard_input.just_pressed(KeyCode::ArrowUp) {
        new_selection = Some(if menu.selected_item == 0 {
            item_count - 1
        } else {
            menu.selected_item - 1
        });
    } else if keyboard_input.just_pressed(KeyCode::ArrowDown) {
        new_selection = Some((menu.selected_item + 1) % item_count);
    }

    // Update selection if changed
    if let Some(new_idx) = new_selection {
        menu.selected_item = new_idx;
        selection_events.send(MenuItemSelected(new_idx));
        keyboard_nav_events.send(KeyboardNavigate(new_idx)); // Send the new event
    }

    // Handle enter key for selection
    if keyboard_input.just_pressed(KeyCode::Enter) {
        click_events.send(MenuItemClicked {
            state: **app_state,
            pos: menu.selected_item,
        });
    }

    // Handle escape key
    if keyboard_input.just_pressed(KeyCode::Escape) {
        escape_events.send(MenuEscapeEvent);
    }
}

/// Updates the visual state of menu items based on selection and hover states.
/// Sets appropriate colors for both the background and text elements.
pub fn update_menu_item_visuals(
    menu_query: Query<&MenuRoot>,
    mut menu_items: Query<(
        &mut BackgroundColor,
        &MenuItemInteractive,
        &Interaction,
        &Children,
    )>,
    mut text_queries: ParamSet<(
        Query<&mut TextColor>,
        Query<(&mut TextColor, &Parent), With<PrincipalMenuText>>,
    )>,
) {
    // Skip if there are no menus
    let Ok(menu) = menu_query.get_single() else {
        return;
    };
    let selected_item = menu.selected_item;

    // Update visual state for each menu item
    for (mut bg_color, item, interaction, children) in menu_items.iter_mut() {
        let is_selected = item.identifier == selected_item;
        let is_hovered = *interaction == Interaction::Hovered;

        // Set background color for parent
        let selected_bg = Color::srgba(0.8, 0.3, 0.3, 0.02);
        let target_bg_color = if is_selected {
            selected_bg
        } else {
            Color::NONE
        };

        if bg_color.0 != target_bg_color {
            bg_color.0 = target_bg_color;
        }

        // Calculate the target text color based on selection and hover state
        let target_text_color = match (is_selected, is_hovered) {
            (true, true) => colors::MENU_ITEM_COLOR_ON.with_alpha(1.0), // Selected and hovered
            (true, false) => colors::MENU_ITEM_COLOR_ON,                // Selected
            (false, true) => colors::MENU_ITEM_COLOR_OFF.with_alpha(0.8), // Just hovered
            (false, false) => colors::MENU_ITEM_COLOR_OFF,              // Neither
        };

        // First, try to find a child with the PrincipalMenuText marker
        let mut updated = false;
        let mut principal_query = text_queries.p1();
        for (mut text_color, parent) in principal_query.iter_mut() {
            // Check if this principal text's parent is in our children list
            if children.iter().any(|&child_id| parent.get() == child_id) {
                if text_color.0 != target_text_color {
                    text_color.0 = target_text_color;
                }
                updated = true;
                break;
            }
        }

        // If we didn't find any PrincipalMenuText, fall back to the old behavior
        if !updated {
            if let Some(child) = children.first() {
                let mut standard_query = text_queries.p0();
                if let Ok(mut text_color) = standard_query.get_mut(*child) {
                    if text_color.0 != target_text_color {
                        text_color.0 = target_text_color;
                    }
                }
            }
        }
    }
}
