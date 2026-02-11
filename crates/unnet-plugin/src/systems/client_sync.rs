use bevy::audio::SpatialListener;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use rand::prelude::*;
use unassets_core::resources::upscale::UpscaleIndex;
use unbehavior::behavior::Behavior;
use unbehavior::roomdb::RoomDB;
use unbehavior::state::TileState;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::resources::board_topology::{BoardEntityField, BoardTopology};
use unevents_core::events::roomchanged::{InteractionExecutionType, RoomStateSyncEvent};
use unevents_core::events::sound::SoundEvent;
use unfoundation_core::types::gear::EquipmentPosition;
use ungear_core::components::core::Battery;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungearitems_core::components::flashlight::Flashlight;
use ungearitems_core::components::sage::{SageSmokeParticle, SmokeParticleTimer};
use unghost_core::assets::GhostAssets;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_influence::GhostInfluence;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::resources::ghost_guess::GhostGuess;
use uninteraction_core::interaction::{ExecuteInteractionEvent, Toggleable};
use unmetrics_core::metrics::SendMetric;
use unnet_core::messages::{
    GearDetails, GearSyncState, NetworkDataEvent, NetworkMessage, SnapshotMsg, TransientEvent,
};
use unnet_core::network_id::{NetworkId, ToBeDespawned};
use unnet_core::resources::MissionEndRequested;
use unplayer_core::assets::PlayerAssets;
use unplayer_core::components::{
    Hiding, MainPlayer, PlayerInput, PlayerSpectating, PlayerSprite, Stamina,
};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::game::{GameSprite, MapTileSprite};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{
    AlphaModulator, LightSensitive, ResolutionFactor, ShadowCaster, SpectralInfluence,
    UltravioletSensitive,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::quadcc::QuadCC;
use unsettings_core::video::VideoSettings;

use unspatial_core::boardposition::{BoardPosition, MapEntityFieldBPos};
use unspatial_core::components::NetworkOriginalMapPosition;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::GhostTag;
use untags_core::tags::PlayerTag;
use untruck_core::components::in_truck::InTruck;
use untruck_core::components::truck_ui_button::TruckUIButton;
use untruck_core::types::repellent_tracker::RepellentCraftTracker;
use untruck_core::types::truck_button::{TruckButtonState, TruckButtonType};
use untypes_core::cli::CliOptions;
use untypes_core::cli::NetMode;
use untypes_core::states::{AppState, GameState};

use super::utils::LastSyncedGearState;
use crate::metrics;

#[derive(SystemParam)]
pub(crate) struct SnapshotAppStates<'w> {
    pub game_next_state: ResMut<'w, NextState<GameState>>,
    pub current_game_state: Res<'w, State<GameState>>,
    pub current_app_state: Res<'w, State<AppState>>,
    pub app_next_state: ResMut<'w, NextState<AppState>>,
}

#[derive(SystemParam)]
#[allow(clippy::type_complexity)]
pub(crate) struct ClientSnapshotParams<'w, 's> {
    pub commands: Commands<'w, 's>,
    pub cli: Res<'w, CliOptions>,
    pub asset_server: Res<'w, AssetServer>,
    pub player_assets: Res<'w, PlayerAssets>,
    pub ghost_assets: Res<'w, GhostAssets>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub materials1: ResMut<'w, Assets<CustomMaterial1>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Res<'w, bevy_persistent::Persistent<VideoSettings>>,
    pub audio_settings: Res<'w, bevy_persistent::Persistent<unsettings_core::audio::AudioSettings>>,
    pub control_settings:
        Res<'w, bevy_persistent::Persistent<unsettings_core::controls::ControlKeys>>,
    pub query_players: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut Position,
            &'static mut AnimationTimer,
            &'static mut unspatial_core::direction::Direction,
            Option<&'static Hiding>,
            Option<&'static InTruck>,
            Option<&'static mut PlayerGear>,
            Option<&'static MainPlayer>,
            &'static mut Stamina,
            Option<&'static PlayerSpectating>,
            &'static mut PlayerSprite,
            &'static mut PlayerInput,
        ),
        (Without<GhostTag>, Without<Behavior>, Without<GhostBreach>),
    >,
    pub query_ghosts: Query<
        'w,
        's,
        (
            &'static NetworkId,
            &'static mut Position,
            &'static mut GhostSprite,
            &'static mut GhostBehaviorDynamics,
        ),
        (
            With<GhostTag>,
            Without<PlayerSprite>,
            Without<Behavior>,
            Without<GhostBreach>,
        ),
    >,
    pub ev_room_sync: MessageWriter<'w, RoomStateSyncEvent>,
    pub board_field: Res<'w, BoardEntityField>,
    pub board_topo: Res<'w, BoardTopology>,
    pub query_tiles: Query<
        'w,
        's,
        &'static Behavior,
        (
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<GhostBreach>,
        ),
    >,
    pub ev_interaction: MessageWriter<'w, ExecuteInteractionEvent>,
    pub room_db: ResMut<'w, RoomDB>,
    pub states: SnapshotAppStates<'w>,
    pub query_gear: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkId,
            &'static mut Position,
            Option<&'static mut Toggleable>,
            Option<&'static mut DeployedGear>,
            Option<&'static mut Battery>,
            Option<&'static mut Flashlight>,
            Option<&'static mut ungearitems_core::components::sage::SageBundleData>,
            Option<&'static mut ungearitems_core::components::repellentflask::RepellentFlask>,
            Option<&'static mut ungearitems_core::components::thermometer::Thermometer>,
            Option<&'static mut ungearitems_core::components::emfmeter::EMFMeter>,
            Option<&'static mut ungearitems_core::components::spiritbox::SpiritBox>,
            Option<&'static mut LastSyncedGearState>,
        ),
        (
            Without<unbehavior::components::Movable>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<Behavior>,
            Without<GhostBreach>,
        ),
    >,
    pub query_net_entities: Query<'w, 's, (Entity, &'static NetworkId)>,
    pub query_tbd: Query<'w, 's, Entity, With<ToBeDespawned>>,
    pub ev_sound: MessageWriter<'w, SoundEvent>,
    pub ghost_guess: ResMut<'w, GhostGuess>,
    pub query_buttons: Query<'w, 's, &'static mut TruckUIButton>,
    pub summary_data: ResMut<'w, SummaryData>,
    pub repellent_craft_tracker: ResMut<'w, RepellentCraftTracker>,
    pub mission_end_requested: ResMut<'w, MissionEndRequested>,
    pub query_breach: Query<
        'w,
        's,
        (
            Entity,
            &'static mut Position,
            Option<&'static mut MapEntityFieldBPos>,
        ),
        (
            With<GhostBreach>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<Behavior>,
        ),
    >,
    pub query_orig_pos: Query<
        'w,
        's,
        (
            Entity,
            &'static NetworkOriginalMapPosition,
            Option<&'static GhostInfluence>,
        ),
    >,
    pub query_movable: Query<
        'w,
        's,
        (
            Entity,
            Option<&'static NetworkId>,
            &'static mut Position,
            &'static NetworkOriginalMapPosition,
            Option<&'static mut EquipmentPosition>,
        ),
        (
            With<unbehavior::components::Movable>,
            Without<PlayerSprite>,
            Without<GhostTag>,
            Without<GhostBreach>,
        ),
    >,
}

pub(crate) fn spawn_breach_locally(params: &mut ClientSnapshotParams, snapshot_pos: [f32; 3]) {
    let breach_img_size = Vec2::new(32.0, 64.0);
    let anchor = unghost_core::assets::GHOST_BREACH_ANCHOR;
    let sprite_anchor = Vec2::new(
        breach_img_size.x * (anchor.x + 0.5),
        breach_img_size.y * (0.5 - anchor.y),
    );
    let mesh_handle = params
        .meshes
        .add(Mesh::from(QuadCC::new(breach_img_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(params.ghost_assets.breach.clone());
    material.data.color = Color::BLACK.with_alpha(0.0).into();
    material.data.y_anchor = anchor.y;
    let material_handle = params.materials1.add(material);

    let pos = Position {
        x: snapshot_pos[0],
        y: snapshot_pos[1],
        z: snapshot_pos[2],
        visual_priority: 0.0,
    };

    params
        .commands
        .spawn(Mesh2d(mesh_handle))
        .insert(MeshMaterial2d(material_handle))
        .insert(pos)
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(SpriteLayer(0.01))
        .insert(GhostBreach)
        .insert(LightSensitive {
            exposure_factor: 1.1,
            bias: 0.02,
        })
        .insert(UltravioletSensitive {
            intensity: 1.0,
            color_shift: 1.0,
        })
        .insert(AlphaModulator {
            frequency: 0.92,
            amplitude: 0.5,
        });
}

pub(crate) fn spawn_remote_player(
    params: &mut ClientSnapshotParams,
    id: NetworkId,
    initial_pos: Position,
) -> Entity {
    let mut player_image = params.player_assets.character.clone();
    let mut player_rf = 1.0;
    if let Some(resolved) = params.upscale_idx.resolve(
        "img/characters-model1-demo.png",
        params.video_settings.max_upscale_factor.factor(),
    ) {
        player_image = params.asset_server.load(resolved.path);
        player_rf = resolved.factor;
    }

    let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
    let anchor = unplayer_core::assets::PLAYER_ANCHOR;
    let sprite_anchor = Vec2::new(
        sprite_size.x * (anchor.x + 0.5),
        sprite_size.y * (0.5 - anchor.y),
    );
    let src_mesh_handle = params
        .meshes
        .add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(player_image.clone());
    material.data.sheet_cols = 16;
    material.data.sheet_rows = 4;
    material.data.sprite_width = 32.0 * player_rf;
    material.data.sprite_height = 32.0 * player_rf;
    material.data.upscale_factor = player_rf;
    material.data.y_anchor = anchor.y;

    let material_handle = params.materials1.add(material);

    let mut ec = params.commands.spawn(Mesh2d(src_mesh_handle.clone()));
    ec.insert(MeshMaterial2d(material_handle))
        .insert(Transform::from_xyz(0.0, 0.0, 0.0).with_scale(Vec3::new(
            1.0 / player_rf,
            1.0 / player_rf,
            1.0 / player_rf,
        )))
        .insert(ResolutionFactor(player_rf))
        .insert(GameSprite)
        .insert(unrender_std::components::game::MapTileSprite)
        .insert(SpriteLayer(0.00001));

    ec.insert(PlayerSprite::new(id, initial_pos))
        .insert(id)
        .insert(PlayerInput::default())
        .insert(VisibilityData::default())
        .insert(PlayerTag)
        .insert(ShadowCaster::default())
        .insert(initial_pos)
        .insert(MapEntityFieldBPos(BoardPosition::default()))
        .insert(unbehavior::components::Movable)
        .insert(LightSensitive {
            exposure_factor: 1.1,
            bias: 0.01,
        })
        .insert(unspatial_core::direction::Direction::new_right())
        .insert(AnimationTimer::from_range(
            Timer::from_seconds(0.20, TimerMode::Repeating),
            CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
        ))
        .insert(Stamina::default())
        .insert(unnavigation_core::components::waypoint::WaypointQueue::default())
        .insert(PlayerGear::default())
        .insert(MapColor {
            color: Color::WHITE,
        });

    ec.with_children(|parent| {
        parent
            .spawn(Sprite {
                image: params.ghost_assets.focus_ring_vignette.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            })
            .insert(
                Transform::from_scale(Vec3::splat(1.1 * player_rf))
                    .with_translation(Vec3::new(0.0, 0.1, 0.01)),
            )
            .insert(unrender_std::components::focus_ring::FocusRing::default());
    });

    ec.id()
}

pub(crate) fn spawn_remote_gear(
    params: &mut ClientSnapshotParams,
    g_sync: &GearSyncState,
) -> Entity {
    let entity = params
        .gear_registry
        .spawn(&mut params.commands, g_sync.kind);
    params.commands.entity(entity).insert(g_sync.id);

    // Apply immediate state to avoid race conditions (1-frame delay)
    params.commands.entity(entity).insert(Position {
        x: g_sync.position[0],
        y: g_sync.position[1],
        z: g_sync.position[2],
        visual_priority: 0.0,
    });
    params.commands.entity(entity).insert(Toggleable {
        is_on: g_sync.is_on,
    });
    // Do NOT insert LastSyncedGearState here. Applying it in the next frame
    // via the main sync loop ensures that all gear-specific components
    // (which might have just been added by the registry builder) are
    // correctly updated with the server's data.

    match &g_sync.details {
        GearDetails::Flashlight(status) => {
            params.commands.entity(entity).insert(Flashlight {
                status: status.clone(),
                ..default()
            });
        }
        GearDetails::RepellentFlask {
            qty,
            active,
            liquid_content,
        } => {
            params.commands.entity(entity).insert(
                ungearitems_core::components::repellentflask::RepellentFlask {
                    qty: *qty,
                    active: *active,
                    liquid_content: *liquid_content,
                },
            );
        }
        GearDetails::Thermometer { temp } => {
            params.commands.entity(entity).insert(
                ungearitems_core::components::thermometer::Thermometer {
                    temp: *temp,
                    ..default()
                },
            );
        }
        GearDetails::EMF { level } => {
            params.commands.entity(entity).insert(
                ungearitems_core::components::emfmeter::EMFMeter {
                    emf: *level,
                    emf_level: ungearitems_core::components::emfmeter::EMFLevel::from_milligauss(
                        *level,
                    ),
                    ..default()
                },
            );
        }
        GearDetails::SpiritBox {
            charge,
            ghost_answer,
        } => {
            params.commands.entity(entity).insert(
                ungearitems_core::components::spiritbox::SpiritBox {
                    charge: *charge,
                    ghost_answer: *ghost_answer,
                    ..default()
                },
            );
        }
        _ => {}
    }

    if g_sync.is_deployed {
        params
            .commands
            .entity(entity)
            .insert(DeployedGear {
                direction: Vec2::from_array(g_sync.deployed_direction).into(),
            })
            .insert(unbehavior::components::FloorItemCollidable);
    }
    entity
}

pub(crate) fn client_apply_snapshots_system(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut params: ClientSnapshotParams,
    local_player: Res<unnet_core::resources::LocalPlayer>,
    mut local_tick: Local<u64>,
) {
    let measure = metrics::CLIENT_APPLY_SNAPSHOTS.time_measure();
    if !matches!(params.cli.net_mode, NetMode::Join { .. }) {
        measure.end_ms();
        return;
    }
    *local_tick += 1;
    let mut net_to_entity: std::collections::HashMap<NetworkId, Entity> = params
        .query_net_entities
        .iter()
        .map(|(e, id)| (*id, e))
        .collect();

    for ev in ev_reader.read() {
        if let NetworkMessage::Snapshot(snapshot) = &ev.message {
            let SnapshotMsg {
                tick: _,
                is_full_sync,
                app_state: server_app_state,
                game_state: server_game_state,
                can_end_mission,
                players,
                ghosts,
                rooms,
                map_tiles,
                gear,
                player_gear,
                events,
                evidences_found,
                evidences_missing,
                ghost_type_guess,
                ghosts_discarded,
                mission_result,
                repellent_crafted_count,
                breach_position,
                ghost_type,
                haunted_objects,
                movable_objects,
            } = snapshot.as_ref();

            // Identify MainPlayer and their held items for client-side prediction
            let mut main_player_net_id = None;
            let mut main_player_predictive_entities = std::collections::HashSet::new();
            for (_, id, _, _, _, _, _, gear, main_player, _, _, _, _) in params.query_players.iter()
            {
                if main_player.is_some() {
                    main_player_net_id = Some(*id);
                    if let Some(gear) = gear {
                        if let Some(e) = gear.left_hand {
                            main_player_predictive_entities.insert(e);
                        }
                        if let Some(e) = gear.right_hand {
                            main_player_predictive_entities.insert(e);
                        }
                        for &e in &gear.inventory {
                            main_player_predictive_entities.insert(e);
                        }
                        if let Some(held) = &gear.held_item {
                            main_player_predictive_entities.insert(held.entity);
                        }
                    }
                    break;
                }
            }

            let is_full_sync = *is_full_sync;
            // Sync AppState - but allow independent Summary transition
            let dominated_by_server_app = matches!(
                server_app_state,
                AppState::Loading | AppState::MainMenu | AppState::Summary
            );
            let local_in_summary = *params.states.current_app_state.get() == AppState::Summary;

            if dominated_by_server_app
                || (!local_in_summary
                    && *server_app_state != *params.states.current_app_state.get())
            {
                params.states.app_next_state.set(*server_app_state);

                // If the server forced us into Summary, we must exit any in-game state (like Truck)
                if *server_app_state == AppState::Summary {
                    params.states.game_next_state.set(GameState::None);
                }
            }

            // Sync GameState - but NOT Truck or Pause states (those are per-player local)
            // Only sync if Host is in a "global" state that affects everyone
            let dominated_by_server = matches!(server_game_state, GameState::NpcHelp);
            let local_in_truck = *params.states.current_game_state.get() == GameState::Truck;
            let local_in_pause = *params.states.current_game_state.get() == GameState::Pause;
            let server_in_truck = *server_game_state == GameState::Truck;
            let server_in_pause = *server_game_state == GameState::Pause;

            if dominated_by_server
                || (!local_in_truck
                    && !local_in_pause
                    && !server_in_truck
                    && !server_in_pause
                    && *server_game_state != *params.states.current_game_state.get())
            {
                // Only sync if we're not locally in Truck or Pause
                // This allows Client to stay in Truck/Pause while Host is in None
                params.states.game_next_state.set(*server_game_state);
            }
            // If local_in_truck or local_in_pause is true, we keep our local state

            // Sync MissionEndRequested
            params.mission_end_requested.0 = *can_end_mission;

            // Sync repellent craft count
            params.repellent_craft_tracker.crafted_count = *repellent_crafted_count;

            // Sync breach position
            match breach_position {
                Some(snapshot_pos) => {
                    if let Ok((_entity, mut pos, _o_bpos)) = params.query_breach.single_mut() {
                        pos.x = snapshot_pos[0];
                        pos.y = snapshot_pos[1];
                        pos.z = snapshot_pos[2];
                    } else if !is_full_sync {
                        spawn_breach_locally(&mut params, *snapshot_pos);
                    }
                }
                None => {
                    for (entity, _, _) in params.query_breach.iter() {
                        params.commands.entity(entity).despawn();
                    }
                }
            }

            // Sync ghost type
            if let Some(t) = ghost_type {
                for (_, _, mut ghost, _) in params.query_ghosts.iter_mut() {
                    if ghost.class != *t {
                        debug!(
                            "Client: Correcting ghost type from {:?} to {:?}",
                            ghost.class, t
                        );
                        ghost.class = *t;
                    }
                }
            }

            // Sync haunted objects
            let mut snap_haunted_entities = std::collections::HashSet::new();

            // Build a lookup map for the client's current entities by their original position.
            // This is O(N) where N is total map entities.
            let mut orig_pos_lookup = std::collections::HashMap::new();
            for (entity, orig, _influence) in params.query_orig_pos.iter() {
                orig_pos_lookup.insert(
                    (
                        orig.position.x as i32,
                        orig.position.y as i32,
                        orig.position.z as i32,
                        orig.tileset.clone(),
                        orig.tileuid,
                    ),
                    entity,
                );
            }

            for haunt_sync in haunted_objects {
                let key = (
                    haunt_sync.original_position[0],
                    haunt_sync.original_position[1],
                    haunt_sync.original_position[2],
                    haunt_sync.tileset.clone(),
                    haunt_sync.tileuid,
                );

                if let Some(&entity) = orig_pos_lookup.get(&key) {
                    snap_haunted_entities.insert(entity);
                    let needs_update = match params.query_orig_pos.get(entity) {
                        Ok((_, _, Some(inf))) => inf.influence_type != haunt_sync.influence_type,
                        _ => true,
                    };
                    if needs_update {
                        params.commands.entity(entity).insert((
                            GhostInfluence {
                                influence_type: haunt_sync.influence_type,
                                charge_value: 0.0,
                            },
                            SpectralInfluence::default(),
                        ));
                    }
                }
            }

            // Remove influence from objects that the host says are not haunted
            for (entity, _, influence) in params.query_orig_pos.iter() {
                if influence.is_some() && !snap_haunted_entities.contains(&entity) {
                    params
                        .commands
                        .entity(entity)
                        .remove::<GhostInfluence>()
                        .remove::<SpectralInfluence>();
                }
            }

            // Sync movable objects
            let mut mov_orig_pos_lookup = std::collections::HashMap::new();
            for (entity, _mid, _pos, orig, _eq) in params.query_movable.iter() {
                let key = (
                    orig.position.x as i32,
                    orig.position.y as i32,
                    orig.position.z as i32,
                    orig.tileset.clone(),
                    orig.tileuid,
                );
                mov_orig_pos_lookup.insert(key, entity);
            }

            for mov_sync in movable_objects {
                let key = (
                    mov_sync.original_position[0],
                    mov_sync.original_position[1],
                    mov_sync.original_position[2],
                    mov_sync.tileset.clone(),
                    mov_sync.tileuid,
                );

                if let Some(&entity) = mov_orig_pos_lookup.get(&key)
                    && let Ok((_entity, id, mut pos, _orig, _eq_pos)) =
                        params.query_movable.get_mut(entity)
                {
                    if id.is_none() {
                        params.commands.entity(entity).insert(mov_sync.id);
                    }

                    let is_held_by_main = (main_player_net_id.is_some()
                        && main_player_net_id == mov_sync.held_by)
                        || main_player_predictive_entities.contains(&entity);

                    if !is_held_by_main {
                        pos.x = mov_sync.current_position[0];
                        pos.y = mov_sync.current_position[1];
                        pos.z = mov_sync.current_position[2];
                    }

                    match mov_sync.held_by {
                        Some(_) => {
                            params
                                .commands
                                .entity(entity)
                                .remove::<unbehavior::components::FloorItemCollidable>();
                        }
                        None => {
                            params
                                .commands
                                .entity(entity)
                                .insert(unbehavior::components::FloorItemCollidable);
                        }
                    }
                } else {
                    warn!(
                        "Client: Could not find movable object for sync key {:?}",
                        key
                    );
                }
            }

            let mut seen_ids = std::collections::HashSet::new();
            for p in players {
                seen_ids.insert(p.id);
            }
            for g in gear {
                seen_ids.insert(g.id);
            }
            for g in ghosts {
                seen_ids.insert(g.id);
            }
            for m in movable_objects {
                seen_ids.insert(m.id);
            }

            let tbd_entities: std::collections::HashSet<Entity> = params.query_tbd.iter().collect();

            for (entity, id) in params.query_net_entities.iter() {
                if !seen_ids.contains(id) {
                    let is_main = params
                        .query_players
                        .get(entity)
                        .map(|q| q.8.is_some())
                        .unwrap_or(false);
                    if !is_main && !tbd_entities.contains(&entity) {
                        params
                            .commands
                            .entity(entity)
                            .insert(ToBeDespawned { in_frames: 2 });
                    }
                } else {
                    // If we see it again in a snapshot, stop the countdown
                    params.commands.entity(entity).remove::<ToBeDespawned>();
                }
            }

            // Update players
            for p_state in players {
                let p_entity = if let Some(e) = net_to_entity.get(&p_state.id) {
                    *e
                } else {
                    debug!("Spawning remote player {:?}", p_state.id);
                    let initial_pos = Position {
                        x: p_state.position[0],
                        y: p_state.position[1],
                        z: p_state.position[2],
                        visual_priority: 0.0,
                    };
                    let ent = spawn_remote_player(&mut params, p_state.id, initial_pos);
                    // Insert into net_to_entity immediately to prevent duplicate
                    // spawns when multiple snapshots are processed in the same
                    // frame. The entity was created via Commands (deferred) so it
                    // won't be queryable until next frame, but
                    // spawn_remote_player already applied initial state.
                    net_to_entity.insert(p_state.id, ent);

                    // If this is OUR player, tag it as MainPlayer
                    if local_player.0 == Some(p_state.id) {
                        info!("Tagging spawned player {:?} as MainPlayer", p_state.id);
                        params
                            .commands
                            .entity(ent)
                            .insert(unplayer_core::components::MainPlayer)
                            .insert(unrender_std::components::visuals::Viewer {
                                id: p_state.id,
                                ..default()
                            })
                            .insert(SpatialListener::new(
                                -params.audio_settings.sound_output.to_ear_offset(),
                            ))
                            .insert(unplayer_core::components::PlayerInputMapping {
                                controls: **params.control_settings,
                            });
                    }

                    continue;
                };

                if let Ok((
                    _p_entity,
                    _id,
                    mut pos,
                    mut anim,
                    mut dir,
                    hiding,
                    in_truck,
                    _,
                    main_player,
                    mut stamina,
                    spectating,
                    mut player_sprite,
                    mut player_input,
                )) = params.query_players.get_mut(p_entity)
                {
                    let old_pos = if main_player.is_none() {
                        Some(Vec2::new(pos.x, pos.y))
                    } else {
                        None
                    };

                    if main_player.is_none() || *local_tick < 10 {
                        pos.x = p_state.position[0];
                        pos.y = p_state.position[1];
                        pos.z = p_state.position[2];

                        dir.dx = p_state.orientation[0];
                        dir.dy = p_state.orientation[1];
                    }

                    // Spectating
                    if p_state.is_spectating && spectating.is_none() {
                        params.commands.entity(p_entity).insert(PlayerSpectating);
                    } else if !p_state.is_spectating && spectating.is_some() {
                        params
                            .commands
                            .entity(p_entity)
                            .remove::<PlayerSpectating>();
                    }

                    // 2.4 Hiding
                    if main_player.is_none() {
                        match (p_state.is_hiding, hiding) {
                            (true, None) => {
                                params
                                    .commands
                                    .entity(p_entity)
                                    .insert(Hiding { hiding_spot: None });
                            }
                            (false, Some(_)) => {
                                params.commands.entity(p_entity).remove::<Hiding>();
                            }
                            _ => {}
                        }
                    }

                    // InTruck visuals for remote players
                    if main_player.is_none() {
                        match (p_state.is_in_truck, in_truck) {
                            (true, None) => {
                                params
                                    .commands
                                    .entity(p_entity)
                                    .insert(InTruck)
                                    .insert(Hiding { hiding_spot: None });
                            }
                            (false, Some(_)) => {
                                params
                                    .commands
                                    .entity(p_entity)
                                    .remove::<InTruck>()
                                    .remove::<Hiding>();
                            }
                            _ => {}
                        }
                    }

                    // Sync Health & Sanity
                    // This ensures the client sees damage and sanity drain on the main player
                    player_sprite.health = p_state.health;
                    player_sprite.sanity = p_state.sanity;
                    if main_player.is_none() {
                        player_input.target_position =
                            p_state.target_position.map(|t| Vec2::new(t[0], t[1]));
                    }

                    if main_player.is_none() {
                        stamina.current = p_state.stamina;
                        stamina.running = p_state.is_running;
                        // No need to sync frame directly as it causes jitter with local animation timer.
                        // The range and stamina.running are enough for local reproduction.
                    }

                    if main_player.is_none() {
                        let animation_speed_factor = if p_state.is_running { 1.5 } else { 1.0 };
                        if let Some(old_pos) = old_pos {
                            let velocity = Vec2::new(pos.x - old_pos.x, pos.y - old_pos.y);
                            if velocity.length_squared() > 0.00001 {
                                let dscreen = perspective::direction_to_screen_coord(Direction {
                                    dx: velocity.x,
                                    dy: velocity.y,
                                    dz: 0.0,
                                });
                                anim.set_range(
                                    CharacterAnimation::from_dir(
                                        dscreen.x * 60.0 * animation_speed_factor,
                                        dscreen.y * 120.0 * animation_speed_factor,
                                    )
                                    .to_vec(),
                                );
                            } else {
                                let dscreen = perspective::direction_to_screen_coord(*dir)
                                    .normalize_or_zero();
                                anim.set_range(
                                    CharacterAnimation::from_dir(dscreen.x * 0.5, dscreen.y * 0.5)
                                        .to_vec(),
                                );
                            }
                        }
                    }
                }
            }

            // Update gear
            for g_sync in gear {
                let g_entity = if let Some(e) = net_to_entity.get(&g_sync.id) {
                    *e
                } else {
                    debug!(
                        "Spawning remote gear {:?} (kind: {:?})",
                        g_sync.id, g_sync.kind
                    );
                    let ent = spawn_remote_gear(&mut params, g_sync);
                    // Insert into net_to_entity immediately to prevent duplicate
                    // spawns when multiple snapshots are processed in the same
                    // frame. The entity was created via Commands (deferred) so it
                    // won't be queryable until next frame, but
                    // spawn_remote_gear already applied initial state from
                    // g_sync.
                    net_to_entity.insert(g_sync.id, ent);
                    continue;
                };

                if let Ok((
                    g_entity,
                    _,
                    mut pos,
                    toggle,
                    deployed,
                    battery,
                    flashlight,
                    sage,
                    repellent,
                    thermometer,
                    emf_meter,
                    spiritbox,
                    mut last_synced,
                )) = params.query_gear.get_mut(g_entity)
                {
                    let is_owned_by_main_player =
                        main_player_predictive_entities.contains(&g_entity);

                    if !is_owned_by_main_player {
                        pos.x = g_sync.position[0];
                        pos.y = g_sync.position[1];
                        pos.z = g_sync.position[2];
                    }

                    let details_changed = if let Some(ref last) = last_synced {
                        last.details != g_sync.details
                    } else {
                        true
                    };
                    let is_on_changed = if let Some(ref last) = last_synced {
                        last.is_on != g_sync.is_on
                    } else {
                        true
                    };

                    if is_on_changed || details_changed {
                        if !is_owned_by_main_player && let Some(mut t) = toggle {
                            t.is_on = g_sync.is_on;
                        }
                        if let Some(ref mut last) = last_synced {
                            last.is_on = g_sync.is_on;
                        }
                    }

                    match (g_sync.is_deployed, deployed) {
                        (true, None) => {
                            params
                                .commands
                                .entity(g_entity)
                                .insert(DeployedGear {
                                    direction: Vec2::from_array(g_sync.deployed_direction).into(),
                                })
                                .insert(unbehavior::components::FloorItemCollidable);
                        }
                        (true, Some(mut d)) => {
                            d.direction = Vec2::from_array(g_sync.deployed_direction).into();
                        }
                        (false, Some(_)) => {
                            params
                                .commands
                                .entity(g_entity)
                                .remove::<DeployedGear>()
                                .remove::<unbehavior::components::FloorItemCollidable>()
                                .remove::<Sprite>()
                                .remove::<Transform>()
                                .remove::<Visibility>()
                                .remove::<unrender_std::components::game::GameSprite>()
                                .remove::<unrender_std::components::sprite_layer::SpriteLayer>()
                                .remove::<MapColor>();
                        }
                        _ => {}
                    }

                    if let Some(mut b) = battery {
                        b.level = g_sync.battery;
                    }

                    if is_on_changed || details_changed {
                        match &g_sync.details {
                            unnet_core::messages::GearDetails::Flashlight(status) => {
                                if !is_owned_by_main_player && let Some(mut f) = flashlight {
                                    f.status = status.clone();
                                }
                            }
                            unnet_core::messages::GearDetails::Sage {
                                consumed,
                                is_active,
                                remaining_secs,
                            } => {
                                if let Some(mut s) = sage {
                                    s.consumed = *consumed;
                                    s.is_active = *is_active;

                                    let elapsed =
                                        s.burn_timer.duration().as_secs_f32() - remaining_secs;
                                    s.burn_timer.set_elapsed(std::time::Duration::from_secs_f32(
                                        elapsed.max(0.0),
                                    ));
                                }
                            }
                            unnet_core::messages::GearDetails::RepellentFlask {
                                qty,
                                active,
                                liquid_content,
                            } => {
                                if let Some(mut r) = repellent {
                                    r.qty = *qty;
                                    r.active = *active;
                                    r.liquid_content = *liquid_content;
                                }
                            }
                            unnet_core::messages::GearDetails::Thermometer { temp } => {
                                if let Some(mut t) = thermometer {
                                    t.temp = *temp;
                                }
                            }
                            unnet_core::messages::GearDetails::EMF { level } => {
                                if let Some(mut e) = emf_meter {
                                    e.emf = *level;
                                    e.emf_level =
                                    ungearitems_core::components::emfmeter::EMFLevel::from_milligauss(
                                        e.emf,
                                    );
                                }
                            }
                            unnet_core::messages::GearDetails::SpiritBox {
                                charge,
                                ghost_answer,
                            } => {
                                if let Some(mut s) = spiritbox {
                                    s.charge = *charge;
                                    s.ghost_answer = *ghost_answer;
                                }
                            }
                            unnet_core::messages::GearDetails::None => {}
                        }
                        if let Some(ref mut last) = last_synced {
                            last.details = g_sync.details.clone();
                        } else {
                            params
                                .commands
                                .entity(g_entity)
                                .insert(LastSyncedGearState {
                                    is_on: g_sync.is_on,
                                    details: g_sync.details.clone(),
                                });
                        }
                    }
                }
            }

            // Update player gear
            for pg_state in player_gear {
                for (_, id, _, _, _, _, _, gear, _, _, _, _, _) in params.query_players.iter_mut() {
                    if *id == pg_state.player_id
                        && let Some(mut gear) = gear
                    {
                        let old_left = gear.left_hand;
                        let old_right = gear.right_hand;
                        let old_held = gear.held_item.as_ref().map(|h| h.entity);

                        gear.left_hand = pg_state
                            .left_hand
                            .and_then(|nid| net_to_entity.get(&nid))
                            .cloned();
                        gear.right_hand = pg_state
                            .right_hand
                            .and_then(|nid| net_to_entity.get(&nid))
                            .cloned();
                        gear.inventory = pg_state
                            .inventory
                            .iter()
                            .filter_map(|nid| net_to_entity.get(nid))
                            .cloned()
                            .collect();
                        gear.held_item = pg_state
                            .held_item
                            .and_then(|nid| net_to_entity.get(&nid))
                            .map(|&entity| ungear_core::components::playergear::HeldObject {
                                entity,
                            });
                        let new_held = gear.held_item.as_ref().map(|h| h.entity);

                        // Derivation: Update EquipmentPosition on referenced gear
                        if let Some(e) = gear.left_hand {
                            params.commands.entity(e).insert(EquipmentPosition::Hand(
                                unfoundation_core::types::gear::Hand::Left,
                            ));
                        }
                        if let Some(e) = gear.right_hand {
                            params.commands.entity(e).insert(EquipmentPosition::Hand(
                                unfoundation_core::types::gear::Hand::Right,
                            ));
                        }
                        for e in &gear.inventory {
                            params.commands.entity(*e).insert(EquipmentPosition::Stowed);
                        }

                        if old_left != gear.left_hand
                            || old_right != gear.right_hand
                            || old_held != new_held
                        {
                            debug!(
                                "Player {:?} gear state: left={:?}, right={:?}, inv_count={}",
                                id,
                                gear.left_hand,
                                gear.right_hand,
                                gear.inventory.len()
                            );
                        }
                    }
                }
            }

            // Update ghosts
            for g_state in ghosts {
                for (id, mut pos, mut ghost, mut dynamics) in params.query_ghosts.iter_mut() {
                    if *id == g_state.id {
                        pos.x = g_state.position[0];
                        pos.y = g_state.position[1];
                        pos.z = g_state.position[2];
                        ghost.warp = g_state.warp;
                        ghost.hunt_warning_active = g_state.hunt_warning_active;
                        ghost.hunt_warning_intensity = g_state.hunt_warning_intensity;
                        ghost.hunt_target = g_state.hunt_target;
                        ghost.calm_time_secs = g_state.calm_time_secs;
                        ghost.repellent_hits_delta = g_state.repellent_hits_delta;
                        ghost.repellent_misses_delta = g_state.repellent_misses_delta;
                        ghost.repellent_hits = g_state.repellent_hits;
                        ghost.class = g_state.class;

                        dynamics.freezing_temp_clarity = g_state.freezing_temp_clarity;
                        dynamics.floating_orbs_clarity = g_state.floating_orbs_clarity;
                        dynamics.uv_ectoplasm_clarity = g_state.uv_ectoplasm_clarity;
                        dynamics.emf_level5_clarity = g_state.emf_level5_clarity;
                        dynamics.evp_recording_clarity = g_state.evp_recording_clarity;
                        dynamics.spirit_box_clarity = g_state.spirit_box_clarity;
                        dynamics.rl_presence_clarity = g_state.rl_presence_clarity;
                        dynamics.cpm500_clarity = g_state.cpm500_clarity;
                        dynamics.visual_alpha_multiplier = g_state.visual_alpha_multiplier;
                        dynamics.rage_tendency_multiplier = g_state.rage_tendency_multiplier;
                    }
                }
            }

            // Update rooms
            let mut room_changed = false;
            for r_sync in rooms {
                let new_state = if r_sync.state == 1 {
                    TileState::On
                } else {
                    TileState::Off
                };
                if let Some(state) = params
                    .room_db
                    .room_state
                    .get_mut(&r_sync.name)
                    .filter(|s| **s != new_state)
                {
                    *state = new_state;
                    room_changed = true;
                }
            }
            if room_changed {
                params.ev_room_sync.write(RoomStateSyncEvent);
            }

            // Update map tiles
            for t_sync in map_tiles {
                let bpos = BoardPosition {
                    x: t_sync.x as i64,
                    y: t_sync.y as i64,
                    z: t_sync.z as i64,
                };
                let rel_x = bpos.x;
                let rel_y = bpos.y;
                let rel_z = bpos.z;
                let shape = params.board_field.0.shape();

                if rel_x >= 0
                    && rel_y >= 0
                    && rel_z >= 0
                    && rel_x < shape[0] as i64
                    && rel_y < shape[1] as i64
                    && rel_z < shape[2] as i64
                {
                    let entities =
                        &params.board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
                    if entities.is_empty() {
                        debug!(
                            "Client: No entities found at board position {:?} (Rel: {:?}, Board shape: {:?}, Origin: {:?})",
                            bpos,
                            (rel_x, rel_y, rel_z),
                            shape,
                            params.board_topo.origin
                        );
                    }
                    for &entity in entities {
                        if params
                            .query_tiles
                            .get(entity)
                            .ok()
                            .filter(|beh| {
                                // Filter: Is this the correct sprite?
                                let key_cvo = beh.key_cvo().to_key_string();
                                if key_cvo != t_sync.cvo_key {
                                    return false;
                                }
                                // Filter: Has this sprite changed? Does it require a change?
                                beh.cfg().tileset != t_sync.tileset
                                    || beh.cfg().tileuid != t_sync.tileuid
                            })
                            .is_some()
                        {
                            debug!(
                                "Client: Applying map tile update at {:?} (tileset: {}, tileuid: {})",
                                bpos, t_sync.tileset, t_sync.tileuid
                            );
                            params.ev_interaction.write(ExecuteInteractionEvent {
                                entity,
                                ietype: InteractionExecutionType::ChangeState,
                                force_tuid: Some(t_sync.tileuid),
                            });
                        }
                    }
                } else {
                    debug!(
                        "Client: Map tile update out of bounds: {:?} (Rel: {:?}, Board shape: {:?}, Origin: {:?})",
                        bpos,
                        (rel_x, rel_y, rel_z),
                        shape,
                        params.board_topo.origin
                    );
                }
            }

            // Update events
            for event in events {
                match event {
                    TransientEvent::PlaySound {
                        sound_file,
                        volume,
                        position,
                    } => {
                        params.ev_sound.write(SoundEvent {
                            sound_file: sound_file.clone(),
                            volume: *volume,
                            position: position.map(|p| Position {
                                x: p[0],
                                y: p[1],
                                z: p[2],
                                visual_priority: 0.0,
                            }),
                            broadcast: false,
                        });
                    }
                    TransientEvent::SpawnParticle {
                        particle_type,
                        position,
                    } => {
                        if particle_type == "smoke" {
                            let mut rng = unfoundation_core::random_seed::rng();
                            let pos = Position {
                                x: position[0],
                                y: position[1],
                                z: position[2],
                                visual_priority: 0.0,
                            };
                            params
                                .commands
                                .spawn(Sprite {
                                    image: params.asset_server.load("img/smoke.png"),
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
                                .insert(unspatial_core::direction::Direction {
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
            }

            // Sync GhostGuess
            let ghost_guess_changed = is_full_sync
                || params.ghost_guess.ghost_type != *ghost_type_guess
                || params.ghost_guess.evidences_found.len() != evidences_found.len()
                || params.ghost_guess.evidences_missing.len() != evidences_missing.len()
                || params.ghost_guess.ghosts_discarded.len() != ghosts_discarded.len()
                || !evidences_found
                    .iter()
                    .all(|e| params.ghost_guess.evidences_found.contains(e))
                || !evidences_missing
                    .iter()
                    .all(|e| params.ghost_guess.evidences_missing.contains(e))
                || !ghosts_discarded
                    .iter()
                    .all(|g| params.ghost_guess.ghosts_discarded.contains(g));

            if ghost_guess_changed {
                params.ghost_guess.ghost_type = *ghost_type_guess;
                params.ghost_guess.evidences_found = evidences_found.iter().cloned().collect();
                params.ghost_guess.evidences_missing = evidences_missing.iter().cloned().collect();
                params.ghost_guess.ghosts_discarded = ghosts_discarded.iter().cloned().collect();

                // Sync TruckUIButtons
                for mut button in params.query_buttons.iter_mut() {
                    match button.class {
                        TruckButtonType::Evidence(e) => {
                            if params.ghost_guess.evidences_found.contains(&e) {
                                button.status = TruckButtonState::Pressed;
                            } else if params.ghost_guess.evidences_missing.contains(&e) {
                                button.status = TruckButtonState::Discard;
                            } else {
                                button.status = TruckButtonState::Off;
                            }
                        }
                        TruckButtonType::Ghost(gt) => {
                            if params.ghost_guess.ghost_type == Some(gt) {
                                button.status = TruckButtonState::Pressed;
                            } else {
                                button.status = TruckButtonState::Off;
                            }
                        }
                        _ => {}
                    }
                }
            }

            // Sync Mission Result
            if let Some(res) = &**mission_result {
                params.summary_data.time_taken_secs = res.time_taken_secs;
                params.summary_data.ghost_types = res.ghost_types.clone();
                params.summary_data.repellent_used_amt = res.repellent_used_amt;
                params.summary_data.ghosts_unhaunted = res.ghosts_unhaunted;
                params.summary_data.base_score = res.base_score;
                params.summary_data.difficulty_multiplier = res.difficulty_multiplier;
                params.summary_data.grade_multiplier = res.grade_multiplier;
                params.summary_data.average_sanity = res.average_sanity;
                params.summary_data.player_count = res.player_count as usize;
                params.summary_data.alive_count = res.alive_count as usize;
                params.summary_data.full_score = res.full_score;
                params.summary_data.mission_successful = res.mission_successful;
                params.summary_data.money_earned = res.money_earned;
                params.summary_data.grade_achieved = res.grade_achieved;
                params.summary_data.required_deposit = res.required_deposit;
                params.summary_data.mission_reward_base = res.mission_reward_base;
                params.summary_data.deposit_originally_held = res.deposit_originally_held;
                params.summary_data.deposit_returned_to_bank = res.deposit_returned_to_bank;
                params.summary_data.costs_deducted_from_deposit = res.costs_deducted_from_deposit;
            }
        } else if let NetworkMessage::MissionSummary { result } = &ev.message {
            params.summary_data.time_taken_secs = result.time_taken_secs;
            params.summary_data.ghost_types = result.ghost_types.clone();
            params.summary_data.repellent_used_amt = result.repellent_used_amt;
            params.summary_data.ghosts_unhaunted = result.ghosts_unhaunted;
            params.summary_data.base_score = result.base_score;
            params.summary_data.difficulty_multiplier = result.difficulty_multiplier;
            params.summary_data.grade_multiplier = result.grade_multiplier;
            params.summary_data.average_sanity = result.average_sanity;
            params.summary_data.player_count = result.player_count as usize;
            params.summary_data.alive_count = result.alive_count as usize;
            params.summary_data.full_score = result.full_score;
            params.summary_data.mission_successful = result.mission_successful;
            params.summary_data.money_earned = result.money_earned;
            params.summary_data.grade_achieved = result.grade_achieved;
            params.summary_data.required_deposit = result.required_deposit;
            params.summary_data.mission_reward_base = result.mission_reward_base;
            params.summary_data.deposit_originally_held = result.deposit_originally_held;
            params.summary_data.deposit_returned_to_bank = result.deposit_returned_to_bank;
            params.summary_data.costs_deducted_from_deposit = result.costs_deducted_from_deposit;

            // Force state transition to Summary, as Host stops sending snapshots once it enters Summary
            params.states.app_next_state.set(AppState::Summary);
            params.states.game_next_state.set(GameState::None);
        }
    }
    measure.end_ms();
}

pub(crate) fn delayed_despawn_system(
    mut commands: Commands,
    mut query: Query<(Entity, &mut ToBeDespawned)>,
) {
    for (entity, mut tbd) in query.iter_mut() {
        if tbd.in_frames == 0 {
            commands.entity(entity).despawn();
        } else {
            tbd.in_frames -= 1;
        }
    }
}
