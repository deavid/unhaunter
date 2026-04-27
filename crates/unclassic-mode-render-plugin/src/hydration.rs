use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::{GameSprite, MapTileSprite, ResolutionFactor};
use unboard_core::resources::visibility_data::VisibilityData;
use uninput_core::components::{PlayerInput, PlayerInputMapping};
use unlight_core::components::LightSensitive;
use unlocomotion_core::animation::{AnimationTimer, CharacterAnimation};
use unplayer_core::components::PlayerSprite;
use unrender_std::components::focus_ring::FocusRing;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::ShadowCaster;
use unrender_std::custom_material1::CustomMaterial1;
use unrender_std::utils::quadcc::QuadCC;
use unreplicon_core::resources::LocalPlayer;
use unsettings_core::video::VideoSettings;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untmxmap_core::resources::upscale::UpscaleIndex;

/// Marker inserted once a player entity has been fully hydrated with visuals and input.
#[derive(Component)]
pub(crate) struct PlayerHydrated;

#[derive(SystemParam)]
pub(crate) struct HydrationParam<'w> {
    pub local_player_role: Option<Res<'w, unreplicon_core::resources::LocalPlayerRole>>,
    pub asset_server: Res<'w, AssetServer>,
    pub player_assets: Option<Res<'w, unplayer_core::assets::PlayerAssets>>,
    pub ghost_assets: Option<Res<'w, unghost_core::assets::GhostAssets>>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Option<Res<'w, Persistent<VideoSettings>>>,
    pub materials1: Option<ResMut<'w, Assets<unrender_std::custom_material1::CustomMaterial1>>>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub audio_settings: Option<Res<'w, Persistent<unsettings_core::audio::AudioSettings>>>,
    pub control_settings: Option<Res<'w, Persistent<unsettings_core::controls::ControlKeys>>>,
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
        let is_local = local_player.uuid == player_sprite.id;
        debug!(
            "hydrate_players_system: processing entity {:?} uuid={} is_local={} local_player={:?}",
            entity, player_sprite.id, is_local, local_player.uuid
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
            .insert(unbehavior_core::components::Movable)
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default())
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(unplayer_core::components::PlayerTag)
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
            ec.insert(VisibilityData::default());
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

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_players_system.run_if(in_state(uncommon_states_core::UIContextState::InGame)),
    );
}
