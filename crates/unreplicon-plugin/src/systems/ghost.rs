use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientMessageAppExt, FromClient, Replicated, ServerMessageAppExt,
};
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::events::{JournalEvidenceToggled, JournalGhostToggled};
use unghost_core::tags::GhostTag;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unmission_core::summary::SummaryData;
use unmission_core::types::SimulationState;
use unorchestrator_core::UIContextState;
use unreplicon_core::components::{
    MissionGoalEntity, RepliconGhostSpawningActive, ServerGamePhase,
};
use unreplicon_core::messages::{
    GhostSoundFieldBroadcast, RequestJournalEvidenceToggle, RequestJournalGhostToggle,
    SpawnParticleNetEvent,
};
use unreplicon_core::resources::{AuthorityRole, is_pure_client};
use unspatial_core::lerp_position::LerpPosition;
use unspatial_core::position::Position;

pub(super) fn app_setup(app: &mut App) {
    // Register Phase 2 replicated components.
    // Ghost domain types are now registered in unghost-plugin.
    // SummaryData is now registered in unmission-plugin.
    app.replicate::<MissionGoalEntity>();

    // Register server → client messages.
    app.add_server_message::<SpawnParticleNetEvent>(Channel::Ordered);
    app.add_server_message::<GhostSoundFieldBroadcast>(Channel::Ordered);

    // Register client → server messages.
    app.add_client_message::<RequestJournalEvidenceToggle>(Channel::Ordered);
    app.add_client_message::<RequestJournalGhostToggle>(Channel::Ordered);

    // Server: setup and teardown ghost replication entities.
    app.add_systems(
        OnEnter(SimulationState::Spawning),
        setup_ghost_entities.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        OnEnter(SimulationState::Spawning),
        setup_goal_entity.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        OnEnter(SimulationState::TearingDown),
        cleanup_ghost_entities.run_if(resource_exists::<AuthorityRole>),
    );

    // Resource Bridges (Singleton Entity -> Resource)
    app.add_systems(
        Update,
        (
            sync_ghost_guess_to_mission_goal,
            sync_summary_data_to_mission_goal,
        )
            .run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        Update,
        (
            sync_mission_goal_to_ghost_guess,
            sync_mission_goal_to_summary_data,
        )
            .run_if(is_pure_client),
    );

    // Server: journal request handlers.
    app.add_systems(
        Update,
        (handle_journal_evidence_toggle, handle_journal_ghost_toggle)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(SimulationState::Ready)),
    );

    // Server: mission lifecycle
    app.add_systems(
        Update,
        (sync_mission_result_phase, server_teardown_grace_period)
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
}

fn setup_ghost_entities(
    q_ghost: Query<(Entity, &Position), With<GhostTag>>,
    q_breach: Query<Entity, With<GhostBreach>>,
    mut commands: Commands,
) {
    commands.insert_resource(RepliconGhostSpawningActive);

    for (entity, pos) in q_ghost.iter() {
        commands
            .entity(entity)
            .insert((Replicated, LerpPosition::new(*pos)));
        info!(
            "setup_ghost_entities: ghost entity {:?} marked Replicated with LerpPosition",
            entity
        );
    }

    for entity in q_breach.iter() {
        commands.entity(entity).insert(Replicated);
        info!(
            "setup_ghost_entities: breach entity {:?} marked Replicated",
            entity
        );
    }
}

fn setup_goal_entity(mut commands: Commands) {
    // Singleton entity for journal + mission-result replication.
    commands.spawn((
        Replicated,
        MissionGoalEntity,
        GhostGuess::default(),
        SummaryData::default(),
    ));
    info!("setup_goal_entity: MissionGoalEntity spawned");
}

fn cleanup_ghost_entities(q_goal: Query<Entity, With<MissionGoalEntity>>, mut commands: Commands) {
    commands.remove_resource::<RepliconGhostSpawningActive>();
    for entity in q_goal.iter() {
        commands.entity(entity).despawn();
    }
}

/// Bridge: Sync GhostGuess resource to singleton entity (Server).
fn sync_ghost_guess_to_mission_goal(
    res: Res<GhostGuess>,
    mut q_goal: Query<&mut GhostGuess, With<MissionGoalEntity>>,
) {
    if !res.is_changed() {
        return;
    }
    for mut comp in q_goal.iter_mut() {
        *comp = res.clone();
    }
}

/// Bridge: Sync SummaryData resource to singleton entity (Server).
fn sync_summary_data_to_mission_goal(
    res: Res<SummaryData>,
    mut q_goal: Query<&mut SummaryData, With<MissionGoalEntity>>,
) {
    if !res.is_changed() {
        return;
    }
    for mut comp in q_goal.iter_mut() {
        *comp = res.clone();
    }
}

/// Bridge: Sync singleton entity to GhostGuess resource (Client).
fn sync_mission_goal_to_ghost_guess(
    q_goal: Query<&GhostGuess, (With<MissionGoalEntity>, Changed<GhostGuess>)>,
    mut res: ResMut<GhostGuess>,
) {
    for comp in q_goal.iter() {
        *res = comp.clone();
    }
}

/// Bridge: Sync singleton entity to SummaryData resource (Client).
fn sync_mission_goal_to_summary_data(
    q_goal: Query<&SummaryData, (With<MissionGoalEntity>, Changed<SummaryData>)>,
    mut res: ResMut<SummaryData>,
) {
    for comp in q_goal.iter() {
        *res = comp.clone();
    }
}

fn handle_journal_evidence_toggle(
    mut reader: MessageReader<FromClient<RequestJournalEvidenceToggle>>,
    mut writer: MessageWriter<JournalEvidenceToggled>,
) {
    for msg in reader.read() {
        writer.write(JournalEvidenceToggled {
            evidence: msg.message.evidence,
            discard: msg.message.discard,
            mark_as_found: msg.message.mark_as_found,
        });
    }
}

fn handle_journal_ghost_toggle(
    mut reader: MessageReader<FromClient<RequestJournalGhostToggle>>,
    mut writer: MessageWriter<JournalGhostToggled>,
) {
    for msg in reader.read() {
        writer.write(JournalGhostToggled {
            ghost_type: msg.message.ghost_type,
            discard: msg.message.discard,
        });
    }
}

fn sync_mission_result_phase(
    summary: Res<SummaryData>,
    mut q_server_phase: Query<&mut ServerGamePhase>,
) {
    if !summary.is_changed() {
        return;
    }

    for mut sas in q_server_phase.iter_mut() {
        *sas = ServerGamePhase::Ended;
    }
}

fn server_teardown_grace_period(
    mut timer: Local<Option<Timer>>,
    time: Res<Time>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    mut q_server_phase: Query<&mut ServerGamePhase>,
) {
    if timer.is_none() {
        *timer = Some(Timer::from_seconds(5.0, TimerMode::Once));
    }

    let Some(grace_timer) = timer.as_mut() else {
        return;
    };
    grace_timer.tick(time.delta());
    if !grace_timer.is_finished() {
        return;
    }

    for mut phase in q_server_phase.iter_mut() {
        *phase = ServerGamePhase::Lobby;
    }
    next_sim_state.set(SimulationState::Unloaded);
    next_app_state.set(UIContextState::Lobby);
    *timer = None;
}

fn on_server_phase_lobby(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Lobby {
            info!("ServerGamePhase::Lobby observed while InGame — returning to lobby");
            next_app_state.set(UIContextState::Lobby);
        }
    }
}
