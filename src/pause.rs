use crate::{
    materials::{self, UIPanelMaterial},
    root,
};
use bevy::prelude::*;

#[derive(Debug, Component)]
pub struct PauseUI;

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::GameState::Pause), setup_ui)
        .add_systems(OnExit(root::GameState::Pause), cleanup)
        .add_systems(Update, keyboard);
}

const PAUSEUI_BGCOLOR: Color = Color::rgba(0.082, 0.094, 0.118, 0.6);
const PAUSEUI_PANEL_BGCOLOR: Color = Color::rgba(0.106, 0.129, 0.157, 0.8);
const PAUSEUI_ACCENT_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 1.0);
const PAUSEUI_TEXT_COLOR: Color = Color::rgba(0.7, 0.82, 0.85, 1.0);

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    mut next_state: ResMut<NextState<root::State>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Pause {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
    if keyboard_input.just_pressed(KeyCode::KeyQ) {
        game_next_state.set(root::GameState::None);
        next_state.set(root::State::MainMenu);
    }
}

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<PauseUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn setup_ui(
    mut commands: Commands,
    mut materials: ResMut<Assets<materials::UIPanelMaterial>>,
    handles: Res<root::GameAssets>,
) {
    const MARGIN_PERCENT: f32 = 0.5;
    const MARGIN: UiRect = UiRect::percent(
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
        MARGIN_PERCENT,
    );
    commands
        .spawn(NodeBundle {
            background_color: PAUSEUI_BGCOLOR.into(),

            style: Style {
                position_type: PositionType::Absolute,
                min_width: Val::Percent(50.0),
                min_height: Val::Percent(30.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Row,
                column_gap: Val::Percent(MARGIN_PERCENT),
                padding: MARGIN,
                margin: MARGIN,
                ..default()
            },
            ..default()
        })
        .insert(PauseUI)
        .with_children(|parent| {
            // Mid content
            parent
                .spawn(MaterialNodeBundle {
                    material: materials.add(UIPanelMaterial {
                        color: PAUSEUI_PANEL_BGCOLOR,
                    }),

                    style: Style {
                        border: UiRect::all(Val::Px(1.0)),
                        padding: UiRect::all(Val::Px(1.0)),
                        min_width: Val::Px(10.0),
                        min_height: Val::Px(10.0),
                        justify_content: JustifyContent::FlexStart,
                        flex_direction: FlexDirection::Column,
                        row_gap: Val::Percent(MARGIN_PERCENT),
                        flex_grow: 1.0,
                        ..default()
                    },
                    ..default()
                })
                .with_children(|mid_blk| {
                    let title = TextBundle::from_section(
                        "Pause",
                        TextStyle {
                            font: handles.fonts.londrina.w300_light.clone(),
                            font_size: 35.0,
                            color: PAUSEUI_ACCENT_COLOR,
                        },
                    )
                    .with_style(Style {
                        height: Val::Px(40.0),
                        ..default()
                    });

                    mid_blk.spawn(title);
                    // Journal contents
                    mid_blk.spawn(NodeBundle {
                        border_color: PAUSEUI_ACCENT_COLOR.into(),
                        style: Style {
                            border: UiRect::top(Val::Px(1.50)),
                            height: Val::Px(0.0),
                            ..default()
                        },
                        ..default()
                    });

                    mid_blk.spawn(
                        TextBundle::from_section(
                            "The game is paused. Hit [ESC] again to resume or [Q] to Quit.",
                            TextStyle {
                                font: handles.fonts.chakra.w300_light.clone(),
                                font_size: 25.0,
                                color: PAUSEUI_TEXT_COLOR,
                            },
                        )
                        .with_style(Style {
                            margin: UiRect::all(Val::Px(4.0)),
                            ..default()
                        }),
                    );

                    // ----
                    mid_blk.spawn(NodeBundle {
                        style: Style {
                            justify_content: JustifyContent::FlexStart,
                            flex_direction: FlexDirection::Column,
                            row_gap: Val::Percent(MARGIN_PERCENT),
                            flex_grow: 1.0,
                            ..default()
                        },
                        ..default()
                    });
                });
        });

    // ---
}
