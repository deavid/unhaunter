use bevy::prelude::*;
use bevy::utils::Instant;
use uncore::colors;
use uncore::difficulty::{CurrentDifficulty, Difficulty};
use uncore::events::loadlevel::LoadLevelEvent;
use uncore::platform::plt::{FONT_SCALE, UI_SCALE};
use uncore::resources::difficulty_state::DifficultySelectionState;
use uncore::resources::maps::Maps;
use uncore::states::AppState;
use uncore::states::MapHubState;
use uncore::types::root::game_assets::GameAssets;
use uncoremenu::{
    components::*,
    systems::{MenuEscapeEvent, MenuItemClicked, MenuItemSelected},
    templates,
};

/// UI component marker for the difficulty selection screen
#[derive(Component, Debug)]
pub struct DifficultySelectionUI;

/// UI component marker for the difficulty description text
#[derive(Component, Debug)]
pub struct DifficultyDescriptionUI;

/// Component that associates a menu item with a specific difficulty level
#[derive(Component, Debug, Clone, Copy)]
pub struct DifficultySelectionItem {
    pub difficulty: Difficulty,
}

/// Registers all systems needed for the difficulty selection screen
pub fn app_setup(app: &mut App) {
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

/// Sets up the difficulty selection screen UI and initializes the difficulty state
pub fn setup_systems(mut commands: Commands, handles: Res<GameAssets>) {
    setup_ui(&mut commands, &handles);
    let default_difficulty = Difficulty::all().next().unwrap_or_default();
    commands.insert_resource(DifficultySelectionState {
        selected_difficulty: default_difficulty,
        selected_map_idx: 0,
        state_entered_at: Instant::now(),
    });
}

/// Cleans up the difficulty selection screen UI
/// Note: DifficultySelectionState is kept as map_selection needs selected_map_idx
pub fn cleanup_systems(mut commands: Commands, qtui: Query<Entity, With<DifficultySelectionUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

/// Handles clicks on difficulty options and the "Go Back" button
/// - On difficulty selection: Sets the global difficulty and loads the level or manual
/// - On "Go Back": Returns to map selection
pub fn handle_difficulty_click(
    mut ev_menu_clicks: EventReader<MenuItemClicked>,
    mut next_hub_state: ResMut<NextState<MapHubState>>,
    mut difficulty_resource: ResMut<CurrentDifficulty>,
    difficulty_selection_state: Res<DifficultySelectionState>,
    maps: Res<Maps>,
    mut ev_load_level: EventWriter<LoadLevelEvent>,
    mut next_app_state: ResMut<NextState<AppState>>,
    q_items: Query<(&DifficultySelectionItem, &MenuItemInteractive)>,
) {
    // Grace period to prevent accidental clicks
    if difficulty_selection_state
        .state_entered_at
        .elapsed()
        .as_secs_f32()
        < 0.1
    {
        ev_menu_clicks.clear();
        return;
    }

    for ev in ev_menu_clicks.read() {
        let total_difficulties = Difficulty::all().count();

        if let Some((item_data, _)) = q_items
            .iter()
            .find(|(_, interactive)| interactive.identifier == ev.0)
        {
            difficulty_resource.0 = item_data.difficulty.create_difficulty_struct();

            start_preplay_manual_or_load(
                &difficulty_resource.0,
                &mut next_app_state,
                &difficulty_selection_state,
                &maps,
                &mut ev_load_level,
            );

            next_hub_state.set(MapHubState::None);
            break;
        } else if ev.0 == total_difficulties {
            next_hub_state.set(MapHubState::MapSelection);
            break;
        }
    }
}

/// Updates the description text when a different difficulty is selected
/// Must preserve state between difficulty hovers to fix description not updating
pub fn update_difficulty_description(
    mut ev_menu_selection: EventReader<MenuItemSelected>,
    mut difficulty_selection_state: ResMut<DifficultySelectionState>,
    mut q_desc_text: Query<(&mut Text, &mut TextColor), With<DifficultyDescriptionUI>>,
    q_items: Query<(&DifficultySelectionItem, &MenuItemInteractive)>,
) {
    if difficulty_selection_state
        .state_entered_at
        .elapsed()
        .as_secs_f32()
        < 0.1
    {
        ev_menu_selection.clear();
        return;
    }

    for ev in ev_menu_selection.read() {
        let total_difficulties = Difficulty::all().count();

        if let Ok((mut text, mut text_color)) = q_desc_text.get_single_mut() {
            if ev.0 == total_difficulties {
                // "Go Back" item selected
                text.0 = "Select a difficulty...".to_string();
                text_color.0 = colors::MENU_ITEM_COLOR_OFF;
                continue;
            }

            // Find the difficulty associated with the selected menu item
            if let Some((item_data, _)) = q_items
                .iter()
                .find(|(_, interactive)| interactive.identifier == ev.0)
            {
                let selected_difficulty = item_data.difficulty;
                let dif_struct = selected_difficulty.create_difficulty_struct();

                let new_text = format!(
                    "Difficulty <{}>:\n{}\n\nScore Bonus: {:.2}x",
                    dif_struct.difficulty_name,
                    dif_struct.difficulty_description,
                    dif_struct.difficulty_score_multiplier
                );

                text.0 = new_text;
                text_color.0 = colors::MENU_ITEM_COLOR_OFF;
                difficulty_selection_state.selected_difficulty = selected_difficulty;
            }
        }
    }
}

/// Handles ESC key press to return to map selection
pub fn handle_difficulty_escape(
    mut ev_escape: EventReader<MenuEscapeEvent>,
    mut next_state: ResMut<NextState<MapHubState>>,
) {
    if ev_escape.read().next().is_some() {
        next_state.set(MapHubState::MapSelection);
    }
}

/// Helper function to either load the level directly or show the preplay manual
/// based on the selected difficulty's tutorial chapter
fn start_preplay_manual_or_load(
    difficulty_struct: &uncore::difficulty::DifficultyStruct,
    next_game_state: &mut ResMut<NextState<AppState>>,
    difficulty_selection_state: &Res<DifficultySelectionState>,
    maps: &Res<Maps>,
    ev_load_level: &mut EventWriter<LoadLevelEvent>,
) {
    if difficulty_struct.tutorial_chapter.is_none() {
        let map_filepath = maps.maps[difficulty_selection_state.selected_map_idx]
            .path
            .clone();
        ev_load_level.send(LoadLevelEvent { map_filepath });
        next_game_state.set(AppState::Loading);
    } else {
        next_game_state.set(AppState::PreplayManual);
    }
}

/// Creates the UI for the difficulty selection screen using templates
pub fn setup_ui(commands: &mut Commands, handles: &GameAssets) {
    let initial_difficulty = Difficulty::all().next().unwrap_or_default();
    let initial_dif_struct = initial_difficulty.create_difficulty_struct();
    let initial_desc = format!(
        "Difficulty <{}>:\n{}\n\nScore Bonus: {:.2}x",
        initial_dif_struct.difficulty_name,
        initial_dif_struct.difficulty_description,
        initial_dif_struct.difficulty_score_multiplier
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
            templates::create_background(parent, handles);
            templates::create_logo(parent, handles);
            templates::create_breadcrumb_navigation(
                parent,
                handles,
                "New Game",
                "Select Difficulty",
            );

            let mut content_area = templates::create_selectable_content_area(parent, handles, 0);

            content_area.with_children(|content| {
                // Left column: difficulty list
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
                        let difficulties: Vec<Difficulty> = Difficulty::all().collect();
                        for (idx, difficulty) in difficulties.iter().enumerate() {
                            templates::create_content_item(
                                list_column,
                                difficulty.difficulty_name(),
                                idx,
                                idx == 0,
                                handles,
                            )
                            .insert(DifficultySelectionItem {
                                difficulty: *difficulty,
                            })
                            .insert(MenuItemInteractive {
                                identifier: idx,
                                selected: idx == 0,
                            });
                        }

                        templates::create_content_item(
                            list_column,
                            "Go Back",
                            difficulties.len(),
                            false,
                            handles,
                        )
                        .insert(MenuItemInteractive {
                            identifier: difficulties.len(),
                            selected: false,
                        });
                    });

                // Right column: difficulty description
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
                                font: handles.fonts.titillium.w300_light.clone(),
                                font_size: 19.0 * FONT_SCALE,
                                font_smoothing: bevy::text::FontSmoothing::AntiAliased,
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
                handles,
                Some(
                    "[Up]/[Down]: Change Difficulty    |    [Enter]: Select    |    [ESC]: Go Back"
                        .to_string(),
                ),
            );
        });
}
