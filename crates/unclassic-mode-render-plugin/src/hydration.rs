use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::components::physics::{FluidEmitter, ThermalEmitter};
use unboard_core::resources::visibility_data::VisibilityData;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::GhostBehaviorDynamics;
use unghost_core::components::ghost_sprite::GhostSprite;
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerInputMapping, PlayerSprite};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unrender_std::components::focus_ring::FocusRing;
use unrender_std::components::game::{GameSprite, MapTileSprite};
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{
    AlphaModulator, EctoplasmVisuals, Emissive, Ethereal, LightSensitive, ResolutionFactor,
    ShadowCaster, SpectralClarity, UltravioletSensitive,
};
use unrender_std::materials::CustomMaterial1;
use unrender_std::utils::quadcc::QuadCC;
use unreplicon_core::resources::LocalPlayer;
use unsettings_core::video::VideoSettings;
use unsoundfield_core::components::SoundFieldSource;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::tags::GhostTag;
use untmxmap_core::resources::upscale::UpscaleIndex;

/// Marker inserted once a player entity has been fully hydrated with visuals and input.
#[derive(Component)]
pub(crate) struct PlayerHydrated;

/// Marker inserted once a ghost entity has been fully hydrated with visuals.
#[derive(Component)]
pub(crate) struct GhostHydrated;

/// Marker inserted once a breach entity has been fully hydrated with visuals.
#[derive(Component)]
pub(crate) struct BreachHydrated;

#[derive(SystemParam)]
pub(crate) struct HydrationParam<'w> {
    pub local_player_role: Option<Res<'w, untypes_core::roles::LocalPlayerRole>>,
    pub asset_server: Res<'w, AssetServer>,
    pub player_assets: Option<Res<'w, unplayer_core::assets::PlayerAssets>>,
    pub ghost_assets: Option<Res<'w, unghost_core::assets::GhostAssets>>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Option<Res<'w, Persistent<VideoSettings>>>,
    pub materials1: Option<ResMut<'w, Assets<unrender_std::materials::CustomMaterial1>>>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub images: Option<Res<'w, Assets<Image>>>,
    pub audio_settings: Option<Res<'w, Persistent<unsettings_core::audio::AudioSettings>>>,
    pub control_settings: Option<Res<'w, Persistent<unsettings_core::controls::ControlKeys>>>,
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

/// Sets up local visual and physics components on a ghost entity that arrived via replication
/// (i.e. on Join clients). Triggered by `Added<GhostSprite>` without visuals, which
/// means the server has just replicated the ghost entity to us.
/// Reactive system: fires whenever a PlayerSprite component appears on an entity,
/// either freshly spawned locally (host/offline) or replicated from the server
/// (pure client). Attaches all visual, audio, and input components.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_players_system(
    mut p: HydrationParam,
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    q_added: Query<(Entity, &PlayerSprite, &Position), Without<PlayerHydrated>>,
) {
    // Headless guard: dedicated servers have no local player and must not run this.
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, player_sprite, pos) in q_added.iter() {
        let spawn_pos = *pos;
        let is_local = local_player
            .0
            .map(|uuid| uuid == player_sprite.id)
            .unwrap_or(false);
        debug!(
            "hydrate_players_system: processing entity {:?} uuid={} is_local={} local_player={:?}",
            entity, player_sprite.id, is_local, local_player.0
        );

        // --- Resolve asset handles and resolution factor ---
        let mut player_image = p
            .player_assets
            .as_ref()
            .map(|a| a.character.clone())
            .unwrap_or_default();
        let mut player_rf = 1.0f32;

        if let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p.upscale_idx.resolve(
                "img/characters-model1-demo.png",
                video_settings.max_upscale_factor.factor(),
            )
        {
            player_image = p.asset_server.load(resolved.path);
            player_rf = resolved.factor;
        }

        // --- Compute mesh geometry ---
        let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
        let anchor = unplayer_core::assets::PLAYER_ANCHOR;
        let sprite_anchor = Vec2::new(
            sprite_size.x * (anchor.x + 0.5),
            sprite_size.y * (0.5 - anchor.y),
        );

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut ec = commands.entity(entity);

        // --- Attach shared components (all local-player nodes) ---
        ec.insert(GameSprite)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(ShadowCaster::default())
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(unspatial_core::direction::Direction::new_right())
            .insert(unbehavior::components::Movable)
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default())
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(untags_core::tags::PlayerTag)
            .insert(unspatial_core::lerp_position::LerpPosition::new(*pos))
            .insert(PlayerInput::default())
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ));

        // --- Attach visual mesh (requires meshes and materials1 to be present) ---
        if let Some(meshes) = &mut p.meshes {
            let src_mesh_handle = meshes.add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

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

        // --- Attach controls and camera: local vs. remote branch ---
        if is_local {
            // This is the player that sits at this screen. Give it input, camera
            // targeting, and spatial audio.
            if let Some(control_settings) = &p.control_settings {
                // Triple-deref: &Res<Persistent<ControlKeys>> → Persistent<ControlKeys> → ControlKeys
                ec.insert(PlayerInputMapping {
                    controls: ***control_settings,
                });
            }
            ec.insert(MainPlayer).insert(VisibilityData::default());
            if let Some(audio_settings) = &p.audio_settings {
                ec.insert(SpatialListener::new(
                    -audio_settings.sound_output.to_ear_offset(),
                ));
            }
        } else {
            // Remote player visible on this screen — no input mapping, use null keys.
            ec.insert(PlayerInputMapping {
                controls: unsettings_core::controls::ControlKeys::NONE,
            });
        }

        // --- Attach FocusRing child entity ---
        // FIXME WARNING: This child entity is spawned client-local during hydration and is NOT
        // replicated. When bevy_replicon despawns the parent (player) entity on the client it may
        // NOT call despawn_recursive, leaving this FocusRing child as an orphaned entity. This is
        // an untested code path — must be verified in multiplayer.
        // See: docs/replicon_refactor/17_server_entity_replication_inventory.md
        if let Some(ghost_assets) = &p.ghost_assets {
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

        // NOTE: Do NOT push the entity into board_entity_field here.
        // The skeleton already has MapEntityFieldBPos inserted by setup_mission_players.
        // A separate spatial-sync system reacts to Added<MapEntityFieldBPos> and populates
        // the grid for all peers uniformly. Duplicating that push here would:
        //   (a) cause a double-push on the Host (which is both Authority and LocalPlayer), and
        //   (b) leave the grid empty on the Dedicated Server, which skips this system entirely.

        commands.entity(entity).insert(PlayerHydrated);
        info!(
            "hydrate_players_system: hydrated entity {:?} player_uuid={} is_local={}",
            entity, player_sprite.id, is_local
        );
    }
}

/// Reactive system: fires whenever a GhostSprite component appears on an entity,
/// either just spawned by classic_mode_orchestrator (host/offline) or just replicated
/// from the server (pure client). Attaches all ghost visual components.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_ghosts_system(
    mut p: HydrationParam,
    mut commands: Commands,
    q_added: Query<(Entity, &Position, &GhostSprite), Without<GhostHydrated>>,
) {
    // Headless guard: dedicated servers have no local player and no visuals needed.
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, pos, _ghost) in q_added.iter() {
        let ghost_spawn = *pos;

        // --- Resolve asset handles and resolution factor ---
        let mut ghost_image = p
            .ghost_assets
            .as_ref()
            .map(|a| a.ghost.clone())
            .unwrap_or_default();
        let mut ghost_rf = 1.0f32;

        if let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p
                .upscale_idx
                .resolve("img/ghost.png", video_settings.max_upscale_factor.factor())
        {
            ghost_image = p.asset_server.load(resolved.path);
            ghost_rf = resolved.factor;
        }

        // --- Compute mesh geometry from image dimensions ---
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

        let mut ec = commands.entity(entity);

        // --- Attach non-replicated gameplay components ---
        // These match what classic_mode_orchestrator inserts on the authority at spawn time.
        ec.insert(GameSprite)
            .insert(MapEntityFieldBPos(pos.to_board_position()))
            .insert(LightSensitive {
                exposure_factor: 0.5,
                bias: 0.01,
            })
            .insert(UltravioletSensitive {
                intensity: 1.0,
                ..default()
            })
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundFieldSource::default())
            .insert(unspatial_core::lerp_position::LerpPosition::new(*pos));

        // --- Attach visual mesh ---
        if let (Some(meshes), Some(materials1)) = (&mut p.meshes, &mut p.materials1) {
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

        // --- Attach FocusRing child entity ---
        // FIXME WARNING: This child entity is spawned client-local during hydration and is NOT
        // replicated. When bevy_replicon despawns the parent (ghost) entity on the client it may
        // NOT call despawn_recursive, leaving this FocusRing child as an orphaned entity. This is
        // an untested code path — must be verified in multiplayer.
        // See: docs/replicon_refactor/17_server_entity_replication_inventory.md
        if let Some(ghost_assets) = &p.ghost_assets {
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

        // NOTE: MapEntityFieldBPos is inserted above (in the gameplay components block).
        // On the Host it already exists from classic_mode_orchestrator, so re-inserting it
        // is a no-op for Added<> (does not double-fire populate_grid_on_spawn).
        // On Join Clients it is NOT replicated, so hydration must add it here to register
        // the entity in BoardEntityField — without which the tiles lighting system never
        // processes the entity and it remains invisible (Color::NONE).

        commands.entity(entity).insert(GhostHydrated);
        info!(
            "hydrate_ghosts_system: hydrated ghost entity {:?} at {:?}",
            entity, ghost_spawn
        );
    }
}

/// Reactive system: fires whenever a GhostBreach component appears on an entity
/// (locally spawned on authority/host, or replicated to a join client).
/// Attaches all visual components for the breach effect.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_breach_system(
    mut p: HydrationParam,
    mut commands: Commands,
    q_added: Query<(Entity, &Position), (With<GhostBreach>, Without<BreachHydrated>)>,
) {
    // Headless guard: dedicated servers have no local player and no visuals needed.
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, pos) in q_added.iter() {
        let breach_pos = *pos;

        // --- Resolve image size ---
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

        let mut ec = commands.entity(entity);

        // --- Attach non-replicated gameplay components ---
        // These match what classic_mode_orchestrator inserts on the authority at spawn time.
        ec.insert(GameSprite)
            .insert(MapEntityFieldBPos(breach_pos.to_board_position()))
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
            .insert(SoundFieldSource::default());

        // --- Attach visual mesh ---
        if let (Some(meshes), Some(materials1), Some(ghost_assets)) =
            (&mut p.meshes, &mut p.materials1, &p.ghost_assets)
        {
            let mesh_handle = meshes.add(Mesh::from(QuadCC::new(breach_img_size, sprite_anchor)));
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

        // --- Attach FocusRing child entity ---
        // FIXME WARNING: This child entity is spawned client-local during hydration and is NOT
        // replicated. When bevy_replicon despawns the parent (breach) entity on the client it may
        // NOT call despawn_recursive, leaving this FocusRing child as an orphaned entity. This is
        // an untested code path — must be verified in multiplayer.
        // See: docs/replicon_refactor/17_server_entity_replication_inventory.md
        if let Some(ghost_assets) = &p.ghost_assets {
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

        commands.entity(entity).insert(BreachHydrated);
        info!(
            "hydrate_breach_system: hydrated breach entity {:?} at {:?}",
            entity, breach_pos
        );
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            hydrate_players_system.run_if(in_state(untypes_core::states::AppState::InGame)),
            hydrate_ghosts_system.run_if(in_state(untypes_core::states::AppState::InGame)),
            hydrate_breach_system.run_if(in_state(untypes_core::states::AppState::InGame)),
        ),
    );
}
