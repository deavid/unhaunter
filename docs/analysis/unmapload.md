# Crate Analysis: `unmapload`

## Core Responsibilities

`unmapload` is the high-level orchestrator for initializing a game level in _Unhaunter_. While `untmxmap` handles the
raw data extraction from Tiled files, `unmapload` takes that data and constructs the actual game state. It is
responsible for:

- **Level Lifecycle Management:** Cleaning up the previous level (despawning entities) and initializing the new one.
- **Entity Spawning:** Instantiating all game entities, including map tiles, players, ghosts, items, and ambient sounds,
  based on the map data.
- **Board Initialization:** Populating the `BoardData` resource with collision, lighting, temperature, and sound fields.
- **Room Management:** Analyzing the map to define rooms and their properties (`RoomDB`).
- **Game Logic Setup:** Configuring initial game state, such as ghost influence areas and difficulty settings.

## Public API

The crate exposes:

- **`UnhaunterMapLoadPlugin`**: The main plugin that registers the `load_level_handler` system and other
  setup/finalization systems.
- **`LoadLevelSystemParam`**: A complex `SystemParam` struct that aggregates the multitude of resources (assets, board
  data, difficulty, settings) required to load a level. This simplifies the function signatures of the loading systems.

## Dependencies

- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `unstd`
- `undifficulty`
- `unlight`
- `ungear`
- `ungearitems`
- `unsettings`
- `bevy`
- `bevy_persistent`
- `ndarray`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate focuses on the "Level Loading" phase. It effectively delegates
  sub-tasks to internal modules (`entity_spawning`, `tile_spawning`, `level_finalization`), which keeps the main handler
  readable.
- **Coupling:** As a "Builder" or "Factory" for the game state, this crate is inherently coupled to almost every other
  part of the system. It needs to know about physics, rendering, AI, and items to instantiate them. This is expected for
  this specific role.

### 2D/3D Coupling

- **Conclusion:** **High Coupling.**
- **Reasoning:**
  - **Tiled Integration:** The loading logic is driven by iterating over Tiled map layers (`MapLayerType`) and tiles,
    which are 2D concepts.
  - **Sprite-Based Rendering:** It uses `SpriteDB` and `CustomMaterial1` to create visual entities, which are explicitly
    2D.
  - **Grid Logic:** It populates `BoardData` and `CollisionFieldData` based on a grid structure derived from the 2D map
    layout.
  - **Pivot Implication:** Moving to 3D would require rewriting this crate to spawn 3D meshes, colliders, and lights
    instead of sprites and 2D grid data. The logic for _what_ to spawn (e.g., "spawn a ghost here") might remain, but
    _how_ it is spawned will change completely.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic**.
- **Reasoning:** This crate defines _how_ an _Unhaunter_ level is put together. It decides that a specific Tiled object
  property means "spawn a ghost" or "this is a hiding spot." It applies the specific rules of the game to the generic
  data provided by the engine.

### Other Notes

- **Performance:** The use of `ndarray` for field data suggests a focus on efficient data storage and access for the
  game board.
- **Modularity:** The separation of `entity_spawning` and `tile_spawning` is a good design choice, making it easier to
  add new entity types without cluttering the main loading logic.
