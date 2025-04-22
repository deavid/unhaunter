use crate::scrollbar;
use crate::scrollbar::ScrollableListContainer;
use bevy::prelude::*;
use bevy::ui::ScrollPosition;
use bevy::utils::Instant;
use uncore::events::map_selected::MapSelectedEvent;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::MenuMouseTracker;
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
            (
                handle_menu_events,
                handle_esc_key,
                scrollbar::update_scroll_position,
                scrollbar::update_scrollbar,
                scrollbar::handle_scrollbar_interactions,
            )
                .run_if(in_state(MapHubState::MapSelection)),
        )
        .add_systems(
            PostUpdate,
            scrollbar::ensure_selected_item_visible.run_if(in_state(MapHubState::MapSelection)),
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
                // Create a container for both the content and scrollbar
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    })
                    .with_children(|scrollable_container| {
                        // Scrollable list container
                        scrollable_container
                            .spawn(Node {
                                width: Val::Percent(90.0),
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::Start,
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
                                map_list.spawn(Node {
                                    min_height: Val::Px(16.0),
                                    flex_basis: Val::Px(16.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior {
                                    should_block_lower: false,
                                    ..default()
                                });

                                templates::create_content_item(map_list, "Go Back", maps.maps.len(), false, handles);
                                map_list.spawn(Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(64.0),
                                    flex_basis: Val::Px(64.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior {
                                    should_block_lower: false,
                                    ..default()
                                });
                            });

                        // Scrollbar container
                        scrollbar::build_scrollbar_ui(scrollable_container, handles);
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
