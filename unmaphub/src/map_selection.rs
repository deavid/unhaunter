use bevy::input::mouse::{MouseScrollUnit, MouseWheel};
use bevy::picking::focus::HoverMap;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::ui::ScrollPosition;
use bevy::utils::Instant;
use uncore::events::map_selected::MapSelectedEvent;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::{MenuItemInteractive, MenuMouseTracker};
use uncoremenu::events::KeyboardNavigate;
use uncoremenu::systems::{MenuItemClicked, MenuItemSelected};
use uncoremenu::templates;

/// UI component marker for the map selection screen
#[derive(Component, Debug)]
pub struct MapSelectionUI;

/// Component that links a menu item to its corresponding map index
#[derive(Component, Debug)]
pub struct MapSelectionItem {
    pub map_idx: usize,
}

/// Marker component for the scrollable container that holds the map list
#[derive(Component, Debug)]
pub struct ScrollableListContainer;

/// Resource that tracks the state of map selection, including timing for event debouncing
#[derive(Resource, Debug)]
pub struct MapSelectionState {
    pub selected_map_idx: usize,
    pub state_entered_at: Instant,
}

impl Default for MapSelectionState {
    fn default() -> Self {
        Self {
            selected_map_idx: 0,
            state_entered_at: Instant::now(),
        }
    }
}

/// Sets up all systems and resources needed for the map selection screen
pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::MapSelection), setup_systems)
        .add_systems(OnExit(MapHubState::MapSelection), cleanup_systems)
        .add_systems(
            Update,
            (handle_menu_events, handle_esc_key, update_scroll_position)
                .run_if(in_state(MapHubState::MapSelection)),
        )
        .add_systems(
            PostUpdate,
            ensure_selected_item_visible.run_if(in_state(MapHubState::MapSelection)),
        )
        .init_resource::<MapSelectionState>()
        .add_event::<MapSelectedEvent>()
        .add_event::<KeyboardNavigate>();
}

/// Creates the initial UI and state for the map selection screen
pub fn setup_systems(mut commands: Commands, maps: Res<Maps>, handles: Res<GameAssets>) {
    setup_ui(&mut commands, &handles, &maps);
    commands.insert_resource(MapSelectionState {
        selected_map_idx: 0,
        state_entered_at: Instant::now(),
    });
}

/// Cleans up the map selection UI when exiting the state
pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<MapSelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Handles menu selection and click events, managing transitions between states
pub fn handle_menu_events(
    mut menu_selection: EventReader<MenuItemSelected>,
    mut menu_clicks: EventReader<MenuItemClicked>,
    mut map_selection_state: ResMut<MapSelectionState>,
    maps: Res<Maps>,
    mut next_state: ResMut<NextState<MapHubState>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut ev_map_selected: EventWriter<MapSelectedEvent>,
) {
    const GRACE_PERIOD_SECS: f32 = 0.1;
    let time_in_state = map_selection_state.state_entered_at.elapsed().as_secs_f32();

    if time_in_state < GRACE_PERIOD_SECS {
        menu_clicks.clear();
        menu_selection.clear();
        return;
    }

    for ev in menu_selection.read() {
        map_selection_state.selected_map_idx = ev.0;
    }

    for ev in menu_clicks.read() {
        let selected_idx = ev.0;
        if selected_idx < maps.maps.len() {
            ev_map_selected.send(MapSelectedEvent {
                map_idx: selected_idx,
            });
            next_state.set(MapHubState::DifficultySelection);
        } else {
            root_next_state.set(AppState::MainMenu);
            next_state.set(MapHubState::None);
        }
    }
}

/// Handles ESC key press to return to the main menu
pub fn handle_esc_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut next_state: ResMut<NextState<MapHubState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        root_next_state.set(AppState::MainMenu);
        next_state.set(MapHubState::None);
    }
}

/// Ensures that the keyboard-selected item remains visible in the scrollable list by adjusting scroll position
pub fn ensure_selected_item_visible(
    mut keyboard_nav_events: EventReader<KeyboardNavigate>,
    mut container_query: Query<
        (Entity, &Node, &ComputedNode, &mut ScrollPosition),
        With<ScrollableListContainer>,
    >,
    items_query: Query<(&Node, &ComputedNode, &GlobalTransform, &MenuItemInteractive)>,
) {
    if keyboard_nav_events.is_empty() {
        return;
    }

    const SCROLL_MARGIN_PX: f32 = 80.0;

    if let Ok((_entity, _node, container_computed, mut scroll_position)) =
        container_query.get_single_mut()
    {
        let container_height = container_computed.size().y;
        let current_scroll_y = scroll_position.offset_y;

        let mut sorted_items_data: Vec<_> = items_query
            .iter()
            .map(|(_, computed, _, item)| (item.identifier, computed.size().y))
            .collect();
        sorted_items_data.sort_by_key(|(id, _)| *id);

        for ev in keyboard_nav_events.read() {
            let selected_idx = ev.0;
            let mut current_y_offset = 0.0;
            let mut selected_item_top_y = 0.0;
            let mut selected_item_height = 0.0;
            let mut found = false;

            for (id, height) in sorted_items_data.iter() {
                if *id == selected_idx {
                    selected_item_top_y = current_y_offset;
                    selected_item_height = *height;
                    found = true;
                    break;
                }
                current_y_offset += height;
            }

            if !found {
                continue;
            }

            let item_bottom_in_content = selected_item_top_y + selected_item_height;
            let visible_top_with_margin = current_scroll_y + SCROLL_MARGIN_PX;
            let visible_bottom_with_margin = current_scroll_y + container_height - SCROLL_MARGIN_PX;

            let mut new_scroll_y = current_scroll_y;

            if selected_item_top_y < visible_top_with_margin {
                new_scroll_y = selected_item_top_y - SCROLL_MARGIN_PX;
            } else if item_bottom_in_content > visible_bottom_with_margin {
                new_scroll_y = item_bottom_in_content - container_height + SCROLL_MARGIN_PX;
            }

            new_scroll_y = new_scroll_y.max(0.0);

            if (new_scroll_y - current_scroll_y).abs() > 0.1 {
                scroll_position.offset_y = new_scroll_y;
            }

            break;
        }
    }
}

/// Updates the scroll position based on mouse wheel input when hovering over the list
pub fn update_scroll_position(
    mut mouse_wheel_events: EventReader<MouseWheel>,
    hover_map: Res<HoverMap>,
    mut scrolled_node_query: Query<&mut ScrollPosition, With<ScrollableListContainer>>,
) {
    const LINE_HEIGHT: f32 = 21.0;

    for mouse_wheel_event in mouse_wheel_events.read() {
        let scroll_amount = match mouse_wheel_event.unit {
            MouseScrollUnit::Line => mouse_wheel_event.y * LINE_HEIGHT,
            MouseScrollUnit::Pixel => mouse_wheel_event.y,
        };

        if scroll_amount == 0.0 {
            continue;
        }

        for (_pointer, pointer_map) in hover_map.iter() {
            for (entity, _hit) in pointer_map.iter() {
                if let Ok(mut scroll_position) = scrolled_node_query.get_mut(*entity) {
                    scroll_position.offset_y -= scroll_amount;
                    scroll_position.offset_y = scroll_position.offset_y.max(0.0);
                }
            }
        }
    }
}

/// Creates the map selection UI layout including background, logo, breadcrumb navigation, and scrollable list
pub fn setup_ui(commands: &mut Commands, handles: &GameAssets, maps: &Maps) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MapSelectionUI)
        .with_children(|parent| {
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);
            templates::create_breadcrumb_navigation(parent, handles, "New Game", "Select Map");

            let mut content_area = templates::create_selectable_content_area(parent, handles, 0);
            content_area.insert(MenuMouseTracker::default());

            content_area.with_children(|content| {
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        align_items: AlignItems::FlexStart,
                        justify_content: JustifyContent::FlexStart,
                        overflow: Overflow::scroll_y(),
                        ..default()
                    })
                    .insert(ScrollableListContainer)
                    .insert(ScrollPosition::default())
                    .with_children(|map_list| {
                        for (idx, map) in maps.maps.iter().enumerate() {
                            templates::create_content_item(map_list, map.name.clone(), idx, idx == 0, handles)
                                .insert(MapSelectionItem { map_idx: idx });
                        }

                        templates::create_content_item(map_list, "Go Back", maps.maps.len(), false, handles);
                    });

                content.spawn(Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    ..default()
                });
            });

            templates::create_help_text(
                parent,
                handles,
                Some("Select a map to play    |    [Up]/[Down]: Change    |    [Enter]: Select    |    [ESC]: Go Back".into()),
            );
        });
}
