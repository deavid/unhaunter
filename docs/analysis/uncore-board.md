# Crate Analysis: `uncore-board`

## Core Responsibilities

This crate is the heart of the game's spatial and environmental logic. It defines how the game world (the "board") is structured, how objects behave within it, how light propagates, and how map data from the Tiled editor is processed and represented in-game. It forms the backbone for navigation, collision, and basic physical interactions.

## Public API

The crate exposes three main modules:

### `behavior` Module
This module is central to how entities are configured from map data.
-   **`Behavior` (Component)**: The primary component in this module, it holds a `SpriteConfig` (private) and `Properties` (public). It links Tiled map properties to in-game component attachments and behaviors.
-   **`Properties`**: Aggregates `Movement`, `Light`, `Util`, `Display`, and `Object` structs, which define granular behaviors like `walkable`, `opaque`, `pickable`, `throwable`, `global_z` adjustment, etc.
-   **`Class` (Enum)**: Categorizes map objects (e.g., `Floor`, `Wall`, `Door`, `Switch`, `NPC`, `VanEntry`, `StairsUp`).
-   **`Orientation` (Enum)**: Defines object orientation (e.g., `XAxis`, `YAxis`, `Both`, `None`). Used heavily in 2D isometric context (e.g., for door pivot).
-   **`TileState` (Enum)**: Defines object states (e.g., `On`/`Off` for switches, `Open`/`Closed` for doors).
-   **`SpriteConfig`**: Stores parsed Tiled properties (`class`, `variant`, `orientation`, `state`, `tileset`, `tileuid`) and is responsible for attaching appropriate Bevy components to entities based on these properties (e.g., `Collision`, `Interactive`, `Light`, `Door`).
-   **`BehaviorProperties`**: Utility for extracting typed properties directly from `tiled::PropertyValue` objects.
-   **ECS Components (from `behavior/component.rs`)**:
    -   `Ground`, `Collision`, `Opaque`, `UVSurface`, `Light`, `Door`, `Stairs`, `FloorItemCollidable`: Marker/data components for physical attributes.
    -   `InteractableByGhost`: Marker for entities ghosts can interact with.
    -   `RoomState`: Component for room-specific logic.
    -   `Interactive`: For interactable objects, including sound files and a `control_point_delta` for interaction points on doors.
    -   `NpcHelpDialog`: For NPC dialogue sourced from map properties.

### `components` Module (Duplicate)
This module contains core spatial components that are **duplicated** in `uncore-components/src/components/board/`.
-   **`BoardPosition` (Component)**: Integer (i64) 3D grid coordinates, fundamental for map indexing and grid-based logic. It also exposes `size_x_meters`, `size_y_meters`, `size_z_meters` for tile dimensions.
-   **`ChunkIterator`, `CellIterator`**: Iterators for spatial partitioning of the map into chunks and individual cells.
-   **`Direction` (Component)**: Floating-point (f32) 3D vector. Critically, it contains `to_screen_coord` that uses isometric projection constants to convert 3D direction vectors to 2D screen coordinates.
-   **`MapColor` (Component)**: Simple color for map tiles.
-   **`Position` (Component)**: Floating-point (f32) 3D logical position. Contains `to_screen_coord` which uses isometric projection constants, directly coupling it to the 2D isometric view. Includes `global_z` for fine-tuning vertical rendering in 2D.

### `types` Module
This module defines various data structures related to map data and lighting.
-   **`fielddata.rs`**: `LightFieldData` and `CollisionFieldData` for representing cell properties in the light and collision simulation grids.
-   **`light.rs`**: `LightType` (enum: Visible, Red, InfraRedNV, UltraViolet) and `LightData` (intensity for each type).
-   **`prebaked_lighting_data.rs`**: Structures (`PrebakedLightingData`, `WaveEdge`, `LightInfo`, `PrebakedMetadata`) for storing precomputed lighting information to optimize runtime light propagation, especially for dynamic light sources.
-   **`quadcc.rs`**: **(2D Coupling)** Defines `QuadCC` which is a 2D rectangle that implements `From<QuadCC> for Mesh`, explicitly generating 2D meshes for rendering.
-   **`tiledmap` Module**:
    -   **`MapTile`**: Represents a single tile, includes `flip_x` for 2D rendering.
    -   **`MapLayer`**: Represents a Tiled map layer, including `floor_number`, `parent_floor_name`, and `z_offset` for vertical layering.
    -   **`MapLayerGroup`**: For recursive iteration over map layers.

## Dependencies

-   `uncore-foundation`
-   `bevy`
-   `bevy_platform`
-   `serde`
-   `serde_json`
-   `fastapprox`
-   `ordered-float`
-   `rand`
-   `tiled`
-   `anyhow`

## Architectural Analysis & Notes

### SOLID Principles
-   **Single Responsibility Principle (SRP):**
    -   The `behavior` module is quite cohesive, focusing on translating Tiled map properties into entity components and behaviors.
    -   The `components` and `types` modules are also reasonably focused on spatial primitives and map/lighting data, respectively.
    -   The major SRP violation is the duplication of core spatial components (`BoardPosition`, `Position`, `Direction`, etc.) between `uncore-board/src/components/` and `uncore-components/src/components/board/`. This needs to be resolved.

### 2D/3D Coupling
-   **Conclusion:** **Very High Coupling.**
-   **Reasoning:** This crate is profoundly coupled to the 2D isometric rendering pipeline:
    -   **Isometric Projection Constants:** Defined directly in `src/components/mod.rs` and used in `Direction` and `Position` for `to_screen_coord` conversions. This is the cornerstone of the isometric view.
    -   **Tile Dimensions:** Hardcoded `size_x_meters`, `size_y_meters`, `size_z_meters` in `BoardPosition` reflect a specific grid layout.
    -   **`z_offset` in `MapLayer` and `global_z` in `Position`**: These fields are explicitly designed to manage vertical layering and drawing order in a 2D isometric representation, not true 3D depth.
    -   **`MapTile::flip_x`**: Directly relates to 2D sprite rendering.
    -   **`quadcc.rs`**: Explicitly generates 2D meshes.
    -   **Tiled Map Integration**: While Tiled can define 3D maps, its use here with `MapLayer` and its properties suggests an interpretation geared towards 2D isometric.
-   A pivot to 3D would necessitate a complete re-evaluation and likely replacement of the `to_screen_coord` methods, the interpretation of Z-coordinates, and the `quadcc` mesh generation. The core `BoardPosition` and `Position` would need to shed their 2D-specific projection logic.

### Game Logic vs. Engine Logic
-   This crate leans heavily towards **Engine-level Concepts** for a grid-based, Tiled-map-driven game, but with significant game-specific influences due to the 2D isometric assumptions.
    -   **Engine-level:** The concepts of `BoardPosition`, `Position` (as generic 3D points), `Direction`, `ChunkIterator`, and generic `LightType` are reusable. The `Behavior` system for deriving components from map properties is also a powerful engine feature.
    -   **Game-specific:** The hardcoded isometric projection logic, `Class` enum (specific object types in *Unhaunter*), and detailed light parameters in `Properties` are more tied to this particular game.
-   To generalize this crate for a "ghost game engine," the 2D isometric coupling needs to be extracted, and the `Class` enum might need to become more extensible or generic.

### Other Notes
-   **File Duplication:** The most critical architectural issue found is the **duplication of core spatial components** (`BoardPosition`, `Position`, `Direction`, `ChunkIterator`, `MapColor`) between `uncore-board/src/components/` and `uncore-components/src/components/board/`. This is a clear violation of DRY (Don't Repeat Yourself) and SRP, and it needs to be resolved immediately during refactoring. It creates confusion, maintenance headaches, and potential for bugs.
-   **Tiled Integration:** The deep integration with the `tiled` crate and its `PropertyValue` system is robust, allowing for rich map-driven entity configuration.
-   **Lighting System:** The presence of `prebaked_lighting_data.rs` indicates a sophisticated, optimized lighting system, which is a valuable engine feature.
