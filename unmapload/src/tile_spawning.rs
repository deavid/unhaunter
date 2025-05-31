//! # Tile Spawning Module
//!
//! This module handles the spawning and processing of individual map tiles.
//! It converts tile data from Tiled into game entities with appropriate components and behaviors.

use bevy::prelude::*;
use uncore::behavior::{TileState, Util};
use uncore::components::board::boardposition::MapEntityFieldBPos;
use uncore::components::board::position::Position;
use uncore::components::game::{GameSprite, MapTileSprite};
use uncore::types::tiledmap::map::{MapLayer, MapTile};

use crate::level_setup::LoadLevelSystemParam;

/// Processes a single map tile: spawns entity, adds components, checks for special types.
///
/// This function handles all aspects of creating a tile entity in the game world:
/// - Positioning based on map coordinates
/// - Handling flipping and orientation
/// - Adding appropriate game components
/// - Registering special tile types (player spawn, ghost spawn, etc.)
/// - Adding to the movable objects list if appropriate
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
/// * `player_spawn_points`/`ghost_spawn_points`/`van_entry_points` - Lists to collect special points
/// * `movable_objects` - List to collect movable object entities
/// * `c` - Counter used for ensuring unique z-ordering
pub fn process_and_spawn_tile(
    tile: &MapTile,
    layer: &MapLayer,
    map_min_x: i32,
    map_min_y: i32,
    map_size: (usize, usize, usize),
    floor_z: usize,
    p: &mut LoadLevelSystemParam,
    commands: &mut Commands,
    player_spawn_points: &mut Vec<Position>,
    ghost_spawn_points: &mut Vec<Position>,
    van_entry_points: &mut Vec<Position>,
    movable_objects: &mut Vec<Entity>,
    c: &mut f32,
) {
    // Get the map tile components from the SpriteDB
    let mt = p
        .sdb
        .map_tile
        .get(&(tile.tileset.clone(), tile.tileuid))
        .expect("Map references non-existent tileset+tileuid");

    // Spawn the base entity
    let mut entity = {
        let mut b = mt.bundle.clone();
        let mut beh = mt.behavior.clone();

        // Handle sprite flipping
        if tile.flip_x {
            b.transform.scale.x = -1.0;
            // Adjust the light receiving offset for flipped sprites
            let (ox, oy) = beh.p.display.light_recv_offset;
            beh.p.display.light_recv_offset = (ox, -oy);
        }

        // Create transparent material initially (will fade in later)
        let mut mat = p.materials1.get(&b.material).unwrap().clone();
        mat.data.color.alpha = 0.0;
        let mat = p.materials1.add(mat);
        b.material = MeshMaterial2d(mat);

        commands.spawn(b)
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
        global_z: 0.0,
    };

    // Ensure unique z-ordering within the same floor level
    *c += 0.000000001;
    pos.global_z = f32::from(mt.behavior.p.display.global_z) + *c;

    // Position for spawn points (slightly adjusted)
    let new_pos = Position {
        global_z: 0.0001,
        ..pos
    };

    // Handle special tile types based on utility
    match &mt.behavior.p.util {
        Util::PlayerSpawn => {
            player_spawn_points.push(new_pos);
        }
        Util::GhostSpawn => {
            ghost_spawn_points.push(new_pos);
        }
        Util::RoomDef(name) => {
            p.roomdb
                .room_tiles
                .insert(pos.to_board_position(), name.to_owned());
            p.roomdb.room_state.insert(name.clone(), TileState::Off);
        }
        Util::Van => {
            van_entry_points.push(new_pos);
        }
        Util::None => {}
    }

    // Add behavior-specific components
    mt.behavior.default_components(&mut entity, layer);

    // Clone and configure behavior for this tile instance
    let mut beh = mt.behavior.clone();

    // Register the entity in the board's map entity field
    p.bf.map_entity_field[pos.to_board_position().ndidx()].push(entity.id());

    // Handle horizontal flipping for behavior
    beh.flip(tile.flip_x);

    // Add board position component
    entity.insert(MapEntityFieldBPos(pos.to_board_position()));

    // Check if the object is movable
    if mt.behavior.p.object.movable {
        // FIXME: It does not check if the item is in a valid room, since the rooms are
        // still being constructed at this point. This is something to fix later on.
        movable_objects.push(entity.id());
    }

    // Add standard components to all tile entities
    entity
        .insert(beh)
        .insert(GameSprite)
        .insert(MapTileSprite)
        .insert(pos)
        .insert(Visibility::Hidden);
}
