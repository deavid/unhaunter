use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::{GameSprite, MapTileSprite, ResolutionFactor};
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::components::presentation::spectral::SpectralClarity;
use unmapload_core::assets::GRID_1X1X4_ANCHOR;
use unrender_std::components::focus_ring::FocusRing;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::{AlphaModulator, EctoplasmVisuals, Emissive, Ethereal};
use unrender_std::custom_material1::CustomMaterial1;
use unrender_std::utils::quadcc::QuadCC;
use unsettings_core::video::VideoSettings;
use unspatial_core::position::Position;
use untmxmap_core::resources::upscale::UpscaleIndex;

/// Marker inserted once a ghost entity has been fully hydrated with visuals.
#[derive(Component)]
pub(crate) struct GhostHydrated;

/// Marker inserted once a breach entity has been fully hydrated with visuals.
#[derive(Component)]
pub(crate) struct BreachHydrated;

/// System parameter containing only the resources needed for ghost/breach hydration.
#[derive(SystemParam)]
pub(crate) struct GhostHydrationParam<'w> {
    pub local_player_role: Option<Res<'w, unreplicon_core::resources::LocalPlayerRole>>,
    pub asset_server: Res<'w, AssetServer>,
    pub ghost_assets: Option<Res<'w, unghost_core::assets::GhostAssets>>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Option<Res<'w, Persistent<VideoSettings>>>,
    pub materials1: Option<ResMut<'w, Assets<CustomMaterial1>>>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub images: Option<Res<'w, Assets<Image>>>,
}

/// Reactive system: fires whenever a GhostSprite component appears on an entity.
/// Attaches all ghost visual components.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_ghosts_system(
    mut p: GhostHydrationParam,
    mut commands: Commands,
    q_added: Query<(Entity, &Position, &GhostSprite, Option<&GameSprite>), Without<GhostHydrated>>,
) {
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, pos, _ghost, game_sprite) in q_added.iter() {
        if game_sprite.is_none() {
            debug!(
                "hydrate_ghosts_system: waiting for GameSprite before visual hydration on ghost {:?}",
                entity
            );
            continue;
        }

        let ghost_spawn = *pos;

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

        let mut ghost_img_size = Vec2::new(128.0, 128.0);
        if let Some(images) = &p.images
            && let Some(img) = images.get(ghost_image.id())
        {
            ghost_img_size = Vec2::new(
                img.texture_descriptor.size.width as f32,
                img.texture_descriptor.size.height as f32,
            );
        }

        let anchor = GRID_1X1X4_ANCHOR;
        let sprite_anchor = Vec2::new(
            ghost_img_size.x * (anchor.x + 0.5),
            ghost_img_size.y * (0.5 - anchor.y),
        );

        let mut ec = commands.entity(entity);

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
                .insert(MapColor {
                    color: Color::WHITE,
                })
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

        commands.entity(entity).insert(GhostHydrated);
        info!(
            "hydrate_ghosts_system: hydrated ghost entity {:?} at {:?}",
            entity, ghost_spawn
        );
    }
}

/// Reactive system: fires whenever a GhostBreach component appears on an entity.
/// Attaches all visual components for the breach effect.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_breach_system(
    mut p: GhostHydrationParam,
    mut commands: Commands,
    q_added: Query<
        (Entity, &Position, Option<&GameSprite>),
        (With<GhostBreach>, Without<BreachHydrated>),
    >,
) {
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, pos, game_sprite) in q_added.iter() {
        if game_sprite.is_none() {
            debug!(
                "hydrate_breach_system: waiting for GameSprite before visual hydration on breach {:?}",
                entity
            );
            continue;
        }

        let breach_pos = *pos;

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

        let anchor = GRID_1X1X4_ANCHOR;
        let sprite_anchor = Vec2::new(
            breach_img_size.x * (anchor.x + 0.5),
            breach_img_size.y * (0.5 - anchor.y),
        );

        let mut ec = commands.entity(entity);

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
                .insert(MapColor {
                    color: Color::WHITE,
                })
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
            hydrate_ghosts_system.run_if(in_state(uncommon_states_core::UIContextState::InGame)),
            hydrate_breach_system.run_if(in_state(uncommon_states_core::UIContextState::InGame)),
        ),
    );
}
