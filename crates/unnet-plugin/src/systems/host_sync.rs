use crate::resources::{HandshakeState, NetworkConn};
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use unevents_core::events::sound::SoundEvent;
use ungear_core::components::core::Battery;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungearitems_core::components::flashlight::Flashlight;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_influence::GhostInfluence;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::ghost_guess::GhostGuess;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{
    GearSyncState, GhostState, HauntedObjectSync, MapTileState, MovableObjectSync, NetworkMessage,
    PlayerGearState, PlayerState, RoomSync, SnapshotMsg,
};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::MissionEndRequested;
use unplayer_core::components::{Hiding, PlayerInput, PlayerSpectating, PlayerSprite, Stamina};
use unrender_std::components::animation::AnimationTimer;
use unspatial_core::components::NetworkOriginalMapPosition;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::GhostTag;
use untruck_core::components::in_truck::InTruck;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;
use untypes_core::cli::CliOptions;
use untypes_core::cli::NetMode;
use untypes_core::states::{AppState, GameState};

use super::utils::extract_gear_details;
use crate::metrics;

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct HostSnapshotParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub query_players: Query<
        'w,
        's,
        (
            &'static PlayerSprite,
            &'static Position,
            &'static unspatial_core::direction::Direction,
            Option<&'static Hiding>,
            Option<&'static InTruck>,
            Option<&'static PlayerGear>,
            Option<&'static Stamina>,
            &'static AnimationTimer,
            Option<&'static PlayerSpectating>,
            Option<&'static PlayerInput>,
        ),
    >,
    pub query_ghosts: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static Position,
            &'static GhostSprite,
            &'static GhostBehaviorDynamics,
        ),
        With<GhostTag>,
    >,
    pub time: Res<'w, Time>,
    pub room_db: Res<'w, RoomDB>,
    pub query_map_tiles: Query<'w, 's, (&'static Position, &'static Behavior), With<Interactive>>,
    pub query_gear: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static ungear_core::types::gear::kind::GearKind,
            &'static Position,
            Option<&'static Toggleable>,
            Option<&'static DeployedGear>,
            Option<&'static Battery>,
            Option<&'static Flashlight>,
            Option<&'static ungearitems_core::components::sage::SageBundleData>,
            Option<&'static ungearitems_core::components::repellentflask::RepellentFlask>,
            Option<&'static ungearitems_core::components::thermometer::Thermometer>,
            Option<&'static ungearitems_core::components::emfmeter::EMFMeter>,
            Option<&'static ungearitems_core::components::spiritbox::SpiritBox>,
        ),
    >,
    pub query_net_id: Query<'w, 's, &'static NetworkId>,
    pub game_state: Res<'w, State<GameState>>,
    pub app_state: Res<'w, State<AppState>>,
    pub ghost_guess: Res<'w, GhostGuess>,
    pub summary_data: Res<'w, SummaryData>,
    pub changed_tiles: ResMut<'w, unnet_core::resources::ChangedTiles>,
    pub mission_end_requested: Res<'w, MissionEndRequested>,
    pub repellent_craft_tracker: Res<'w, RepellentCraftTracker>,
    pub query_breach: Query<'w, 's, &'static Position, With<GhostBreach>>,
    pub query_influence:
        Query<'w, 's, (&'static NetworkOriginalMapPosition, &'static GhostInfluence)>,
    pub query_movable: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static NetworkId>,
            &'static Position,
            &'static NetworkOriginalMapPosition,
        ),
        With<unbehavior::components::Movable>,
    >,
}

pub(crate) fn host_send_snapshots_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    mut host_params: HostSnapshotParams,
    mut ev_sound: MessageReader<SoundEvent>,
    mut ev_transient: MessageReader<unnet_core::messages::TransientEvent>,
) {
    let measure = metrics::HOST_SEND_SNAPSHOTS.time_measure();
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }
    let NetworkConn::Host { clients, .. } = &mut *conn else {
        measure.end_ms();
        return;
    };
    if clients.is_empty() {
        measure.end_ms();
        return;
    }

    let tick = (host_params.time.elapsed_secs() * 60.0) as u64;
    let players: Vec<PlayerState> = host_params
        .query_players
        .iter()
        .map(
            |(player, pos, dir, hiding, in_truck, _, stamina, anim, spectating, input)| {
                PlayerState {
                    id: player.id,
                    position: [pos.x, pos.y, pos.z],
                    orientation: [dir.dx, dir.dy],
                    target_position: input.and_then(|i| i.target_position).map(|v| [v.x, v.y]),
                    is_hiding: hiding.is_some(),
                    is_in_truck: in_truck.is_some(),
                    stamina: stamina.map(|s| s.current).unwrap_or(100.0),
                    health: player.health,
                    sanity: player.sanity,
                    is_running: stamina.map(|s| s.running).unwrap_or(false),
                    frame: anim.idx() as u16,
                    is_spectating: spectating.is_some(),
                }
            },
        )
        .collect();

    let player_gear = host_params
        .query_players
        .iter()
        .filter_map(|(p, _, _, _, _, gear, _, _, _, _)| {
            gear.map(|g| PlayerGearState {
                player_id: p.id,
                left_hand: g
                    .left_hand
                    .and_then(|e| host_params.query_net_id.get(e).ok().cloned()),
                right_hand: g
                    .right_hand
                    .and_then(|e| host_params.query_net_id.get(e).ok().cloned()),
                inventory: g
                    .inventory
                    .iter()
                    .filter_map(|&e| host_params.query_net_id.get(e).ok().cloned())
                    .collect(),
                held_item: g
                    .held_item
                    .as_ref()
                    .and_then(|h| host_params.query_net_id.get(h.entity).ok().cloned()),
            })
        })
        .collect();

    let ghosts = host_params
        .query_ghosts
        .iter()
        .map(|(id, pos, ghost, dynamics)| GhostState {
            id: *id,
            position: [pos.x, pos.y, pos.z],
            warp: ghost.warp,
            hunt_warning_active: ghost.hunt_warning_active,
            hunt_warning_intensity: ghost.hunt_warning_intensity,
            hunt_target: ghost.hunt_target,
            calm_time_secs: ghost.calm_time_secs,
            repellent_hits_delta: ghost.repellent_hits_delta,
            repellent_misses_delta: ghost.repellent_misses_delta,
            repellent_hits: ghost.repellent_hits,
            class: ghost.class,
            freezing_temp_clarity: dynamics.freezing_temp_clarity,
            floating_orbs_clarity: dynamics.floating_orbs_clarity,
            uv_ectoplasm_clarity: dynamics.uv_ectoplasm_clarity,
            emf_level5_clarity: dynamics.emf_level5_clarity,
            evp_recording_clarity: dynamics.evp_recording_clarity,
            spirit_box_clarity: dynamics.spirit_box_clarity,
            rl_presence_clarity: dynamics.rl_presence_clarity,
            cpm500_clarity: dynamics.cpm500_clarity,
            visual_alpha_multiplier: dynamics.visual_alpha_multiplier,
            rage_tendency_multiplier: dynamics.rage_tendency_multiplier,
        })
        .collect();

    let rooms = host_params
        .room_db
        .room_state
        .iter()
        .map(|(name, state): (&String, &TileState)| RoomSync {
            name: name.clone(),
            state: match state {
                TileState::On => 1,
                _ => 0,
            },
        })
        .collect();

    let gear: Vec<_> = host_params
        .query_gear
        .iter()
        .map(
            |(
                id,
                kind,
                pos,
                toggle,
                deployed,
                battery,
                flashlight,
                sage,
                repellent,
                thermometer,
                emfm,
                spiritbox,
            )| {
                let details =
                    extract_gear_details(flashlight, sage, repellent, thermometer, emfm, spiritbox);

                GearSyncState {
                    id: *id,
                    kind: *kind,
                    position: [pos.x, pos.y, pos.z],
                    is_on: toggle.map(|t| t.is_on).unwrap_or(false),
                    is_deployed: deployed.is_some(),
                    deployed_direction: deployed
                        .map(|d| [d.direction.dx, d.direction.dy])
                        .unwrap_or([0.0, 0.0]),
                    details,
                    battery: battery.map(|b| b.level).unwrap_or(0.0),
                }
            },
        )
        .collect();

    let mut events: Vec<unnet_core::messages::TransientEvent> = ev_sound
        .read()
        .filter(|ev| ev.broadcast)
        .map(|ev| unnet_core::messages::TransientEvent::PlaySound {
            sound_file: ev.sound_file.clone(),
            volume: ev.volume,
            position: ev.position.map(|p| [p.x, p.y, p.z]),
        })
        .collect();

    events.extend(ev_transient.read().cloned());

    let any_needs_full = clients
        .iter()
        .any(|c| c.handshake == HandshakeState::Completed && c.needs_full_sync);

    let delta_tiles: Vec<MapTileState> = host_params.changed_tiles.0.drain(..).collect();
    let full_tiles: Option<Vec<MapTileState>> = if any_needs_full {
        Some(
            host_params
                .query_map_tiles
                .iter()
                .map(|(pos, beh)| MapTileState {
                    x: pos.x as i32,
                    y: pos.y as i32,
                    z: pos.z as i32,
                    tileset: beh.cfg().tileset.clone(),
                    tileuid: beh.cfg().tileuid,
                    cvo_key: beh.key_cvo().to_key_string(),
                })
                .collect(),
        )
    } else {
        None
    };

    let breach_position = host_params
        .query_breach
        .single()
        .ok()
        .map(|pos| [pos.x, pos.y, pos.z]);

    let ghost_type = host_params
        .query_ghosts
        .iter()
        .next()
        .map(|(_, _, ghost, _)| ghost.class);

    let haunted_objects = host_params
        .query_influence
        .iter()
        .map(|(orig, influence)| HauntedObjectSync {
            original_position: [
                orig.position.x as i32,
                orig.position.y as i32,
                orig.position.z as i32,
            ],
            tileset: orig.tileset.clone(),
            tileuid: orig.tileuid,
            influence_type: influence.influence_type,
        })
        .collect();

    let movable_objects =
        host_params
            .query_movable
            .iter()
            .map(|(entity, nid, pos, orig)| {
                let mid = match nid {
                    Some(id) => *id,
                    None => {
                        let mut rng = rand::rng();
                        let new_id = NetworkId(rng.random_range(1000..u64::MAX));
                        host_params.commands.entity(entity).insert(new_id);
                        new_id
                    }
                };
                let held_by = host_params.query_players.iter().find_map(
                    |(p, _, _, _, _, gear, _, _, _, _)| {
                        gear.and_then(|g| {
                            g.held_item.as_ref().and_then(|h| {
                                if host_params.query_net_id.get(h.entity).ok() == Some(&mid) {
                                    Some(p.id)
                                } else {
                                    None
                                }
                            })
                        })
                    },
                );
                // Sending all objects: For some reason, sending only if is_changed causes issues on the client on high RTT
                MovableObjectSync {
                    id: mid,
                    original_position: [
                        orig.position.x as i32,
                        orig.position.y as i32,
                        orig.position.z as i32,
                    ],
                    tileset: orig.tileset.clone(),
                    tileuid: orig.tileuid,
                    current_position: [pos.x, pos.y, pos.z],
                    held_by,
                }
            })
            .collect();

    let base_snapshot = SnapshotMsg {
        tick,
        is_full_sync: false,
        app_state: *host_params.app_state.get(),
        // Pause is a local-only state; broadcast None instead so clients aren't affected
        game_state: if *host_params.game_state.get() == GameState::Pause {
            GameState::None
        } else {
            *host_params.game_state.get()
        },
        can_end_mission: host_params.mission_end_requested.0,
        players,
        ghosts,
        rooms,
        map_tiles: delta_tiles,
        gear,
        player_gear,
        events,
        evidences_found: host_params
            .ghost_guess
            .evidences_found
            .iter()
            .cloned()
            .collect(),
        evidences_missing: host_params
            .ghost_guess
            .evidences_missing
            .iter()
            .cloned()
            .collect(),
        ghost_type_guess: host_params.ghost_guess.ghost_type,
        ghosts_discarded: host_params
            .ghost_guess
            .ghosts_discarded
            .iter()
            .cloned()
            .collect(),
        mission_result: Box::new(if *host_params.app_state.get() == AppState::Summary {
            Some(unnet_core::messages::MissionResult {
                time_taken_secs: host_params.summary_data.time_taken_secs,
                ghost_types: host_params.summary_data.ghost_types.clone(),
                repellent_used_amt: host_params.summary_data.repellent_used_amt,
                ghosts_unhaunted: host_params.summary_data.ghosts_unhaunted,
                base_score: host_params.summary_data.base_score,
                difficulty_multiplier: host_params.summary_data.difficulty_multiplier,
                grade_multiplier: host_params.summary_data.grade_multiplier,
                average_sanity: host_params.summary_data.average_sanity,
                player_count: host_params.summary_data.player_count as u32,
                alive_count: host_params.summary_data.alive_count as u32,
                full_score: host_params.summary_data.full_score,
                mission_successful: host_params.summary_data.mission_successful,
                money_earned: host_params.summary_data.money_earned,
                grade_achieved: host_params.summary_data.grade_achieved,
                required_deposit: host_params.summary_data.required_deposit,
                mission_reward_base: host_params.summary_data.mission_reward_base,
                deposit_originally_held: host_params.summary_data.deposit_originally_held,
                deposit_returned_to_bank: host_params.summary_data.deposit_returned_to_bank,
                costs_deducted_from_deposit: host_params.summary_data.costs_deducted_from_deposit,
            })
        } else {
            None
        }),
        repellent_crafted_count: host_params.repellent_craft_tracker.crafted_count,
        breach_position,
        ghost_type,
        haunted_objects,
        movable_objects,
    };

    for client in clients.iter_mut() {
        if client.handshake != HandshakeState::Completed {
            continue;
        }
        if client.needs_full_sync {
            let mut full_snap = base_snapshot.clone();
            full_snap.is_full_sync = true;
            full_snap.map_tiles = full_tiles.clone().unwrap_or_default();
            client.needs_full_sync = false;
            client
                .write_queue
                .push_back(NetworkMessage::Snapshot(Box::new(full_snap)));
        } else {
            client
                .write_queue
                .push_back(NetworkMessage::Snapshot(Box::new(base_snapshot.clone())));
        }
    }
    measure.end_ms();
}

pub(crate) fn host_send_summary_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    summary_data: Res<SummaryData>,
) {
    let measure = metrics::HOST_SEND_SUMMARY.time_measure();
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        measure.end_ms();
        return;
    }
    info!("Network: Sending MissionSummary to clients");
    conn.host_broadcast(NetworkMessage::MissionSummary {
        result: unnet_core::messages::MissionResult {
            time_taken_secs: summary_data.time_taken_secs,
            ghost_types: summary_data.ghost_types.clone(),
            repellent_used_amt: summary_data.repellent_used_amt,
            ghosts_unhaunted: summary_data.ghosts_unhaunted,
            base_score: summary_data.base_score,
            difficulty_multiplier: summary_data.difficulty_multiplier,
            grade_multiplier: summary_data.grade_multiplier,
            average_sanity: summary_data.average_sanity,
            player_count: summary_data.player_count as u32,
            alive_count: summary_data.alive_count as u32,
            full_score: summary_data.full_score,
            mission_successful: summary_data.mission_successful,
            money_earned: summary_data.money_earned,
            grade_achieved: summary_data.grade_achieved,
            required_deposit: summary_data.required_deposit,
            mission_reward_base: summary_data.mission_reward_base,
            deposit_originally_held: summary_data.deposit_originally_held,
            deposit_returned_to_bank: summary_data.deposit_returned_to_bank,
            costs_deducted_from_deposit: summary_data.costs_deducted_from_deposit,
        },
    });
    measure.end_ms();
}
