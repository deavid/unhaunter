// Unified Mission Selection System
//
// This module provides a unified mission selection UI that serves both Campaign and Custom
// mission modes. It replaces the previous separate implementation that used different screens for
// Campaign and Custom missions.
//
// Key features:
// - Mode-specific filtering: Shows only mission types based on CurrentMissionSelectMode
// - Player level filtering: Shows available missions and locked missions separately
// - Unified sorting: Applies consistent order->name->path sorting for all missions
// - Progress tracking: Shows completion badges for missions the player has finished
// - Different navigation flows based on mode:
//   * Campaign: Main Menu -> Mission Selection -> Game
//   * Custom: Main Menu -> Difficulty Selection -> Mission Selection -> Game
// - Proper UI mapping between list items and the original maps collection

use bevy::picking::PickingBehavior;
use bevy::prelude::*;
use bevy::ui::ScrollPosition;
use bevy_persistent::Persistent;

use uncore::colors;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::loadlevel::LoadLevelEvent;
use uncore::platform::plt::FONT_SCALE;
use uncore::resources::maps::Maps;
use uncore::resources::mission_select_mode::{CurrentMissionSelectMode, MissionSelectMode};
use uncore::states::{AppState, MapHubState};
use uncore::types::grade::Grade;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::scrollbar::ScrollableListContainer;
use uncoremenu::{
    components::{MenuItemInteractive, MenuRoot},
    scrollbar,
    systems::MenuItemSelected,
    systems::{MenuEscapeEvent, MenuItemClicked},
    templates,
};
use unmaphub::badge_utils::BadgeUtils;

// Component to delay input processing to avoid immediate selection
#[derive(Component, Default)]
struct InputDebounce {
    timer: Timer,
}

// Marker component for the unified Mission Select UI root node
#[derive(Component)]
pub struct MissionSelectUI;

// Component for the camera in mission selection
#[derive(Component)]
struct MissionSelectCamera;

// Component for the description text area
#[derive(Component)]
pub struct MissionDescriptionText;

// Component for the preview image area
#[derive(Component)]
pub struct MissionPreviewImage;

// Resource to track UI mapping to map indices
// This is critical for translating UI item indices to actual map indices
// after filtering and sorting operations are applied
#[derive(Resource, Debug, Default)]
pub struct UIMissionMapping {
    pub ui_to_map_index: Vec<usize>, // Maps UI index to original map index in maps_resource.maps
}

// Setup function for unified mission selection systems
pub fn app_setup(app: &mut App) {
    app.init_resource::<UIMissionMapping>()
        .add_systems(OnEnter(AppState::MissionSelect), setup_ui)
        .add_systems(OnExit(AppState::MissionSelect), cleanup_ui)
        // Register the interaction systems to run in the correct state
        .add_systems(
            Update,
            (
                update_input_debounce,    // Update the input debounce timer
                update_mission_selection, // Handles selection changes (hover/arrow keys)
                handle_selection_input,   // Handles confirming selection or going back
            )
                .chain() // Ensure selection updates before click/keyboard potentially changes state
                .run_if(in_state(AppState::MissionSelect)),
        );
}

// System to update the input debounce timer
fn update_input_debounce(time: Res<Time>, mut query: Query<&mut InputDebounce>) {
    for mut debounce in query.iter_mut() {
        debounce.timer.tick(time.delta());
    }
}

// System to clean up UI when exiting this state
fn cleanup_ui(
    mut commands: Commands,
    query: Query<Entity, With<MissionSelectUI>>,
    camera_query: Query<Entity, With<MissionSelectCamera>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }

    // Also clean up the camera
    for entity in camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// System to handle mission selection clicks or keyboard confirmation (Enter/Escape)
fn handle_selection_input(
    mut ev_menu_clicks: EventReader<MenuItemClicked>,
    mut ev_escape: EventReader<MenuEscapeEvent>, // Use the uncoremenu escape event
    keyboard_input: Res<ButtonInput<KeyCode>>,   // To check for Enter key
    menu_root: Query<&MenuRoot>,                 // To get the currently selected item for Enter key
    debounce_query: Query<&InputDebounce>,       // To check if input is ready
    maps_resource: Res<Maps>,                    // Access maps resource
    ui_mapping: Res<UIMissionMapping>,           // Access UI to map index mapping
    mission_select_mode: Res<CurrentMissionSelectMode>, // Current mode (Campaign/Custom)
    mut difficulty_resource: ResMut<CurrentDifficulty>, // To set the chosen difficulty
    mut ev_load_level: EventWriter<LoadLevelEvent>, // To trigger level load
    mut next_app_state: ResMut<NextState<AppState>>, // To change AppState
    mut next_map_hub_state: ResMut<NextState<MapHubState>>, // For Custom mode "Go Back"
    mut player_profile: ResMut<Persistent<unprofile::data::PlayerProfileData>>, // Player profile data
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>, // Query description text
) {
    // Check if we should ignore input events
    if let Ok(debounce) = debounce_query.get_single() {
        if !debounce.timer.finished() {
            // Still in debounce period, ignore all inputs
            ev_menu_clicks.clear();
            ev_escape.clear();
            return;
        }
    }

    let mut selected_identifier: Option<usize> = None;

    // Check for click events first
    if let Some(click_ev) = ev_menu_clicks.read().last() {
        selected_identifier = Some(click_ev.0);
    }

    // Clear the events reader after checking
    ev_menu_clicks.clear();

    // Check for Enter key press if no click happened
    if selected_identifier.is_none() && keyboard_input.just_pressed(KeyCode::Enter) {
        if let Ok(root) = menu_root.get_single() {
            selected_identifier = Some(root.selected_item);
        }
    }

    // Check for Escape key press
    let mut go_back = false;
    if ev_escape.read().last().is_some() {
        go_back = true;
    }
    // Clear the escape events reader
    ev_escape.clear();

    // Process the identified action (click or Enter)
    if let Some(ui_index) = selected_identifier {
        // Check if this is the "Go Back" button at the end of the list
        match ui_index.cmp(&ui_mapping.ui_to_map_index.len()) {
            std::cmp::Ordering::Equal => {
                // This is the "Go Back" button at the end of the list
                go_back = true;
            }
            std::cmp::Ordering::Less => {
                // It's a mission selection - get the original index from our mapping
                let original_map_idx = ui_mapping.ui_to_map_index[ui_index];
                let map = &maps_resource.maps[original_map_idx];
                let mission_data = &map.mission_data;

                // Set the CurrentDifficulty based on the mode
                match mission_select_mode.0 {
                    MissionSelectMode::Campaign => {
                        // For campaign, use the map's predefined difficulty
                        difficulty_resource.0 = mission_data.difficulty.create_difficulty_struct();
                        info!(
                            "Setting difficulty for mission: {:?} (Mode: Campaign)",
                            mission_data.difficulty
                        );
                    }
                    MissionSelectMode::Custom => {
                        // For custom, the difficulty is already set from the previous screen
                        // But we can confirm it's still correct
                        info!(
                            "Using pre-selected difficulty for mission: {:?} (Mode: Custom)",
                            difficulty_resource.0.difficulty_name
                        );
                    }
                }

                let desired_total_deposit = mission_data.required_deposit;

                // Access the player's profile directly
                let current_held_deposit = player_profile.progression.insurance_deposit;
                let additional_bank_needed = desired_total_deposit - current_held_deposit;

                // Handle the deposit based on what's needed
                match additional_bank_needed.cmp(&0) {
                    std::cmp::Ordering::Greater => {
                        if player_profile.progression.bank >= additional_bank_needed {
                            player_profile.progression.bank -= additional_bank_needed;
                            player_profile.progression.insurance_deposit += additional_bank_needed;
                        } else {
                            warn!(
                                "Insufficient money in bank for deposit. Required: ${}, Available: ${}",
                                desired_total_deposit, player_profile.progression.bank
                            );
                            if let Ok(mut text) = q_desc_text.get_single_mut() {
                                text.0 = format!(
                                    "Insufficient Money in Bank for deposit. Required: ${}, Available: ${}",
                                    desired_total_deposit, player_profile.progression.bank
                                );
                            }
                            return;
                        }
                    }
                    std::cmp::Ordering::Less => {
                        let refund_to_bank = -additional_bank_needed;
                        player_profile.progression.bank += refund_to_bank;
                        player_profile.progression.insurance_deposit -= refund_to_bank;
                    }
                    std::cmp::Ordering::Equal => {}
                }

                // Persist the updated player profile
                if let Err(e) = player_profile.persist() {
                    error!("Failed to persist PlayerProfileData: {:?}", e);
                    panic!("Profile persistence failed!");
                }

                // Proceed with loading the mission
                ev_load_level.send(LoadLevelEvent {
                    map_filepath: mission_data.map_filepath.clone(),
                });
                next_app_state.set(AppState::Loading);
                return; // Exit early after starting load
            }
            _ => {
                // Index is greater than the number of missions, do nothing
            }
        }
    }

    // Handle Go Back action (from Escape or clicking "Go Back")
    if go_back {
        match mission_select_mode.0 {
            MissionSelectMode::Campaign => {
                next_app_state.set(AppState::MainMenu);
                info!("Returning to MainMenu from mission selection.");
            }
            MissionSelectMode::Custom => {
                // For custom, go back to difficulty selection
                next_app_state.set(AppState::MapHub);
                next_map_hub_state.set(MapHubState::DifficultySelection);
                info!("Returning to DifficultySelection from mission selection.");
            }
        }
    }
}

// System to update the mission description and image when selection changes
pub fn update_mission_selection(
    mut ev_menu_selection: EventReader<MenuItemSelected>, // Read selection events
    asset_server: Res<AssetServer>,                       // To load images
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>, // Query description text
    mut q_preview_image: Query<&mut ImageNode, With<MissionPreviewImage>>, // Query preview image
    maps_resource: Res<Maps>,                             // Access maps resource
    ui_mapping: Res<UIMissionMapping>,                    // Access UI to map index mapping
    mission_select_mode: Res<CurrentMissionSelectMode>,   // Current mode (Campaign/Custom)
    difficulty_resource: Res<CurrentDifficulty>,          // For Custom mode difficulty info
) {
    // No need to filter maps here, we use the UI mapping instead
    for ev in ev_menu_selection.read() {
        let selected_ui_idx = ev.0; // Get the UI index from the event

        // Check if the index is within the bounds of our UI mapping
        if selected_ui_idx < ui_mapping.ui_to_map_index.len() {
            // Get the original map index
            let original_map_idx = ui_mapping.ui_to_map_index[selected_ui_idx];
            let map = &maps_resource.maps[original_map_idx];
            let mission_data = &map.mission_data;

            // Update Description Text
            if let Ok(mut text) = q_desc_text.get_single_mut() {
                let base_reward = mission_data.mission_reward_base;
                let required_deposit = mission_data.required_deposit;
                let potential_reward_range = format!(
                    "${:.0} - ${:.0}",
                    base_reward as f64 * 0.5,
                    base_reward as f64 * 5.0
                );

                // Format the difficulty information based on the mode
                let difficulty_info = match mission_select_mode.0 {
                    MissionSelectMode::Campaign => {
                        let dif = mission_data.difficulty.create_difficulty_struct();
                        format!(
                            "Difficulty: <{}> ({}x score)",
                            dif.difficulty_name, dif.difficulty_score_multiplier
                        )
                    }
                    MissionSelectMode::Custom => {
                        format!(
                            "Challenge: <{}> ({}x score)",
                            difficulty_resource.0.difficulty_name,
                            difficulty_resource.0.difficulty_score_multiplier
                        )
                    }
                };

                text.0 = format!(
                    "Mission: <{}>\nLocation: {}\n{}\n\n{}\n\n{}\nRequired Deposit: ${}\nReward: ${} ({})",
                    mission_data.display_name,
                    mission_data.location_name,
                    mission_data.location_address,
                    mission_data.flavor_text,
                    difficulty_info,
                    required_deposit,
                    base_reward,
                    potential_reward_range,
                );
            } else {
                warn!("MissionDescriptionText not found in UI.");
            }

            // Update Preview Image
            if let Ok(mut image) = q_preview_image.get_single_mut() {
                image.image = asset_server.load(&mission_data.preview_image_path);
            } else {
                warn!("MissionPreviewImage not found in UI.");
            }
        } else {
            // Handle selection of "Go Back" or out-of-bounds index
            // Reset description and maybe show a default image or hide preview
            if let Ok(mut text) = q_desc_text.get_single_mut() {
                text.0 = "Select a mission to view details.".to_string();
            }
            // We keep the last image shown for the "Go Back" option
        }
    }
}

// System to set up the unified mission selection UI
pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    player_profile_resource: Res<Persistent<unprofile::data::PlayerProfileData>>,
    maps_resource: Res<Maps>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    difficulty_resource: Res<CurrentDifficulty>,
    mut ui_mapping: ResMut<UIMissionMapping>,
) {
    info!(
        "Setting up MissionSelectUI for mode: {:?}",
        mission_select_mode.0
    );

    // Add a camera for the UI
    commands.spawn(Camera2d).insert(MissionSelectCamera);

    let player_level = player_profile_resource.progression.player_level;

    // STEP 1: Filter maps based on mission select mode
    // This is the critical first filtering step that was missing
    let filtered_maps: Vec<(usize, &uncore::types::root::map::Map)> = maps_resource
        .maps
        .iter()
        .enumerate() // Keep original index for UIMissionMapping
        .filter(|(_, map)| match mission_select_mode.0 {
            MissionSelectMode::Campaign => map.mission_data.is_campaign_mission,
            MissionSelectMode::Custom => !map.mission_data.is_campaign_mission,
        })
        .collect();

    // STEP 2: Apply player level filtering to get available and locked maps
    let (available_maps, locked_maps): (Vec<_>, Vec<_>) = filtered_maps
        .into_iter()
        .partition(|(_, map)| player_level >= map.mission_data.min_player_level);

    // Create or clear the UI to map index mapping
    ui_mapping.ui_to_map_index.clear();

    // Create the base entity first so we can reference it
    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MissionSelectUI) // Marker for cleanup
        .insert(InputDebounce {
            // Add InputDebounce component
            timer: Timer::from_seconds(0.5, TimerMode::Once), // 0.5 seconds debounce
        })
        .id();

    if available_maps.is_empty() && locked_maps.is_empty() {
        warn!(
            "No missions available for mode {:?} at player level {}. Cannot build selection UI.",
            mission_select_mode.0, player_level
        );
        // Spawn a message indicating no missions are available
        commands
            .entity(root_entity)
            .with_children(|parent| {
                let mode_name = match mission_select_mode.0 {
                    MissionSelectMode::Campaign => "campaign",
                    MissionSelectMode::Custom => "custom",
                };

                parent
                    .spawn(Text::new(format!(
                        "No {} missions available at your current level.",
                        mode_name
                    )))
                    .insert(TextFont {
                        font: handles.fonts.londrina.w300_light.clone(),
                        font_size: 24.0 * FONT_SCALE,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    })
                    .insert(TextColor(colors::MENU_ITEM_COLOR_OFF));

                // Create a simple "Go Back" button that's automatically selected
                parent
                    .spawn(Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(100.0),
                        align_items: AlignItems::Center,
                        justify_content: JustifyContent::Center,
                        margin: UiRect::top(Val::Px(30.0)),
                        ..default()
                    })
                    .with_children(|button_container| {
                        templates::create_content_item(
                            button_container,
                            "Go Back",
                            0,    // Make this the only and selected item
                            true, // Selected by default
                            &handles,
                        )
                        .insert(MenuItemInteractive {
                            identifier: 0,
                            selected: true,
                        })
                        .insert(Button)
                        .insert(Interaction::None);
                    });
            })
            .insert(MenuRoot { selected_item: 0 }); // Add MenuRoot to handle navigation when empty

        return;
    }

    // STEP 3: Apply unified sorting logic to both available and locked maps
    // Sorting function that applies the unified sorting logic: order → display_name → path
    let sort_maps = |a: &(usize, &uncore::types::root::map::Map),
                     b: &(usize, &uncore::types::root::map::Map)| {
        // First sort by order
        let a_order = a.1.mission_data.order.as_str();
        let b_order = b.1.mission_data.order.as_str();

        let order_cmp = a_order.cmp(b_order);
        if order_cmp != std::cmp::Ordering::Equal {
            return order_cmp;
        }

        // Then by display name
        let name_cmp =
            a.1.mission_data
                .display_name
                .cmp(&b.1.mission_data.display_name);
        if name_cmp != std::cmp::Ordering::Equal {
            return name_cmp;
        }

        // Finally by path
        a.1.path.cmp(&b.1.path)
    };

    let mut sorted_available_maps = available_maps;
    let mut sorted_locked_maps = locked_maps;

    sorted_available_maps.sort_by(sort_maps);
    sorted_locked_maps.sort_by(sort_maps);

    // Set up initial description and image for the first map if available
    let (initial_desc, initial_preview_image_path) = if !sorted_available_maps.is_empty() {
        let initial_map = &sorted_available_maps[0].1;
        let initial_mission = &initial_map.mission_data;

        // Format difficulty info based on mode
        let difficulty_info = match mission_select_mode.0 {
            MissionSelectMode::Campaign => {
                let dif = initial_mission.difficulty.create_difficulty_struct();
                format!(
                    "Difficulty: <{}> ({}x score)",
                    dif.difficulty_name, dif.difficulty_score_multiplier
                )
            }
            MissionSelectMode::Custom => {
                format!(
                    "Challenge: <{}> ({}x score)",
                    difficulty_resource.0.difficulty_name,
                    difficulty_resource.0.difficulty_score_multiplier
                )
            }
        };

        let initial_description = format!(
            "Mission: <{}>\nLocation: {}\n{}\n\n{}\n\n{}\nRequired Deposit: ${}\nReward: ${}",
            initial_mission.display_name,
            initial_mission.location_name,
            initial_mission.location_address,
            initial_mission.flavor_text,
            difficulty_info,
            initial_mission.required_deposit,
            initial_mission.mission_reward_base,
        );

        // Default preview image path
        let initial_preview = initial_mission.preview_image_path.clone();
        (initial_description, initial_preview)
    } else {
        // No available maps, use a placeholder
        let mode_name = match mission_select_mode.0 {
            MissionSelectMode::Campaign => "campaign",
            MissionSelectMode::Custom => "custom",
        };

        (
            format!(
                "No available {} missions at your current level.\n\nComplete missions to gain experience and unlock more content.",
                mode_name
            ),
            "img/placeholder_mission.png".to_string(), // Default image path
        )
    };

    // Get the title text based on the mode
    let title_text = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Campaign",
        MissionSelectMode::Custom => "Custom Mission",
    };

    // Get the subtitle based on the mode
    let subtitle_text = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Select Mission".to_string(),
        MissionSelectMode::Custom => {
            format!("Select Map ({})", difficulty_resource.0.difficulty_name)
        }
    };

    commands.entity(root_entity).with_children(|parent| {
            templates::create_background(parent, &handles);
            templates::create_logo(parent, &handles);
            templates::create_breadcrumb_navigation(
                parent,
                &handles,
                title_text,
                &subtitle_text,
            );

            // Create the main content area using the template
            let mut content_area = templates::create_selectable_content_area(
                parent,
                &handles,
                0, // Default selected index is 0
            );

            content_area.with_children(|content| {
                // Left Column: Contains the scrollable list and the scrollbar
                content
                    .spawn(Node { // Container for list + scrollbar
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    })
                    .with_children(|list_and_scrollbar_container| {
                        // Node for the scrollable list itself
                        list_and_scrollbar_container
                            .spawn(Node {
                                width: Val::Percent(90.0), // List takes most space
                                height: Val::Percent(100.0),
                                flex_direction: FlexDirection::Column,
                                align_items: AlignItems::FlexStart,
                                justify_content: JustifyContent::Start,
                                overflow: Overflow::scroll_y(),
                                ..default()
                            })
                            .insert(ScrollableListContainer) // Mark for scrollbar logic
                            .insert(ScrollPosition::default()) // Add ScrollPosition component
                            .with_children(|mission_list| {
                                let mut current_ui_index = 0;

                                // STEP 4: Populate list items from sorted_available_maps (unlocked missions)
                                // This is where we populate the UIMissionMapping
                                for (idx, (original_idx, map)) in sorted_available_maps.iter().enumerate() {
                                    // CRITICAL: Store the mapping from UI index to original map index
                                    // This is necessary for correctly identifying maps later
                                    ui_mapping.ui_to_map_index.push(*original_idx);

                                    // Create a custom menu item with map name on left and badge on right
                                    let selected = idx == 0;
                                    create_mission_list_item(
                                        mission_list,
                                        &handles,
                                        map,
                                        &player_profile_resource,
                                        current_ui_index,
                                        selected,
                                    );

                                    current_ui_index += 1;
                                }

                                // Add divider if we have both available and locked maps
                                if !sorted_available_maps.is_empty() && !sorted_locked_maps.is_empty() {
                                    mission_list.spawn(Node {
                                        min_height: Val::Px(16.0),
                                        flex_basis: Val::Px(16.0),
                                        flex_shrink: 0.0,
                                        ..Default::default()
                                    }).insert(PickingBehavior { should_block_lower: false, ..default() });
                                }

                                // Add level-locked missions (non-interactable)
                                // Note: These are NOT added to ui_mapping because they're non-interactive
                                for (_original_idx, map) in sorted_locked_maps.iter().map(|(idx, map)| (*idx, map)) {
                                    let mission_data = &map.mission_data;

                                    // Create a non-interactive mission item with "locked" appearance
                                    create_locked_mission_item(mission_list, &handles, mission_data);

                                    // Note: We don't increment current_ui_index since locked maps aren't interactive
                                    // And we don't add them to ui_mapping
                                }

                                // Add space after locked missions if any
                                if !sorted_locked_maps.is_empty() {
                                    mission_list.spawn(Node {
                                        min_height: Val::Px(16.0),
                                        flex_basis: Val::Px(16.0),
                                        flex_shrink: 0.0,
                                        ..default()
                                    }).insert(PickingBehavior { should_block_lower: false, ..default() });
                                }

                                // Add "Go Back" button - this is NOT part of ui_mapping
                                // But its ui_index is equal to ui_mapping.len() for identification
                                templates::create_content_item(
                                    mission_list,
                                    "Go Back",
                                    current_ui_index, // Index after all available missions
                                    false,
                                    &handles,
                                )
                                // Add specific marker or use index check in handler
                                .insert(MenuItemInteractive {
                                    identifier: current_ui_index,
                                    selected: false
                                });

                                mission_list.spawn(Node {
                                    width: Val::Percent(100.0),
                                    min_height: Val::Px(64.0),
                                    flex_basis: Val::Px(64.0),
                                    flex_shrink: 0.0,
                                    ..default()
                                }).insert(PickingBehavior { should_block_lower: false, ..default() });
                            });

                        // Build the scrollbar UI (using the function now in uncoremenu::scrollbar)
                        scrollbar::build_scrollbar_ui(list_and_scrollbar_container, &handles);
                    });

                // Right Column: Mission Description and Preview Image
                content
                    .spawn(Node { // Container for right column content
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::left(Val::Px(15.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|desc_column| {
                        // Spawn the Preview Image node
                        desc_column.spawn((
                                Node {
                                    width: Val::Percent(80.0),
                                    aspect_ratio: Some(16.0/9.0),
                                    margin: UiRect::bottom(Val::Px(10.0)),
                                    border: UiRect::all(Val::Px(1.0)),
                                    ..default()
                                },
                                ImageNode {
                                    image: asset_server.load(&initial_preview_image_path),
                                    ..default()
                                },
                                BorderColor(colors::TRUCKUI_ACCENT2_COLOR),
                                MissionPreviewImage, // Marker
                            ));

                        // Spawn the Description Text node using Text::new and components
                        desc_column
                            .spawn(Node {
                                // Background container with black semi-transparent background
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(colors::PANEL_BGCOLOR.with_alpha(0.95)))
                            .with_children(|text_container| {
                                text_container
                                    .spawn(Text::new(initial_desc))
                                    .insert(TextFont { // TextFont component
                                        font: handles.fonts.titillium.w300_light.clone(),
                                        font_size: 19.0 * FONT_SCALE,
                                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                                    })
                                    .insert(TextColor(colors::MENU_DESC_TEXT_COLOR)) // TextColor component
                                    .insert(TextLayout { // TextLayout component
                                        justify: JustifyText::Left, // Justify left
                                        ..default()
                                    })
                                    .insert(MissionDescriptionText); // Marker component
                            });
                    });
            });

            // Help text at the bottom - change based on mode
            let help_text = match mission_select_mode.0 {
                MissionSelectMode::Campaign =>
                    "Select a mission    |    [Up]/[Down]: Change    |    [Enter]: Start Mission    |    [ESC]: Go Back",
                MissionSelectMode::Custom =>
                    "Select a map    |    [Up]/[Down]: Change    |    [Enter]: Start Mission    |    [ESC]: Back to Difficulty Selection",
            };

            templates::create_help_text(
                parent,
                &handles,
                Some(help_text.to_string()),
            );

            // Add the persistent player status bar
            templates::create_player_status_bar(parent, &handles, &player_profile_resource);
        });
}

// Helper function to create a mission list item in the UI
fn create_mission_list_item(
    mission_list: &mut ChildBuilder,
    handles: &GameAssets,
    map: &uncore::types::root::map::Map,
    player_profile: &unprofile::data::PlayerProfileData,
    ui_index: usize,
    is_selected: bool,
) -> Entity {
    let mission_data = &map.mission_data;
    let map_path = &map.path;

    let entity_id = mission_list
        .spawn(Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(8.0 * FONT_SCALE), Val::Px(6.0 * FONT_SCALE)),
            margin: UiRect::vertical(Val::Px(2.0 * FONT_SCALE)),
            ..default()
        })
        .insert(MenuItemInteractive {
            identifier: ui_index,
            selected: is_selected,
        })
        .insert(Button)
        .insert(Interaction::None)
        .insert(BackgroundColor(if is_selected {
            Color::srgba(0.3, 0.3, 0.3, 0.1)
        } else {
            Color::NONE
        }))
        .with_children(|parent| {
            // Container for the content with space-between justification
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Mission name (left-aligned)
                    row.spawn((
                        Text::new(mission_data.display_name.clone()),
                        TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(if !is_selected {
                            colors::MENU_ITEM_COLOR_OFF
                        } else {
                            colors::MENU_ITEM_COLOR_ON
                        }),
                    ));

                    // Get the player's statistics for this map, if any

                    let player_stats = player_profile.map_statistics.get(map_path);

                    // If the player has completed this mission before, show their best grade
                    // Otherwise, show NA grade
                    let grade = if let Some(stats) = player_stats {
                        if stats.total_missions_completed > 0 {
                            stats.best_grade
                        } else {
                            Grade::NA
                        }
                    } else {
                        Grade::NA
                    };

                    // Create the badge for the mission's grade
                    BadgeUtils::create_badge(row, handles, grade, 32.0, false);
                });
        })
        .id();

    entity_id
}

// Helper function to create a locked mission list item
fn create_locked_mission_item(
    mission_list: &mut ChildBuilder,
    handles: &GameAssets,
    mission_data: &uncore::types::mission_data::MissionData,
) {
    mission_list
        .spawn(Node {
            width: Val::Percent(100.0),
            padding: UiRect::axes(Val::Px(8.0 * FONT_SCALE), Val::Px(6.0 * FONT_SCALE)),
            margin: UiRect::vertical(Val::Px(2.0 * FONT_SCALE)),
            ..default()
        })
        .insert(BackgroundColor(Color::NONE))
        .with_children(|parent| {
            // Container for the content
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .with_children(|row| {
                    // Mission name with "locked" text
                    row.spawn((
                        Text::new(format!(
                            "{} (Level {} required)",
                            mission_data.display_name, mission_data.min_player_level
                        )),
                        TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)), // Grayed out text
                    ));

                    // Lock icon
                    row.spawn(Text::new("🔒"))
                        .insert(TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5))); // Grayed out
                });
        });
}
