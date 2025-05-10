use bevy::picking::PickingBehavior;
use bevy::prelude::*;
use bevy::ui::ScrollPosition;
use bevy_persistent::Persistent;

use uncore::colors;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::loadlevel::LoadLevelEvent;
use uncore::platform::plt::FONT_SCALE;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
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

// Marker component for the Campaign Mission Select UI root node
#[derive(Component)]
pub struct CampaignMissionSelectUI;

// Component for the camera in campaign mission selection
#[derive(Component)]
struct CampaignCamera;

// Component to link a UI list item to a specific Map with MissionData
#[derive(Component, Debug)]
pub struct CampaignMissionItem {
    pub mission_index: usize,
}

// Component for the description text area
#[derive(Component)]
pub struct MissionDescriptionText;

// Component for the preview image area
#[derive(Component)]
pub struct MissionPreviewImage;

// Setup function for campaign mission selection systems
pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::CampaignMissionSelect), setup_ui)
        .add_systems(OnExit(AppState::CampaignMissionSelect), cleanup_ui)
        // Register the interaction systems to run in the correct state
        .add_systems(
            Update,
            (
                update_input_debounce,    // Update the input debounce timer
                update_mission_selection, // Handles selection changes (hover/arrow keys)
                handle_selection_input,   // Handles confirming selection or going back
            )
                .chain() // Ensure selection updates before click/keyboard potentially changes state
                .run_if(in_state(AppState::CampaignMissionSelect)),
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
    query: Query<Entity, With<CampaignMissionSelectUI>>,
    camera_query: Query<Entity, With<CampaignCamera>>,
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
    _q_items: Query<&CampaignMissionItem>, // Query items to get mission index (optional, can use identifier directly)
    maps_resource: Res<Maps>,              // Access maps resource
    mut difficulty_resource: ResMut<CurrentDifficulty>, // To set the chosen difficulty
    mut ev_load_level: EventWriter<LoadLevelEvent>, // To trigger level load
    mut next_app_state: ResMut<NextState<AppState>>, // To change AppState
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

    // Get campaign missions
    let campaign_maps: Vec<_> = maps_resource
        .maps
        .iter()
        .filter(|map| map.mission_data.is_campaign_mission)
        .collect();

    // Process the identified action (click or Enter)
    if let Some(identifier) = selected_identifier {
        // Use match statement instead of if chain as suggested by clippy
        match identifier.cmp(&campaign_maps.len()) {
            std::cmp::Ordering::Equal => {
                // It's the "Go Back" button
                go_back = true;
            }
            std::cmp::Ordering::Less => {
                // It's a mission selection
                let map = &campaign_maps[identifier];
                let mission_data = &map.mission_data;

                // Set the CurrentDifficulty based on the mission's fixed difficulty
                difficulty_resource.0 = mission_data.difficulty.create_difficulty_struct();
                info!(
                    "Setting difficulty for campaign mission: {:?}",
                    mission_data.difficulty
                );

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
        next_app_state.set(AppState::MainMenu);
        info!("Returning to MainMenu from CampaignMissionSelect.");
    }
}

// System to update the mission description and image when selection changes
pub fn update_mission_selection(
    mut ev_menu_selection: EventReader<MenuItemSelected>, // Read selection events
    asset_server: Res<AssetServer>,                       // To load images
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>, // Query description text
    mut q_preview_image: Query<&mut ImageNode, With<MissionPreviewImage>>, // Query preview image
    maps_resource: Res<Maps>,                             // Access maps resource
) {
    // Get campaign missions
    let campaign_maps: Vec<_> = maps_resource
        .maps
        .iter()
        .filter(|map| map.mission_data.is_campaign_mission)
        .collect();

    for ev in ev_menu_selection.read() {
        let selected_idx = ev.0; // Get the index from the event

        // Check if the index is within the bounds of loaded missions
        if selected_idx < campaign_maps.len() {
            let map = &campaign_maps[selected_idx];
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

                text.0 = format!(
                    "Mission: <{}>\nLocation: {}\n{}\n\n{}\n\nRequired Deposit: ${}\nReward: ${} ({})",
                    mission_data.display_name,
                    mission_data.location_name,
                    mission_data.location_address,
                    mission_data.flavor_text,
                    required_deposit,
                    base_reward,
                    potential_reward_range,
                );
            } else {
                warn!("MissionDescriptionText not found in UI.");
            }

            // Update Preview Image (using ImageNode for Bevy 0.15)
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

// System to set up the campaign mission selection UI
pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    asset_server: Res<AssetServer>, // Needed for loading preview images
    player_profile_resource: Res<Persistent<unprofile::data::PlayerProfileData>>, // Player profile data
    maps_resource: Res<Maps>, // Access maps resource
) {
    info!("Setting up CampaignMissionSelectUI...");

    // Add a camera for the UI
    commands.spawn(Camera2d).insert(CampaignCamera);

    let player_level = player_profile_resource.progression.player_level;

    // Filter and collect maps that have campaign mission data
    let available_missions: Vec<_> = maps_resource
        .maps
        .iter()
        .filter(|map| {
            map.mission_data.is_campaign_mission
                && player_level >= map.mission_data.min_player_level
        })
        .collect();

    if available_missions.is_empty() {
        warn!(
            "No campaign missions available for player level {}. Cannot build selection UI.",
            player_level
        );
        // Spawn a message indicating no missions are available
        commands
            .spawn(Node {
                // Root node for cleanup
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            })
            .insert(CampaignMissionSelectUI)
            .with_children(|parent| {
                parent
                    .spawn(Text::new("No campaign missions found."))
                    .insert(TextFont {
                        font: handles.fonts.londrina.w300_light.clone(),
                        font_size: 24.0 * FONT_SCALE,
                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                    })
                    .insert(TextColor(colors::MENU_ITEM_COLOR_OFF));
            });
        return;
    }

    // Sort missions by order (for campaign progression)
    let mut sorted_missions = available_missions.clone();
    sorted_missions.sort_by(|a, b| {
        let a_order = a.mission_data.order.as_str();
        let b_order = b.mission_data.order.as_str();
        a_order.cmp(b_order)
    });

    let initial_mission = &sorted_missions[0].mission_data;
    let initial_desc = format!(
        "Mission: <{}>\nLocation: {}\n{}\n\n{}",
        initial_mission.display_name,
        initial_mission.location_name,
        initial_mission.location_address,
        initial_mission.flavor_text,
    );
    // Default preview image path
    let initial_preview_image_path = initial_mission.preview_image_path.clone();

    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(CampaignMissionSelectUI) // Marker for cleanup
        .insert(InputDebounce {
            // Add InputDebounce component
            timer: Timer::from_seconds(0.5, TimerMode::Once), // 0.5 seconds debounce
        })
        .id();

    commands.entity(root_entity).with_children(|parent| {
            templates::create_background(parent, &handles);
            templates::create_logo(parent, &handles);
            templates::create_breadcrumb_navigation(
                parent,
                &handles,
                "Campaign",
                "Select Mission",
            );

            // Create the main content area using the template
            // This template now implicitly adds MenuRoot and MenuMouseTracker
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
                                // Populate list items from sorted_missions
                                for (idx, map) in sorted_missions.iter().enumerate() {
                                    let mission_data = &map.mission_data;
                                    // Create a custom menu item with map name on left and badge on right
                                    let selected = idx == 0;
                                    let mut entity = mission_list.spawn(Node {
                                        width: Val::Percent(100.0),
                                        padding: UiRect::axes(Val::Px(8.0 * FONT_SCALE), Val::Px(6.0 * FONT_SCALE)),
                                        margin: UiRect::vertical(Val::Px(2.0 * FONT_SCALE)),
                                        ..default()
                                    });

                                    // Add interactive behavior components
                                    let item = entity.insert(MenuItemInteractive {
                                        identifier: idx,
                                        selected,
                                    })
                                    .insert(Button)
                                    .insert(Interaction::None)
                                    .insert(BackgroundColor(if selected {
                                        Color::srgba(0.3, 0.3, 0.3, 0.1)
                                    } else {
                                        Color::NONE
                                    }))
                                    .insert(CampaignMissionItem { mission_index: idx });

                                    // Create a layout with name on left and badge on right
                                    item.with_children(|parent| {
                                        // Container for the content with space-between justification
                                        parent.spawn(Node {
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
                                                TextColor(if !selected {
                                                    colors::MENU_ITEM_COLOR_OFF
                                                } else {
                                                    colors::MENU_ITEM_COLOR_ON
                                                }),
                                            ));

                                            // Get the player's statistics for this map, if any
                                            let player_stats = player_profile_resource
                                                .map_statistics
                                                .get(&map.path);

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
                                            BadgeUtils::create_badge(row, &handles, grade, 32.0, false);
                                        });
                                    });
                                }

                                mission_list.spawn(Node {
                                    min_height: Val::Px(16.0),
                                    flex_basis: Val::Px(16.0),
                                    flex_shrink: 0.0,
                                    ..Default::default()
                                }).insert(PickingBehavior { should_block_lower: false, ..default() });

                            // Add "Go Back" button
                            templates::create_content_item(
                                mission_list,
                                "Go Back",
                                sorted_missions.len(), // Index after last mission
                                false,
                                &handles,
                            )
                            // Add specific marker or use index check in handler
                            .insert(MenuItemInteractive { identifier: sorted_missions.len(), selected: false });

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
                                    image: asset_server.load(initial_preview_image_path),
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

            // Help text at the bottom
            templates::create_help_text(
                parent,
                &handles,
                Some(
                    "Select a campaign mission    |    [Up]/[Down]: Change    |    [Enter]: Start Mission    |    [ESC]: Go Back"
                        .to_string(),
                ),
            );

            // Add the persistent player status bar
            templates::create_player_status_bar(parent, &handles, &player_profile_resource);
        });
}
