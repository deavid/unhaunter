use bevy::prelude::*;
use uncommon_app_core::platform::plt::{FONT_SCALE, UI_SCALE};
use uncommon_app_core::states::AppState;
use uninput_core::states::InGameUiState;
use unmission_core::events::QuitMissionEvent;
use unrender_std::custom_material2::UIPanelMaterial;
use unreplicon_core::resources::HostGone;

use crate::assets::PauseAssets;

#[derive(Debug, Component)]
struct PauseUI;

const PAUSEUI_BGCOLOR: Color = Color::srgba(0.082, 0.094, 0.118, 0.6);
const PAUSEUI_PANEL_BGCOLOR: Color = Color::srgba(0.106, 0.129, 0.157, 0.8);
const PAUSEUI_ACCENT_COLOR: Color = Color::srgba(0.290, 0.596, 0.706, 1.0);
const PAUSEUI_TEXT_COLOR: Color = Color::srgba(0.7, 0.82, 0.85, 1.0);

fn keyboard(
    game_state: Res<State<InGameUiState>>,
    mut game_next_state: ResMut<NextState<InGameUiState>>,
    mut ev_quit: MessageWriter<QuitMissionEvent>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    host_gone: Res<HostGone>,
) {
    if *game_state.get() != InGameUiState::Pause {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) && !host_gone.0 {
        game_next_state.set(InGameUiState::Running);
    }
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        game_next_state.set(InGameUiState::Running);
        ev_quit.write(QuitMissionEvent);
    }
}

fn cleanup(mut commands: Commands, qtui: Query<Entity, With<PauseUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn();
    }
}

fn keyboard_pause(
    app_state: Res<State<AppState>>,
    game_state: Res<State<InGameUiState>>,
    mut game_next_state: ResMut<NextState<InGameUiState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *app_state.get() != AppState::InGame {
        return;
    }

    let can_pause = *game_state.get() == InGameUiState::Running;
    if *game_state.get() == InGameUiState::Pause {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) && can_pause {
        game_next_state.set(InGameUiState::Pause);
    }
}

fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<UIPanelMaterial>>,
    ui_assets: Res<PauseAssets>,
    host_gone: Res<HostGone>,
) {
    let (p_text, p_sub_text) = if host_gone.0 {
        (
            "Mission Unavailable",
            "Host has left the mission or disconnected. Press Q to exit.",
        )
    } else {
        ("Pause", "Press ESC to resume or Q to quit mission")
    };
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    commands
        .spawn(Node {
            position_type: PositionType::Absolute,
            min_width: Val::Percent(50.0),
            min_height: Val::Percent(30.0),
            justify_content: JustifyContent::FlexStart,
            flex_direction: FlexDirection::Row,
            column_gap: Val::Percent(MARGIN_PERCENT),
            padding: MARGIN,
            margin: MARGIN,
            ..default()
        })
        .insert(BackgroundColor(PAUSEUI_BGCOLOR))
        .insert(PauseUI)
        .with_children(|parent| {
            // Mid content
            parent
                .spawn(MaterialNode(materials.add(UIPanelMaterial {
                    color: PAUSEUI_PANEL_BGCOLOR.into(),
                })))
                .insert(Node {
                    padding: UiRect::all(Val::Px(1.0)),
                    min_width: Val::Px(10.0),
                    min_height: Val::Px(10.0),
                    justify_content: JustifyContent::FlexStart,
                    flex_direction: FlexDirection::Column,
                    row_gap: Val::Percent(MARGIN_PERCENT),
                    flex_grow: 1.0,
                    ..default()
                })
                .with_children(|mid_blk| {
                    mid_blk
                        .spawn(Text::new(p_text))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 35.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(PAUSEUI_ACCENT_COLOR))
                        .insert(Node {
                            height: Val::Px(40.0 * UI_SCALE),
                            ..default()
                        });
                    mid_blk.spawn(Node {
                        border: UiRect::top(Val::Px(1.50)),
                        height: Val::Px(0.0),
                        ..default()
                    });
                    mid_blk
                        .spawn(Text::new(p_sub_text))
                        .insert(TextFont {
                            font: ui_assets.font_londrina_light.clone(),
                            font_size: 20.0 * FONT_SCALE,
                            ..default()
                        })
                        .insert(TextColor(PAUSEUI_TEXT_COLOR));
                });
        });
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(InGameUiState::Pause), setup_ui);
    app.add_systems(OnExit(InGameUiState::Pause), cleanup);
    app.add_systems(Update, keyboard.run_if(in_state(InGameUiState::Pause)));
    app.add_systems(Update, keyboard_pause.run_if(in_state(AppState::InGame)));
}
