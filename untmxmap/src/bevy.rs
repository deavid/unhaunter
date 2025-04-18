// ------------ Bevy map loading utils --------------------
use bevy::{prelude::*, utils::HashMap};
use std::path::{Path, PathBuf};
use uncore::{
    events::loadlevel::FloorLevelMapping,
    types::tiledmap::map::{MapLayer, MapLayerGroup},
};
use unstd::{
    materials::CustomMaterial1,
    tiledmap::{AtlasData, MapTileSet, MapTileSetDb},
};

use super::load::load_tile_layer_iter;

/// Helps trimming the extra assets/ folder for Bevy
pub fn resolve_tiled_image_path(img_path: &Path) -> PathBuf {
    use normalize_path::NormalizePath;

    img_path
        .strip_prefix("assets/")
        .unwrap_or(img_path)
        .normalize()
        .to_owned()
}

/// Contains information about a floor level in the map
#[derive(Debug)]
struct FloorLevel {
    display_name: String,
    layers: Vec<(usize, MapLayer)>,
}

pub fn bevy_load_map(
    map: tiled::Map,
    asset_server: &AssetServer,
    texture_atlases: &mut ResMut<Assets<TextureAtlasLayout>>,
    tilesetdb: &mut ResMut<MapTileSetDb>,
) -> (Vec<(usize, MapLayer)>, FloorLevelMapping) {
    // Preload all tilesets referenced:
    for tileset in map.tilesets().iter() {
        // If an image is included, this is a tilemap. If no image is included this is a
        // sprite collection. Sprite collections are not supported right now.
        let data = if let Some(image) = &tileset.image {
            let img_src = resolve_tiled_image_path(&image.source);

            // FIXME: When the images are loaded onto the GPU it seems that we need at least 1
            // pixel of empty space .. so that the GPU can sample surrounding pixels properly.
            // .. This contrasts with how Tiled works, as it assumes a perfect packing if
            // possible.
            const MARGIN: u32 = 1;

            // TODO: Ideally we would prefer to preload, upscale by nearest to 2x or 4x, and
            // add a 2px margin. Recreating .. the texture on the fly.
            let texture: Handle<Image> = asset_server.load(img_src);
            let rows = tileset.tilecount / tileset.columns;
            let atlas1 = TextureAtlasLayout::from_grid(
                UVec2::new(
                    tileset.tile_width + tileset.spacing - MARGIN,
                    tileset.tile_height + tileset.spacing - MARGIN,
                ),
                tileset.columns,
                rows,
                Some(UVec2::new(MARGIN, MARGIN)),
                Some(UVec2::new(0, 0)),
            );
            let mut cmat = CustomMaterial1::from_texture(texture);
            cmat.data.sheet_rows = rows;
            cmat.data.sheet_cols = tileset.columns;
            cmat.data.sheet_idx = 0;
            cmat.data.sprite_width = tileset.tile_width as f32 + tileset.spacing as f32;
            cmat.data.sprite_height = tileset.tile_height as f32 + tileset.spacing as f32;
            let atlas1_handle = texture_atlases.add(atlas1);
            AtlasData::Sheet((atlas1_handle.clone(), cmat))
        } else {
            let mut images: Vec<(Handle<Image>, CustomMaterial1)> = vec![];
            for (_tileid, tile) in tileset.tiles() {
                // tile.collision
                if let Some(image) = &tile.image {
                    let img_src = resolve_tiled_image_path(&image.source);
                    dbg!(&img_src);
                    let img_handle: Handle<Image> = asset_server.load(img_src);
                    let cmat = CustomMaterial1::from_texture(img_handle.clone());
                    images.push((img_handle, cmat));
                }
            }
            AtlasData::Tiles(images)
        };

        // NOTE: tile.offset_x/y is used when drawing, instead we want the center point.
        let anchor_bottom_px = tileset.properties.get("Anchor::bottom_px").and_then(|x| {
            if let tiled::PropertyValue::IntValue(n) = x {
                Some(n)
            } else {
                None
            }
        });
        let y_anchor: f32 = if let Some(n) = anchor_bottom_px {
            // find the fraction from the total image:
            let f = *n as f32 / (tileset.tile_height + tileset.spacing) as f32;

            // from the center:
            f - 0.5
        } else {
            -0.25
        };
        let mts = MapTileSet {
            tileset: tileset.clone(),
            data,
            y_anchor,
        };

        // Store the tileset in memory in case we need to do anything with it later on.
        if tilesetdb.db.insert(tileset.name.to_string(), mts).is_some() {
            eprintln!(
                "ERROR: Already existing tileset loaded with name {:?} - make sure you don't have the same tileset loaded twice",
                tileset.name.to_string()
            );
            // panic!();
        }
    }
    let map_layers = load_tile_layer_iter(map.layers());
    let grp = MapLayerGroup { layers: map_layers };

    // Process map layers by floor level
    let mut floor_levels: HashMap<i32, FloorLevel> = HashMap::new();
    let mut ungrouped_layers: Vec<(usize, MapLayer)> = Vec::new();
    let mut layer_index = 0;

    // First pass: group layers by floor level
    for layer in grp.layers.iter() {
        if layer.user_class == Some("FloorLevel".to_string()) {
            if let Some(floor_number) = get_floor_number(layer) {
                let display_name = get_floor_display_name(layer)
                    .unwrap_or_else(|| format!("Floor {}", floor_number));

                // Extract child layers from this floor level group
                let mut floor_layers = Vec::new();
                if let uncore::types::tiledmap::map::MapLayerType::Group(group) = &layer.data {
                    for (i, mut child_layer) in group.layers.iter().cloned().enumerate() {
                        // Set floor information for each child layer
                        child_layer.floor_number = Some(floor_number);
                        child_layer.parent_floor_name = Some(layer.name.clone());
                        floor_layers.push((i, child_layer));
                    }
                }
                info!("Floor level number: {floor_number} - name: {display_name}");
                floor_levels.insert(
                    floor_number,
                    FloorLevel {
                        // number: floor_number,
                        display_name,
                        layers: floor_layers,
                    },
                );
            } else {
                // If it's a FloorLevel but has no number, use a default of 0
                warn!(
                    "Unrecognized layer structure {} - layer class but no number",
                    layer.name
                );
                ungrouped_layers.push((layer_index, layer.clone()));
                layer_index += 1;
            }
        } else {
            // Not a floor level group, add to ungrouped layers
            warn!(
                "Unrecognized layer structure {} - unrecognized layer class",
                layer.name
            );
            ungrouped_layers.push((layer_index, layer.clone()));
            layer_index += 1;
        }
    }

    // Sort floor levels by their number
    let mut sorted_floor_numbers: Vec<i32> = floor_levels.keys().cloned().collect();
    sorted_floor_numbers.sort();

    // Create mappings between floor numbers and z-coordinates
    let mut floor_to_z: HashMap<i32, usize> = HashMap::new();
    let mut z_to_floor: HashMap<usize, i32> = HashMap::new();
    let mut floor_display_names: HashMap<i32, String> = HashMap::new();

    // Create contiguous z-coordinates
    for (z, &floor_num) in sorted_floor_numbers.iter().enumerate() {
        floor_to_z.insert(floor_num, z);
        z_to_floor.insert(z, floor_num);
        if let Some(level) = &floor_levels.get(&floor_num) {
            floor_display_names.insert(floor_num, level.display_name.clone());
        }
    }

    // Build the final list of layers
    let mut final_layers: Vec<(usize, MapLayer)> = Vec::new();

    // Add all layers from each floor level, starting with floor 0 or the lowest floor
    for &floor_num in &sorted_floor_numbers {
        if let Some(level) = floor_levels.get(&floor_num) {
            for (i, layer) in &level.layers {
                final_layers.push((*i, layer.clone()));
            }
        } else {
            error!("Unexpected {floor_num} not found?");
        }
    }

    // Add any layers that weren't part of a floor group
    for (i, layer) in ungrouped_layers {
        // Always include ungrouped layers, regardless of visibility
        final_layers.push((i, layer));
    }

    // No longer filter by visibility - include all layers
    let layers: Vec<(usize, MapLayer)> = final_layers;

    let mapping = FloorLevelMapping {
        floor_to_z,
        z_to_floor,
        floor_display_names,
    };

    (layers, mapping)
}

/// Helper function to extract the floor number from a layer's properties
fn get_floor_number(layer: &MapLayer) -> Option<i32> {
    if let Some(tiled::PropertyValue::IntValue(num)) =
        layer.user_properties.get("FloorLevel::number")
    {
        Some(*num)
    } else {
        warn!("Incorrect type for FloorLevel::number or property not found");
        None
    }
}

/// Helper function to extract the floor display name from a layer's properties
fn get_floor_display_name(layer: &MapLayer) -> Option<String> {
    if let Some(tiled::PropertyValue::StringValue(name)) =
        layer.user_properties.get("FloorLevel::display_name")
    {
        Some(name.clone())
    } else {
        warn!("Incorrect type for FloorLevel::display_name or property not found");
        None
    }
}
