use bevy::prelude::*;
use bevy::ui::ScrollPosition;
use bevy::utils::Instant;
use bevy_persistent::Persistent; // Added for player profile
use uncore::assets::tmxmap::TmxMap; // Added for TMX map assets
use uncore::events::map_selected::MapSelectedEvent;
use uncore::platform::plt::FONT_SCALE; // Added for FONT_SCALE
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::components::MenuMouseTracker;
use uncoremenu::events::KeyboardNavigate;
use uncoremenu::scrollbar;
use uncoremenu::scrollbar::ScrollableListContainer;
use uncoremenu::systems::{MenuItemClicked, MenuItemSelected};
use uncoremenu::templates;
use unprofile::data::PlayerProfileData; // Added for player profile data // Added for TextColor, though Color::WHITE is used directly.

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
            selected_map_idx: 0, // Defaulting to the first map
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
        .init_resource::<MapSelectionState>() // Ensures it's initialized if not already
        .add_event::<MapSelectedEvent>()
        .add_event::<KeyboardNavigate>();
}

/// Creates the initial UI and state for the map selection screen
pub fn setup_systems(
    mut commands: Commands,
    maps: Res<Maps>,
    handles: Res<GameAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>, // Added player_profile
    tmx_assets: Res<Assets<TmxMap>>,                    // Added tmx_assets
) {
    setup_ui(&mut commands, &handles, &maps, &player_profile, &tmx_assets); // Pass new resources
    // Initialize or re-initialize state upon entering
    commands.insert_resource(MapSelectionState {
        selected_map_idx: 0, // Always start with the first map selected
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
    maps_res: Res<Maps>, // Renamed from maps to maps_res to avoid conflict
    mut next_state: ResMut<NextState<MapHubState>>,
    mut root_next_state: ResMut<NextState<AppState>>,
    mut ev_map_selected: EventWriter<MapSelectedEvent>,
    player_profile: Res<Persistent<PlayerProfileData>>, // Added player_profile
    tmx_assets: Res<Assets<TmxMap>>,                    // Added tmx_assets
) {
    const GRACE_PERIOD_SECS: f32 = 0.1;
    let time_in_state = map_selection_state.state_entered_at.elapsed().as_secs_f32();

    if time_in_state < GRACE_PERIOD_SECS {
        menu_clicks.clear();
        menu_selection.clear();
        return;
    }

    // Filter maps based on player level first
    let player_level = player_profile.get().progression.player_level;
    let available_maps: Vec<_> = maps_res
        .maps
        .iter()
        .enumerate()
        .filter(|(_, map_data)| {
            if let Some(tmx_asset) = tmx_assets.get(&map_data.handle) {
                player_level >= tmx_asset.props.min_player_level
            } else {
                true // If TMX asset not found, include it (or handle error)
            }
        })
        .map(|(original_idx, map_data)| (original_idx, map_data.clone())) // Keep original index
        .collect();

    for ev in menu_selection.read() {
        // ev.0 is the index in the *filtered* list (available_maps)
        if ev.0 < available_maps.len() {
            map_selection_state.selected_map_idx = ev.0; // Store filtered list index
        }
    }

    for ev in menu_clicks.read() {
        let selected_filtered_idx = ev.0;
        if selected_filtered_idx < available_maps.len() {
            // Get the original map index from the filtered list
            let original_map_idx = available_maps[selected_filtered_idx].0;
            ev_map_selected.send(MapSelectedEvent {
                map_idx: original_map_idx, // Send the original map index
            });
            next_state.set(MapHubState::DifficultySelection);
        } else {
            // This is the "Go Back" item, its index is available_maps.len()
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
pub fn setup_ui(
    commands: &mut Commands,
    handles: &GameAssets,
    maps_res: &Maps,                                // Renamed from maps to maps_res
    player_profile: &Persistent<PlayerProfileData>, // Added player_profile
    tmx_assets: &Assets<TmxMap>,                    // Added tmx_assets
) {
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MapSelectionUI)
        .id();

    commands.entity(root_entity).with_children(|parent| {
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);
            templates::create_breadcrumb_navigation(
                parent,
                handles,
                "Custom Mission", // Updated breadcrumb
                "Select Map",
            );

            let mut content_area = templates::create_selectable_content_area(parent, handles, 0);
            content_area.insert(MenuMouseTracker::default()); // Added MenuMouseTracker

            content_area.with_children(|content| {
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    })
                    .with_children(|scrollable_container| {
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
                                let player_level = player_profile.get().progression.player_level;
                                let available_maps: Vec<_> = maps_res
                                    .maps
                                    .iter()
                                    .filter(|map_data| {
                                        if let Some(tmx_asset) = tmx_assets.get(&map_data.handle) {
                                            player_level >= tmx_asset.props.min_player_level
                                        } else {
                                            true // If TMX asset not found, include it (or handle error)
                                        }
                                    })
                                    .collect();

                                if available_maps.is_empty() {
                                    // Optional: Display a message if no maps are available
                                    map_list.spawn(Text::new("No maps available for your current level."))
                                        .insert(TextFont { // Use TextFont component
                                            font: handles.fonts.londrina.w300_light.clone(),
                                            font_size: 20.0 * FONT_SCALE, // Apply FONT_SCALE
                                            font_smoothing: bevy::text::FontSmoothing::AntiAliased, // Add for consistency
                                        })
                                        .insert(TextColor(Color::WHITE)); // Use TextColor component
                                }

                                for (idx, map) in available_maps.iter().enumerate() {
                                    templates::create_content_item(map_list, map.name.clone(), idx, idx == 0, handles)
                                        .insert(MapSelectionItem { map_idx: idx }); // map_idx now refers to index in available_maps
                                }
                                map_list.spawn(Node {
                                    min_height: Val::Px(16.0),
                                    flex_basis: Val::Px(16.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior { // Allow picking through for scroll
                                    should_block_lower: false,
                                    ..default()
                                });

                                templates::create_content_item(map_list, "Go Back", available_maps.len(), false, handles);
                                map_list.spawn(Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(64.0),
                                    flex_basis: Val::Px(64.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                }).insert(PickingBehavior { // Allow picking through for scroll
                                    should_block_lower: false,
                                    ..default()
                                });
                            });
                        scrollbar::build_scrollbar_ui(scrollable_container, handles);
                    });
                // Placeholder for map preview or description (right column)
                content.spawn(Node {
                    width: Val::Percent(50.0),
                    height: Val::Percent(100.0),
                    flex_direction: FlexDirection::Column,
                    // TODO: Add image/text here for map preview if desired
                    ..default()
                });
            });

            templates::create_help_text(
                parent,
                handles,
                Some("Select a map to investigate    |    [Up]/[Down]: Change    |    [Enter]: Select    |    [ESC]: Go Back".into()),
            );

            // Add the persistent player status bar
            templates::create_player_status_bar(parent, handles, player_profile.get());
        });
}
