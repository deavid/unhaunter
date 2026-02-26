//! Phase 4: Ghost, Evidence, and Mission-flow replication systems.
//!
//! This module handles:
//! - Replicating ghost position and state from the server to all clients.
//! - Replicating journal evidence + ghost-type guess so all players share it.
//! - Distributing smoke-particle spawn events from server to clients.
//! - Receiving client journal-toggle requests and applying them to the server's
//!   `GhostGuess` resource.

use bevy::prelude::*;
use bevy_replicon::prelude::{
    AppRuleExt, Channel, ClientMessageAppExt, FromClient, Replicated, ServerMessageAppExt,
    ServerState,
};
use rand::RngExt;
use unboard_core::components::mapcolor::MapColor;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use unfoundation_core::types::grade::Grade;
use ungearitems_core::components::sage::{SageSmokeParticle, SmokeParticleTimer};
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::ghost_guess::GhostGuess;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unreplicon_core::components::{
    MissionGoalEntity, RepliconGhostSpawningActive, ServerGamePhase,
};
use unreplicon_core::messages::{
    RequestJournalEvidenceToggle, RequestJournalGhostToggle, SpawnParticleNetEvent,
};
use unreplicon_core::net_components::{
    EvidenceFoundNet, GhostStateNet, MissionResultNet, NetworkPosition,
};
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::GhostTag;
use untypes_core::states::AppState;

/// Speed at which ghost position is interpolated toward its network value on clients.
const LERP_SPEED: f32 = 15.0;

pub(super) fn app_setup(app: &mut App) {
    // Register Phase 4 replicated components.
    app.replicate::<GhostStateNet>();
    app.replicate::<EvidenceFoundNet>();
    app.replicate::<MissionResultNet>();

    // Register server → client messages.
    app.add_server_message::<SpawnParticleNetEvent>(Channel::Ordered);

    // Register client → server messages.
    app.add_client_message::<RequestJournalEvidenceToggle>(Channel::Ordered);
    app.add_client_message::<RequestJournalGhostToggle>(Channel::Ordered);

    // Server: setup and teardown ghost replication entities.
    app.add_systems(
        OnEnter(AppState::InGame),
        setup_ghost_entities.run_if(in_state(ServerState::Running)),
    );
    app.add_systems(
        OnExit(AppState::InGame),
        cleanup_ghost_entities.run_if(in_state(ServerState::Running)),
    );

    // Server: state sync and journal request handlers.
    app.add_systems(
        Update,
        (
            sync_ghost_state_to_net,
            sync_evidence_to_net,
            handle_journal_evidence_toggle,
            handle_journal_ghost_toggle,
        )
            .run_if(in_state(ServerState::Running))
            .run_if(in_state(AppState::InGame)),
    );

    // Server: write mission result when summary screen activates.
    app.add_systems(
        Update,
        sync_mission_result_to_net
            .run_if(in_state(ServerState::Running))
            .run_if(in_state(AppState::Summary)),
    );

    // Client: apply replicated state to local world.
    app.add_systems(
        Update,
        (
            interpolate_ghost_position,
            apply_ghost_state_net,
            apply_evidence_net,
            handle_spawn_particle,
        )
            .run_if(in_state(AppState::InGame))
            .run_if(not(in_state(ServerState::Running))),
    );

    // Client: apply mission result — runs in any non-server state so it fires
    // even as the client is about to re-enter InGame after a mission ends.
    app.add_systems(
        Update,
        apply_mission_result_net.run_if(not(in_state(ServerState::Running))),
    );
}

/// Server: called once on `OnEnter(AppState::InGame)`.
///
/// 1. Inserts `RepliconGhostSpawningActive` so that `classic_mode_orchestrator`
///    in `unclassic-mode-plugin` skips local ghost spawning for Join clients.
/// 2. Adds `(Replicated, NetworkPosition, GhostStateNet)` to the host's ghost entity
///    so bevy_replicon replicates it to connected clients.
/// 3. Spawns a singleton `MissionGoalEntity` carrying journal + mission-result
///    replicated components.
fn setup_ghost_entities(
    q_ghost: Query<(Entity, &Position), With<GhostTag>>,
    mut commands: Commands,
) {
    commands.insert_resource(RepliconGhostSpawningActive);

    for (entity, pos) in q_ghost.iter() {
        commands.entity(entity).insert((
            Replicated,
            NetworkPosition {
                x: pos.x,
                y: pos.y,
                z: pos.z,
            },
            GhostStateNet::default(),
        ));
        info!(
            "setup_ghost_entities: ghost entity {:?} marked Replicated",
            entity
        );
    }

    // Singleton entity for journal + mission-result replication.
    commands.spawn((
        Replicated,
        MissionGoalEntity,
        EvidenceFoundNet::default(),
        MissionResultNet::default(),
    ));
    info!("setup_ghost_entities: MissionGoalEntity spawned");
}

/// Server: remove the spawning-active marker and despawn the goal entity on InGame exit.
///
/// `bevy_replicon` propagates the despawn to all connected clients automatically.
fn cleanup_ghost_entities(q_goal: Query<Entity, With<MissionGoalEntity>>, mut commands: Commands) {
    commands.remove_resource::<RepliconGhostSpawningActive>();
    for entity in q_goal.iter() {
        commands.entity(entity).despawn();
    }
}

/// Server: copy `GhostSprite` + `GhostBehaviorDynamics` → `NetworkPosition` + `GhostStateNet`.
fn sync_ghost_state_to_net(
    mut q_ghost: Query<
        (
            &Position,
            &GhostSprite,
            &GhostBehaviorDynamics,
            &mut NetworkPosition,
            &mut GhostStateNet,
        ),
        With<GhostTag>,
    >,
) {
    for (pos, sprite, dynamics, mut net_pos, mut net_state) in q_ghost.iter_mut() {
        net_pos.x = pos.x;
        net_pos.y = pos.y;
        net_pos.z = pos.z;

        net_state.is_hunting = sprite.hunting > 0.1;
        net_state.hunt_warning_active = sprite.hunt_warning_active;
        net_state.hunt_warning_intensity = sprite.hunt_warning_intensity;
        net_state.hunt_target = sprite.hunt_target;
        net_state.visual_alpha_multiplier = dynamics.visual_alpha_multiplier;
    }
}

/// Server: copy `GhostGuess` resource → `EvidenceFoundNet` on the `MissionGoalEntity`.
fn sync_evidence_to_net(
    ghost_guess: Option<Res<GhostGuess>>,
    mut q_goal: Query<&mut EvidenceFoundNet, With<MissionGoalEntity>>,
) {
    let Some(ghost_guess) = ghost_guess else {
        return;
    };
    let Ok(mut net) = q_goal.single_mut() else {
        return;
    };
    net.found = ghost_guess.evidences_found.iter().cloned().collect();
    net.missing = ghost_guess.evidences_missing.iter().cloned().collect();
    net.ghost_type_guess = ghost_guess.ghost_type;
    net.ghosts_discarded = ghost_guess.ghosts_discarded.iter().cloned().collect();
}

/// Server: handle `RequestJournalEvidenceToggle` messages from clients.
fn handle_journal_evidence_toggle(
    mut reader: MessageReader<FromClient<RequestJournalEvidenceToggle>>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for FromClient { message, .. } in reader.read() {
        if message.mark_as_found {
            ghost_guess.evidences_found.insert(message.evidence);
            ghost_guess.evidences_missing.remove(&message.evidence);
        } else {
            ghost_guess.evidences_found.remove(&message.evidence);
            ghost_guess.evidences_missing.insert(message.evidence);
        }
    }
}

/// Server: handle `RequestJournalGhostToggle` messages from clients.
fn handle_journal_ghost_toggle(
    mut reader: MessageReader<FromClient<RequestJournalGhostToggle>>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    for FromClient { message, .. } in reader.read() {
        ghost_guess.ghost_type = message.ghost_type;
    }
}

/// Client: lerp ghost `Position` toward server-authoritative `NetworkPosition`.
fn interpolate_ghost_position(
    mut q_ghost: Query<(&NetworkPosition, &mut Position), With<GhostStateNet>>,
    time: Res<Time>,
) {
    let alpha = (LERP_SPEED * time.delta_secs()).min(1.0);
    for (net_pos, mut pos) in q_ghost.iter_mut() {
        pos.x += (net_pos.x - pos.x) * alpha;
        pos.y += (net_pos.y - pos.y) * alpha;
        pos.z += (net_pos.z - pos.z) * alpha;
    }
}

/// Client: apply `GhostStateNet` changes to the local `GhostSprite` + `GhostBehaviorDynamics`.
///
/// Only runs on non-server nodes; the server writes in the opposite direction.
fn apply_ghost_state_net(
    mut q_ghost: Query<
        (&GhostStateNet, &mut GhostSprite, &mut GhostBehaviorDynamics),
        Changed<GhostStateNet>,
    >,
) {
    for (net, mut sprite, mut dynamics) in q_ghost.iter_mut() {
        sprite.hunting = if net.is_hunting { 1.0 } else { 0.0 };
        sprite.hunt_warning_active = net.hunt_warning_active;
        sprite.hunt_warning_intensity = net.hunt_warning_intensity;
        sprite.hunt_target = net.hunt_target;
        dynamics.visual_alpha_multiplier = net.visual_alpha_multiplier;
    }
}

/// Client: apply `EvidenceFoundNet` changes to the local `GhostGuess` resource.
///
/// Runs on non-server nodes only; the server writes GhostGuess directly.
fn apply_evidence_net(
    q_net: Query<&EvidenceFoundNet, Changed<EvidenceFoundNet>>,
    mut ghost_guess: Option<ResMut<GhostGuess>>,
) {
    let Ok(net) = q_net.single() else {
        return;
    };
    let Some(ref mut ghost_guess) = ghost_guess else {
        return;
    };
    ghost_guess.evidences_found = net.found.iter().cloned().collect();
    ghost_guess.evidences_missing = net.missing.iter().cloned().collect();
    ghost_guess.ghost_type = net.ghost_type_guess;
    ghost_guess.ghosts_discarded = net.ghosts_discarded.iter().cloned().collect();
}

/// Client: handle `SpawnParticleNetEvent` messages broadcast from the server and
/// spawn the visual particle entity locally.
fn handle_spawn_particle(
    mut reader: MessageReader<SpawnParticleNetEvent>,
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    let mut rng = random_seed::rng();
    for event in reader.read() {
        if event.particle_type == "smoke" {
            let pos = Position {
                x: event.position[0],
                y: event.position[1],
                z: event.position[2],
                visual_priority: 0.0,
            };
            commands
                .spawn(Sprite {
                    image: asset_server.load("img/smoke.png"),
                    color: Color::NONE,
                    ..default()
                })
                .insert(
                    Transform::from_translation(perspective::to_screen_coord(pos))
                        .with_scale(Vec3::new(0.2, 0.2, 0.2)),
                )
                .insert(SageSmokeParticle)
                .insert(GameSprite)
                .insert(pos)
                .insert(Direction {
                    dx: rng.random_range(-0.9..0.9),
                    dy: rng.random_range(-0.9..0.9),
                    dz: rng.random_range(-0.5..0.5),
                })
                .insert(MapColor {
                    color: Color::srgba(1.0, 1.0, 1.0, 0.20),
                })
                .insert(SmokeParticleTimer(Timer::from_seconds(
                    5.0,
                    TimerMode::Once,
                )))
                .insert(SpriteLayer::default());
        }
    }
}

/// Server: when the server enters `AppState::Summary`, write the authoritative
/// `SummaryData` into `MissionResultNet` and set `ServerGamePhase::Ended` so
/// clients can follow the transition to the summary screen.
///
/// Runs every frame while in `AppState::Summary` with a server `ServerState`,
/// but uses `Res::is_changed()` so the actual write only happens once when
/// `SummaryData` is first populated by `calculate_rewards_and_grades`.
fn sync_mission_result_to_net(
    summary: Res<SummaryData>,
    mut q_goal: Query<&mut MissionResultNet, With<MissionGoalEntity>>,
    mut q_server_phase: Query<&mut ServerGamePhase>,
) {
    if !summary.is_changed() {
        return;
    }

    // Signal to clients that the server has moved to the summary state.
    for mut sas in q_server_phase.iter_mut() {
        *sas = ServerGamePhase::Ended;
    }

    let Ok(mut net) = q_goal.single_mut() else {
        warn!("sync_mission_result_to_net: no MissionGoalEntity found");
        return;
    };

    net.time_taken_secs = summary.time_taken_secs;
    net.ghost_types = summary.ghost_types.clone();
    net.repellent_used_amt = summary.repellent_used_amt;
    net.ghosts_unhaunted = summary.ghosts_unhaunted;
    net.base_score = summary.base_score;
    net.difficulty_multiplier = summary.difficulty_multiplier;
    net.grade_multiplier = summary.grade_multiplier;
    net.average_sanity = summary.average_sanity;
    net.player_count = summary.player_count as u32;
    net.alive_count = summary.alive_count as u32;
    net.full_score = summary.full_score;
    net.map_path = summary.map_path.clone();
    net.mission_successful = summary.mission_successful;
    net.money_earned = summary.money_earned;
    net.grade_achieved = format!("{}", summary.grade_achieved);
    net.required_deposit = summary.required_deposit;
    net.mission_reward_base = summary.mission_reward_base;
    net.deposit_originally_held = summary.deposit_originally_held;
    net.deposit_returned_to_bank = summary.deposit_returned_to_bank;
    net.costs_deducted_from_deposit = summary.costs_deducted_from_deposit;
    net.ready = true;

    info!(
        "sync_mission_result_to_net: written MissionResultNet (score={}, grade={})",
        net.full_score, net.grade_achieved
    );
}

/// Client: apply `MissionResultNet` to the local `SummaryData` resource and
/// transition to `AppState::Summary` when the server marks the result as ready.
///
/// Only runs on non-server nodes.  The `SummaryData` transition is normally driven
/// by `update_time` (player deaths), but the server's authoritative result takes
/// precedence for score breakdown, grade, and financial fields.
fn apply_mission_result_net(
    q_net: Query<&MissionResultNet, Changed<MissionResultNet>>,
    mut summary: ResMut<SummaryData>,
    current_difficulty: Res<CurrentDifficulty>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    let Ok(net) = q_net.single() else {
        return;
    };
    if !net.ready {
        return;
    }

    summary.time_taken_secs = net.time_taken_secs;
    summary.ghost_types = net.ghost_types.clone();
    summary.repellent_used_amt = net.repellent_used_amt;
    summary.ghosts_unhaunted = net.ghosts_unhaunted;
    summary.base_score = net.base_score;
    summary.difficulty_multiplier = net.difficulty_multiplier;
    summary.grade_multiplier = net.grade_multiplier;
    summary.average_sanity = net.average_sanity;
    summary.player_count = net.player_count as usize;
    summary.alive_count = net.alive_count as usize;
    summary.full_score = net.full_score;
    summary.map_path = net.map_path.clone();
    summary.mission_successful = net.mission_successful;
    summary.money_earned = net.money_earned;
    summary.grade_achieved = Grade::from(net.grade_achieved.as_str());
    summary.required_deposit = net.required_deposit;
    summary.mission_reward_base = net.mission_reward_base;
    summary.deposit_originally_held = net.deposit_originally_held;
    summary.deposit_returned_to_bank = net.deposit_returned_to_bank;
    summary.costs_deducted_from_deposit = net.costs_deducted_from_deposit;
    // Preserve the local difficulty resource so grade display is correct.
    summary.difficulty = current_difficulty.clone();
    // animated_final_score starts at 0; the UI score-count animation drives it.
    summary.animated_final_score = 0;

    info!(
        "apply_mission_result_net: received summary (score={}, grade={})",
        net.full_score, net.grade_achieved
    );

    next_state.set(AppState::Summary);
}
