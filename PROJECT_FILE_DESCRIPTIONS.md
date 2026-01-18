# Project Index for AI Assistants

- **Purpose:** Outline file/directory roles in the \`unhaunter\` project. Aids code navigation and modification
  understanding.
- **"Feature X Relevance":** Indicates files likely touched when adding features related to their described purpose.

## General Project Files

- \`.cargo/config.toml\`: Cargo target-specific build flags.
- \`Cargo.toml\`: Main Rust workspace manifest.
- \`Justfile\`: Command runner recipes (build, run, test).
- \`CHANGELOG.md\`: Version history.
- \`NOTES.md\`: Dev scratchpad.
- \`README.md\`: Project overview and instructions.
- \`INSTALLING_DEPS.md\`: Dependency installation guide.
- \`RELEASING.md\`: Release process documentation.
- \`unhaunter/src/assetidx_updater.rs\`: Generates \`\*.assetidx\` files for WASM.
- \`clippy.toml\`: Rust linter config.
- \`index.html\`: WASM entry point.
- \`catall.sh\`: Helper to list project files for AI.
- \`unhaunter/src/bin/unhaunter.rs\`: Native executable entry point.
- \`unhaunter/src/lib.rs\`: Main library entry point (WASM & Native).

## Data and Assets (\`assets/\`)

- \`fonts/\`: TTF fonts.
- \`img/\`: Sprites, UI, tiles (PNG, Aseprite).
- \`maps/\`: Tiled maps (\`.tmx\`, \`.tsx\`) and metadata.
- \`music/\` & \`sounds/\`: Audio files (\`.ogg\`).
- \`manual/\`: In-game manual content.
- \`phrasebooks/\`: Ghost communication data (\`.ron\`).
- \`sample_ghosts/\`: Ghost definitions (\`.ron\`).
- \`shaders/\`: Custom WGSL shaders.
- \`index/\`: Generated asset indices.

## Crate Structure (Workspace)

The project is modularized into many crates to separate concerns and improve compile times.

### Core Architecture (\`un\*-core\`)

Low-level foundation shared across the project. Zero game logic (no systems or observers).

- **\`unfoundation-core\`**: Base types, colors, platform utils.
- **\`untypes-core\`**: Common data structures (enums, structs) used globally.
  - \`CliOptions\` (Command line arguments), \`AppState\` and \`GameState\` enums.
- **\`unevents-core\`**: Shared ECS events.
  - \`GhostInteractionEvent\`, \`LevelLoadedEvent\`, \`RoomChangedEvent\`.
- **\`unassets-core\`**: Tiled map integration and asset lookup infrastructure.
  - \`AssetIdx\` (Asset lookup), \`TmxMap\` (Tiled map loader), \`Maps\` (Global map resource).
- **\`unboard-core\`**: Spatial system, grid/board logic, collision, \`Behavior\` component.
- **\`unspatial-core\`**: Spatial types and coordinate systems.
- **\`untags-core\`**: Marker components for entity identification.
- **\`unnoise-core\`**: Perlin noise generation.
- **\`unmetrics-core\`**: Performance metrics and diagnostics data.
- **\`unsettings-core\`**: Application and game configuration.
- **\`unui-core\`**: Shared UI components and resources (e.g., \`MouseVisibility\`).
- **\`unsummary-core\`**: Data contract for the end-of-mission summary.
- **\`unbehavior\`**: Logic and components for entity behavior patterns.
- **\`uninteraction-core\`**: Generic entity interaction data (e.g., \`Toggleable\`, \`Triggered\`).
- **\`unnavigation-core\`**: Pathfinding components and data.
- **\`unprofile-core\`**: User profile data structures.
- **\`unthermal-core\`**: Thermal-specific data types and components.
- **\`untiled-core\`**: Low-level Tiled map data structures.
- **\`undifficulty-core\`**: Difficulty levels and configuration.
- **\`unfog-core\`**: Components and resources for the miasma/fog system.
- **\`ungear-core\`**: Base gear traits, types, and resources.
- **\`ungearitems-core\`**: Specific item components and traits (e.g., EMF).
- **\`unghost-core\`**: Core ghost components and evidence decay logic.
- **\`unmanual-core\`**: Data structures and asset handles for the in-game manual.
- **\`unmapload-core\`**: Map loading metadata and shared logic.
- **\`unmenu-core\`**: Shared UI templates and mission selection data.
- **\`unpicking-core\`**: Low-level picking logic and backend components.
- **\`unplayer-core\`**: Core player components and animation data.
- **\`unsound-core\`**: Sound-related resources and types.
- **\`untruck-core\`**: Truck-specific components, journal, and loadout data.
- **\`unwalkie-core\`**: Walkie-talkie traits, events, and resources.

### Plugins & Gameplay Systems

High-level game flow and specific gameplay mechanics. Logic is contained in \`Plugin\` implementations.

- **\`unroot-plugin\`**: Root application plugin, startup logic and framepace management.
- **\`unui-plugin\`**: Centralized UI asset loading and theme management.
- **`unclassic-plugin`**: Main game loop, scene management, high-level coordination.
- **\`uncampaign-plugin\`**: Campaign progression, mission unlocking, persistent state.
- **\`unprofile-plugin\`**: User profile management, save/load logic.
- **\`unsummary-plugin\`**: End-of-mission summary screen systems.
- **\`unmapload-plugin\`**: Map loading orchestration and setup.
- **\`unmainmenu-plugin\`**: Main menu UI and logic.
- **\`unmenu-plugin\`**: Core menu systems and shared menu components.
- **\`unmenusettings-plugin\`**: Settings menu implementation.
- **\`unmaphub-plugin\`**: Map selection hub.
- **\`unplayer-plugin\`**: Player controller, movement, interaction, stats.
- **\`unghost-plugin\`**: Ghost AI, behavior, hunting logic, evidence generation.
- **\`unnpc-plugin\`**: NPC interaction, dialog systems.
- **\`unfog-plugin\`**: Miasma and fog systems/data.
- **\`ungear-plugin\`**: Gear logic and item components (e.g., \`Electronic\`, \`Battery\`).
- **\`ungearitems-plugin\`**: Implementation of specific items (EMF, Flashlight, etc.).
- **\`unlight-plugin\`**: Lighting logic and propagation (using \`unrender-std\`).
- **\`unwalkie-plugin\`**: Walkie-talkie communication logic and types.
- **\`untruck-plugin\`**: Truck-based mission setup, loadout, and journal systems.
- **\`unrender-plugin\`**: Higher-level rendering orchestration.
- **\`unpicking-plugin\`**: Custom picking backend for map sprites.
- **\`unsound-plugin\`**: Audio playback and sound triggering logic.
- **\`unthermal-plugin\`**: Thermal vision and heat signature simulation.
- **\`unmetrics-plugin\`**: Performance monitoring and reporting.
- **\`unclassic-mode-plugin\`**: Classic mode gameplay logic and mission evaluation.
- **\`unmanual-plugin\`**: In-game manual UI, chapters, and navigation logic.
- **\`unmission-plugin\`**: Mission lifecycle events and summary data preparation.
- **\`unsettings-plugin\`**: Persistence of application and gameplay settings.
- **\`untmxmap-plugin\`**: Tiled map loading integration and Bevy compatibility.

### Tools & Utilities

- **\`unrender-std\`**: Shared rendering components (\`GameSprite\`), materials (\`CustomMaterial1\`).
- **\`crates/tools/\`**:
  - \`ghost_radio\`: CLI tool for testing ghost communication/phrasebooks.
  - \`ghost_list\`: Tool for managing/listing ghost definitions.
  - \`text_to_speech\`: Tool for generating TTS audio assets.
