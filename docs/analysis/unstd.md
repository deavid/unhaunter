# Crate Analysis: `unstd`

## Core Responsibilities

`unstd` acts as a "standard library" or "core implementation" crate for the _Unhaunter_ game. It aggregates foundational
plugins, resources, and systems to set up the Bevy application. It bridges the gap between the abstract definitions in
`uncore-*` crates and the concrete runtime environment. Its responsibilities include:

- **Application Setup:** Initializing the Bevy app, states (`AppState`, `GameState`), and platform-specific settings
  (e.g., frame limiting).
- **Asset Loading:** Centralized loading of global assets like fonts, UI images, and character sprites.
- **Board Rendering Pipeline:** Implementing the systems that translate logical board positions into visual 2D screen
  coordinates (isometric projection).
- **Tile Management:** Managing Tiled map resources (`MapTileSetDb`) and optimizing tile component creation
  (`SpriteDB`).
- **Custom Materials:** Defining custom 2D shaders and materials (`CustomMaterial1`).

## Public API

The crate exposes several key modules and plugins:

- **`plugins` Module**:

  - **`UnhaunterRootPlugin`**: The primary entry point for initializing the game's core loop. It sets up states, loads
    assets, and handles platform differences (WASM vs. Native).
  - **`UnhaunterBoardPlugin`**: Initializes board-related resources (`BoardData`, `VisibilityData`, `SpriteDB`,
    `RoomDB`) and adds the `apply_perspective` system.

- **`board` Module**:

  - **`SpriteDB`**: A resource that caches pre-built Bevy components for map tiles to optimize map loading. It indexes
    tiles by their visual characteristics (Class, Variant, Orientation).

- **`materials` Module**:

  - **`CustomMaterial1`**: A custom `Material2d` implementation with a specialized shader, likely for handling sprite
    sheet effects or lighting.

- **`tiledmap` Module**:
  - **`MapTileSetDb`**: A resource storing loaded Tiled tilesets (`MapTileSet`), bridging the raw asset data with the
    game's rendering system.

## Dependencies

- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-systems`
- `undifficulty`
- `bevy`
- `bevy_platform` (custom wrapper/fork?)
- `bevy_picking`
- `bevy_framepace`
- `tiled`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is somewhat of a "glue" crate, aggregating various concerns
  (setup, assets, rendering). While `UnhaunterRootPlugin` and `UnhaunterBoardPlugin` separate concerns at the plugin
  level, the crate itself is a collection of "standard" implementations.
- **Dependency Inversion:** It depends on all `uncore-*` crates, which is expected for an implementation layer.

### 2D/3D Coupling

- **Conclusion:** **Very High Coupling.**
- **Reasoning:**
  - **Rendering Pipeline:** The `apply_perspective` system in `UnhaunterBoardPlugin` directly calls
    `pos.to_screen_coord()`, enforcing the 2D isometric projection for all entities with a `Position`.
  - **Materials:** `CustomMaterial1` is explicitly a `Material2d` and contains logic for sprite sheets (`sheet_rows`,
    `sheet_cols`).
  - **Map System:** The `tiledmap` and `board` modules are deeply integrated with the Tiled map editor workflow and 2D
    tile concepts. `SpriteDB` is designed around 2D sprite reuse.
  - **Asset Loading:** The `load_assets` system hardcodes the loading of 2D sprite sheets and texture atlases.

### Game Logic vs. Engine Logic

- **Conclusion:** Mixed, but primarily **Engine Implementation**.
- **Reasoning:**
  - The setup of plugins, states, and asset loaders is generic engine work.
  - The specific assets loaded (e.g., "manual/images/chapter1") are game-specific content.
  - The board rendering and tile management are part of the "2D Isometric Ghost Engine."

### Other Notes

- **Platform Abstraction:** The `arch_setup` module demonstrates good practice by isolating platform-specific code (like
  `bevy_framepace` for native) using `#[cfg]` attributes.
- **Optimization:** `SpriteDB` shows a focus on performance by caching components to avoid re-creating them for every
  tile, which is crucial for large tile-based maps.
