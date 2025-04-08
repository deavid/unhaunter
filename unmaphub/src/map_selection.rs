use bevy::prelude::*;
use std::time::Instant;
use uncore::events::map_selected::MapSelectedEvent;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::systems::{MenuItemClicked, MenuItemSelected};
use uncoremenu::templates;

#[derive(Component, Debug)]
pub struct MapSelectionUI;

#[derive(Component, Debug)]
pub struct MapSelectionItem {
    pub map_idx: usize,
}

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

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::MapSelection), setup_systems)
        .add_systems(OnExit(MapHubState::MapSelection), cleanup_systems)
        .add_systems(
            Update,
            (handle_menu_events, handle_esc_key).run_if(in_state(MapHubState::MapSelection)),
        )
        .init_resource::<MapSelectionState>()
        .add_event::<MapSelectedEvent>();

    info!("Map Selection systems registered");
}

pub fn setup_systems(mut commands: Commands, maps: Res<Maps>, handles: Res<GameAssets>) {
    // Create the UI for the map selection screen
    setup_ui(&mut commands, &handles, &maps);

    // Initialize state with current timestamp
    commands.insert_resource(MapSelectionState {
        selected_map_idx: 0,
        state_entered_at: Instant::now(),
    });
}

pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<MapSelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn handle_menu_events(
    mut menu_selection: EventReader<MenuItemSelected>,
    mut menu_clicks: EventReader<MenuItemClicked>,
    mut map_selection_state: ResMut<MapSelectionState>,
    maps: Res<Maps>,
    mut next_state: ResMut<NextState<MapHubState>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut ev_map_selected: EventWriter<MapSelectedEvent>,
) {
    // Define a small grace period to ignore events from previous state
    const GRACE_PERIOD_SECS: f32 = 0.1;

    // Get time since state entered
    let time_in_state = map_selection_state.state_entered_at.elapsed().as_secs_f32();

    // Ignore events that happened too soon after state transition
    if time_in_state < GRACE_PERIOD_SECS {
        menu_clicks.clear();
        menu_selection.clear();
        return;
    }

    // Handle selection changes
    for ev in menu_selection.read() {
        map_selection_state.selected_map_idx = ev.0;
    }

    // Handle clicks
    for ev in menu_clicks.read() {
        let selected_idx = ev.0;
        if selected_idx < maps.maps.len() {
            // A map was selected
            ev_map_selected.send(MapSelectedEvent {
                map_idx: selected_idx,
            });
            next_state.set(MapHubState::DifficultySelection);
        } else {
            // Go Back was selected
            root_next_state.set(AppState::MainMenu);
            next_state.set(MapHubState::None);
        }
    }
}

pub fn handle_esc_key(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut next_state: ResMut<NextState<MapHubState>>,
) {
    if keyboard_input.just_pressed(KeyCode::Escape) {
        // Transition back to the main menu
        root_next_state.set(AppState::MainMenu);
        next_state.set(MapHubState::None);
    }
}

pub fn setup_ui(commands: &mut Commands, handles: &GameAssets, maps: &Maps) {
    // Create the standard menu layout
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MapSelectionUI)
        .with_children(|parent| {
            // Background and logo using standard components
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);

            // Breadcrumb navigation in left strip
            templates::create_breadcrumb_navigation(
                parent,
                handles,
                "New Game",
                "Select Map"
            );

            // Create content area with semi-transparent background
            templates::create_selectable_content_area(
                parent,
                handles,
                0 // Initial selection
            )
            .with_children(|content| {
                // Left column for map list
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
                    .with_children(|map_list| {
                        // Add map items to the list
                        for (idx, map) in maps.maps.iter().enumerate() {
                            templates::create_content_item(
                                map_list,
                                map.name.clone(),
                                idx,
                                idx == 0, // First item selected by default
                                handles,
                            ).insert(MapSelectionItem { map_idx: idx });
                        }

                        // Add "Go Back" option
                        templates::create_content_item(
                            map_list,
                            "Go Back",
                            maps.maps.len(), // Index after last map
                            false,
                            handles,
                        );
                    });

                // Right column for future content
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        ..default()
                    });
            });

            // Help text
            templates::create_help_text(
                parent,
                handles,
                Some("Select a map to play    |    [Up]/[Down]: Change    |    [Enter]: Select    |    [ESC]: Go Back".into()),
            );
        });

    info!("MapHub - Map Selection menu loaded with new breadcrumb layout");
}
