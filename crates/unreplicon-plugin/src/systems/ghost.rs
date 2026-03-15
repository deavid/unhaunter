use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientMessageAppExt, FromClient, Replicated, ServerMessageAppExt,
};
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::ghost_guess::GhostGuess;
use unrender_std::components::visuals::SpectralClarity;
use unreplicon_core::components::{
    MissionGoalEntity, RepliconGhostSpawningActive, ServerGamePhase,
};
use unreplicon_core::messages::{
    GhostSoundFieldBroadcast, RequestJournalEvidenceToggle, RequestJournalGhostToggle,
    SpawnParticleNetEvent,
};
use unreplicon_core::resources::MissionConcludingCinematic;
use unspatial_core::lerp_position::LerpPosition;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::GhostTag;
use untypes_core::roles::{AuthorityRole, is_pure_client};
use untypes_core::states::AppState;
use untypes_core::states::SimulationState;

pub(super) fn app_setup(app: &mut App) {
    // Register Phase 2 replicated components.
    app.replicate::<GhostTag>();
    app.replicate::<GhostBreach>();
    app.replicate::<GhostSprite>();
    app.replicate::<GhostBehaviorDynamics>();
    app.replicate::<SpectralClarity>();
    app.replicate::<GhostGuess>();
    app.replicate::<SummaryData>();
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

    // Client: observe ServerGamePhase::Concluding to start cinematic
    app.add_systems(
        Update,
        (on_mission_concluding, tick_mission_concluding)
            .run_if(resource_exists::<untypes_core::roles::LocalPlayerRole>)
            .run_if(in_state(AppState::InGame)),
    );

    // Client: if the server returns to Lobby (e.g. after an abort) while we are
    // still InGame, transition back to AppState::Lobby immediately.
    app.add_systems(
        Update,
        on_server_phase_lobby
            .run_if(is_pure_client)
            .run_if(in_state(AppState::InGame)),
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
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for msg in reader.read() {
        if msg.message.discard {
            if ghost_guess
                .evidences_missing
                .contains(&msg.message.evidence)
            {
                ghost_guess.evidences_missing.remove(&msg.message.evidence);
            } else {
                ghost_guess.evidences_missing.insert(msg.message.evidence);
                ghost_guess.evidences_found.remove(&msg.message.evidence);
            }
        } else if msg.message.mark_as_found {
            ghost_guess.evidences_found.insert(msg.message.evidence);
            ghost_guess.evidences_missing.remove(&msg.message.evidence);
        } else {
            ghost_guess.evidences_found.remove(&msg.message.evidence);
            // Non-discard clear maps to "unset".
            ghost_guess.evidences_missing.remove(&msg.message.evidence);
        }
    }
}

fn handle_journal_ghost_toggle(
    mut reader: MessageReader<FromClient<RequestJournalGhostToggle>>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for msg in reader.read() {
        if msg.message.discard {
            if let Some(ghost_type) = msg.message.ghost_type {
                if ghost_guess.ghosts_discarded.contains(&ghost_type) {
                    ghost_guess.ghosts_discarded.remove(&ghost_type);
                } else {
                    ghost_guess.ghosts_discarded.insert(ghost_type);
                    if ghost_guess.ghost_type == Some(ghost_type) {
                        ghost_guess.ghost_type = None;
                    }
                }
            }
        } else {
            ghost_guess.ghost_type = msg.message.ghost_type;
        }
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
    mut next_app_state: ResMut<NextState<AppState>>,
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
    next_app_state.set(AppState::Lobby);
    *timer = None;
}

fn on_server_phase_lobby(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Lobby {
            info!("ServerGamePhase::Lobby observed while InGame — returning to lobby");
            next_app_state.set(AppState::Lobby);
        }
    }
}

fn on_mission_concluding(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    mut commands: Commands,
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Concluding {
            info!("ServerGamePhase::Concluding observed — starting cinematic");
            commands.insert_resource(MissionConcludingCinematic {
                timer: Timer::from_seconds(2.5, TimerMode::Once),
                inputs_blocked: true,
            });
        }
    }
}

fn tick_mission_concluding(
    mut commands: Commands,
    cinematic: Option<ResMut<MissionConcludingCinematic>>,
    summary_data: Option<Res<SummaryData>>,
    authority: Option<Res<AuthorityRole>>,
    q_goal: Query<&SummaryData, With<MissionGoalEntity>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    let Some(mut cinematic) = cinematic else {
        return;
    };

    cinematic.timer.tick(time.delta());
    if !cinematic.timer.just_finished() {
        return;
    }

    // summary_data on authority or summary_data replicated component on client
    let ready = if authority.is_some() {
        summary_data.is_some()
    } else {
        q_goal.iter().next().is_some()
    };
    if ready {
        next_app_state.set(AppState::Summary);
        commands.remove_resource::<MissionConcludingCinematic>();
    }
}
