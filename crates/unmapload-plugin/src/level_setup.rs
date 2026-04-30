//! # Level Setup Module
//!
//! This module handles the core level initialization and setup process.
//! It contains the main level loading handler and defines the primary system parameters needed for level loading.

use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use bevy_platform::collections::{HashMap, HashSet};
use bevy_replicon::prelude::Remote;
use ndarray::Array3;
use unboard_core::entity::GameSprite;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unboard_core::resources::roomdb::{RoomStateMap, RoomTopology};
use unboard_core::types::fielddata::CollisionFieldData;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unmapload_core::events::loadlevel::LevelLoadedEvent;
use unmapload_core::resources::SpriteDB;
use unmission_core::events::MapGeometryInitializedEvent;
use unmission_core::types::SimulationState;
use unreplicon_core::resources::AuthorityRole;
use unspatial_core::position::Position;
use untmxmap_core::events::LevelDataEvent;
use untmxmap_core::tiled::MapTileSetDb;
use untmxmap_core::types::map::MapLayerType;

use crate::resources::LevelLoadingStatus;
use crate::sprite_db;
use crate::tile_spawning;

/// System parameter for loading levels, providing access to various resources.
///
/// This struct contains references to all resources needed throughout the level loading process:
/// - Core data resources (BoardTopology, RoomTopology, RoomStateMap, SpriteDB, etc.)
/// - Asset handling resources (AssetServer, Meshes, Materials, etc.)
/// - Game configuration resources (Difficulty, Controls, Audio settings)
///
/// Using this as a system parameter simplifies function signatures throughout the level loading process.
#[derive(SystemParam)]
pub(crate) struct LoadLevelSystemParam<'w, 's> {
    pub bf: ResMut<'w, BoardTopology>,
    pub bef: ResMut<'w, BoardEntityField>,
    pub bcf: ResMut<'w, BoardCollisionField>,
    pub tilesetdb: Res<'w, MapTileSetDb>,
    pub sdb: ResMut<'w, SpriteDB>,
    pub roomtopo: ResMut<'w, RoomTopology>,
    pub roomstate: ResMut<'w, RoomStateMap>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub loading_status: ResMut<'w, LevelLoadingStatus>,
    pub authority: Option<Res<'w, AuthorityRole>>,
    pub existing_tmx_entities:
        Query<'w, 's, (Entity, &'static unbehavior_core::components::TmxEntityId)>,
}

/// Loads a new level based on the `LevelLoadedEvent`.
///
/// This function is the primary handler for level loading in the game. It:
/// - Cleans up existing game entities
/// - Processes map data to determine size and floor levels
/// - Initializes field data (temperature, collision, lighting, etc.)
/// - Spawns all tile entities and special entities
///
/// # Arguments
/// * `ev` - Event reader for LevelLoadedEvent
/// * `commands` - Command buffer for entity operations
/// * `qgs` - Query for existing game sprites to despawn
/// * `p` - Level system parameters containing all needed resources
/// * `ev_geometry_init` - Event writer to signal when level geometry is ready
fn load_level_handler(
    mut ev: MessageReader<LevelDataEvent>,
    mut commands: Commands,
    qgs: Query<Entity, (With<GameSprite>, Without<Remote>)>,
    q_remote_tmx: Query<Entity, (With<unbehavior_core::components::TmxEntityId>, With<Remote>)>,
    q_positions: Query<&Position>,
    mut p: LoadLevelSystemParam,
    mut ev_geometry_init: MessageWriter<MapGeometryInitializedEvent>,
    mut ev_level_loaded: MessageWriter<LevelLoadedEvent>,
    time: Res<Time>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
    ui_state: Res<State<UIContextState>>,
    sim_state: Res<State<SimulationState>>,
    maps: Option<Res<untmxmap_core::resources::maps::Maps>>,
) {
    // Get the loaded event or return early if none
    let Some(loaded_event) = ev.read().last() else {
        return;
    };

    let local_gamesprites = qgs.iter().count();
    let remote_tmx_entities = q_remote_tmx.iter().count();
    let existing_tmx_count = p.existing_tmx_entities.iter().count();
    let is_authority = p.authority.is_some();

    // Log WHO is loading, WHEN (state), and WHAT entities exist at load start.
    // This distinguishes: dedicated server vs client, first vs second mission,
    // and whether zombie entities survived from a previous load.
    if local_gamesprites > 0 {
        warn!(
            "LOAD_HANDLER_CONTEXT: map={} ui_state={:?} sim_state={:?} authority={} local_gamesprites={} remote_tmx={} existing_tmx={} — local GameSprites exist at load start, expected 0",
            loaded_event.map_filepath,
            ui_state.get(),
            sim_state.get(),
            is_authority,
            local_gamesprites,
            remote_tmx_entities,
            existing_tmx_count
        );
    } else {
        info!(
            "LOAD_HANDLER_CONTEXT: map={} ui_state={:?} sim_state={:?} authority={} local_gamesprites=0 remote_tmx={} existing_tmx={}",
            loaded_event.map_filepath,
            ui_state.get(),
            sim_state.get(),
            is_authority,
            remote_tmx_entities,
            existing_tmx_count
        );
    }

    info!("Starting level load: {}", loaded_event.map_filepath);

    next_sim_state.set(SimulationState::Loading);
    *p.loading_status = LevelLoadingStatus::JustStarted;

    // --- 1. Cleanup & Reset ---
    // Despawn existing game entities (except remotely replicated!)
    let scheduled_for_despawn: HashSet<Entity> = qgs.iter().collect();
    if scheduled_for_despawn.is_empty() {
        debug!(
            "LEVEL_LOAD_CLEANUP_BEGIN: map={} scheduled_game_sprite_despawns=0 existing_tmx_entities_before_cleanup={}",
            loaded_event.map_filepath, existing_tmx_count
        );
    } else {
        warn!(
            "LEVEL_LOAD_ZOMBIE_GAMESPRITES: map={} {} local GameSprite entities scheduled for despawn at load start",
            loaded_event.map_filepath,
            scheduled_for_despawn.len()
        );
    }
    for entity in &qgs {
        commands.entity(entity).despawn();
    }

    // Reset core data structures
    p.roomstate.room_state.clear();
    p.roomtopo.room_tiles.clear();
    p.sdb.clear();

    // --- 2. Map Geometry Calculation ---
    // Calculate the bounding box of all tiles in the map to determine dimensions.
    // Note: Tiled Y coordinates are negated to match our internal isometric coordinate system.
    let tile_layers_iter = || {
        loaded_event.layers.iter().filter_map(|(_, layer)| {
            if let MapLayerType::Tiles(tiles) = &layer.data {
                Some((tiles, layer))
            } else {
                None
            }
        })
    };

    let (min_x, min_y, max_x, max_y) = tile_layers_iter()
        .flat_map(|(tiles, _)| tiles.v.iter())
        .fold(
            (i32::MAX, i32::MAX, i32::MIN, i32::MIN),
            |(mix, miy, max, may), tile| {
                (
                    mix.min(tile.pos.x),
                    miy.min(-tile.pos.y),
                    max.max(tile.pos.x),
                    may.max(-tile.pos.y),
                )
            },
        );

    // Add margin for neighbor checking and visual padding
    const MARGIN: i32 = 3;
    let origin = (min_x - MARGIN, min_y - MARGIN, 0);
    let map_size = (
        (max_x - min_x + 1 + 2 * MARGIN) as usize,
        (max_y - min_y + 1 + 2 * MARGIN) as usize,
        loaded_event.floor_mapping.floor_to_z.len(),
    );

    debug!(
        "Level geometry initialized: size {:?}, origin {:?}",
        map_size, origin
    );

    // --- 3. Topology & Resource Setup ---
    {
        let bf = &mut p.bf;
        bf.map_size = map_size;
        bf.origin = origin;
        bf.map_path = loaded_event.map_filepath.clone();
        bf.ambient_temp = p.difficulty.0.ambient_temperature();
        bf.level_ready_time = time.elapsed_secs();
        bf.floor_z_map = loaded_event.floor_mapping.floor_to_z.clone();
        bf.z_floor_map = loaded_event.floor_mapping.z_to_floor.clone();
        bf.floor_mapping = loaded_event.floor_mapping.clone();

        // Populate mission financial data from the Maps index so that T2 orchestrator
        // can copy it into SummaryData without importing a T3 type directly.
        if let Some(ref maps) = maps {
            if let Some(map_entry) = maps
                .maps
                .iter()
                .find(|m| m.path == loaded_event.map_filepath)
            {
                bf.mission_reward_base = map_entry.mission_data.mission_reward_base;
                bf.required_deposit = map_entry.mission_data.required_deposit;
            } else {
                warn!(
                    "level_setup: No Maps entry found for map path '{}'; mission_reward_base and required_deposit will default to 0",
                    loaded_event.map_filepath
                );
            }
        } else {
            warn!(
                "level_setup: Maps resource not available; mission_reward_base and required_deposit will default to 0"
            );
        }

        // Re-allocate board data fields with calculated dimensions
        p.bcf.0 = Array3::from_elem(map_size, CollisionFieldData::default());
        p.bef.0 = Array3::default(map_size);
    }

    // Broadcast geometry for other systems (minimaps, light grids, etc.)
    ev_geometry_init.write(MapGeometryInitializedEvent { map_size, origin });

    // --- 4. Asset Preparation ---
    sprite_db::populate_sprite_db(&mut p);

    // --- 5. Entity Spawning ---
    let existing_tmx_map: HashMap<_, _> = p
        .existing_tmx_entities
        .iter()
        .map(|(e, id)| (id.clone(), e))
        .collect();

    let stale_stitch_candidates: Vec<_> = existing_tmx_map
        .iter()
        .filter_map(|(tmx_id, entity)| {
            scheduled_for_despawn
                .contains(entity)
                .then_some((tmx_id.clone(), *entity))
        })
        .collect();

    if stale_stitch_candidates.is_empty() {
        debug!(
            "LEVEL_LOAD_STITCH_SCAN: map={} existing_tmx_entities={} stale_candidates=0",
            loaded_event.map_filepath,
            existing_tmx_map.len()
        );
    } else {
        warn!(
            "LEVEL_LOAD_STITCH_SCAN: map={} existing_tmx_entities={} stale_candidates={} entities scheduled for despawn are still available for Tmx stitching in this load",
            loaded_event.map_filepath,
            existing_tmx_map.len(),
            stale_stitch_candidates.len()
        );
        for (tmx_id, entity) in stale_stitch_candidates.iter().take(12) {
            warn!(
                "LEVEL_LOAD_STALE_STITCH_CANDIDATE: map={} entity={:?} tmx_id={:?}",
                loaded_event.map_filepath, entity, tmx_id
            );
        }
    }

    let mut depth_counter = 0.0;
    for (layer_idx, (maptiles, layer)) in tile_layers_iter().enumerate() {
        // Get floor z-index from the layer's floor_mapping
        let floor_z = layer
            .floor_number
            .and_then(|num| p.bf.floor_z_map.get(&num))
            .copied()
            .unwrap_or(0);

        for tile in &maptiles.v {
            tile_spawning::process_and_spawn_tile(
                tile,
                layer,
                layer_idx,
                origin.0,
                origin.1,
                map_size,
                floor_z,
                &mut p,
                &mut commands,
                &mut depth_counter,
                &existing_tmx_map,
                &scheduled_for_despawn,
                &q_positions,
                &loaded_event.map_filepath,
            );
        }
    }

    debug!("Map spawning complete: {}", loaded_event.map_filepath);
    ev_level_loaded.write(LevelLoadedEvent);
}

pub(crate) fn reset_level_resources(
    mut roomtopo: ResMut<RoomTopology>,
    mut roomstate: ResMut<RoomStateMap>,
    mut sdb: ResMut<SpriteDB>,
    mut next_sim_state: ResMut<NextState<SimulationState>>,
) {
    roomtopo.reset();
    roomstate.reset();
    sdb.clear();
    next_sim_state.set(SimulationState::Unloaded);
}

fn teardown_map_entities(
    mut commands: Commands,
    qgs: Query<Entity, (With<GameSprite>, Without<Remote>)>,
    mut bf: ResMut<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut bef: ResMut<BoardEntityField>,
) {
    for entity in qgs.iter() {
        commands.entity(entity).despawn();
    }

    bf.reset();
    bcf.reset();
    bef.reset();

    debug!("Map entities torn down and topological resources reset.");
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        OnExit(UIContextState::InGame),
        reset_level_resources,
    );
    app.add_systems(OnEnter(SimulationState::Unloaded), teardown_map_entities);
    app.add_systems(
        PostUpdate,
        load_level_handler.run_if(bevy::prelude::on_message::<LevelDataEvent>),
    );
}
