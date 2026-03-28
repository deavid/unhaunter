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

use crate::assets::CampaignAssets;
use crate::badge_utils::BadgeUtils;
use bevy::picking::Pickable;
use bevy::prelude::*;
use bevy::ui::ComputedNode;
use bevy::ui::ScrollPosition;
use bevy_persistent::Persistent;
use uncommon_app_core::platform::plt::FONT_SCALE;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unmaphub_core::states::MapHubState;
use unmapload_core::events::loadlevel::LoadLevelEvent;
use unmenu_core::assets::MenuAssets;
use unmenu_core::colors as foundation_colors;
use unmenu_core::colors;
use unmenu_core::components::MenuMouseTracker;
use unmenu_core::events::KeyboardNavigate;
use unmenu_core::mission_select::{CurrentMissionSelectMode, MissionSelectMode};
use unmenu_core::scrollbar::ScrollableListContainer;
use unmenu_core::{
    components::{MenuItemInteractive, MenuRoot},
    events::MenuItemSelected,
    events::{MenuEscapeEvent, MenuItemClicked},
    scrollbar, templates,
};
use unorchestrator_core::UIContextState;
use untmxmap_core::resources::maps::Maps;

/// Marker component for the unified Mission Select UI root node
#[derive(Component)]
pub(crate) struct MissionSelectUI;

/// Component for the camera in mission selection
#[derive(Component)]
struct MissionSelectCamera;

/// Component for the description text area
#[derive(Component)]
pub(crate) struct MissionDescriptionText;

/// Component for the preview image area
#[derive(Component)]
pub(crate) struct MissionPreviewImage;

/// Resource to track UI mapping to map indices
/// This is critical for translating UI item indices to actual map indices
/// after filtering and sorting operations are applied
#[derive(Resource, Debug, Default)]
pub(crate) struct UIMissionMapping {
    /// Maps UI index to original map index in maps_resource.maps
    pub ui_to_map_index: Vec<usize>,
}

#[derive(Resource, Default)]
pub(crate) struct InitialScrollTarget(Option<usize>);

/// Setup function for unified mission selection systems
pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<UIMissionMapping>()
        .init_resource::<InitialScrollTarget>()
        .add_systems(OnEnter(UIContextState::MissionSelect), setup_ui)
        .add_systems(OnExit(UIContextState::MissionSelect), cleanup_ui)
        .add_systems(
            Update,
            (
                update_mission_selection,
                handle_selection_input,
                trigger_initial_scroll_if_needed,
            )
                .chain()
                .run_if(in_state(UIContextState::MissionSelect)),
        );
}

// System to clean up UI when exiting this state
fn cleanup_ui(
    mut commands: Commands,
    query: Query<Entity, With<MissionSelectUI>>,
    camera_query: Query<Entity, With<MissionSelectCamera>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn();
    }

    for entity in camera_query.iter() {
        commands.entity(entity).despawn();
    }
}

// System to handle mission selection clicks or keyboard confirmation (Enter/Escape)
fn handle_selection_input(
    mut ev_menu_clicks: MessageReader<MenuItemClicked>,
    mut ev_escape: MessageReader<MenuEscapeEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    menu_root: Query<&MenuRoot>,
    maps_resource: Res<Maps>,
    ui_mapping: Res<UIMissionMapping>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    mut difficulty_resource: ResMut<CurrentDifficulty>,
    mut ev_load_level: MessageWriter<LoadLevelEvent>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut player_profile: ResMut<Persistent<unprofile_core::profile::PlayerProfileData>>,
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>,
) {
    let mut selected_identifier: Option<usize> = None;

    if let Some(click_ev) = ev_menu_clicks.read().last() {
        if click_ev.state != UIContextState::MissionSelect {
            warn!(
                "MenuItemClicked event received in state: {:?}",
                click_ev.state
            );
            return;
        }
        selected_identifier = Some(click_ev.pos);
    }
    ev_menu_clicks.clear();

    if selected_identifier.is_none()
        && keyboard_input.just_pressed(KeyCode::Enter)
        && let Ok(root) = menu_root.single()
    {
        selected_identifier = Some(root.selected_item);
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
                        difficulty_resource.0 = mission_data.difficulty;
                        info!(
                            "Setting difficulty for mission: {:?} (Mode: Campaign)",
                            mission_data.difficulty
                        );
                    }
                    MissionSelectMode::Custom => {
                        info!(
                            "Using pre-selected difficulty for mission: {:?} (Mode: Custom)",
                            difficulty_resource.0.difficulty_name()
                        );
                    }
                }

                if let Err(error_msg) = player_profile
                    .progression
                    .update_deposit(mission_data.required_deposit)
                {
                    info!("{}", error_msg);
                    if let Ok(mut text) = q_desc_text.single_mut() {
                        text.0 = error_msg;
                    }
                    return;
                }

                if let Err(e) = player_profile.persist() {
                    error!("Failed to persist PlayerProfileData: {:?}", e);
                    panic!("Profile persistence failed!");
                }

                ev_load_level.write(LoadLevelEvent {
                    map_filepath: mission_data.map_filepath.clone(),
                });
                // SP-5: exit to InGame is handled by SimulationState observer in unreplicon-plugin.
                next_app_state.set(UIContextState::MissionLoading);
                return;
            }
            _ => {}
        }
    }

    if go_back {
        match mission_select_mode.0 {
            MissionSelectMode::Campaign => {
                next_app_state.set(UIContextState::MainMenu);
                info!("Returning to MainMenu from mission selection.");
            }
            MissionSelectMode::Custom => {
                next_app_state.set(UIContextState::MapHub);
                next_map_hub_state.set(MapHubState::DifficultySelection);
                info!("Returning to DifficultySelection from mission selection.");
            }
        }
    }
}

fn format_mission_details(
    m: &untmxmap_core::types::mission_data::TmxMissionData,
    mode: MissionSelectMode,
    dif: &CurrentDifficulty,
) -> (String, String) {
    let (dname, dmult, prefix) = match mode {
        MissionSelectMode::Campaign => (
            m.difficulty.difficulty_name().to_string(),
            m.difficulty.difficulty_score_multiplier(),
            "Difficulty",
        ),
        MissionSelectMode::Custom => (
            dif.0.difficulty_name().to_string(),
            dif.0.difficulty_score_multiplier(),
            "Challenge",
        ),
    };
    let text = format!(
        "Mission: <{}>\nLocation: {}\n{}\n\n{}\n\n{}: <{}> ({}x score)\nRequired Deposit: ${}\nReward: ${} (${:.0} - ${:.0})",
        m.display_name,
        m.location_name,
        m.location_address,
        m.flavor_text,
        prefix,
        dname,
        dmult,
        m.required_deposit,
        m.mission_reward_base,
        m.mission_reward_base as f64 * 0.5,
        m.mission_reward_base as f64 * 5.0
    );
    let img = if m.preview_image_path.is_empty() {
        "img/placeholder_mission.png"
    } else {
        &m.preview_image_path
    };
    (text, img.to_string())
}

/// System to update the mission description and image when selection changes
pub(crate) fn update_mission_selection(
    mut ev_menu_selection: MessageReader<MenuItemSelected>,
    asset_server: Res<AssetServer>,
    mut q_desc_text: Query<&mut Text, With<MissionDescriptionText>>,
    mut q_preview_image: Query<&mut ImageNode, With<MissionPreviewImage>>,
    maps_resource: Res<Maps>,
    ui_mapping: Res<UIMissionMapping>,
    mission_select_mode: Res<CurrentMissionSelectMode>,
    difficulty_resource: Res<CurrentDifficulty>,
) {
    for ev in ev_menu_selection.read() {
        if ev.0 < ui_mapping.ui_to_map_index.len() {
            let map = &maps_resource.maps[ui_mapping.ui_to_map_index[ev.0]];
            let (text, img) = format_mission_details(
                &map.mission_data,
                mission_select_mode.0,
                &difficulty_resource,
            );

            if let Ok(mut t) = q_desc_text.single_mut() {
                t.0 = text;
            }
            if let Ok(mut i) = q_preview_image.single_mut() {
                i.image = asset_server.load(img);
            }
        } else if let Ok(mut t) = q_desc_text.single_mut() {
            t.0 = "Select a mission to view details.".to_string();
        }
    }
}

fn sort_maps(
    a: &(usize, &untmxmap_core::types::root::map::Map),
    b: &(usize, &untmxmap_core::types::root::map::Map),
) -> std::cmp::Ordering {
    a.1.mission_data
        .order
        .as_str()
        .cmp(b.1.mission_data.order.as_str())
        .then_with(|| {
            a.1.mission_data
                .display_name
                .cmp(&b.1.mission_data.display_name)
        })
        .then_with(|| a.1.path.cmp(&b.1.path))
}

fn get_most_advanced_affordable_mission_idx(
    sorted_available_maps: &[(usize, &untmxmap_core::types::root::map::Map)],
    player_profile: &unprofile_core::profile::PlayerProfileData,
) -> usize {
    sorted_available_maps
        .iter()
        .enumerate()
        .rev()
        .find(|(_, (_, map_info))| {
            let needed = map_info.mission_data.required_deposit
                - player_profile.progression.insurance_deposit;
            needed <= 0 || player_profile.progression.bank >= needed
        })
        .map(|(idx, _)| idx)
        .unwrap_or(0)
}

/// System to set up the unified mission selection UI
pub(crate) fn setup_ui(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    campaign_assets: Res<CampaignAssets>,
    asset_server: Res<AssetServer>,
    player_profile_resource: Res<Persistent<unprofile_core::profile::PlayerProfileData>>,
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
    let mut filtered_maps: Vec<(usize, &untmxmap_core::types::root::map::Map)> = maps_resource
        .maps
        .iter()
        .enumerate()
        .filter(|(_, map)| match mission_select_mode.0 {
            MissionSelectMode::Campaign => map.mission_data.is_campaign_mission,
            MissionSelectMode::Custom => !map.mission_data.is_campaign_mission,
        })
        .collect();

    filtered_maps.sort_by(sort_maps);

    let (available_maps, locked_maps): (Vec<_>, Vec<_>) = filtered_maps
        .into_iter()
        .partition(|(_, map)| player_level >= map.mission_data.min_player_level);

    ui_mapping.ui_to_map_index.clear();

    let root_entity = commands
        .spawn((
            Node {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                position_type: PositionType::Absolute,
                flex_direction: FlexDirection::Column,
                ..default()
            },
            MissionSelectUI,
        ))
        .id();

    if available_maps.is_empty() && locked_maps.is_empty() {
        warn!(
            "No missions available for mode {:?} at player level {}.",
            mission_select_mode.0, player_level
        );
        commands
            .entity(root_entity)
            .with_children(|p| {
                let mname = match mission_select_mode.0 {
                    MissionSelectMode::Campaign => "campaign",
                    MissionSelectMode::Custom => "custom",
                };
                p.spawn(Text::new(format!(
                    "No {} missions available at your level.",
                    mname
                )))
                .insert((
                    TextFont {
                        font: menu_assets.font_londrina_light.clone(),
                        font_size: 24.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(colors::MENU_ITEM_COLOR_OFF),
                ));
                templates::create_content_item(p, "Go Back", 0, true, &menu_assets).insert((
                    MenuItemInteractive {
                        identifier: 0,
                        selected: true,
                    },
                    Button,
                    Interaction::None,
                ));
            })
            .insert(MenuRoot { selected_item: 0 });
        return;
    }

    let mut default_sel = 0;
    let (initial_desc, initial_img) = if !available_maps.is_empty() {
        if mission_select_mode.0 == MissionSelectMode::Campaign {
            default_sel =
                get_most_advanced_affordable_mission_idx(&available_maps, &player_profile_resource);
        }
        format_mission_details(
            &available_maps[default_sel].1.mission_data,
            mission_select_mode.0,
            &difficulty_resource,
        )
    } else {
        (
            format!(
                "No available {} missions at your level.\n\nComplete missions to unlock more.",
                match mission_select_mode.0 {
                    MissionSelectMode::Campaign => "campaign",
                    MissionSelectMode::Custom => "custom",
                }
            ),
            "img/placeholder_mission.png".to_string(),
        )
    };

    let title = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Campaign",
        MissionSelectMode::Custom => "Custom Mission",
    };
    let subtitle = match mission_select_mode.0 {
        MissionSelectMode::Campaign => "Select Mission".to_string(),
        MissionSelectMode::Custom => {
            format!(
                "Select Map\n  ({})",
                difficulty_resource.0.difficulty_name()
            )
        }
    };

    commands.entity(root_entity).with_children(|p| {
        templates::create_background(p, &menu_assets);
        templates::create_logo(p, &menu_assets);
        templates::create_breadcrumb_navigation(p, &menu_assets, title, &subtitle);

        let mut content_area = templates::create_selectable_content_area(p, &menu_assets, default_sel);
        content_area.insert(MenuMouseTracker::default()).with_children(|c| {
            // Mission List Pane
            c.spawn(Node { width: Val::Percent(50.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Row, ..default() })
                .with_children(|lp| {
                    lp.spawn((
                        Node {
                            width: Val::Percent(90.0),
                            height: Val::Percent(100.0),
                            flex_direction: FlexDirection::Column,
                            justify_content: JustifyContent::Start,
                            overflow: Overflow::scroll_y(),
                            ..default()
                        },
                        ScrollableListContainer,
                        ScrollPosition::default(),
                    )).with_children(|list| {
                        let mut curr_ui_idx = 0;
                        for (idx, (orig_idx, map)) in available_maps.iter().enumerate() {
                            ui_mapping.ui_to_map_index.push(*orig_idx);
                            create_mission_list_item(list, &menu_assets, &campaign_assets, map, &player_profile_resource, curr_ui_idx, idx == default_sel, &mission_select_mode, &difficulty_resource);
                            curr_ui_idx += 1;
                        }
                        if !available_maps.is_empty() && !locked_maps.is_empty() {
                            list.spawn(Node { min_height: Val::Px(16.0), ..default() }).insert(Pickable::default());
                        }
                        if let Some((_, map)) = locked_maps.first() {
                            list.spawn(Node { width: Val::Percent(100.0), padding: UiRect::axes(Val::Px(8.0 * FONT_SCALE), Val::Px(6.0 * FONT_SCALE)), ..default() })
                                .insert(Pickable::default())
                                .with_children(|li| {
                                    li.spawn(Node { width: Val::Percent(100.0), flex_direction: FlexDirection::Row, justify_content: JustifyContent::SpaceBetween, ..default() })
                                        .with_children(|row| {
                                            row.spawn((Text::new(format!("Unlock Level {} for more", map.mission_data.min_player_level)), TextFont { font: menu_assets.font_titillium_regular.clone(), font_size: 24.0 * FONT_SCALE, ..default() }, TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5)), unmenu_core::components::PrincipalMenuText));
                                            row.spawn((Text::new("🔒"), TextFont { font: menu_assets.font_titillium_regular.clone(), font_size: 24.0 * FONT_SCALE, ..default() }, TextColor(Color::srgba(0.5, 0.5, 0.5, 0.5))));
                                        });
                                });
                        }
                        if !locked_maps.is_empty() {
                            list.spawn(Node { min_height: Val::Px(16.0), ..default() }).insert(Pickable::default());
                        }
                        templates::create_content_item(list, "Go Back", curr_ui_idx, false, &menu_assets).insert(MenuItemInteractive { identifier: curr_ui_idx, selected: false });
                        list.spawn(Node { width: Val::Percent(100.0), min_height: Val::Px(64.0), ..default() }).insert(Pickable::default());
                    });
                    scrollbar::build_scrollbar_ui(lp, &menu_assets);
                });

            // Mission Detail Pane
            c.spawn(Node { width: Val::Percent(50.0), height: Val::Percent(100.0), flex_direction: FlexDirection::Column, padding: UiRect::left(Val::Px(15.0)), justify_content: JustifyContent::Center, ..default() })
                .with_children(|dp| {
                    dp.spawn((Node { width: Val::Percent(80.0), aspect_ratio: Some(16.0/9.0), margin: UiRect::bottom(Val::Px(10.0)), border: UiRect::all(Val::Px(1.0)), ..default() }, ImageNode { image: asset_server.load(initial_img), ..default() }, BorderColor::all(Color::srgba(0.290, 0.596, 0.706, 0.2)), MissionPreviewImage));
                    dp.spawn((Node { width: Val::Percent(100.0), padding: UiRect::all(Val::Px(10.0)), ..default() }, BackgroundColor(foundation_colors::PANEL_BGCOLOR.with_alpha(0.95))))
                        .with_children(|tc| {
                            tc.spawn((Text::new(initial_desc), TextFont { font: menu_assets.font_titillium_light.clone(), font_size: 19.0 * FONT_SCALE, ..default() }, TextColor(colors::MENU_DESC_TEXT_COLOR), MissionDescriptionText));
                        });
                });
        });

        let help = match mission_select_mode.0 {
            MissionSelectMode::Campaign => "Select a mission    |    [Up]/[Down]: Change    |    [Enter]: Start Mission    |    [ESC]: Go Back",
            MissionSelectMode::Custom => "Select a map    |    [Up]/[Down]: Change    |    [Enter]: Start Mission    |    [ESC]: Back to Difficulty Selection",
        };
        templates::create_help_text(p, &menu_assets, Some(help.to_string()));
        templates::create_player_status_bar(p, &menu_assets, &player_profile_resource);
    });

    initial_scroll_target.0 = (!available_maps.is_empty()).then_some(default_sel);
}

/// Helper function to create a mission list item in the UI
fn create_mission_list_item(
    mission_list: &mut ChildSpawnerCommands,
    menu_assets: &MenuAssets,
    campaign_assets: &CampaignAssets,
    map: &untmxmap_core::types::root::map::Map,
    player_profile: &unprofile_core::profile::PlayerProfileData,
    ui_index: usize,
    is_selected: bool,
    mode: &CurrentMissionSelectMode,
    dif: &CurrentDifficulty,
) -> Entity {
    let m = &map.mission_data;
    mission_list
        .spawn((
            Node {
                width: Val::Percent(100.0),
                padding: UiRect::axes(Val::Px(8.0 * FONT_SCALE), Val::Px(6.0 * FONT_SCALE)),
                margin: UiRect::vertical(Val::Px(2.0 * FONT_SCALE)),
                ..default()
            },
            MenuItemInteractive {
                identifier: ui_index,
                selected: is_selected,
            },
            Button,
            Interaction::None,
            BackgroundColor(if is_selected {
                Color::srgba(0.3, 0.3, 0.3, 0.1)
            } else {
                Color::NONE
            }),
            Pickable::default(),
        ))
        .with_children(|p| {
            p.spawn((
                Node {
                    width: Val::Percent(100.0),
                    flex_direction: FlexDirection::Row,
                    justify_content: JustifyContent::SpaceBetween,
                    align_items: AlignItems::Center,
                    ..default()
                },
                Pickable::default(),
            ))
            .with_children(|row| {
                row.spawn((
                    Text::new(m.display_name.clone()),
                    TextFont {
                        font: menu_assets.font_titillium_regular.clone(),
                        font_size: 24.0 * FONT_SCALE,
                        ..default()
                    },
                    TextColor(if !is_selected {
                        colors::MENU_ITEM_COLOR_OFF
                    } else {
                        colors::MENU_ITEM_COLOR_ON
                    }),
                    unmenu_core::components::PrincipalMenuText,
                    Pickable::default(),
                ));
                let tdif = match mode.0 {
                    MissionSelectMode::Campaign => m.difficulty,
                    MissionSelectMode::Custom => dif.0,
                };
                BadgeUtils::create_badge(
                    row,
                    campaign_assets,
                    player_profile.get_map_grade(&map.path, &tdif),
                    32.0,
                    false,
                );
            });
        })
        .id()
}

fn trigger_initial_scroll_if_needed(
    mut initial_scroll_target: ResMut<InitialScrollTarget>,
    mut ev_keyboard_nav: MessageWriter<KeyboardNavigate>,
    container_query: Query<&ComputedNode, With<ScrollableListContainer>>,
) {
    if let Some(target_idx) = initial_scroll_target.0
        && let Ok(container_node) = container_query.single()
        && container_node.size().y > 0.0
    {
        ev_keyboard_nav.write(KeyboardNavigate(target_idx));
        initial_scroll_target.0 = None;
    }
}
