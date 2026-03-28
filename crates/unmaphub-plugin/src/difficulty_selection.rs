use bevy::prelude::*;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_app_core::states::AppState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty::Difficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unmaphub_core::states::MapHubState;
use unmenu_core::assets::MenuAssets;
use unmenu_core::colors;
use unmenu_core::mission_select::{CurrentMissionSelectMode, MissionSelectMode};
use unmenu_core::{
    components::*,
    events::{MenuEscapeEvent, MenuItemClicked, MenuItemSelected},
    templates,
};

/// UI component marker for the difficulty selection screen
#[derive(Component, Debug)]
pub(crate) struct DifficultySelectionUI;

/// UI component marker for the difficulty description text
#[derive(Component, Debug)]
pub(crate) struct DifficultyDescriptionUI;

/// Component that associates a menu item with a specific difficulty level
#[derive(Component, Debug, Clone, Copy)]
pub(crate) struct DifficultySelectionItem {
    pub difficulty: Difficulty,
}

/// Registers all systems needed for the difficulty selection screen
pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(MapHubState::DifficultySelection), setup_systems)
        .add_systems(OnExit(MapHubState::DifficultySelection), cleanup_systems)
        .add_systems(
            Update,
            (
                handle_difficulty_click,
                update_difficulty_description,
                handle_difficulty_escape,
            )
                .run_if(in_state(MapHubState::DifficultySelection)),
        );
}

/// Sets up the difficulty selection screen UI
pub(crate) fn setup_systems(
    mut commands: Commands,
    menu_assets: Res<MenuAssets>,
    mut ev_menu_clicks: MessageReader<MenuItemClicked>,
    mut ev_menu_selection: MessageReader<MenuItemSelected>,
) {
    // Clear event readers to prevent phantom inputs on state entry
    ev_menu_clicks.clear();
    ev_menu_selection.clear();

    // Filter for non-tutorial difficulties to display
    let available_difficulties: Vec<Difficulty> = Difficulty::all()
        .filter(|d| !d.is_tutorial_difficulty())
        .collect();

    setup_ui(&mut commands, &menu_assets, &available_difficulties);
}

/// Cleans up the difficulty selection screen UI
pub(crate) fn cleanup_systems(
    mut commands: Commands,
    qtui: Query<Entity, With<DifficultySelectionUI>>,
) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }
}

/// Handles clicks on difficulty options and the "Go Back" button
pub(crate) fn handle_difficulty_click(
    mut ev_menu_clicks: MessageReader<MenuItemClicked>,
    mut next_hub_state: ResMut<NextState<MapHubState>>,
    mut difficulty_resource: ResMut<CurrentDifficulty>,
    mut next_app_state: ResMut<NextState<AppState>>,
    q_items: Query<(&DifficultySelectionItem, &MenuItemInteractive)>,
    mut mission_select_mode: ResMut<CurrentMissionSelectMode>,
) {
    // Get the list of non-tutorial difficulties actually displayed in the UI
    let displayed_difficulties: Vec<Difficulty> = Difficulty::all()
        .filter(|d| !d.is_tutorial_difficulty())
        .collect();

    for ev in ev_menu_clicks.read() {
        let total_displayed_difficulties = displayed_difficulties.len();
        if ev.state != AppState::MapHub {
            warn!("MenuItemClicked event received in state: {:?}", ev.state);
            continue;
        }
        if let Some((item_data, _)) = q_items
            .iter()
            .find(|(_, interactive)| interactive.identifier == ev.pos)
        {
            // Ensure the clicked item is a non-tutorial difficulty
            if !item_data.difficulty.is_tutorial_difficulty() {
                // Set the difficulty based on selection
                difficulty_resource.0 = item_data.difficulty;

                // Set the mission select mode to Custom
                mission_select_mode.0 = MissionSelectMode::Custom;

                // Instead of loading directly, transition to the unified mission selection screen
                next_app_state.set(AppState::MissionSelect);
                next_hub_state.set(MapHubState::None);

                info!(
                    "Selected difficulty: {:?}, transitioning to MissionSelect",
                    item_data.difficulty
                );
            } else {
                warn!(
                    "Clicked on a tutorial difficulty in Custom Mission mode. This should not happen."
                );
            }
            break;
        } else if ev.pos == total_displayed_difficulties {
            // This is the "Go Back" item
            next_app_state.set(AppState::MainMenu);
            next_hub_state.set(MapHubState::None);
            break;
        }
    }
}

/// Updates the description text when a different difficulty is selected
pub(crate) fn update_difficulty_description(
    mut ev_menu_selection: MessageReader<MenuItemSelected>,
    mut q_desc_text: Query<(&mut Text, &mut TextColor), With<DifficultyDescriptionUI>>,
    q_items: Query<(&DifficultySelectionItem, &MenuItemInteractive)>,
) {
    // Get the list of non-tutorial difficulties actually displayed in the UI
    let displayed_difficulties: Vec<Difficulty> = Difficulty::all()
        .filter(|d| !d.is_tutorial_difficulty())
        .collect();

    for ev in ev_menu_selection.read() {
        let total_displayed_difficulties = displayed_difficulties.len();

        if let Ok((mut text, mut text_color)) = q_desc_text.single_mut() {
            if ev.0 == total_displayed_difficulties {
                text.0 = "Select a challenge level for your custom mission.".to_string();
                text_color.0 = colors::MENU_ITEM_COLOR_OFF;
                continue;
            }

            if let Some((item_data, _)) = q_items
                .iter()
                .find(|(_, interactive)| interactive.identifier == ev.0)
            {
                // Ensure the selected item is a non-tutorial difficulty
                if !item_data.difficulty.is_tutorial_difficulty() {
                    let selected_difficulty = item_data.difficulty;

                    let new_text = format!(
                        "Challenge: <{}>:\n{}\n\nScore Bonus: {:.2}x",
                        selected_difficulty.difficulty_name(),
                        selected_difficulty.difficulty_description(),
                        selected_difficulty.difficulty_score_multiplier()
                    );

                    text.0 = new_text;
                    text_color.0 = colors::MENU_ITEM_COLOR_OFF;
                }
            }
        }
    }
}

/// Handles ESC key press to return to main menu
pub(crate) fn handle_difficulty_escape(
    mut ev_escape: MessageReader<MenuEscapeEvent>,
    mut next_hub_state: ResMut<NextState<MapHubState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if ev_escape.read().next().is_some() {
        // Go back to main menu since map selection no longer exists
        next_app_state.set(AppState::MainMenu);
        next_hub_state.set(MapHubState::None);
    }
}

/// Creates the UI for the difficulty selection screen using templates
/// This now takes a Vec<Difficulty> containing only non-tutorial difficulties.
pub(crate) fn setup_ui(
    commands: &mut Commands,
    menu_assets: &MenuAssets,
    available_difficulties: &[Difficulty],
) {
    // Use the first available non-tutorial difficulty for initial description
    let initial_difficulty = available_difficulties.first().copied().unwrap_or_else(|| {
        warn!("No non-tutorial difficulties passed to setup_ui. Defaulting description.");
        Difficulty::StandardChallenge // Fallback
    });

    let initial_dif_struct = initial_difficulty;
    let initial_desc = format!(
        "Challenge: <{}>:\n{}\n\nScore Bonus: {:.2}x",
        initial_dif_struct.difficulty_name(),
        initial_dif_struct.difficulty_description(),
        initial_dif_struct.difficulty_score_multiplier()
    );

    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            position_type: PositionType::Absolute,
            ..default()
        })
        .insert(DifficultySelectionUI)
        .with_children(|parent| {
            templates::create_background(parent, menu_assets);
            templates::create_logo(parent, menu_assets);
            templates::create_breadcrumb_navigation(
                parent,
                menu_assets,
                "Custom Mission", // Changed from "New Game"
                "Select Difficulty",
            );

            let mut content_area =
                templates::create_selectable_content_area(parent, menu_assets, 0);
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
                        padding: UiRect::right(Val::Px(10.0 * UI_SCALE)),
                        ..default()
                    })
                    .with_children(|list_column| {
                        // Iterate only over the filtered, non-tutorial difficulties
                        for (idx, difficulty) in available_difficulties.iter().enumerate() {
                            templates::create_content_item(
                                list_column,
                                difficulty.difficulty_name(),
                                idx,
                                idx == 0,
                                menu_assets,
                            )
                            .insert(DifficultySelectionItem {
                                difficulty: *difficulty,
                            })
                            .insert(MenuItemInteractive {
                                // Ensure MenuItemInteractive uses the correct index
                                identifier: idx,
                                selected: idx == 0,
                            });
                        }

                        templates::create_content_item(
                            list_column,
                            "Go Back",
                            available_difficulties.len(), // Index for "Go Back" is after all difficulties
                            false,
                            menu_assets,
                        )
                        .insert(MenuItemInteractive {
                            // Ensure MenuItemInteractive uses the correct index
                            identifier: available_difficulties.len(),
                            selected: false,
                        });
                    });

                content
                    .spawn(Node {
                        width: Val::Percent(50.0),
                        height: Val::Percent(100.0),
                        flex_direction: FlexDirection::Column,
                        padding: UiRect::left(Val::Px(10.0 * UI_SCALE)),
                        ..default()
                    })
                    .with_children(|desc_column| {
                        desc_column.spawn((
                            Text::new(initial_desc),
                            TextFont {
                                font: menu_assets.font_titillium_light.clone(),
                                font_size: 19.0 * FONT_SCALE,
                                ..default()
                            },
                            TextColor(colors::MENU_ITEM_COLOR_OFF),
                            Node {
                                margin: UiRect::top(Val::Px(15.0 * UI_SCALE)),
                                ..default()
                            },
                            DifficultyDescriptionUI,
                        ));
                    });
            });

            templates::create_help_text(
                parent,
                menu_assets,
                Some(
                    "[Up]/[Down]: Change Difficulty    |    [Enter]: Select    |    [ESC]: Go Back"
                        .to_string(),
                ),
            );
        });
}
