//! # Tile Spawning Module
//!
//! This module handles the spawning and processing of individual map tiles.
//! It converts tile data from Tiled into game entities with appropriate components and behaviors.

use bevy::prelude::*;
use unbehavior::behavior::Util;
use unboard_core::components::spawning::VanEntryPoint;
use unmapload_core::components::PendingTiledLayerProperties;
use unrender_std::components::game::{GameSprite, MapTileSprite};
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::components::NetworkOriginalMapPosition;
use unspatial_core::position::Position;
use untiled_core::tiledmap::map::{MapLayer, MapTile};
use untypes_core::hydration::HydrationStage;

use crate::level_setup::LoadLevelSystemParam;

/// Processes a single map tile: spawns entity, adds components, checks for special types.
///
/// This function handles all aspects of creating a tile entity in the game world:
/// - Positioning based on map coordinates
/// - Handling flipping and orientation
/// - Adding appropriate game components
/// - Hydration stage initialization
///
/// # Arguments
///
/// * `tile` - The tile data from the Tiled map
/// * `layer` - The map layer containing the tile
/// * `map_min_x`/`map_min_y` - Map origin coordinates
/// * `map_size` - Size of the map (width, height, floors)
/// * `floor_z` - Z-coordinate (floor level) for this tile
/// * `p` - System parameters containing necessary resources
/// * `commands` - Command buffer for entity creation
/// * `c` - Counter used for ensuring unique z-ordering
pub(crate) fn process_and_spawn_tile(
    tile: &MapTile,
    layer: &MapLayer,
    map_min_x: i32,
    map_min_y: i32,
    map_size: (usize, usize, usize),
    floor_z: usize,
    p: &mut LoadLevelSystemParam,
    commands: &mut Commands,
    c: &mut f32,
) {
    // Get the map tile components from the SpriteDB
    let mt = p
        .sdb
        .map_tile
        .get(&(tile.tileset.clone(), tile.tileuid))
        .expect("Map references non-existent tileset+tileuid");

    // Spawn the base entity
    let entity_id = {
        let mut b = mt.bundle.clone();
        let mut beh = mt.behavior.clone();

        let rf = b.resolution_factor.ratio();
        b.transform.scale = Vec3::new(rf, rf, 1.0);

        // Handle sprite flipping
        if tile.flip_x {
            b.transform.scale.x = -rf;
            // Adjust the light receiving offset for flipped sprites
            let (ox, oy) = beh.p.display.light_recv_offset;
            beh.p.display.light_recv_offset = (ox, -oy);
        }

        // Create transparent material initially (will fade in later)
        if !p.cli.is_headless() {
            let mut mat = p.materials1.get(&b.material).expect("Material not found in tile_spawning").clone();
            mat.data.color.alpha = 0.0;
            let mat = p.materials1.add(mat);
            b.material = MeshMaterial2d(mat);
        }

        commands
            .spawn(b)
            .insert(HydrationStage::<1>)
            .insert(PendingTiledLayerProperties(layer.user_properties.clone()))
            .id()
    };

    // Calculate position on the map
    const MAP_MARGIN: i32 = 3;
    let t_x = (tile.pos.x - map_min_x) as f32;
    let t_y = (-tile.pos.y - map_min_y) as f32;

    // Validate position is within bounds
    assert!(
        t_x >= MAP_MARGIN as f32,
        "out of bounds X < v {:?} => {t_x},{t_y}",
        tile.pos
    );
    assert!(
        t_y >= MAP_MARGIN as f32,
        "out of bounds Y < v {:?} => {t_x},{t_y}",
        tile.pos
    );
    assert!(
        t_x < map_size.0 as f32,
        "out of bounds X > v {:?} => {t_x},{t_y}",
        tile.pos
    );
    assert!(
        t_y < map_size.1 as f32,
        "out of bounds Y > v {:?} => {t_x},{t_y}",
        tile.pos
    );

    // Create position component with z-offset applied
    let mut pos = Position {
        x: t_x,
        y: t_y,
        z: floor_z as f32 + layer.z_offset, // Apply the z-offset directly
        visual_priority: 0.0,
    };

    // Ensure unique z-ordering within the same floor level
    *c += 0.000000001;
    pos.visual_priority = f32::from(mt.behavior.p.display.visual_priority) + *c;

    // Handle special tile types based on utility (spawning separate entities)
    if matches!(&mt.behavior.p.util, Util::Van) {
        // Position for spawn points (slightly adjusted)
        let new_pos = Position {
            visual_priority: 0.0001,
            ..pos
        };
        commands.spawn((new_pos, VanEntryPoint, GameSprite));
    }

    // Now finish setting up the main tile entity
    let mut entity = commands.entity(entity_id);

    // Clone and configure behavior for this tile instance
    let mut beh = mt.behavior.clone();

    // Register the entity in the board's map entity field
    p.bef.0[pos.to_board_position().ndidx()].push(entity_id);

    // Handle horizontal flipping for behavior
    beh.flip(tile.flip_x);

    // Add board position component
    entity
        .insert(MapEntityFieldBPos(pos.to_board_position()))
        .insert(NetworkOriginalMapPosition {
            position: pos.to_board_position(),
            tileset: beh.cfg().tileset.clone(),
            tileuid: beh.cfg().tileuid,
        });

    // Add standard components to all tile entities
    let mut transform = Transform::from_xyz(t_x, t_y, pos.visual_priority);
    let rf = mt.bundle.resolution_factor.ratio();
    transform.scale = Vec3::new(rf, rf, 1.0);
    if tile.flip_x {
        transform.scale.x = -rf;
    }
    entity
        .insert(beh)
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(pos)
        .insert(Visibility::Hidden)
        .insert(transform);
}
