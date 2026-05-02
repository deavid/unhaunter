use bevy::prelude::*;
use bevy_replicon::prelude::{AppRuleExt, Channel, ServerMessageAppExt};
use uncommon_states_core::UIContextState;
use uninvestigation_core::components::ghost_guess::GhostGuess;
use unmission_core::types::SimulationState;
use unreplicon_core::components::{
    MissionGoalEntity, RepliconGhostSpawningActive, ServerGamePhase,
};
use unreplicon_core::messages::{GhostSoundFieldBroadcast, SpawnParticleNetEvent};
use unreplicon_core::repellent_tracker::RepellentCraftTracker;
use unreplicon_core::resources::{AuthorityRole, is_pure_client};

pub(super) fn app_setup(app: &mut App) {
    // Register Phase 2 replicated components.
    // GhostGuess remains the shared replicated mission whiteboard.
    app.replicate::<MissionGoalEntity>();
    app.replicate::<RepellentCraftTracker>();

    // Register server → client messages.
    app.add_server_message::<SpawnParticleNetEvent>(Channel::Ordered);
    app.add_server_message::<GhostSoundFieldBroadcast>(Channel::Ordered);

    app.add_systems(
        OnEnter(SimulationState::Spawning),
        setup_goal_entity.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        cleanup_ghost_entities.run_if(resource_exists::<AuthorityRole>),
    );

    // Server: mission lifecycle
    app.add_systems(
        Update,
        server_teardown_grace_period
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::TearingDown)),
    );

    // Client: if the server returns to Lobby (e.g. after an abort) while we are
    // still InGame, transition back to AppState::Lobby immediately.
    app.add_systems(
        Update,
        on_server_phase_lobby
            .run_if(is_pure_client)
            .run_if(in_state(UIContextState::InGame)),
    );

    // Client: mirror ServerGamePhase into SimulationState so that systems gated
    // on SimulationState::Ready stop ticking when the mission is ending.
    app.add_systems(
        Update,
        sync_sim_state_from_server_phase
            .run_if(is_pure_client)
            .run_if(in_state(UIContextState::InGame)),
    );
}

fn setup_goal_entity(mut commands: Commands) {
    // Singleton entity for shared mission whiteboard replication.
    commands.spawn((
        bevy_replicon::prelude::Replicated,
        MissionGoalEntity,
        GhostGuess::default(),
    ));
    info!("setup_goal_entity: MissionGoalEntity spawned");
}

fn cleanup_ghost_entities(q_goal: Query<Entity, With<MissionGoalEntity>>, mut commands: Commands) {
    commands.remove_resource::<RepliconGhostSpawningActive>();
    for entity in q_goal.iter() {
        commands.entity(entity).despawn();
        info!("MissionGoalEntity despawned");
    }
}

fn server_teardown_grace_period(
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut q_server_phase: Query<&mut ServerGamePhase, With<unreplicon_core::components::LobbyInfo>>,
    q_gamesprites: Query<
        Entity,
        (
            With<unboard_core::entity::GameSprite>,
            Without<bevy_replicon::prelude::Remote>,
        ),
    >,
    ui_state: Res<State<UIContextState>>,
) {
    if timer.is_none() {
        let gs_count = q_gamesprites.iter().count();
        warn!(
            "TEARDOWN_GRACE_START: ui_state={:?} local_gamesprites={} — server beginning 5s grace period before returning to Lobby",
            ui_state.get(),
            gs_count
        );
        *timer = Some(Timer::from_seconds(5.0, TimerMode::Once));
    }

    let Some(grace_timer) = timer.as_mut() else {
        return;
    };
    grace_timer.tick(time.delta());
    if !grace_timer.is_finished() {
        return;
    }

    let gs_count = q_gamesprites.iter().count();
    warn!(
        "TEARDOWN_GRACE_END: ui_state={:?} local_gamesprites={} — server returning to Lobby. If gamesprites > 0, they will survive into mission 2.",
        ui_state.get(),
        gs_count
    );

    for mut phase in q_server_phase.iter_mut() {
        *phase = ServerGamePhase::Lobby;
    }
    next_sim_state.set(SimulationState::Unloaded);
    next_app_state.set(UIContextState::Lobby);
    *timer = None;
}

fn on_server_phase_lobby(
    q_phase: Query<
        &ServerGamePhase,
        (
            Changed<ServerGamePhase>,
            With<unreplicon_core::components::LobbyInfo>,
        ),
    >,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Lobby {
            info!("ServerGamePhase::Lobby observed while InGame — returning to lobby");
            next_sim_state.set(SimulationState::Unloaded);
            next_app_state.set(UIContextState::Lobby);
        }
    }
}

/// Client-only: mirrors the replicated `ServerGamePhase` into the local
/// `SimulationState` so that systems gated on `SimulationState::Ready` stop
/// ticking when the mission ends on the server.
fn sync_sim_state_from_server_phase(
    q_phase: Query<
        &ServerGamePhase,
        (
            Changed<ServerGamePhase>,
            With<unreplicon_core::components::LobbyInfo>,
        ),
    >,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    for phase in q_phase.iter() {
        match phase {
            ServerGamePhase::InProgress => {
                next_sim_state.set(SimulationState::Ready);
            }
            ServerGamePhase::Concluding | ServerGamePhase::Ended => {
                next_sim_state.set(SimulationState::TearingDown);
            }
            ServerGamePhase::Lobby => {
                next_sim_state.set(SimulationState::Unloaded);
            }
        }
    }
}
