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

### Core Architecture (`uncore-*`)

Low-level foundation shared across the project.

- **`uncore-foundation`**: Base types, colors, platform utils. Zero game logic.
  - `GhostType` (Enum of ghost types), `GhostPersonality`, `Evidence`, `Difficulty`.
- **`uncore-types`**: Common data structures (enums, structs) used globally.
  - `GameAssets` (Resource holding handles to all loaded assets), `GearKind`, `EvidenceStatus`.
- **`uncore-components`**: Shared ECS components.
  - `GhostSprite`, `PlayerSprite`.
- **`uncore-events`**: Shared ECS events.
  - `GhostInteractionEvent`, `LevelLoadedEvent`, `RoomChangedEvent`.
- **`uncore-resources`**: Shared ECS resources.
  - `GameState` (Tracks current state: Menu, Game, etc), `BoardData` (Grid/collision data), `RoomDB`.
- **`uncore-assets`**: Asset loading infrastructure and handles.
  - `AssetIdx` (Asset lookup), `TmxMap` (Tiled map asset).
- **`uncore-board`**: Spatial system, grid/board logic, collision, `Behavior` component.
  - `Behavior` (Entity behavior on board), `Light`, `Door`, `Stairs`.
- **`unspatial`**: Spatial types and coordinate systems.
  - `BoardPosition` (Grid position), `Direction`, `Position`.
- **`untags`**: Marker components for entity identification.
  - `PlayerTag`, `GhostTag`, `InteractableTag`.
- **`unnoise`**: Perlin noise generation.
- **`unmetrics`**: Performance metrics and diagnostics.

### Game Logic & Progression

High-level game flow and state management.

- **`ungame`**: Main game loop, scene management, high-level coordination.
  - `UnhaunterGamePlugin`.
- **`uncampaign`**: Campaign progression, mission unlocking, persistent state.
  - `UnhaunterCampaignPlugin`.
- **`undifficulty-core`**: Difficulty levels and configuration.
  - `DifficultyStruct` (Configuration for a difficulty level).
- **`unprofile`**: User profile management, save/load logic.
  - `PlayerProfileData` (Persistent player data), `StatisticsData`.
- **`unsummary`**: End-of-mission summary screen and logic.
  - `UnhaunterSummaryPlugin`.
- **`unmapload`**: Map loading orchestration and setup.
  - `UnhaunterMapLoadPlugin`.

### Entities & Gameplay Systems

Specific gameplay mechanics and entity behaviors.

- **`unplayer`**: Player controller, movement, interaction, stats (sanity/health).
  - `UnhaunterPlayerPlugin`.
- **`unplayer-core`**: Core player components and resources.
  - `PlayerInput`.
- **`unghost`**: Ghost AI, behavior, hunting logic, evidence generation.
  - `UnhaunterGhostPlugin`.
- **`unghost-core`**: Core ghost components and resources.
  - `HauntState`, `CurrentEvidenceReadings`.
- **`unnpc`**: NPC interaction, dialog systems.
  - `UnhaunterNPCPlugin`.
- **`uninteraction`**: Generic entity interaction system.
  - `Target`, `PositionTarget`.
- **`unnavigation`**: Pathfinding and collision handling.
  - `CollisionHandler`.
- **`ungear`**: Inventory system, equipment slots, deployment logic.
  - `Gear` (Component for gear items), `PlayerGear` (Inventory), `DeployedGear`.
- **`ungearitems`**: **[GEAR LOGIC]** Implementation of specific items (EMF, Flashlight, etc.).
  - `Flashlight`, `EMFMeter`, `Thermometer`, `SpiritBox`, `UVTorch`.
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
