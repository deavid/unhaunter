// ------------ Bevy map loading utils --------------------
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use std::path::{Path, PathBuf};
use unassets_core::resources::upscale::UpscaleIndex;
use unboard_core::types::floor::FloorLevelMapping;
use unrender_std::materials::CustomMaterial1;
use unsettings_core::video::VideoSettings;
use untiled_core::tiled::{AtlasData, MapTileSet, MapTileSetDb};
use untiled_core::tiledmap::map::{MapLayer, MapLayerGroup, MapLayerType};

use super::load::load_tile_layer_iter;

/// Helps trimming the extra assets/ folder for Bevy
pub(crate) fn resolve_tiled_image_path(img_path: &Path) -> PathBuf {
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

pub(crate) fn bevy_load_map(
    map: tiled::Map,
    asset_server: &AssetServer,
    o_texture_atlases: &mut Option<ResMut<Assets<TextureAtlasLayout>>>,
    tilesetdb: &mut ResMut<MapTileSetDb>,
    upscale_idx: &UpscaleIndex,
    video_settings: &VideoSettings,
    headless: bool,
) -> (Vec<(usize, MapLayer)>, FloorLevelMapping) {
    // Preload all tilesets referenced:
    for tileset in map.tilesets().iter() {
        let mut factor = 1.0;
        // If an image is included, this is a tilemap. If no image is included this is a
        // sprite collection. Sprite collections are not supported right now.
        let data = if let Some(texture_atlases) = o_texture_atlases
            && !headless
        {
            if let Some(image) = &tileset.image {
                let img_src = resolve_tiled_image_path(&image.source);
                let img_src_str = img_src.to_string_lossy();

                let (loading_src, f) = if let Some(resolved) =
                    upscale_idx.resolve(&img_src_str, video_settings.max_upscale_factor.factor())
                {
                    (PathBuf::from(resolved.path), resolved.factor)
                } else {
                    (img_src, 1.0)
                };
                factor = f;

                let texture: Handle<Image> = asset_server.load(loading_src);
                let rows = tileset.tilecount / tileset.columns;
                let atlas1 = TextureAtlasLayout::from_grid(
                    UVec2::new(
                        (tileset.tile_width as f32 * factor) as u32,
                        (tileset.tile_height as f32 * factor) as u32,
                    ),
                    tileset.columns,
                    rows,
                    Some(UVec2::new(
                        (tileset.spacing as f32 * factor) as u32,
                        (tileset.spacing as f32 * factor) as u32,
                    )),
                    Some(UVec2::new(
                        (tileset.margin as f32 * factor) as u32,
                        (tileset.margin as f32 * factor) as u32,
                    )),
                );
                let mut cmat = CustomMaterial1::from_texture(texture);
                cmat.data.sheet_rows = rows;
                cmat.data.sheet_cols = tileset.columns;
                cmat.data.sheet_idx = 0;
                cmat.data.sprite_width = tileset.tile_width as f32 * factor;
                cmat.data.sprite_height = tileset.tile_height as f32 * factor;
                cmat.data.padding = tileset.spacing as f32 * factor;
                cmat.data.margin = tileset.margin as f32 * factor;
                cmat.data.upscale_factor = factor;
                let atlas1_handle = texture_atlases.add(atlas1);
                AtlasData::Sheet((atlas1_handle.clone(), cmat))
            } else {
                let mut images: Vec<(Handle<Image>, CustomMaterial1)> = vec![];
                for (_tileid, tile) in tileset.tiles() {
                    // tile.collision
                    if let Some(image) = &tile.image {
                        let img_src = resolve_tiled_image_path(&image.source);
                        let img_src_str = img_src.to_string_lossy();

                        let (loading_src, f) = if let Some(resolved) = upscale_idx
                            .resolve(&img_src_str, video_settings.max_upscale_factor.factor())
                        {
                            (PathBuf::from(resolved.path), resolved.factor)
                        } else {
                            (img_src, 1.0)
                        };
                        factor = f;
                        let img_handle: Handle<Image> = asset_server.load(loading_src);
                        let mut cmat = CustomMaterial1::from_texture(img_handle.clone());
                        cmat.data.sprite_width = (image.width as f32) * factor;
                        cmat.data.sprite_height = (image.height as f32) * factor;
                        cmat.data.upscale_factor = factor;
                        images.push((img_handle, cmat));
                    }
                }
                AtlasData::Tiles(images)
            }
        } else {
            AtlasData::Headless
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
            factor,
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
                let mut floor_layers: Vec<(usize, MapLayer)> = Vec::new();
                if let MapLayerType::Group(group) = &layer.data {
                    for (i, mut child_layer) in group.layers.iter().cloned().enumerate() {
                        // Set floor information for each child layer
                        child_layer.floor_number = Some(floor_number);
                        child_layer.parent_floor_name = Some(layer.name.clone());
                        floor_layers.push((i, child_layer));
                    }
                }
                debug!("Floor level number: {floor_number} - name: {display_name}");
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
    let mut ghost_attracting_objects: HashMap<i32, i32> = HashMap::new();
    let mut ghost_repelling_objects: HashMap<i32, i32> = HashMap::new();

    // Create contiguous z-coordinates
    for (z, &floor_num) in sorted_floor_numbers.iter().enumerate() {
        floor_to_z.insert(floor_num, z);
        z_to_floor.insert(z, floor_num);

        // For each floor level, check if there are ghost influence requirements
        if let Some(floor_level_layer) = grp.layers.iter().find(|l| {
            l.user_class == Some("FloorLevel".to_string()) && get_floor_number(l) == Some(floor_num)
        }) {
            // Extract floor display name
            if let Some(level) = &floor_levels.get(&floor_num) {
                floor_display_names.insert(floor_num, level.display_name.clone());
            }

            // Extract ghost influence requirements
            if let Some(attracting) = get_floor_ghost_attracting_objects(floor_level_layer) {
                ghost_attracting_objects.insert(floor_num, attracting);
                debug!("Floor {floor_num} requires {attracting} ghost attracting objects");
            }

            if let Some(repelling) = get_floor_ghost_repelling_objects(floor_level_layer) {
                ghost_repelling_objects.insert(floor_num, repelling);
                debug!("Floor {floor_num} requires {repelling} ghost repelling objects");
            }
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
        ghost_attracting_objects,
        ghost_repelling_objects,
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

/// Helper function to extract the quantity of ghost attracting objects for a floor
pub(crate) fn get_floor_ghost_attracting_objects(layer: &MapLayer) -> Option<i32> {
    if let Some(value) = layer
        .user_properties
        .get("FloorLevel::quantity_ghost_attracting_objects")
    {
        match value {
            tiled::PropertyValue::IntValue(num) => Some(*num),
            tiled::PropertyValue::StringValue(s) => s.parse::<i32>().ok(),
            _ => None,
        }
    } else {
        None
    }
}

/// Helper function to extract the quantity of ghost repelling objects for a floor
pub(crate) fn get_floor_ghost_repelling_objects(layer: &MapLayer) -> Option<i32> {
    if let Some(value) = layer
        .user_properties
        .get("FloorLevel::quantity_ghost_repelling_objects")
    {
        match value {
            tiled::PropertyValue::IntValue(num) => Some(*num),
            tiled::PropertyValue::StringValue(s) => s.parse::<i32>().ok(),
            _ => None,
        }
    } else {
        None
    }
}
