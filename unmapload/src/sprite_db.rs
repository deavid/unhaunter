//! # Sprite Database Module
//!
//! This module is responsible for populating the SpriteDB resource with tile data from tilesets.
//! The SpriteDB serves as a comprehensive lookup for tile visuals and behaviors throughout the game.

use bevy::prelude::*;
use bevy::utils::HashMap;
use uncore::behavior::{Behavior, SpriteConfig};
use uncore::types::quadcc::QuadCC;
use unstd::board::tiledata::{MapTileComponents, PreMesh, TileSpriteBundle};
use unstd::tiledmap::AtlasData;

use crate::level_setup::LoadLevelSystemParam;

/// Pre-computes and populates the SpriteDB resource with tile data from all available tilesets.
///
/// This function is called during level loading to prepare all tile visual data and behavior properties
/// that will be used when spawning tile entities.
///
/// # Arguments
/// * `p` - System parameters containing resources needed for sprite loading
/// * `mesh_tileset` - Hash map to cache mesh handles for each tileset
pub fn populate_sprite_db(
    p: &mut LoadLevelSystemParam,
    mesh_tileset: &mut HashMap<String, Handle<Mesh>>,
) {
    // Iterate through all tilesets in the database
    for (tset_name, tileset) in p.tilesetdb.db.iter() {
        for (tileuid, tiled_tile) in tileset.tileset.tiles() {
            // Create sprite configuration from the tile data
            let sprite_config =
                SpriteConfig::from_tiled_auto(tset_name.clone(), tileuid, &tiled_tile);
            let behavior = Behavior::from_config(sprite_config);

            // Set initial visibility based on behavior
            let visibility = if behavior.p.display.disable {
                Visibility::Hidden
            } else {
                Visibility::Inherited
            };

            // Position tile offscreen initially
            let transform = Transform::from_xyz(-10000.0, -10000.0, -1000.0);

            // Create a sprite bundle based on the tileset type
            let bundle = match &tileset.data {
                AtlasData::Sheet((handle, cmat)) => {
                    let mut cmat = cmat.clone();
                    let tatlas = p.texture_atlases.get(handle).unwrap();

                    // Create or reuse mesh for this tileset
                    let mesh_handle = mesh_tileset
                        .entry(tset_name.to_string())
                        .or_insert_with(|| {
                            let sprite_size = Vec2::new(
                                tatlas.size.x as f32 / cmat.data.sheet_cols as f32 * 1.005,
                                tatlas.size.y as f32 / cmat.data.sheet_rows as f32 * 1.005,
                            );
                            let sprite_anchor = Vec2::new(
                                sprite_size.x / 2.0,
                                sprite_size.y * (0.5 - tileset.y_anchor),
                            );
                            let base_quad = Mesh::from(QuadCC::new(sprite_size, sprite_anchor));
                            p.meshes.add(base_quad)
                        })
                        .clone();
                    cmat.data.sheet_idx = tileuid;

                    // Set alpha initially transparent to all materials
                    cmat.data.color.set_alpha(0.0);
                    cmat.data.gamma = 0.1;
                    cmat.data.gbl = 0.1;
                    cmat.data.gbr = 0.1;
                    cmat.data.gtl = 0.1;
                    cmat.data.gtr = 0.1;
                    let mat = p.materials1.add(cmat);

                    TileSpriteBundle {
                        mesh: PreMesh::Mesh(mesh_handle.into()),
                        material: MeshMaterial2d(mat.clone()),
                        transform,
                        visibility,
                    }
                }
                AtlasData::Tiles(v_img) => {
                    let (image_handle, mut cmat) = v_img[tileuid as usize].clone();
                    cmat.data.sheet_cols = 1;
                    cmat.data.sheet_rows = 1;
                    cmat.data.sheet_idx = 0;

                    // Set alpha initially transparent to all materials
                    cmat.data.color.set_alpha(0.0);
                    cmat.data.gamma = 0.1;
                    cmat.data.gbl = 0.1;
                    cmat.data.gbr = 0.1;
                    cmat.data.gtl = 0.1;
                    cmat.data.gtr = 0.1;
                    let mat = p.materials1.add(cmat);

                    let sprite_anchor = Vec2::new(1.0 / 2.0, 0.5 - tileset.y_anchor);

                    TileSpriteBundle {
                        mesh: PreMesh::Image {
                            sprite_anchor,
                            image_handle,
                        },
                        material: MeshMaterial2d(mat.clone()),
                        transform,
                        visibility,
                    }
                }
            };

            // Store the tile data in the sprite database
            let key_tuid = behavior.key_tuid();
            p.sdb
                .cvo_idx
                .entry(behavior.key_cvo())
                .or_default()
                .push(key_tuid.clone());
            let mt = MapTileComponents { bundle, behavior };
            p.sdb.map_tile.insert(key_tuid, mt);
        }
    }
}
