use bevy::prelude::*;
use unassets_core::resources::maps::Maps;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unengine_core::{GCameraArena, MCamera, MenuUI};
use unrender_std::components::game::{GameSound, GameSprite};
use untypes_core::cli::CliOptions;
use untypes_core::roles::{AuthorityRole, LobbyPresenceRole, LocalPlayerRole};
use untypes_core::states::{AppState, BootState, GameState, SimulationState};

pub fn setup_menu_camera(mut commands: Commands) {
    commands.spawn(Camera2d).insert(MCamera);
}

pub fn cleanup_menu(
    mut commands: Commands,
    qc: Query<Entity, With<MCamera>>,
    qm: Query<Entity, With<MenuUI>>,
) {
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }
    for ui_entity in qm.iter() {
        commands.entity(ui_entity).despawn();
    }
}

pub fn cleanup_game(
    mut commands: Commands,
    qc: Query<Entity, With<GCameraArena>>,
    qgs: Query<Entity, With<GameSprite>>,
    qs: Query<Entity, With<GameSound>>,
    mut bf: ResMut<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut bef: ResMut<BoardEntityField>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    bf.reset();
    bcf.reset();
    bef.reset();
    next_sim_state.set(SimulationState::Unloaded);

    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // Despawn game sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn();
    }

    // Despawn game sound
    for s in qs.iter() {
        commands.entity(s).despawn();
    }
}

pub fn simulation_state_transitions(
    app_state: Res<State<AppState>>,
    sim_state: Res<State<SimulationState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    if *app_state.get() == AppState::InGame && *sim_state.get() == SimulationState::Spawning {
        next_sim_state.set(SimulationState::Ready);
    }
}

pub fn keyboard_state_transitions(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
) {
    if *app_state.get() != AppState::InGame {
        return;
    }

    let can_pause = *game_state.get() == GameState::Running;
    if *game_state.get() == GameState::Pause {
        return;
    }

    if keyboard_input.just_pressed(KeyCode::Escape) && can_pause {
        game_next_state.set(GameState::Pause);
    }
}

pub(crate) fn set_boot_ready_when_maps_loaded(
    maps: Option<Res<Maps>>,
    boot_state: Res<State<BootState>>,
    mut next_boot: ResMut<NextState<BootState>>,
) {
    if *boot_state == BootState::Ready {
        return;
    } // one-way gate
    if let Some(m) = maps.as_ref().filter(|m| !m.maps.is_empty()) {
        info!("BootState -> Ready ({} maps loaded)", m.maps.len());
        next_boot.set(BootState::Ready);
    }
}

pub(crate) fn insert_roles_at_startup(cli: Res<CliOptions>, mut commands: Commands) {
    if cli.dedicated {
        // Dedicated server
        commands.insert_resource(AuthorityRole);
        commands.insert_resource(LobbyPresenceRole);
        debug!("Roles inserted: AuthorityRole, LobbyPresenceRole");
    } else {
        match cli.net_mode {
            untypes_core::cli::CliNetMode::Offline => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole");
            }
            untypes_core::cli::CliNetMode::PeerHost { .. } => {
                commands.insert_resource(AuthorityRole);
                commands.insert_resource(LocalPlayerRole);
                commands.insert_resource(LobbyPresenceRole);
                debug!("Roles inserted: AuthorityRole, LocalPlayerRole, LobbyPresenceRole");
            }
            untypes_core::cli::CliNetMode::Join { .. } => {
                commands.insert_resource(LocalPlayerRole);
                commands.insert_resource(LobbyPresenceRole);
                debug!("Roles inserted: LocalPlayerRole, LobbyPresenceRole");
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::MainMenu), setup_menu_camera);
    app.add_systems(OnExit(AppState::MainMenu), cleanup_menu);
    app.add_systems(OnExit(AppState::InGame), cleanup_game);
    app.add_systems(
        Update,
        (keyboard_state_transitions, simulation_state_transitions),
    );
}
