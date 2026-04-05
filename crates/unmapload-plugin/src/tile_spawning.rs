//! # Tile Spawning Module
//!
//! This module handles the spawning and processing of individual map tiles.
//! It converts tile data from Tiled into game entities with appropriate components and behaviors.

use bevy::prelude::*;
use bevy_platform::collections::HashSet;
use bevy_replicon::prelude::Replicated;
use unbehavior_core::behavior::Util;
use unbehavior_core::components::PendingProperties;
use unbehavior_core::components::TmxEntityId;
use unboard_core::components::spawning::VanEntryPoint;
use unboard_core::entity::{GameSprite, MapTileSprite};
use unmapload_core::components::TileVisualRef;
use unmapload_core::hydration::HydrationStage;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;
use untmxmap_core::types::map::{MapLayer, MapTile};

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
    layer_idx: usize,
    map_min_x: i32,
    map_min_y: i32,
    map_size: (usize, usize, usize),
    floor_z: usize,
    p: &mut LoadLevelSystemParam,
    commands: &mut Commands,
    c: &mut f32,
    existing_tmx_map: &bevy_platform::collections::HashMap<TmxEntityId, Entity>,
    scheduled_for_despawn: &HashSet<Entity>,
    q_positions: &Query<&Position>,
    map_filepath: &str,
) {
    // Get the map tile components from the SpriteDB
    let mt = p
        .sdb
        .map_tile
        .get(&(tile.tileset.clone(), tile.tileuid))
        .expect("Map references non-existent tileset+tileuid");

    // Check if this is a dynamic tile that already exists as a replicated entity.
    // When a client joins, server-replicated dynamic entities (doors, switches, etc.)
    // arrive before the map loads. Instead of spawning a duplicate, we attach components
    // to the existing entity using insert_if_new to preserve server-authoritative state.
    let stitch_tmx_id = TmxEntityId {
        layer_idx,
        x: tile.pos.x,
        y: tile.pos.y,
    };

    let stitch_entity = existing_tmx_map.get(&stitch_tmx_id).copied();

    let mut beh = mt.behavior.clone();

    // Calculate position on the map
    const MAP_MARGIN: i32 = 3;
    let t_x = (tile.pos.x - map_min_x) as f32;
    let t_y = (-tile.pos.y - map_min_y) as f32;

    // FIXME: Position is replicated meaning that the incoming position might be already be different.
    // ... This means that the BoardEntityField (p.bef) might be wrong.

    // Create position component with z-offset applied
    let mut pos = Position {
        x: t_x,
        y: t_y,
        z: floor_z as f32 + layer.z_offset, // Apply the z-offset directly
        visual_priority: 0.0,
    };

    // Handle special tile types based on utility (spawning separate entities)
    if matches!(&mt.behavior.p.util, Util::Van) {
        // Position for spawn points (slightly adjusted)
        let new_pos = Position {
            visual_priority: 0.0001,
            ..pos
        };
        commands.spawn((new_pos, VanEntryPoint, GameSprite));
    }

    // Helper to determine what we do with the stitch entity
    let (should_reuse, existing_to_reuse) = stitch_entity.map_or((false, None), |existing| {
        if scheduled_for_despawn.contains(&existing) {
            (false, Some(existing)) // Conflict: skip reuse, but keep around for logging
        } else {
            (true, Some(existing)) // Clean reuse
        }
    });

    // Spawn the base entity, or reuse existing replicated entity for stitch.
    let mut entity_commands = if should_reuse {
        let existing = existing_to_reuse.unwrap();
        trace!(
            "TILE_STITCH_REUSE: map={} entity={:?} tmx_id={:?} behavior={}",
            map_filepath,
            existing,
            stitch_tmx_id,
            beh.key_cvo().to_key_string()
        );
        if let Ok(server_pos) = q_positions.get(existing) {
            pos = *server_pos;
        } else {
            warn!(
                "TILE_STITCH_MISSING_POSITION: map={} entity={:?} tmx_id={:?} behavior={} has no Position at stitch time",
                map_filepath,
                existing,
                stitch_tmx_id,
                beh.key_cvo().to_key_string()
            );
        }
        commands.entity(existing)
    } else {
        if let Some(existing) = existing_to_reuse {
            warn!(
                "TILE_STITCH_CONFLICT_AVOIDED: map={} entity={:?} tmx_id={:?} behavior={} was scheduled for despawn. Spawning new entity instead to avoid panic.",
                map_filepath,
                existing,
                stitch_tmx_id,
                beh.key_cvo().to_key_string()
            );
        }
        commands.spawn_empty()
    };

    // Adjust the light receiving offset for flipped sprites.
    if tile.flip_x {
        let (ox, oy) = beh.p.display.light_recv_offset;
        beh.p.display.light_recv_offset = (ox, -oy);
    }

    entity_commands
        .insert(TileVisualRef {
            tileset: tile.tileset.clone(),
            tileuid: tile.tileuid,
            flip_x: tile.flip_x,
        })
        .insert(HydrationStage::<1>)
        .insert(PendingProperties(
            unbehavior::behavior::behavior_properties_from_layer(&layer.user_properties),
        ));

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

    // Ensure unique z-ordering within the same floor level
    *c += 0.000000001;
    pos.visual_priority = mt.behavior.p.display.visual_priority + *c;

    // Clone and configure behavior for this tile instance
    let mut beh = mt.behavior.clone();

    // Register the entity in the board's map entity field
    p.bef.0[pos.to_board_position().ndidx()].push(entity_commands.id());

    // Handle horizontal flipping for behavior
    beh.flip(tile.flip_x);

    // Add board position component
    entity_commands.insert_if_new(MapEntityFieldBPos(pos.to_board_position()));

    // Add standard components to all tile entities
    let transform = Transform::from_xyz(pos.x, pos.y, pos.visual_priority);

    // TODO: This is inserting Behavior which eventually we should replicate, when we do, we need to consider that
    // .. this code considers the map-loaded behavior to be authoritative, and that might be wrong.
    entity_commands.insert_if_new(beh.clone());

    // Determine if this entity is "dynamic" — needs network identity for replication.
    let is_dynamic = beh.p.is_replicated;

    if is_dynamic {
        let tmx_id = TmxEntityId {
            layer_idx,
            x: tile.pos.x,
            y: tile.pos.y,
        };
        entity_commands.insert_if_new(tmx_id);

        // Only the Authority (server / offline host) adds Replicated.
        if p.authority.is_some() {
            entity_commands.insert_if_new(Replicated);
        }
    }

    entity_commands
        .insert_if_new(GameSprite)
        .insert_if_new(MapTileSprite)
        // We intentionally insert `pos` here (even if it matches the server's replicated Position)
        // to guarantee `Changed<Position>` fires, triggering `apply_perspective` to compute the correct isometric Transform.
        .insert(pos)
        .insert(Visibility::Hidden)
        .insert(transform);
}
