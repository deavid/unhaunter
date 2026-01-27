use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use ordered_float::OrderedFloat;
use rand::prelude::IndexedRandom;
use unassets_core::resources::upscale::UpscaleIndex;
use unbehavior::components::Movable;
use unbehavior::roomdb::RoomDB;
use unboard_core::components::physics::{FluidEmitter, SoundEmitter, ThermalEmitter};
use unboard_core::components::spawning::{HostileSpawnPoint, PlayerSpawnPoint, VanEntryPoint};
use unboard_core::resources::board_topology::BoardTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::loadlevel::{LevelReadyEvent, MapEntitiesReadyEvent};
use unfoundation_core::random_seed;
use unfoundation_core::types::sound::SoundType;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use unghost_core::components::GhostBehaviorDynamics;
use unghost_core::components::GhostBreach;
use unghost_core::components::ghost_sprite::GhostSprite;
use unghost_core::resources::haunt_state::HauntState;
use unplayer_core::components::PlayerSprite;
use unplayer_core::components::Stamina;
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::focus_ring::FocusRing;
use unrender_std::components::game::{GameSound, GameSprite, MapTileSprite};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{
    AlphaModulator, EctoplasmVisuals, Ethereal, InfraredSensitive, LightSensitive,
    ResolutionFactor, ShadowCaster, SpectralClarity, UltravioletSensitive, Viewer,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::utils::perspective;
use unrender_std::utils::quadcc::QuadCC;
use unsettings_core::video::VideoSettings;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use untags_core::tags::{GhostTag, PlayerTag};

#[derive(SystemParam)]
pub(crate) struct ClassicModeSystemParam<'w> {
    pub asset_server: Res<'w, AssetServer>,
    pub haunt_state: ResMut<'w, HauntState>,
    pub player_assets: Res<'w, unplayer_core::assets::PlayerAssets>,
    pub ghost_assets: Res<'w, unghost_core::assets::GhostAssets>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Res<'w, Persistent<VideoSettings>>,
    pub materials1: ResMut<'w, Assets<unrender_std::materials::CustomMaterial1>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub images: Res<'w, Assets<Image>>,
    pub audio_settings: Res<'w, Persistent<unsettings_core::audio::AudioSettings>>,
    pub control_settings: Res<'w, Persistent<unsettings_core::controls::ControlKeys>>,
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
    let player_scoord = perspective::to_screen_coord(player_position);

    let mut player_image = p.player_assets.character.clone();
    let mut player_rf = 1.0;

    if let Some(resolved) = p.upscale_idx.resolve(
        "img/characters-model1-demo.png",
        p.video_settings.max_upscale_factor.factor(),
    ) {
        player_image = p.asset_server.load(resolved.path);
        player_rf = resolved.factor;
    }

    let mut player_gear = PlayerGear::default();
    if p.difficulty.0.player_gear.left_hand.is_some() {
        player_gear.left_hand = Some(
            p.gear_registry
                .spawn(&mut commands, p.difficulty.0.player_gear.left_hand),
        );
    }
    if p.difficulty.0.player_gear.right_hand.is_some() {
        player_gear.right_hand = Some(
            p.gear_registry
                .spawn(&mut commands, p.difficulty.0.player_gear.right_hand),
        );
    }
    for kind in &p.difficulty.0.player_gear.inventory {
        if kind.is_some() {
            player_gear
                .inventory
                .push(p.gear_registry.spawn(&mut commands, *kind));
        }
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
    let mesh_handle = p
        .meshes
        .add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(player_image);
    material.data.sheet_cols = 16;
    material.data.sheet_rows = 4;
    material.data.sprite_width = 32.0 * player_rf;
    material.data.sprite_height = 32.0 * player_rf;
    material.data.upscale_factor = player_rf;
    material.data.y_anchor = anchor.y;

    let material_handle = p.materials1.add(material);

    let player_id = commands
        .spawn(Mesh2d(mesh_handle))
        .insert(MeshMaterial2d(material_handle))
        .insert(
            Transform::from_xyz(player_scoord[0], player_scoord[1], player_scoord[2])
                .with_scale(Vec3::new(1.0 / player_rf, 1.0 / player_rf, 1.0 / player_rf)),
        )
        .insert(ResolutionFactor(player_rf))
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(SpriteLayer(0.00001))
        .insert(PlayerSprite::new(1, player_position).with_controls(**p.control_settings))
        .insert(PlayerTag { id: 1 })
        .insert(ShadowCaster::default())
        .insert(Viewer { id: 1, ..default() })
        .insert(SpatialListener::new(
            -p.audio_settings.sound_output.to_ear_offset(),
        ))
        .insert(player_position)
        .insert(MapEntityFieldBPos(player_position.to_board_position()))
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
        .insert(unnavigation_core::components::waypoint::WaypointQueue::default())
        .insert(player_gear)
        .with_children(|parent| {
            parent
                .spawn(Sprite {
                    image: p.ghost_assets.focus_ring_vignette.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                })
                .insert(
                    Transform::from_scale(Vec3::splat(1.1 * player_rf))
                        .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                )
                .insert(FocusRing::default());
        })
        .id();

    p.board_entity_field.0[player_position.to_board_position().ndidx()].push(player_id);

    // --- Spawn Ghost ---
    p.haunt_state.evidences.clear();

    let ghost_spawn = ghost_spawn_points
        .choose(&mut rng)
        .copied()
        .unwrap_or(Position::new_i64(0, 0, 0));

    let possible_ghost_types: Vec<_> = p.difficulty.0.ghost_set.as_vec();
    let ghost_sprite = GhostSprite::new(ghost_spawn.to_board_position(), &possible_ghost_types);
    let ghost_types = vec![ghost_sprite.class];

    for evidence in ghost_sprite.class.evidences() {
        p.haunt_state.evidences.insert(evidence);
    }
    p.haunt_state.breach_pos = ghost_spawn;

    commands.insert_resource(SummaryData::new(ghost_types, p.difficulty.clone()));

    let breach_id = {
        let breach_img_size = p
            .images
            .get(p.ghost_assets.breach.id())
            .map(|img| {
                Vec2::new(
                    img.texture_descriptor.size.width as f32,
                    img.texture_descriptor.size.height as f32,
                )
            })
            .unwrap_or(Vec2::new(128.0, 128.0));

        let anchor = unmapload_core::assets::GRID_1X1X4_ANCHOR;
        let sprite_anchor = Vec2::new(
            breach_img_size.x * (anchor.x + 0.5),
            breach_img_size.y * (0.5 - anchor.y),
        );
        let mesh_handle = p
            .meshes
            .add(Mesh::from(QuadCC::new(breach_img_size, sprite_anchor)));

        let mut material = CustomMaterial1::from_texture(p.ghost_assets.breach.clone());
        material.data.color = Color::NONE.into();
        material.data.y_anchor = anchor.y;
        let material_handle = p.materials1.add(material);

        let breach_id = commands
            .spawn(Mesh2d(mesh_handle))
            .insert(MeshMaterial2d(material_handle))
            .insert(Transform::from_xyz(-1000.0, -1000.0, -1000.0))
            .insert(GameSprite)
            .insert(MapTileSprite)
            .insert(SpriteLayer(0.01))
            .insert(GhostBreach)
            .insert(ghost_spawn)
            .insert(MapEntityFieldBPos(ghost_spawn.to_board_position()))
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
            })
            .insert(EctoplasmVisuals {
                use_breach_curve: true,
            })
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundEmitter::default())
            .with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: p.ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(0.5))
                            .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                    )
                    .insert(FocusRing::default());
            })
            .id();

        p.board_entity_field.0[ghost_spawn.to_board_position().ndidx()].push(breach_id);
        breach_id
    };

    let mut ghost_image = p.ghost_assets.ghost.clone();
    let mut ghost_rf = 1.0;
    if let Some(resolved) = p.upscale_idx.resolve(
        "img/ghost.png",
        p.video_settings.max_upscale_factor.factor(),
    ) {
        ghost_image = p.asset_server.load(resolved.path);
        ghost_rf = resolved.factor;
    }

    let ghost_img_size = p
        .images
        .get(ghost_image.id())
        .map(|img| {
            Vec2::new(
                img.texture_descriptor.size.width as f32,
                img.texture_descriptor.size.height as f32,
            )
        })
        .unwrap_or(Vec2::new(128.0, 128.0));

    let anchor = unmapload_core::assets::GRID_1X1X4_ANCHOR;
    let sprite_anchor = Vec2::new(
        ghost_img_size.x * (anchor.x + 0.5),
        ghost_img_size.y * (0.5 - anchor.y),
    );
    let mesh_handle = p
        .meshes
        .add(Mesh::from(QuadCC::new(ghost_img_size, sprite_anchor)));

    let mut material = CustomMaterial1::from_texture(ghost_image);
    material.data.color = Color::NONE.into();
    material.data.y_anchor = anchor.y;
    let material_handle = p.materials1.add(material);

    let ghost_id = commands
        .spawn(Mesh2d(mesh_handle))
        .insert(MeshMaterial2d(material_handle))
        .insert(
            Transform::from_xyz(-1000.0, -1000.0, -1000.0).with_scale(Vec3::splat(1.0 / ghost_rf)),
        )
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(ResolutionFactor(ghost_rf))
        .insert(SpriteLayer(10.0))
        .insert(ghost_sprite.with_breachid(breach_id))
        .insert(Ethereal::default())
        .insert(GhostBehaviorDynamics::default())
        .insert(GhostTag)
        .insert(ghost_spawn)
        .insert(MapEntityFieldBPos(ghost_spawn.to_board_position()))
        .insert(Movable)
        .insert(SpectralClarity::default())
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
        .insert(AlphaModulator {
            frequency: 1.0,
            amplitude: 0.5,
        })
        .insert(EctoplasmVisuals {
            use_breach_curve: false,
        })
        .insert(ThermalEmitter {
            room_restricted: true,
            ..default()
        })
        .insert(FluidEmitter::default())
        .insert(SoundEmitter::default())
        .with_children(|parent| {
            parent
                .spawn(Sprite {
                    image: p.ghost_assets.focus_ring_vignette.clone(),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                })
                .insert(
                    Transform::from_scale(Vec3::splat(0.5 * ghost_rf))
                        .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                )
                .insert(FocusRing::default());
        })
        .id();

    p.board_entity_field.0[ghost_spawn.to_board_position().ndidx()].push(ghost_id);

    spawn_ambient_sounds(&p, &mut commands);

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

    ev_level_ready.write(LevelReadyEvent { open_van });
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

pub(crate) fn sync_ghost_visuals(
    haunt_state: Res<HauntState>,
    mut q_ghost: Query<&mut SpectralClarity, With<GhostTag>>,
) {
    for mut clarity in q_ghost.iter_mut() {
        clarity.uv = haunt_state.ghost_dynamics.uv_ectoplasm_clarity;
        clarity.rl = haunt_state.ghost_dynamics.rl_presence_clarity;
        clarity.alpha = haunt_state.ghost_dynamics.visual_alpha_multiplier;
    }
}
