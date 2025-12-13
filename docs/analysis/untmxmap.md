# Crate Analysis: `untmxmap`

## Core Responsibilities

`untmxmap` is the dedicated map loading engine for _Unhaunter_. Its primary responsibility is to bridge the gap between
the raw map data loaded by `uncore-assets` (as `.tmx` and `.tsx` files) and the runtime Bevy ECS. It uses the `tiled`
crate to parse the XML data and then converts it into game-ready entities, components, and resources.

## Public API

The crate exposes the following key components:

- **`plugin` Module**:

  - **`UnhaunterTmxMapPlugin`**: The main plugin that registers the `TmxMapLoader`, `TsxSheetLoader`, and
    `AssetIdxLoader` (which are defined in `uncore-assets` but initialized here). It also sets up the system to listen
    for `LoadLevelEvent`.

- **`map_loader` Module**:

  - **`UnhaunterMapLoader`**: A custom loader that implements `tiled::ResourceReader`. It intercepts file read requests
    from the `tiled` crate and serves data directly from the `TmxMap` and `TsxSheet` assets already loaded in memory by
    Bevy. This avoids double-loading files and works within Bevy's async asset system.

- **`bevy` Module**:

  - **`bevy_load_map`**: The core function that takes a parsed `tiled::Map` and generates the corresponding Bevy
    resources (like `TextureAtlasLayout`) and entities. It handles the creation of `CustomMaterial1` for tilesets and
    organizes layers.

- **`load_level` Module**:
  - **`load_level_handler`**: The system that reacts to `LoadLevelEvent`, triggers the loading process, and emits
    `LevelLoadedEvent` when complete.

## Dependencies

- `uncore-board`
- `uncore-assets`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `unstd`
- `bevy`
- `bevy_platform`
- `normalize-path`
- `tiled`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate has a clear purpose: "Load Tiled Maps into Bevy". It encapsulates
  the complexity of the `tiled` crate and the conversion logic.
- **Interface Segregation:** It consumes the generic `LoadLevelEvent` and produces `LevelLoadedEvent`, decoupling the
  _request_ to load a level from the _implementation_ of how it's loaded.

### 2D/3D Coupling

- **Conclusion:** **Extreme Coupling.**
- **Reasoning:**
  - **Dependency on `tiled`:** The entire crate is built around the `tiled` crate, which is designed for 2D
    orthogonal/isometric maps.
  - **2D Rendering Assets:** It generates `TextureAtlasLayout` and uses `CustomMaterial1` (from `unstd`), which are
    explicitly 2D rendering constructs.
  - **Layer Logic:** The logic for processing layers (`FloorLevel`, `MapLayer`) is derived from the 2D layer structure
    of Tiled.
  - **Pivot Implication:** In a 3D pivot, this crate would likely be replaced entirely by a 3D scene loader (e.g., for
    GLTF) or heavily modified to treat Tiled maps as merely a layout guide for 3D assets (e.g., placing 3D meshes where
    2D tiles are).

### Game Logic vs. Engine Logic

- **Conclusion:** **Engine Implementation**.
- **Reasoning:** This is a classic "Map Loader" subsystem of a game engine. It contains no gameplay rules (like ghost
  behavior or player stats), only the logic to instantiate the world from data.

### Other Notes

- **Memory Loading Strategy:** The `TmxMemoryReader` implementation in `map_loader.rs` is a clever architectural
  pattern. It allows the game to leverage Bevy's asset management (loading bytes asynchronously) while still using the
  synchronous `tiled` parser, without hitting the disk again. This is particularly important for WASM support where
  synchronous file I/O is problematic or impossible.
