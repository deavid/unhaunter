use bevy::app::App;
use bevy::prelude::*;

use crate::root;

#[derive(Component, Debug)]
pub struct TruckUI;

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(root::GameState::Truck), setup_ui)
        .add_systems(OnExit(root::GameState::Truck), cleanup)
        .add_systems(Update, keyboard);
}

pub fn setup_ui(mut commands: Commands, _handles: Res<root::GameAssets>) {
    // Load Truck UI
    const TRUCKUI_BGCOLOR: Color = Color::rgba(0.082, 0.094, 0.118, 0.9);
    // const TRUCKUI_PANEL_BGCOLOR: Color = Color::rgba(0.106, 0.129, 0.157, 0.9);
    // const TRUCKUI_ACCENT_COLOR: Color = Color::rgba(0.290, 0.596, 0.706, 1.0);
    commands
        .spawn(NodeBundle {
            background_color: TRUCKUI_BGCOLOR.into(),

            style: Style {
                width: Val::Percent(98.0),
                height: Val::Percent(96.0),
                justify_content: JustifyContent::FlexStart,
                flex_direction: FlexDirection::Column,
                // border: UiRect::all(Val::Px(1.0)),
                padding: UiRect::all(Val::Percent(1.0)),
                margin: UiRect::percent(1.0, 1.0, 1.0, 1.0),
                ..default()
            },
            ..default()
        })
        .insert(TruckUI);

    // ---
}

pub fn cleanup(mut commands: Commands, qtui: Query<Entity, With<TruckUI>>) {
    for e in qtui.iter() {
        commands.entity(e).despawn_recursive();
    }
}

pub fn keyboard(
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    if *game_state.get() != root::GameState::Truck {
        return;
    }
    if keyboard_input.just_pressed(KeyCode::Escape) {
        game_next_state.set(root::GameState::None);
    }
}
