use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::prelude::IndexedRandom;
use unboard_core::components::spawning::{HostileSpawnPoint, PlayerSpawnPoint, VanEntryPoint};
use unboard_core::resources::board_topology::BoardTopology;
use unboard_core::resources::roomdb::RoomTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unghost_core::difficulty_ext::DifficultyGhostExt;
use unfoundation_core::random_seed;
use unfoundation_core::types::sound::SoundType;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::haunt_state::HauntState;
use unmapload_core::events::loadlevel::{LevelReadyEvent, MapEntitiesReadyEvent};
use unplayer_core::components::PlayerSprite;
use unrender_std::components::game::GameSound;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::GhostTag;
use unbehavior::components::Movable;
use unboard_core::components::physics::{FluidEmitter, SoundEmitter, ThermalEmitter};
use unrender_std::components::visuals::{InfraredSensitive, LightSensitive, UltravioletSensitive};
use unreplicon_core::network_id::NetworkId;

#[derive(SystemParam)]
pub(crate) struct OrchestratorParam<'w> {
    pub local_player_role: Option<Res<'w, untypes_core::roles::LocalPlayerRole>>,
    pub authority_role: Option<Res<'w, untypes_core::roles::AuthorityRole>>,
    pub asset_server: Res<'w, AssetServer>,
    pub haunt_state: ResMut<'w, HauntState>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub board_topology: Res<'w, BoardTopology>,
    pub room_topology: Res<'w, RoomTopology>,
}

pub(crate) fn classic_mode_orchestrator(
    p: OrchestratorParam,
    mut commands: Commands,
    mut ev_level_ready: MessageWriter<LevelReadyEvent>,
    mut ev_entities_ready: MessageReader<MapEntitiesReadyEvent>,
    q_ghost_breach: Query<&Position, With<GhostBreach>>,
    q_player_sprite: Query<&Position, With<PlayerSprite>>,
    q_position: Query<&Position>,
    q_player_spawns: Query<&Position, With<PlayerSpawnPoint>>,
    q_ghost_spawns: Query<&Position, With<HostileSpawnPoint>>,
    q_van_entry: Query<&Position, With<VanEntryPoint>>,
    q_movable: Query<Entity, With<Movable>>,
) {
    let Some(_) = ev_entities_ready.read().next() else {
        return;
    };

    let player_spawn_points: Vec<Position> = q_player_spawns.iter().copied().collect();
    let ghost_spawn_points: Vec<Position> = q_ghost_spawns.iter().copied().collect();
    let van_entry_points: Vec<Position> = q_van_entry.iter().copied().collect();
    let movable_objects: Vec<Entity> = q_movable.iter().collect();

    if player_spawn_points.is_empty() {
        error!("No player spawn points found!!");
        return;
    }

    // --- Determine Player/Van Position ---
    let mut rng = random_seed::rng();
    let player_position = player_spawn_points.choose(&mut rng).copied().unwrap();

    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

    let open_van = dist_to_van < 8.0 && p.difficulty.0.van_auto_open;

    // Join clients do not spawn the ghost locally; they receive the replicated entity
    // from the server and set up its visuals via hydrate_ghosts_system.
    if p.authority_role.is_some() {
        // --- Spawn Ghost ---
        {
            let ghost_spawn = ghost_spawn_points
                .choose(&mut rng)
                .copied()
                .unwrap_or(Position::new_i64(0, 0, 0));

            let possible_ghost_types: Vec<_> = p.difficulty.0.difficulty.ghost_set().as_vec();
            let ghost_sprite =
                GhostSprite::new(ghost_spawn.to_board_position(), &possible_ghost_types);
            let ghost_types = vec![ghost_sprite.class];

            commands.insert_resource(SummaryData::new(ghost_types, p.difficulty.clone()));

            let breach_id = {
                let mut ec = commands.spawn(ghost_spawn);

                ec.insert(GhostBreach)
                    .insert(GameSound {
                        class: SoundType::BackgroundHouse,
                    })
                    .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                        ghost_spawn.to_board_position(),
                    ))
                    .insert(LightSensitive {
                        exposure_factor: 1.1,
                        bias: 0.02,
                    })
                    .insert(UltravioletSensitive {
                        intensity: 1.0,
                        color_shift: 1.0,
                    })
                    .insert(ThermalEmitter {
                        room_restricted: true,
                        ..default()
                    })
                    .insert(FluidEmitter::default())
                    .insert(SoundEmitter::default());

                ec.id()
            };

            let ghost_id_net = NetworkId(0); // Ghost is always 0 in MVP
            let mut ec = commands.spawn(ghost_spawn);

            ec.insert(ghost_sprite.with_breachid(breach_id))
                .insert(p.haunt_state.ghost_dynamics)
                .insert(GhostTag)
                .insert(ghost_id_net)
                .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                    ghost_spawn.to_board_position(),
                ))
                .insert(Movable)
                .insert(LightSensitive {
                    exposure_factor: 0.5,
                    bias: 0.01,
                })
                .insert(UltravioletSensitive {
                    intensity: 1.0,
                    ..default()
                })
                .insert(InfraredSensitive {
                    intensity: 1.0,
                    ..default()
                })
                .insert(ThermalEmitter {
                    room_restricted: true,
                    ..default()
                })
                .insert(FluidEmitter::default())
                .insert(SoundEmitter::default());
            let _ghost_id = ec.id();

            if p.local_player_role.is_some() {
                spawn_ambient_sounds(&p, &mut commands);
            }

            crate::influence_system::assign_ghost_influence(
                &mut commands,
                &movable_objects,
                &q_ghost_breach,
                &q_player_sprite,
                &q_position,
                &p.room_topology,
                &p.board_topology,
                &p.haunt_state,
            );
        }
    } // end if !Join

    ev_level_ready.write(LevelReadyEvent { open_van });
}

fn spawn_ambient_sounds(p: &OrchestratorParam, commands: &mut Commands) {
    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/background-noise-house-1.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
            ..default()
        })
        .insert(GameSound {
            class: SoundType::BackgroundHouse,
        });

    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/ambient-clean.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
            ..default()
        })
        .insert(GameSound {
            class: SoundType::BackgroundStreet,
        });

    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/heartbeat-1.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
            ..default()
        })
        .insert(GameSound {
            class: SoundType::HeartBeat,
        });

    commands
        .spawn(AudioPlayer::new(p.asset_server.load("sounds/insane-1.ogg")))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::Linear(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
            ..default()
        })
        .insert(GameSound {
            class: SoundType::Insane,
        });
}
