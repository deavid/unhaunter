//! Unified Mission Selection System
//!
//! This module provides a unified mission selection UI that serves both Campaign and Custom
//! mission modes. It replaces the previous separate implementation that used different screens for
//! Campaign and Custom missions.
//!
//! Key features:
//! - Mode-specific filtering: Shows only mission types based on CurrentMissionSelectMode
//! - Player level filtering: Shows available missions and locked missions separately
//! - Unified sorting: Applies consistent order->name->path sorting for all missions
//! - Progress tracking: Shows completion badges for missions the player has finished
//! - Different navigation flows based on mode:
//!   * Campaign: Main Menu -> Mission Selection -> Game
//!   * Custom: Main Menu -> Difficulty Selection -> Mission Selection -> Game
//! - Proper UI mapping between list items and the original maps collection

use bevy::picking::PickingBehavior;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
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
use uncoremenu::components::MenuMouseTracker;
use uncoremenu::events::KeyboardNavigate;
use uncoremenu::scrollbar::ScrollableListContainer;
use uncoremenu::{
    components::{MenuItemInteractive, MenuRoot},
    scrollbar,
    systems::MenuItemSelected,
    systems::{MenuEscapeEvent, MenuItemClicked},
    templates,
};
use unmaphub::badge_utils::BadgeUtils;

/// Marker component for the unified Mission Select UI root node
#[derive(Component)]
pub struct MissionSelectUI;

/// Component for the camera in mission selection
#[derive(Component)]
struct MissionSelectCamera;

/// Component for the description text area
#[derive(Component)]
pub struct MissionDescriptionText;

/// Component for the preview image area
#[derive(Component)]
pub struct MissionPreviewImage;

/// Resource to track UI mapping to map indices
/// This is critical for translating UI item indices to actual map indices
/// after filtering and sorting operations are applied
#[derive(Resource, Debug, Default)]
pub struct UIMissionMapping {
    /// Maps UI index to original map index in maps_resource.maps
    pub ui_to_map_index: Vec<usize>,
}

#[derive(Resource, Default)]
pub struct InitialScrollTarget(Option<usize>);

/// Setup function for unified mission selection systems
pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<UIMissionMapping>()
        .init_resource::<InitialScrollTarget>()
        .add_systems(OnEnter(AppState::MissionSelect), setup_ui)
        .add_systems(OnExit(AppState::MissionSelect), cleanup_ui)
        .add_systems(
            Update,
            (
                update_mission_selection,
                handle_selection_input,
                trigger_initial_scroll_if_needed,
            )
                .chain()
                .run_if(in_state(AppState::MissionSelect)),
        );
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

    for entity in camera_query.iter() {
        commands.entity(entity).despawn_recursive();
    }
}

// System to handle mission selection clicks or keyboard confirmation (Enter/Escape)
fn handle_selection_input(
    mut ev_menu_clicks: EventReader<MenuItemClicked>,
    mut ev_escape: EventReader<MenuEscapeEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    menu_root: Query<&MenuRoot>,
    maps_resource: Res<Maps>,
    ui_mapping: Res<UIMissionMapping>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    mut difficulty_resource: ResMut<CurrentDifficulty>,
    mut ev_load_level: EventWriter<LoadLevelEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut player_profile: ResMut<Persistent<unprofile::data::PlayerProfileData>>,
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>,
) {
    let mut selected_identifier: Option<usize> = None;

    if let Some(click_ev) = ev_menu_clicks.read().last() {
        if click_ev.state != AppState::MissionSelect {
            warn!(
                "MenuItemClicked event received in state: {:?}",
                click_ev.state
            );
            return;
        }
        selected_identifier = Some(click_ev.pos);
    }

    ev_menu_clicks.clear();

    if selected_identifier.is_none() && keyboard_input.just_pressed(KeyCode::Enter) {
        if let Ok(root) = menu_root.get_single() {
            selected_identifier = Some(root.selected_item);
        }
    }

    let mut go_back = false;
    if ev_escape.read().last().is_some() {
        go_back = true;
    }
    ev_escape.clear();

    if let Some(ui_index) = selected_identifier {
        match ui_index.cmp(&ui_mapping.ui_to_map_index.len()) {
            std::cmp::Ordering::Equal => {
                go_back = true;
            }
            std::cmp::Ordering::Less => {
                let original_map_idx = ui_mapping.ui_to_map_index[ui_index];
                let map = &maps_resource.maps[original_map_idx];
                let mission_data = &map.mission_data;

                match mission_select_mode.0 {
                    MissionSelectMode::Campaign => {
                        difficulty_resource.0 = mission_data.difficulty.create_difficulty_struct();
                        info!(
                            "Setting difficulty for mission: {:?} (Mode: Campaign)",
                            mission_data.difficulty
                        );
                    }
                    MissionSelectMode::Custom => {
                        info!(
                            "Using pre-selected difficulty for mission: {:?} (Mode: Custom)",
                            difficulty_resource.0.difficulty_name
                        );
                    }
                }

                let desired_total_deposit = mission_data.required_deposit;

                let current_held_deposit = player_profile.progression.insurance_deposit;
                let additional_bank_needed = desired_total_deposit - current_held_deposit;

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

                if let Err(e) = player_profile.persist() {
                    error!("Failed to persist PlayerProfileData: {:?}", e);
                    panic!("Profile persistence failed!");
                }

                ev_load_level.send(LoadLevelEvent {
                    map_filepath: mission_data.map_filepath.clone(),
                });
                next_app_state.set(AppState::Loading);
                return;
            }
            _ => {}
        }
    }

    if go_back {
        match mission_select_mode.0 {
            MissionSelectMode::Campaign => {
                next_app_state.set(AppState::MainMenu);
                info!("Returning to MainMenu from mission selection.");
            }
            MissionSelectMode::Custom => {
                next_app_state.set(AppState::MapHub);
                next_map_hub_state.set(MapHubState::DifficultySelection);
                info!("Returning to DifficultySelection from mission selection.");
            }
        }
    }
}

/// System to update the mission description and image when selection changes
pub fn update_mission_selection(
    mut ev_menu_selection: EventReader<MenuItemSelected>,
    asset_server: Res<AssetServer>,
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>,
    mut q_preview_image: Query<&mut ImageNode, With<MissionPreviewImage>>,
    maps_resource: Res<Maps>,
    ui_mapping: Res<UIMissionMapping>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    difficulty_resource: Res<CurrentDifficulty>,
) {
    for ev in ev_menu_selection.read() {
        let selected_ui_idx = ev.0;

        if selected_ui_idx < ui_mapping.ui_to_map_index.len() {
            let original_map_idx = ui_mapping.ui_to_map_index[selected_ui_idx];
            let map = &maps_resource.maps[original_map_idx];
            let mission_data = &map.mission_data;

            if let Ok(mut text) = q_desc_text.get_single_mut() {
                let base_reward = mission_data.mission_reward_base;
                let required_deposit = mission_data.required_deposit;
                let potential_reward_range = format!(
                    "${:.0} - ${:.0}",
                    base_reward as f64 * 0.5,
                    base_reward as f64 * 5.0
                );

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

            if let Ok(mut image) = q_preview_image.get_single_mut() {
                let initial_preview = if mission_data.preview_image_path.is_empty() {
                    "img/placeholder_mission.png".to_string()
                } else {
                    mission_data.preview_image_path.clone()
                };

                image.image = asset_server.load(&initial_preview);
            } else {
                warn!("MissionPreviewImage not found in UI.");
            }
        } else if let Ok(mut text) = q_desc_text.get_single_mut() {
            text.0 = "Select a mission to view details.".to_string();
        }
    }
}

/// System to set up the unified mission selection UI
pub fn setup_ui(
    mut commands: Commands,
    handles: Res<GameAssets>,
    asset_server: Res<AssetServer>,
    player_profile_resource: Res<Persistent<unprofile::data::PlayerProfileData>>,
    maps_resource: Res<Maps>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    difficulty_resource: Res<CurrentDifficulty>,
    mut ui_mapping: ResMut<UIMissionMapping>,
    mut initial_scroll_target: ResMut<InitialScrollTarget>,
) {
    info!(
        "Setting up MissionSelectUI for mode: {:?}",
        mission_select_mode.0
    );

    commands.spawn(Camera2d).insert(MissionSelectCamera);

    let player_level = player_profile_resource.progression.player_level;

    let filtered_maps: Vec<(usize, &uncore::types::root::map::Map)> = maps_resource
        .maps
        .iter()
        .enumerate()
        .filter(|(_, map)| match mission_select_mode.0 {
            MissionSelectMode::Campaign => map.mission_data.is_campaign_mission,
            MissionSelectMode::Custom => !map.mission_data.is_campaign_mission,
        })
        .collect();

    let (available_maps, locked_maps): (Vec<_>, Vec<_>) = filtered_maps
        .into_iter()
        .partition(|(_, map)| player_level >= map.mission_data.min_player_level);

    ui_mapping.ui_to_map_index.clear();

    let root_entity = commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(MissionSelectUI)
        .id();

    if available_maps.is_empty() && locked_maps.is_empty() {
        warn!(
            "No missions available for mode {:?} at player level {}. Cannot build selection UI.",
            mission_select_mode.0, player_level
        );
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
                            0,
                            true,
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
            .insert(MenuRoot { selected_item: 0 });

        return;
    }

    let sort_maps = |a: &(usize, &uncore::types::root::map::Map),
                     b: &(usize, &uncore::types::root::map::Map)| {
        let a_order = a.1.mission_data.order.as_str();
        let b_order = b.1.mission_data.order.as_str();

        let order_cmp = a_order.cmp(b_order);
        if order_cmp != std::cmp::Ordering::Equal {
            return order_cmp;
        }

        let name_cmp =
            a.1.mission_data
                .display_name
                .cmp(&b.1.mission_data.display_name);
        if name_cmp != std::cmp::Ordering::Equal {
            return name_cmp;
        }

        a.1.path.cmp(&b.1.path)
    };

    let mut sorted_available_maps = available_maps;
    let mut sorted_locked_maps = locked_maps;

    sorted_available_maps.sort_by(sort_maps);
    sorted_locked_maps.sort_by(sort_maps);

    let mut default_selected_idx_in_sorted_list = 0;

    let (initial_desc, initial_preview_image_path) = if !sorted_available_maps.is_empty() {
        // Ensure this is inside the 'if !sorted_available_maps.is_empty() { ... }' block
        // and after 'default_selected_idx_in_sorted_list' has been initialized (e.g., to 0).

        if mission_select_mode.0 == MissionSelectMode::Campaign {
            // Iterate available maps from most advanced (last in the 'sorted_available_maps' list, due to .rev())
            // to least advanced. 'idx' will be the original index from the forward iteration.
            for (idx, (_original_map_idx, map_info)) in
                sorted_available_maps.iter().enumerate().rev()
            {
                let current_map_mission_data = &map_info.mission_data;

                // Get player's financial details for clarity
                let player_bank_balance = player_profile_resource.progression.bank;
                let player_insurance_deposit =
                    player_profile_resource.progression.insurance_deposit;

                // Get map's financial requirements for clarity
                let map_required_total_deposit = current_map_mission_data.required_deposit;

                // Calculate how much *additional* money is needed from the bank,
                // if the map's required deposit is more than what's already in the player's insurance deposit.
                let additional_funds_needed_from_bank =
                    map_required_total_deposit - player_insurance_deposit;

                // Determine if the player can afford this map
                let is_map_affordable = if additional_funds_needed_from_bank > 0 {
                    // Player needs to pull money from their bank account.
                    // Check if bank balance is sufficient for the additional amount needed.
                    player_bank_balance >= additional_funds_needed_from_bank
                } else {
                    // Player's current insurance deposit already covers or exceeds the map's requirement.
                    // No additional funds are needed from the bank, or they might even get a refund from their deposit.
                    // Thus, the map is considered affordable in terms of bank funds.
                    true
                };

                if is_map_affordable {
                    // This map is affordable. Since we are iterating from most advanced to least,
                    // this is the most advanced affordable map.
                    default_selected_idx_in_sorted_list = idx; // Set this map as the default selection.
                    break; // Exit the loop as we've found our target.
                }
            }
            // If the loop completes without finding an affordable map (i.e., 'break' was never called),
            // 'default_selected_idx_in_sorted_list' will retain its initial value (e.g., 0).
            // This will result in the first map in 'sorted_available_maps' being selected as a fallback.
        }

        let initial_map = &sorted_available_maps[default_selected_idx_in_sorted_list].1;
        let initial_mission = &initial_map.mission_data;

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

        let base_reward = initial_mission.mission_reward_base;
        let potential_reward_range = format!(
            "${:.0} - ${:.0}",
            base_reward as f64 * 0.5,
            base_reward as f64 * 5.0
        );

        let initial_description = format!(
            "Mission: <{}>\nLocation: {}\n{}\n\n{}\n\n{}\nRequired Deposit: ${}\nReward: ${} ({})",
            initial_mission.display_name,
            initial_mission.location_name,
            initial_mission.location_address,
            initial_mission.flavor_text,
            difficulty_info,
            initial_mission.required_deposit,
            base_reward,
            potential_reward_range,
        );

        let initial_preview = if initial_mission.preview_image_path.is_empty() {
            "img/placeholder_mission.png".to_string()
        } else {
            initial_mission.preview_image_path.clone()
        };
        (initial_description, initial_preview)
    } else {
        let mode_name = match mission_select_mode.0 {
            MissionSelectMode::Campaign => "campaign",
            MissionSelectMode::Custom => "custom",
        };

        (
            format!(
                "No available {} missions at your current level.\n\nComplete missions to gain experience and unlock more content.",
                mode_name
            ),
            "img/placeholder_mission.png".to_string(),
        )
    };
    let title_text = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Campaign",
        MissionSelectMode::Custom => "Custom Mission",
    };

    let subtitle_text = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Select Mission".to_string(),
        MissionSelectMode::Custom => {
            format!("Select Map\n  ({})", difficulty_resource.0.difficulty_name)
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

            let mut content_area = templates::create_selectable_content_area(
                parent,
                &handles,
                default_selected_idx_in_sorted_list,
            );
            content_area.insert(MenuMouseTracker::default());

            content_area.with_children(|content| {
                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Row,
                        align_items: AlignItems::Stretch,
                        ..default()
                    })
                    .with_children(|list_and_scrollbar_container| {
                        list_and_scrollbar_container
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
                            .with_children(|mission_list| {
                                let mut current_ui_index = 0;

                                for (idx, (original_idx, map)) in sorted_available_maps.iter().enumerate() {
                                    ui_mapping.ui_to_map_index.push(*original_idx);

                                    let selected = idx == default_selected_idx_in_sorted_list;
                                    create_mission_list_item(
                                        mission_list,
                                        &handles,
                                        map,
                                        &player_profile_resource,
                                        current_ui_index,
                                        selected,
                                        &mission_select_mode, // Added
                                        &difficulty_resource, // Added
                                    );

                                    current_ui_index += 1;
                                }

                                if !sorted_available_maps.is_empty() && !sorted_locked_maps.is_empty() {
                                    mission_list.spawn(Node {
                                        min_height: Val::Px(16.0),
                                        flex_basis: Val::Px(16.0),
                                        flex_shrink: 0.0,
                                        ..Default::default()
                                    }).insert(PickingBehavior { should_block_lower: false, ..default() });
                                }

                                // Add locked missions - but just the first one because we just want to show the player that there are locked missions
                                if let Some((_original_idx, map)) = sorted_locked_maps.iter().map(|(idx, map)| (*idx, map)).next() {
                                    let mission_data = &map.mission_data;

                                    create_locked_mission_item(mission_list, &handles, mission_data);
                                }

                                if !sorted_locked_maps.is_empty() {
                                    mission_list.spawn(Node {
                                        min_height: Val::Px(16.0),
                                        flex_basis: Val::Px(16.0),
                                        flex_shrink: 0.0,
                                        ..default()
                                    }).insert(PickingBehavior { should_block_lower: false, ..default() });
                                }

                                templates::create_content_item(
                                    mission_list,
                                    "Go Back",
                                    current_ui_index,
                                    false,
                                    &handles,
                                )
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

                        scrollbar::build_scrollbar_ui(list_and_scrollbar_container, &handles);
                    });

                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::left(Val::Px(15.0)),
                        justify_content: JustifyContent::Center,
                        ..default()
                    })
                    .with_children(|desc_column| {
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
                                MissionPreviewImage,
                            ));

                        desc_column
                            .spawn(Node {
                                width: Val::Percent(100.0),
                                padding: UiRect::all(Val::Px(10.0)),
                                ..default()
                            })
                            .insert(BackgroundColor(colors::PANEL_BGCOLOR.with_alpha(0.95)))
                            .with_children(|text_container| {
                                text_container
                                    .spawn(Text::new(initial_desc))
                                    .insert(TextFont {
                                        font: handles.fonts.titillium.w300_light.clone(),
                                        font_size: 19.0 * FONT_SCALE,
                                        font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                                    })
                                    .insert(TextColor(colors::MENU_DESC_TEXT_COLOR))
                                    .insert(TextLayout {
                                        justify: JustifyText::Left,
                                        ..default()
                                    })
                                    .insert(MissionDescriptionText);
                            });
                    });
            });

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

            templates::create_player_status_bar(parent, &handles, &player_profile_resource);
        });

    // If there are available missions, set the target for initial scroll.
    if !sorted_available_maps.is_empty() {
        initial_scroll_target.0 = Some(default_selected_idx_in_sorted_list);
    } else {
        initial_scroll_target.0 = None;
    }
}

/// Helper function to create a mission list item in the UI
fn create_mission_list_item(
    mission_list: &mut ChildBuilder,
    handles: &GameAssets,
    map: &uncore::types::root::map::Map,
    player_profile: &unprofile::data::PlayerProfileData,
    ui_index: usize,
    is_selected: bool,
    mission_select_mode: &CurrentMissionSelectMode,
    difficulty_resource: &CurrentDifficulty,
) -> Entity {
    let mission_data = &map.mission_data;
    let map_path = &map.path;

    mission_list
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
        .insert(PickingBehavior {
            should_block_lower: false,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(PickingBehavior {
                    should_block_lower: false,
                    ..default()
                })
                .with_children(|row| {
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
                        // Add the PrincipalMenuText marker for visual state updates
                        uncoremenu::components::PrincipalMenuText,
                    ))
                    .insert(PickingBehavior {
                        should_block_lower: false,
                        ..default()
                    });

                    let target_difficulty_for_badge = match mission_select_mode.0 {
                        MissionSelectMode::Campaign => mission_data.difficulty,
                        MissionSelectMode::Custom => difficulty_resource.0.difficulty,
                    };

                    let grade = if let Some(map_difficulties_stats) =
                        player_profile.map_statistics.get(map_path)
                    {
                        if let Some(stats_for_target_difficulty) =
                            map_difficulties_stats.get(&target_difficulty_for_badge)
                        {
                            if stats_for_target_difficulty.total_missions_completed > 0 {
                                stats_for_target_difficulty.best_grade
                            } else {
                                Grade::NA
                            }
                        } else {
                            Grade::NA // No stats for this specific difficulty
                        }
                    } else {
                        Grade::NA // No stats for this map at all
                    };

                    BadgeUtils::create_badge(row, handles, grade, 32.0, false);
                });
        })
        .id()
}

/// Helper function to create a locked mission list item
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
        .insert(PickingBehavior {
            should_block_lower: false,
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn(Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                })
                .insert(PickingBehavior {
                    should_block_lower: false,
                    ..default()
                })
                .with_children(|row| {
                    row.spawn((
                        Text::new(format!(
                            "Unlock Level {} for more missions",
                            mission_data.min_player_level
                        )),
                        TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        },
                        TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)),
                        // Add the PrincipalMenuText marker for visual state updates
                        uncoremenu::components::PrincipalMenuText,
                    ))
                    .insert(PickingBehavior {
                        should_block_lower: false,
                        ..default()
                    });

                    row.spawn(Text::new("🔒"))
                        .insert(TextFont {
                            font: handles.fonts.titillium.w400_regular.clone(),
                            font_size: 24.0 * FONT_SCALE,
                            font_smoothing: bevy::text::FontSmoothing::AntiAliased,
                        })
                        .insert(TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)));
                })
                .insert(PickingBehavior {
                    should_block_lower: false,
                    ..default()
                });
        });
}

fn trigger_initial_scroll_if_needed(
    mut initial_scroll_target: ResMut<InitialScrollTarget>,
    mut ev_keyboard_nav: EventWriter<KeyboardNavigate>,
    container_query: Query<&ComputedNode, With<ScrollableListContainer>>,
) {
    if let Some(target_idx) = initial_scroll_target.0 {
        if let Ok(container_node) = container_query.get_single() {
            if container_node.size().y > 0.0 {
                // Check if container height is calculated
                info!(
                    "Initial scroll: UI ready. Triggering scroll to index: {}",
                    target_idx
                );
                ev_keyboard_nav.send(KeyboardNavigate(target_idx));
                initial_scroll_target.0 = None; // Clear the target so this doesn't run again
            } else {
                // Log that UI is not ready, will retry next frame.
                // info!("Initial scroll: UI not ready yet (container height is 0). Will retry.");
            }
        } else {
            // Log that container is not found, will retry next frame.
            // info!("Initial scroll: ScrollableListContainer not found yet. Will retry.");
        }
    }
}
