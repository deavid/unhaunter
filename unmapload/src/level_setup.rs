//! # Level Setup Module
//!
//! This module handles the core level initialization and setup process.
//! It contains the main level loading handler and defines the primary system parameters needed for level loading.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy::utils::HashMap;
use bevy_persistent::Persistent;
use ndarray::Array3;
use uncore::components::board::position::Position;
use uncore::components::game::{GameSound, GameSprite};
use uncore::difficulty::CurrentDifficulty;
use uncore::events::loadlevel::{LevelLoadedEvent, LevelReadyEvent};
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::types::board::fielddata::{CollisionFieldData, LightFieldData};
use uncore::types::root::game_assets::GameAssets;
use uncore::types::tiledmap::map::MapLayerType;
use unstd::board::spritedb::SpriteDB;
use unstd::materials::CustomMaterial1;
use unstd::tiledmap::MapTileSetDb;

use crate::entity_spawning;
use crate::sprite_db;
use crate::tile_spawning;

/// System parameter for loading levels, providing access to various resources.
///
/// This struct contains references to all resources needed throughout the level loading process:
/// - Core data resources (BoardData, RoomDB, SpriteDB, etc.)
/// - Asset handling resources (AssetServer, Meshes, Materials, etc.)
/// - Game configuration resources (Difficulty, Controls, Audio settings)
///
/// Using this as a system parameter simplifies function signatures throughout the level loading process.
#[derive(SystemParam)]
pub struct LoadLevelSystemParam<'w> {
    pub asset_server: Res<'w, AssetServer>,
    pub bf: ResMut<'w, BoardData>,
    pub materials1: ResMut<'w, Assets<CustomMaterial1>>,
    pub texture_atlases: Res<'w, Assets<TextureAtlasLayout>>,
    pub meshes: ResMut<'w, Assets<Mesh>>,
    pub tilesetdb: Res<'w, MapTileSetDb>,
    pub sdb: ResMut<'w, SpriteDB>,
    pub handles: Res<'w, GameAssets>,
    pub roomdb: ResMut<'w, RoomDB>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub audio_settings: Res<'w, Persistent<unsettings::audio::AudioSettings>>,
    pub control_settings: Res<'w, Persistent<unsettings::controls::ControlKeys>>,
}

/// Marker component to handle ghost influence assignment after level loading is complete
#[derive(Component)]
pub struct AssignGhostInfluenceMarker(pub Vec<Entity>);

/// Loads a new level based on the `LevelLoadedEvent`.
///
/// This function is the primary handler for level loading in the game. It:
/// - Cleans up existing game entities
/// - Processes map data to determine size and floor levels
/// - Initializes field data (temperature, collision, lighting, etc.)
/// - Spawns all tile entities, special entities, and ambient sounds
/// - Coordinates cross-entity systems like ghost influence
///
/// # Arguments
/// * `ev` - Event reader for LevelLoadedEvent
/// * `commands` - Command buffer for entity operations
/// * `qgs` - Query for existing game sprites to despawn
/// * `qgs2` - Query for existing game sounds to despawn
/// * `p` - Level system parameters containing all needed resources
/// * `ev_level_ready` - Event writer to signal when level is ready
#[allow(clippy::too_many_arguments)]
pub fn load_level_handler(
    mut ev: EventReader<LevelLoadedEvent>,
    mut commands: Commands,
    qgs: Query<Entity, With<GameSprite>>,
    qgs2: Query<Entity, With<GameSound>>,
    mut p: LoadLevelSystemParam,
    mut ev_level_ready: EventWriter<LevelReadyEvent>,
) {
    // Get the loaded event or return early if none
    let mut ev_iter = ev.read();
    let Some(loaded_event) = ev_iter.next() else {
        return;
    };
    let layers = &loaded_event.layers;
    let floor_mapping = &loaded_event.floor_mapping;

    // Despawn existing game entities
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }

    // Despawn existing ambient sounds
    for gs in qgs2.iter() {
        commands.entity(gs).despawn_recursive();
    }

    // Set temperature from difficulty
    p.bf.ambient_temp = p.difficulty.0.ambient_temperature;

    // Compute map boundaries by examining all tiles
    let mut map_min_x = i32::MAX;
    let mut map_min_y = i32::MAX;
    let mut map_max_x = i32::MIN;
    let mut map_max_y = i32::MIN;

    // Filter for tile layers and find min/max coordinates
    for (maptiles, _layer) in layers.iter().filter_map(|(_, layer)| {
        if let MapLayerType::Tiles(tiles) = &layer.data {
            Some((tiles, layer))
        } else {
            None
        }
    }) {
        for tile in &maptiles.v {
            map_min_x = tile.pos.x.min(map_min_x);
            map_min_y = (-tile.pos.y).min(map_min_y);
            map_max_x = tile.pos.x.max(map_max_x);
            map_max_y = (-tile.pos.y).max(map_max_y);
        }
    }

    // Add margin for neighbor checking
    const MAP_MARGIN: i32 = 3;
    map_min_x -= MAP_MARGIN;
    map_min_y -= MAP_MARGIN;

    // Use floor mapping from event
    p.bf.floor_z_map = floor_mapping.floor_to_z.clone();
    p.bf.z_floor_map = floor_mapping.z_to_floor.clone();

    // Log floor mapping details
    warn!(
        "Floor mapping from event: {:?} floors found",
        p.bf.floor_z_map.len()
    );
    warn!("Floor z-map: {:?}", p.bf.floor_z_map);
    warn!("Z-floor map: {:?}", p.bf.z_floor_map);

    // Calculate final map size with margins
    let map_size = (
        (map_max_x - map_min_x + 1 + MAP_MARGIN) as usize,
        (map_max_y - map_min_y + 1 + MAP_MARGIN) as usize,
        p.bf.floor_z_map.len(), // Use the number of floors for the z dimension
    );

    info!("Map size: ({map_min_x},{map_min_y}) - ({map_max_x},{map_max_y}) - {map_size:?}");

    // Initialize board data fields
    p.bf.map_size = map_size;
    p.bf.origin = (map_min_x, map_min_y, 0);
    p.bf.temperature_field = Array3::from_elem(map_size, p.bf.ambient_temp);
    p.bf.collision_field = Array3::from_elem(map_size, CollisionFieldData::default());
    p.bf.light_field = Array3::from_elem(map_size, LightFieldData::default());
    p.bf.miasma.pressure_field = Array3::from_elem(map_size, 0.0);
    p.bf.miasma.velocity_field = Array3::from_elem(map_size, Vec2::ZERO);
    p.bf.map_entity_field = Array3::default(map_size);

    // Clear other field data
    p.bf.sound_field.clear();
    p.bf.current_exposure = 10.0;
    p.roomdb.room_state.clear();
    p.roomdb.room_tiles.clear();

    // Spawn ambient sound entities
    entity_spawning::spawn_ambient_sounds(&p, &mut commands);

    // Initialize board data resource
    commands.init_resource::<BoardData>();
    warn!("Level Loaded: {}", &loaded_event.map_filepath);

    // ---------- NEW MAP LOAD ----------
    // Create containers for different entity types
    let mut player_spawn_points: Vec<Position> = vec![];
    let mut ghost_spawn_points: Vec<Position> = vec![];
    let mut van_entry_points: Vec<Position> = vec![];
    let mut mesh_tileset = HashMap::<String, Handle<Mesh>>::new();

    // Clear the sprite database
    p.sdb.clear();

    // Populate sprite database with tile data
    sprite_db::populate_sprite_db(&mut p, &mut mesh_tileset);

    // Process map tiles and spawn entities
    let mut c: f32 = 0.0;
    let mut movable_objects: Vec<Entity> = Vec::new();

    // Process each tile layer
    for (maptiles, layer) in layers.iter().filter_map(|(_, layer)| {
        if let MapLayerType::Tiles(tiles) = &layer.data {
            Some((tiles, layer))
        } else {
            None
        }
    }) {
        // Get floor z-index from the layer's floor_number
        let floor_z = if let Some(floor_num) = layer.floor_number {
            p.bf.floor_z_map.get(&floor_num).copied().unwrap_or(0)
        } else {
            0 // Default to ground floor if no floor number is set
        };

        // Process each tile in the layer
        for tile in &maptiles.v {
            tile_spawning::process_and_spawn_tile(
                tile,
                layer,
                map_min_x,
                map_min_y,
                map_size,
                floor_z,
                &mut p,
                &mut commands,
                &mut player_spawn_points,
                &mut ghost_spawn_points,
                &mut van_entry_points,
                &mut movable_objects,
                &mut c,
            );
        }
    }

    // Validate map for ghost influence system
    if movable_objects.len() < 3 {
        warn!(
            "Map has less than 3 movable objects in rooms. Ghost influence system might not work as intended."
        );
    }

    // Schedule ghost influence assignment for after level loading
    let movable_objects_clone = movable_objects.clone();
    commands.spawn(AssignGhostInfluenceMarker(movable_objects_clone));

    // Spawn player entity
    let open_van = entity_spawning::spawn_player(
        &p,
        &mut commands,
        &mut player_spawn_points,
        &van_entry_points,
    );

    // Spawn ghost and breach entities
    entity_spawning::spawn_ghosts(&mut p, &mut commands, &mut ghost_spawn_points);

    // Send level ready event
    ev_level_ready.send(LevelReadyEvent { open_van });
    warn!("Done: load_level_handler");
}
