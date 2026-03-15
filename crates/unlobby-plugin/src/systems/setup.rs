use crate::systems::{difficulty_select, lobby_main, map_select};
use bevy::prelude::*;
use unengine_core::MenuUI;
use untypes_core::states::{AppState, LobbyScreen};

#[derive(Component)]
struct LobbyCamera;

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<map_select::MapSelectMapping>()
        .init_resource::<difficulty_select::DifficultyMapping>()
        .init_resource::<map_select::StateEntryTimer>()
        .init_resource::<difficulty_select::StateEntryTimer>()
        .init_resource::<lobby_main::StateEntryTimer>()
        .add_systems(OnEnter(AppState::Lobby), enter_lobby)
        .add_systems(OnExit(AppState::Lobby), exit_lobby)
        // LobbyMain
        .add_systems(OnEnter(LobbyScreen::Main), lobby_main::setup_ui)
        .add_systems(OnExit(LobbyScreen::Main), lobby_main::cleanup_ui)
        .add_systems(
            Update,
            (
                lobby_main::handle_clicks,
                lobby_main::update_display,
                lobby_main::update_deployment_status_ui,
            )
                .run_if(in_state(AppState::Lobby).and(in_state(LobbyScreen::Main))),
        )
        // Map Selection
        .add_systems(OnEnter(LobbyScreen::MapSelection), map_select::setup_ui)
        .add_systems(OnExit(LobbyScreen::MapSelection), map_select::cleanup_ui)
        .add_systems(
            Update,
            (map_select::handle_input, map_select::update_preview)
                .run_if(in_state(AppState::Lobby).and(in_state(LobbyScreen::MapSelection))),
        )
        // Difficulty Selection
        .add_systems(
            OnEnter(LobbyScreen::DifficultySelection),
            difficulty_select::setup_ui,
        )
        .add_systems(
            OnExit(LobbyScreen::DifficultySelection),
            difficulty_select::cleanup_ui,
        )
        .add_systems(
            Update,
            (
                difficulty_select::handle_input,
                difficulty_select::update_description,
            )
                .run_if(in_state(AppState::Lobby).and(in_state(LobbyScreen::DifficultySelection))),
        );
}

fn enter_lobby(mut commands: Commands, mut next_screen: ResMut<NextState<LobbyScreen>>) {
    commands.spawn((Camera2d, LobbyCamera));
    next_screen.set(LobbyScreen::Main);
}

fn exit_lobby(
    mut commands: Commands,
    q_cam: Query<Entity, With<LobbyCamera>>,
    q_ui: Query<Entity, With<MenuUI>>,
    mut next_screen: ResMut<NextState<LobbyScreen>>,
) {
    for e in q_cam.iter() {
        commands.entity(e).despawn();
    }
    for e in q_ui.iter() {
        commands.entity(e).despawn();
    }
    next_screen.set(LobbyScreen::None);
}
