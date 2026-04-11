use bevy::prelude::*;
use unboard_core::entity::ResolutionFactor;
use unmapload_core::components::TileVisualRef;
use unrender_std::board::tiledata::PreMesh;
use unrender_std::custom_material1::CustomMaterial1;
use unrender_std::utils::quadcc::QuadCC;
use untmxmap_core::tiled::AtlasData;
use untmxmap_core::tiled::MapTileSetDb;

/// Hydrate core tile visual references into render components on client builds.
fn hydrate_tile_visual_refs(
    mut commands: Commands,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut meshes: ResMut<Assets<Mesh>>,
    tilesetdb: Res<MapTileSetDb>,
    mut q_tiles: Query<(Entity, &TileVisualRef, Option<&mut Transform>), Added<TileVisualRef>>,
) {
    for (entity, visual_ref, transform) in q_tiles.iter_mut() {
        let Some(tileset) = tilesetdb.db.get(&visual_ref.tileset) else {
            warn!(
                "hydrate_tile_visual_refs: missing tileset '{}' for entity {:?}",
                visual_ref.tileset, entity
            );
            continue;
        };

        let visibility = Visibility::Hidden;

        match &tileset.data {
            AtlasData::Sheet((_handle, cmat)) => {
                let mut cmat = cmat.clone();
                cmat.data.upscale_factor = tileset.factor;
                cmat.data.sheet_idx = visual_ref.tileuid;
                cmat.data.y_anchor = tileset.y_anchor;
                cmat.data.color.set_alpha(0.0);
                cmat.data.gamma = 0.1;
                cmat.data.gbl = 0.1;
                cmat.data.gbr = 0.1;
                cmat.data.gtl = 0.1;
                cmat.data.gtr = 0.1;

                let sprite_size = Vec2::new(
                    cmat.data.sprite_width * 1.005,
                    cmat.data.sprite_height * 1.005,
                );
                let sprite_anchor = Vec2::new(
                    sprite_size.x / 2.0,
                    sprite_size.y * (0.5 - tileset.y_anchor),
                );
                let base_quad = Mesh::from(QuadCC::new(sprite_size, sprite_anchor));
                let mesh_handle = meshes.add(base_quad);

                let mat_handle = materials1.add(cmat);
                let rf = ResolutionFactor(tileset.factor).ratio();
                let mut scale = Vec3::new(rf, rf, 1.0);
                if visual_ref.flip_x {
                    scale.x = -rf;
                }

                commands.entity(entity).insert((
                    PreMesh::Mesh(mesh_handle.into()),
                    MeshMaterial2d(mat_handle),
                    ResolutionFactor(tileset.factor),
                    visibility,
                ));

                if let Some(mut transform) = transform {
                    transform.scale = scale;
                } else {
                    commands.entity(entity).insert(Transform::from_scale(scale));
                }
            }
            AtlasData::Tiles(v_img) => {
                let Some((image_handle, mut cmat)) =
                    v_img.get(visual_ref.tileuid as usize).cloned()
                else {
                    warn!(
                        "hydrate_tile_visual_refs: tileuid {} out of range for tileset '{}'",
                        visual_ref.tileuid, visual_ref.tileset
                    );
                    continue;
                };

                cmat.data.sheet_cols = 1;
                cmat.data.sheet_rows = 1;
                cmat.data.sheet_idx = 0;
                cmat.data.y_anchor = tileset.y_anchor;
                cmat.data.color.set_alpha(0.0);
                cmat.data.gamma = 0.1;
                cmat.data.gbl = 0.1;
                cmat.data.gbr = 0.1;
                cmat.data.gtl = 0.1;
                cmat.data.gtr = 0.1;

                let mat_handle = materials1.add(cmat);
                let sprite_anchor = Vec2::new(0.5, 0.5 - tileset.y_anchor);

                let rf = ResolutionFactor(tileset.factor).ratio();
                let mut scale = Vec3::new(rf, rf, 1.0);
                if visual_ref.flip_x {
                    scale.x = -rf;
                }

                commands.entity(entity).insert((
                    PreMesh::Image {
                        sprite_anchor,
                        image_handle,
                    },
                    MeshMaterial2d(mat_handle),
                    ResolutionFactor(tileset.factor),
                    visibility,
                ));

                if let Some(mut transform) = transform {
                    transform.scale = scale;
                } else {
                    commands.entity(entity).insert(Transform::from_scale(scale));
                }
            }
            AtlasData::Headless => {
                debug!(
                    "hydrate_tile_visual_refs: headless atlas for tileset '{}' entity {:?}",
                    visual_ref.tileset, entity
                );
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydrate_tile_visual_refs);
}
