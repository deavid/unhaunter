# Project Index for AI Assistants

- **Purpose:** Outline file/directory roles in the `unhaunter` project. Aids code navigation and modification
  understanding.
- **"Feature X Relevance":** Indicates files likely touched when adding features related to their described purpose.

## General Project Files

- `.cargo/config.toml`: Cargo target-specific build flags.
- `Cargo.toml`: Main Rust workspace manifest.
- `Justfile`: Command runner recipes (build, run, test).
- `CHANGELOG.md`: Version history. Update after changes.
- `NOTES.md`: Dev scratchpad.
- `README.md`: Project overview and instructions.
- `INSTALLING_DEPS.md`: Dependency installation guide.
- `RELEASING.md`: Release process documentation.
- `unhaunter/src/assetidx_updater.rs`: Generates `*.assetidx` files for WASM.
- `clippy.toml`: Rust linter config.
- `index.html`: WASM entry point.
- `catall.sh`: Helper to list project files.
- `unhaunter/src/bin/unhaunter.rs`: Native executable entry point.
- `unhaunter/src/lib.rs`: Main library entry point (WASM & Native).
- `unhaunter/src/app.rs`: **[CORE SETUP]** Main Bevy App assembly (plugins, resources).

## Data and Assets (`assets/`)

- `fonts/`: TTF fonts.
- `img/`: Sprites, UI, tiles (PNG, Aseprite).
- `maps/`: Tiled maps (`.tmx`, `.tsx`) and metadata.
- `music/` & `sounds/`: Audio files (`.ogg`).
- `manual/`: In-game manual content.
- `phrasebooks/`: Ghost communication data (`.ron`).
- `sample_ghosts/`: Ghost definitions (`.ron`).
- `shaders/`: Custom WGSL shaders.
- `index/`: Generated asset indices.

## Crate Structure (Workspace)

The project is modularized into many crates to separate concerns and improve compile times.

### Core Architecture (`un*-core`)

Low-level foundation shared across the project. Zero game logic (no systems or observers).

- **`unfoundation-core`**: Base types, colors, platform utils.
  - `GhostType` (Enum of ghost types), `GhostPersonality`, `Evidence`, `Difficulty`.
- **`untypes-core`**: Common data structures (enums, structs) used globally.
  - `GameAssets` (Resource holding handles to all loaded assets), `GearKind`, `EvidenceStatus`.
- **`unevents-core`**: Shared ECS events.
  - `GhostInteractionEvent`, `LevelLoadedEvent`, `RoomChangedEvent`.
- **`unassets-core`**: Asset loading infrastructure and handles.
  - `AssetIdx` (Asset lookup), `TmxMap` (Tiled map asset).
- **`unboard-core`**: Spatial system, grid/board logic, collision, `Behavior` component.
  - `Behavior` (Entity behavior on board), `Light`, `Door`, `Stairs`.
- **`unspatial-core`**: Spatial types and coordinate systems.
  - `BoardPosition` (Grid position), `Direction`, `Position`.
- **`untags-core`**: Marker components for entity identification.
  - `PlayerTag`, `GhostTag`, `InteractableTag`.
- **`unnoise-core`**: Perlin noise generation.
- **`unmetrics-core`**: Performance metrics and diagnostics data.
- **`unsettings-core`**: Application and game configuration.
- **`unui-core`**: Shared UI components and resources (e.g., `MouseVisibility`).
- **`unsummary-core`**: Data contract for the end-of-mission summary.

### Standard Rendering (`unrender-std`)

- Shared rendering components (`GameSprite`), materials (`CustomMaterial1`), and utilities used by multiple plugins.

### Game Logic & Progression (Plugins)

High-level game flow and state management. Logic is contained in `Plugin` implementations.

- **`ungame-plugin`**: Main game loop, scene management, high-level coordination.
- **`uncampaign-plugin`**: Campaign progression, mission unlocking, persistent state.
- **`undifficulty-core`**: Difficulty levels and configuration.
- **`unprofile-plugin`**: User profile management, save/load logic.
- **`unsummary-plugin`**: End-of-mission summary screen systems.
- **`unmapload-plugin`**: Map loading orchestration and setup.
- **`unmainmenu-plugin`**: Main menu UI and logic.
- **`unmenu-plugin`**: Core menu systems and shared menu components.

### Entities & Gameplay Systems (Plugins)

Specific gameplay mechanics and entity behaviors.

- **`unplayer-plugin`**: Player controller, movement, interaction, stats.
- **`unplayer-core`**: Core player components (e.g., `PlayerInput`).
- **`unghost-plugin`**: Ghost AI, behavior, hunting logic, evidence generation.
- **`unghost-core`**: Core ghost components and resources (e.g., `HauntState`).
- **`unnpc-plugin`**: NPC interaction, dialog systems.
- **`uninteraction-core`**: Generic entity interaction data (e.g., `Toggleable`, `Triggered`).
- **`unnavigation-core`**: Pathfinding components and data.
- **`unfog-plugin` / `unfog-core`**: Miasma and fog systems/data.
- **`ungear-plugin` / `ungear-core`**: Gear logic and item components (e.g., `Electronic`, `Battery`).
- **`unlight-plugin`**: Lighting logic and propagation (using `unrender-std`).
- **`unwalkie-plugin` / `unwalkie-core`**: Walkie-talkie communication logic and types.
- **`unroot-plugin`**: Root application plugin, startup logic.
- **`ungear`**: Inventory system, equipment slots, deployment logic.
  - `Gear` (Component for gear items), `PlayerGear` (Inventory), `DeployedGear`.
- **`ungearitems-core`**: Data definitions for all gear items (Flashlight, EMF, etc.).
  - `Flashlight`, `EMFMeter`, `Thermometer`, `SpiritBox`, `UVTorch`.
- **`ungearitems-plugin`**: **[GEAR LOGIC]** Implementation of specific items (EMF, Flashlight, etc.).
  - `UnhaunterGearItemsPlugin`.
- **`unfog`**: Miasma (fog) rendering and simulation.
  - `MiasmaConfig`.
- **`unlight`**: Lighting engine, visibility calculation, field of view.
  - `UnhaunterLightPlugin`.
- **`unwalkie`**: Walkie-talkie gameplay integration (hints, audio).
  - `UnhaunterWalkiePlugin`.
- **`unwalkiecore`**: Core logic for walkie-talkie state and responses.
  - `WalkieEvent`.
- **`unwalkie_types`**: Data types for the walkie-talkie system.
  - `VoiceLineData`.

### UI & Interface

User interface screens and menus.

- **`unmenu`**: Main menu screen.
  - `UnhaunterMenuPlugin`.
- **`unmenusettings`**: Settings menu implementation.
  - `SettingsState`.
- **`unmaphub`**: Map selection hub.
  - `UnhaunterMapHubPlugin`.
- **`untruck`**: In-game mission hub (Truck UI), CCTV, journal, loadout.
  - `TruckUI`, `TruckGear`.
- **`unmanual`**: In-game manual logic and UI.
  - `Manual`, `UnhaunterManualPlugin`.
- **`unui`**: Shared UI components and utilities.
- **`uncoremenu`**: Shared UI components, templates, and styles.
  - `MenuRoot`, `MenuItem`, `MenuBackground`.

### Infrastructure & Tools

Utilities and external tools.

- **`unroot`**: Main entry point for app setup and asset loading.
  - `UnhaunterRootPlugin`.
- **`unrender`**: Rendering engine, materials, and visibility logic.
  - `UnhaunterBoardPlugin`, `VisibilityData`.
- **`unpicking`**: Custom picking backend for map sprites.
  - `CustomSpritePickingPlugin`.
- **`untmxmap`**: Tiled map (`.tmx`) parsing and loading into Bevy.
  - `UnhaunterTmxMapPlugin`.
- **`untiled`**: Tiled map data structures and tileset management.
  - `MapTileSetDb`.
- **`unsettings`**: Persistent settings schema and serialization.
  - `AudioSettings`, `VideoSettings`, `GameplaySettings`, `ControlKeys`.
- **`crates/tools/`**:
  - `ghost_radio`: CLI tool for testing ghost communication/phrasebooks.
  - `ghost_list`: Tool for managing/listing ghost definitions.
  - `text_to_speech`: Tool for generating TTS audio assets.
