use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use ordered_float::OrderedFloat;
use rand::prelude::IndexedRandom;
use unassets_core::resources::upscale::UpscaleIndex;
use unbehavior::components::Movable;
use unbehavior::roomdb::RoomDB;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::components::physics::{FluidEmitter, SoundEmitter, ThermalEmitter};
use unboard_core::components::spawning::{HostileSpawnPoint, PlayerSpawnPoint, VanEntryPoint};
use unboard_core::resources::board_topology::BoardTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::random_seed;
use unfoundation_core::types::sound::SoundType;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::GhostBehaviorDynamics;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::haunt_state::HauntState;
use unmapload_core::events::loadlevel::{LevelReadyEvent, MapEntitiesReadyEvent};
use unnet_core::network_id::NetworkId;
use unnet_core::resources::{LobbyData, LocalPlayer};
use unplayer_core::components::PlayerDisconnected;
use unplayer_core::components::{
    MainPlayer, PlayerInput, PlayerInputMapping, PlayerSprite, Stamina,
};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::focus_ring::FocusRing;
use unrender_std::components::game::{GameSound, GameSprite, MapTileSprite};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{
    AlphaModulator, EctoplasmVisuals, Emissive, Ethereal, InfraredSensitive, LightSensitive,
    ResolutionFactor, ShadowCaster, SpectralClarity, UltravioletSensitive, Viewer,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::quadcc::QuadCC;
use unsettings_core::video::VideoSettings;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::{GhostTag, PlayerTag};
use unreplicon_core::net_components::{NetworkPosition, PlayerNetInfo};

#[derive(SystemParam)]
pub(crate) struct ClassicModeSystemParam<'w> {
    pub cli: Res<'w, untypes_core::cli::CliOptions>,
    pub asset_server: Res<'w, AssetServer>,
    pub haunt_state: ResMut<'w, HauntState>,
    pub player_assets: Option<Res<'w, unplayer_core::assets::PlayerAssets>>,
    pub ghost_assets: Option<Res<'w, unghost_core::assets::GhostAssets>>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Option<Res<'w, Persistent<VideoSettings>>>,
    pub materials1: Option<ResMut<'w, Assets<unrender_std::materials::CustomMaterial1>>>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub images: Option<Res<'w, Assets<Image>>>,
    pub audio_settings: Option<Res<'w, Persistent<unsettings_core::audio::AudioSettings>>>,
    pub control_settings: Option<Res<'w, Persistent<unsettings_core::controls::ControlKeys>>>,
    pub board_topology: Res<'w, BoardTopology>,
    pub board_entity_field: ResMut<'w, unboard_core::resources::board_topology::BoardEntityField>,
    pub roomdb: Res<'w, RoomDB>,
}

pub(crate) fn classic_mode_orchestrator(
    mut p: ClassicModeSystemParam,
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

    // --- Spawn Player ---
    let mut rng = random_seed::rng();
    let player_position = player_spawn_points.choose(&mut rng).copied().unwrap();

    let mut player_image = p
        .player_assets
        .as_ref()
        .map(|a| a.character.clone())
        .unwrap_or_default();
    let mut player_rf = 1.0;

    if !p.cli.is_headless()
        && let Some(video_settings) = &p.video_settings
        && let Some(resolved) = p.upscale_idx.resolve(
            "img/characters-model1-demo.png",
            video_settings.max_upscale_factor.factor(),
        )
    {
        player_image = p.asset_server.load(resolved.path);
        player_rf = resolved.factor;
    }

    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

    let open_van = dist_to_van < 8.0 && p.difficulty.0.van_auto_open;

    let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
    let anchor = unplayer_core::assets::PLAYER_ANCHOR;
    let sprite_anchor = Vec2::new(
        sprite_size.x * (anchor.x + 0.5),
        sprite_size.y * (0.5 - anchor.y),
    );

    let mut src_mesh_handle = Handle::default();
    if let Some(meshes) = &mut p.meshes {
        src_mesh_handle = meshes.add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));
    }

    let player_ids_to_spawn: Vec<usize> = match p.cli.net_mode {
        untypes_core::cli::NetMode::Offline => vec![1],
        untypes_core::cli::NetMode::Host { .. } => {
            if p.cli.is_headless() {
                vec![]
            } else {
                vec![1]
            }
        }
        untypes_core::cli::NetMode::Join { .. } => vec![],
    };

    for (idx, id) in player_ids_to_spawn.into_iter().enumerate() {
        // In the new model:
        // - Offline: always the main player (only player).
        // - Host: spawns only id=1, which is always the main player.
        // - Join: spawns nothing (vec is empty), so this loop body never runs.
        let is_main_player = true;

        let mut player_gear = PlayerGear::default();
        if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Join { .. }) {
            player_gear =
                spawn_initial_gear(&mut commands, &p.gear_registry, &p.difficulty, id as u64);
        }

        // Pick a spawn point for this player. Use index-based selection to avoid spawning on top of each other.
        let spawn_pos = player_spawn_points
            .get(idx % player_spawn_points.len())
            .copied()
            .unwrap_or(player_position);

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut ec = commands.spawn(spawn_pos);

        if !p.cli.is_headless() {
            let mut material = CustomMaterial1::from_texture(player_image.clone());
            material.data.sheet_cols = 16;
            material.data.sheet_rows = 4;
            material.data.sprite_width = 32.0 * player_rf;
            material.data.sprite_height = 32.0 * player_rf;
            material.data.upscale_factor = player_rf;
            material.data.y_anchor = anchor.y;

            if let Some(materials1) = &mut p.materials1 {
                let material_handle = materials1.add(material);

                ec.insert(Mesh2d(src_mesh_handle.clone()))
                    .insert(MeshMaterial2d(material_handle))
                    .insert(
                        Transform::from_xyz(spawn_scoord[0], spawn_scoord[1], spawn_scoord[2])
                            .with_scale(Vec3::new(
                                1.0 / player_rf,
                                1.0 / player_rf,
                                1.0 / player_rf,
                            )),
                    )
                    .insert(ResolutionFactor(player_rf))
                    .insert(MapTileSprite)
                    .insert(SpriteLayer(0.00001));
            }
        }

        let id_net = NetworkId(id as u64);

        ec.insert(PlayerSprite::new(id_net, spawn_pos))
            .insert(id_net)
            .insert(GameSprite)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(PlayerInput::default())
            .insert(VisibilityData::default())
            .insert(PlayerTag)
            .insert(ShadowCaster::default())
            .insert(spawn_pos)
            .insert(MapEntityFieldBPos(spawn_pos.to_board_position()))
            .insert(Movable)
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(Direction::new_right())
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ))
            .insert(Stamina::default())
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default());

        if let Some(control_settings) = &p.control_settings {
            ec.insert(PlayerInputMapping {
                controls: ***control_settings,
            });
        }

        if is_main_player {
            ec.insert(MainPlayer).insert(Viewer {
                id: id_net,
                ..default()
            });

            if let Some(audio_settings) = &p.audio_settings {
                ec.insert(SpatialListener::new(
                    -audio_settings.sound_output.to_ear_offset(),
                ));
            }
        }
        ec.insert(player_gear);

        if !p.cli.is_headless()
            && let Some(ghost_assets) = &p.ghost_assets
        {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(1.1 * player_rf))
                            .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }
        let player_ent_id = ec.id();

        p.board_entity_field.0[player_position.to_board_position().ndidx()].push(player_ent_id);
    }

    // --- Spawn Ghost ---
    {
        let ghost_spawn = ghost_spawn_points
            .choose(&mut rng)
            .copied()
            .unwrap_or(Position::new_i64(0, 0, 0));

        let possible_ghost_types: Vec<_> = p.difficulty.0.ghost_set.as_vec();
        let ghost_sprite = GhostSprite::new(ghost_spawn.to_board_position(), &possible_ghost_types);
        let ghost_types = vec![ghost_sprite.class];

        commands.insert_resource(SummaryData::new(ghost_types, p.difficulty.clone()));

        let breach_id = {
            let mut breach_img_size = Vec2::new(128.0, 128.0);
            if let Some(ghost_assets) = &p.ghost_assets
                && let Some(images) = &p.images
                && let Some(img) = images.get(ghost_assets.breach.id())
            {
                breach_img_size = Vec2::new(
                    img.texture_descriptor.size.width as f32,
                    img.texture_descriptor.size.height as f32,
                );
            }

            let anchor = unmapload_core::assets::GRID_1X1X4_ANCHOR;
            let sprite_anchor = Vec2::new(
                breach_img_size.x * (anchor.x + 0.5),
                breach_img_size.y * (0.5 - anchor.y),
            );

            let mut ec = commands.spawn(ghost_spawn);
            if !p.cli.is_headless()
                && let (Some(meshes), Some(materials1), Some(ghost_assets)) =
                    (&mut p.meshes, &mut p.materials1, &p.ghost_assets)
            {
                let mesh_handle =
                    meshes.add(Mesh::from(QuadCC::new(breach_img_size, sprite_anchor)));
                let mut material = CustomMaterial1::from_texture(ghost_assets.breach.clone());
                material.data.color = Color::NONE.into();
                material.data.y_anchor = anchor.y;
                let material_handle = materials1.add(material);

                ec.insert(Mesh2d(mesh_handle))
                    .insert(MeshMaterial2d(material_handle))
                    .insert(Transform::from_xyz(-1000.0, -1000.0, -1000.0))
                    .insert(MapTileSprite)
                    .insert(SpriteLayer(0.01))
                    .insert(AlphaModulator {
                        frequency: 0.92,
                        amplitude: 0.5,
                    })
                    .insert(EctoplasmVisuals {
                        use_breach_curve: true,
                    });
            }

            ec.insert(GhostBreach)
                .insert(GameSprite)
                .insert(MapEntityFieldBPos(ghost_spawn.to_board_position()))
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

            if !p.cli.is_headless()
                && let Some(ghost_assets) = &p.ghost_assets
            {
                ec.with_children(|parent| {
                    parent
                        .spawn(Sprite {
                            image: ghost_assets.focus_ring_vignette.clone(),
                            color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                            ..default()
                        })
                        .insert(
                            Transform::from_scale(Vec3::splat(0.5))
                                .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                        )
                        .insert(FocusRing::default());
                });
            }
            let breach_id = ec.id();

            p.board_entity_field.0[ghost_spawn.to_board_position().ndidx()].push(breach_id);
            breach_id
        };

        let mut ghost_image = p
            .ghost_assets
            .as_ref()
            .map(|a| a.ghost.clone())
            .unwrap_or_default();
        let mut ghost_rf = 1.0;

        if !p.cli.is_headless()
            && let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p
                .upscale_idx
                .resolve("img/ghost.png", video_settings.max_upscale_factor.factor())
        {
            ghost_image = p.asset_server.load(resolved.path);
            ghost_rf = resolved.factor;
        }

        let mut ghost_img_size = Vec2::new(128.0, 128.0);
        if let Some(images) = &p.images
            && let Some(img) = images.get(ghost_image.id())
        {
            ghost_img_size = Vec2::new(
                img.texture_descriptor.size.width as f32,
                img.texture_descriptor.size.height as f32,
            );
        }

        let anchor = unmapload_core::assets::GRID_1X1X4_ANCHOR;
        let sprite_anchor = Vec2::new(
            ghost_img_size.x * (anchor.x + 0.5),
            ghost_img_size.y * (0.5 - anchor.y),
        );

        let ghost_id_net = NetworkId(0); // Ghost is always 0 in MVP
        let mut ec = commands.spawn(ghost_spawn);
        if !p.cli.is_headless()
            && let (Some(meshes), Some(materials1)) = (&mut p.meshes, &mut p.materials1)
        {
            let mesh_handle = meshes.add(Mesh::from(QuadCC::new(ghost_img_size, sprite_anchor)));
            let mut material = CustomMaterial1::from_texture(ghost_image);
            material.data.color = Color::NONE.into();
            material.data.y_anchor = anchor.y;
            let material_handle = materials1.add(material);

            ec.insert(Mesh2d(mesh_handle))
                .insert(MeshMaterial2d(material_handle))
                .insert(
                    Transform::from_xyz(-1000.0, -1000.0, -1000.0)
                        .with_scale(Vec3::splat(1.0 / ghost_rf)),
                )
                .insert(MapTileSprite)
                .insert(ResolutionFactor(ghost_rf))
                .insert(SpriteLayer(10.0))
                .insert(Ethereal::default())
                .insert(Emissive::default())
                .insert(SpectralClarity::default())
                .insert(AlphaModulator {
                    frequency: 1.0,
                    amplitude: 0.5,
                })
                .insert(EctoplasmVisuals {
                    use_breach_curve: false,
                });
        }

        ec.insert(ghost_sprite.with_breachid(breach_id))
            .insert(p.haunt_state.ghost_dynamics)
            .insert(GhostTag)
            .insert(ghost_id_net)
            .insert(GameSprite)
            .insert(MapEntityFieldBPos(ghost_spawn.to_board_position()))
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

        if !p.cli.is_headless()
            && let Some(ghost_assets) = &p.ghost_assets
        {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(0.5 * ghost_rf))
                            .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }
        let ghost_id = ec.id();

        p.board_entity_field.0[ghost_spawn.to_board_position().ndidx()].push(ghost_id);

        if !p.cli.is_headless() {
            spawn_ambient_sounds(&p, &mut commands);
        }

        crate::influence_system::assign_ghost_influence(
            &mut commands,
            &movable_objects,
            &q_ghost_breach,
            &q_player_sprite,
            &q_position,
            &p.roomdb,
            &p.board_topology,
            &p.haunt_state,
        );
    }

    ev_level_ready.write(LevelReadyEvent { open_van });
}

/// Spawns a player entity when a remote client completes handshake (host only).
fn spawn_initial_gear(
    commands: &mut Commands,
    gear_registry: &GearSpawnerRegistry,
    difficulty: &CurrentDifficulty,
    player_id: u64,
) -> PlayerGear {
    let mut player_gear = PlayerGear::default();
    let mut gear_id_counter = player_id * 1000;

    if difficulty.0.player_gear.left_hand.is_some() {
        let gear_entity = gear_registry.spawn(commands, difficulty.0.player_gear.left_hand);
        player_gear.left_hand = Some(gear_entity);
        commands
            .entity(gear_entity)
            .insert(NetworkId(gear_id_counter));
        gear_id_counter += 1;
    }
    if difficulty.0.player_gear.right_hand.is_some() {
        let gear_entity = gear_registry.spawn(commands, difficulty.0.player_gear.right_hand);
        player_gear.right_hand = Some(gear_entity);
        commands
            .entity(gear_entity)
            .insert(NetworkId(gear_id_counter));
        gear_id_counter += 1;
    }
    for kind in &difficulty.0.player_gear.inventory {
        if kind.is_some() {
            let gear_entity = gear_registry.spawn(commands, *kind);
            player_gear.inventory.push(gear_entity);
            commands
                .entity(gear_entity)
                .insert(NetworkId(gear_id_counter));
            gear_id_counter += 1;
        }
    }
    player_gear
}

pub(crate) fn spawn_joined_player(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    lobby_data: Res<LobbyData>,
    existing_players: Query<&NetworkId, With<PlayerTag>>,
    q_player_spawns: Query<&Position, With<PlayerSpawnPoint>>,
    q_disconnected: Query<(Entity, &NetworkId), With<PlayerDisconnected>>,
) {
    if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Host { .. }) {
        return;
    }

    for player in &lobby_data.players {
        if !player.connected {
            continue;
        }
        let new_id = player.id;

        // Skip if entity already exists (reconnect case)
        if existing_players.iter().any(|id| *id == new_id) {
            for (entity, disc_id) in q_disconnected.iter() {
                if disc_id == &new_id {
                    info!("Removing Disconnected state for player {:?}", new_id);
                    commands
                        .entity(entity)
                        .remove::<PlayerDisconnected>()
                        .remove::<unplayer_core::components::Hiding>();
                }
            }
            continue;
        }

        // Pick a spawn point. Try to find one not too close to existing players.
        let player_spawn_points: Vec<Position> = q_player_spawns.iter().copied().collect();
        if player_spawn_points.is_empty() {
            error!(
                "No player spawn points found, cannot spawn joined player {:?}",
                new_id
            );
            continue;
        }

        // Use NetworkId to pick a deterministic spawn point. Host is NetworkId(1) → index 0,
        // first client is NetworkId(2) → index 1, etc. This is stable regardless of join/leave order.
        let spawn_idx = (new_id.0 as usize - 1) % player_spawn_points.len();
        let spawn_pos = player_spawn_points[spawn_idx];

        info!(
            "Spawning player {:?} at {:?} (spawn point index {})",
            new_id, spawn_pos, spawn_idx
        );

        // --- Gear ---
        let player_gear =
            spawn_initial_gear(&mut commands, &p.gear_registry, &p.difficulty, new_id.0);

        // --- Visual setup ---
        let mut player_image = p
            .player_assets
            .as_ref()
            .map(|a| a.character.clone())
            .unwrap_or_default();
        let mut player_rf = 1.0;

        if !p.cli.is_headless()
            && let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p.upscale_idx.resolve(
                "img/characters-model1-demo.png",
                video_settings.max_upscale_factor.factor(),
            )
        {
            player_image = p.asset_server.load(resolved.path);
            player_rf = resolved.factor;
        }

        let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
        let anchor = unplayer_core::assets::PLAYER_ANCHOR;
        let sprite_anchor = Vec2::new(
            sprite_size.x * (anchor.x + 0.5),
            sprite_size.y * (0.5 - anchor.y),
        );

        let mut src_mesh_handle = Handle::default();
        if let Some(meshes) = &mut p.meshes {
            src_mesh_handle = meshes.add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));
        }

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut ec = commands.spawn(spawn_pos);
        ec.insert(GameSprite);

        if !p.cli.is_headless() {
            let mut material = CustomMaterial1::from_texture(player_image);
            material.data.sheet_cols = 16;
            material.data.sheet_rows = 4;
            material.data.sprite_width = 32.0 * player_rf;
            material.data.sprite_height = 32.0 * player_rf;
            material.data.upscale_factor = player_rf;
            material.data.y_anchor = anchor.y;

            if let Some(materials1) = &mut p.materials1 {
                let material_handle = materials1.add(material);

                ec.insert(Mesh2d(src_mesh_handle))
                    .insert(MeshMaterial2d(material_handle))
                    .insert(
                        Transform::from_xyz(spawn_scoord[0], spawn_scoord[1], spawn_scoord[2])
                            .with_scale(Vec3::new(
                                1.0 / player_rf,
                                1.0 / player_rf,
                                1.0 / player_rf,
                            )),
                    )
                    .insert(ResolutionFactor(player_rf))
                    .insert(MapTileSprite)
                    .insert(SpriteLayer(0.00001));
            }
        }

        // Remote player: no MainPlayer, no Viewer, no SpatialListener.
        // Use explicit ControlKeys::NONE — ControlKeys::default() is WASD, not NONE!
        ec.insert(PlayerSprite::new(new_id, spawn_pos))
            .insert(new_id)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(PlayerInputMapping {
                controls: unsettings_core::controls::ControlKeys::NONE,
            })
            .insert(PlayerInput::default())
            .insert(VisibilityData::default())
            .insert(PlayerTag)
            .insert(ShadowCaster::default())
            .insert(spawn_pos)
            .insert(MapEntityFieldBPos(spawn_pos.to_board_position()))
            .insert(Movable)
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(Direction::new_right())
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ))
            .insert(Stamina::default())
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default());

        ec.insert(player_gear);

        if !p.cli.is_headless()
            && let Some(ghost_assets) = &p.ghost_assets
        {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(1.1 * player_rf))
                            .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }
        let player_ent_id = ec.id();

        p.board_entity_field.0[spawn_pos.to_board_position().ndidx()].push(player_ent_id);
    }
}

fn spawn_ambient_sounds(p: &ClassicModeSystemParam, commands: &mut Commands) {
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

pub(crate) fn setup_replicated_player_visuals(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    q_new_players: Query<
        (Entity, &PlayerNetInfo, &NetworkPosition),
        (Added<PlayerNetInfo>, Without<PlayerSprite>),
    >,
) {
    for (entity, net_info, net_pos) in q_new_players.iter() {
        let is_local = local_player
            .0
            .map(|id| id.0 == net_info.client_id)
            .unwrap_or(false);

        let net_id = NetworkId(net_info.client_id);
        let spawn_pos = Position {
            x: net_pos.x,
            y: net_pos.y,
            z: net_pos.z,
            visual_priority: 0.0,
        };

        // Both local and remote players get a full gear kit spawned; active gear
        // state is then kept in sync via the *Net components.
        let player_gear =
            spawn_initial_gear(&mut commands, &p.gear_registry, &p.difficulty, net_info.client_id);

        // --- Visual setup ---
        let mut player_image = p
            .player_assets
            .as_ref()
            .map(|a| a.character.clone())
            .unwrap_or_default();
        let mut player_rf = 1.0;

        if !p.cli.is_headless()
            && let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p.upscale_idx.resolve(
                "img/characters-model1-demo.png",
                video_settings.max_upscale_factor.factor(),
            )
        {
            player_image = p.asset_server.load(resolved.path);
            player_rf = resolved.factor;
        }

        let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
        let anchor = unplayer_core::assets::PLAYER_ANCHOR;
        let sprite_anchor = Vec2::new(
            sprite_size.x * (anchor.x + 0.5),
            sprite_size.y * (0.5 - anchor.y),
        );

        let mut src_mesh_handle = Handle::default();
        if let Some(meshes) = &mut p.meshes {
            src_mesh_handle = meshes.add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));
        }

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut ec = commands.entity(entity);
        ec.insert(spawn_pos);
        ec.insert(GameSprite);

        if !p.cli.is_headless() {
            let mut material = CustomMaterial1::from_texture(player_image);
            material.data.sheet_cols = 16;
            material.data.sheet_rows = 4;
            material.data.sprite_width = 32.0 * player_rf;
            material.data.sprite_height = 32.0 * player_rf;
            material.data.upscale_factor = player_rf;
            material.data.y_anchor = anchor.y;

            if let Some(materials1) = &mut p.materials1 {
                let material_handle = materials1.add(material);

                ec.insert(Mesh2d(src_mesh_handle))
                    .insert(MeshMaterial2d(material_handle))
                    .insert(
                        Transform::from_xyz(spawn_scoord[0], spawn_scoord[1], spawn_scoord[2])
                            .with_scale(Vec3::new(
                                1.0 / player_rf,
                                1.0 / player_rf,
                                1.0 / player_rf,
                            )),
                    )
                    .insert(ResolutionFactor(player_rf))
                    .insert(MapTileSprite)
                    .insert(SpriteLayer(0.00001));
            }
        }

        ec.insert(PlayerSprite::new(net_id, spawn_pos))
            .insert(net_id)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(PlayerInput::default())
            .insert(VisibilityData::default())
            .insert(PlayerTag)
            .insert(ShadowCaster::default())
            .insert(MapEntityFieldBPos(spawn_pos.to_board_position()))
            .insert(Movable)
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(Direction::new_right())
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ))
            .insert(Stamina::default())
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default());

        if is_local {
            // Local player: add input, camera targets, and audio listener.
            if let Some(control_settings) = &p.control_settings {
                ec.insert(PlayerInputMapping {
                    controls: ***control_settings,
                });
            }
            ec.insert(MainPlayer).insert(Viewer {
                id: net_id,
                ..default()
            });
            if let Some(audio_settings) = &p.audio_settings {
                ec.insert(SpatialListener::new(
                    -audio_settings.sound_output.to_ear_offset(),
                ));
            }
        } else {
            // Remote player: no input, use NONE control keys.
            ec.insert(PlayerInputMapping {
                controls: unsettings_core::controls::ControlKeys::NONE,
            });
        }

        ec.insert(player_gear);

        if !p.cli.is_headless()
            && let Some(ghost_assets) = &p.ghost_assets
        {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(1.1 * player_rf))
                            .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }

        let player_ent_id = entity;
        p.board_entity_field.0[spawn_pos.to_board_position().ndidx()].push(player_ent_id);

        info!(
            "setup_replicated_player_visuals: entity {:?} client_id={} is_local={}",
            entity, net_info.client_id, is_local
        );
    }
}

pub(crate) fn sync_ghost_visuals(
    mut q_ghost: Query<(&mut SpectralClarity, &GhostBehaviorDynamics), With<GhostTag>>,
) {
    for (mut clarity, dynamics) in q_ghost.iter_mut() {
        clarity.uv = dynamics.uv_ectoplasm_clarity;
        clarity.rl = dynamics.rl_presence_clarity;
        clarity.alpha = dynamics.visual_alpha_multiplier;
    }
}
