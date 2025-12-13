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
- `build.rs`: Generates `*.assetidx` files for WASM.
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
- **`uncore-types`**: Common data structures (enums, structs) used globally.
- **`uncore-components`**: Shared ECS components.
- **`uncore-events`**: Shared ECS events.
- **`uncore-resources`**: Shared ECS resources.
- **`uncore-assets`**: Asset loading infrastructure and handles.
- **`uncore-board`**: Spatial system, grid/board logic, collision, `Behavior` component.
- **`uncore-systems`**: Common ECS systems.

### Game Logic & Progression

High-level game flow and state management.

- **`ungame`**: Main game loop, scene management, high-level coordination.
- **`uncampaign`**: Campaign progression, mission unlocking, persistent state.
- **`undifficulty`**: Difficulty levels and configuration.
- **`unprofile`**: User profile management, save/load logic.
- **`unsummary`**: End-of-mission summary screen and logic.
- **`unmapload`**: Map loading orchestration and setup.

### Entities & Gameplay Systems

Specific gameplay mechanics and entity behaviors.

- **`unplayer`**: Player controller, movement, interaction, stats (sanity/health).
- **`unghost`**: Ghost AI, behavior, hunting logic, evidence generation.
- **`unnpc`**: NPC interaction, dialog systems.
- **`ungear`**: Inventory system, equipment slots, deployment logic.
- **`ungearitems`**: **[GEAR LOGIC]** Implementation of specific items (EMF, Flashlight, etc.).
- **`unfog`**: Miasma (fog) rendering and simulation.
- **`unlight`**: Lighting engine, visibility calculation, field of view.
- **`unwalkie`**: Walkie-talkie gameplay integration (hints, audio).
- **`unwalkiecore`**: Core logic for walkie-talkie state and responses.
- **`unwalkie_types`**: Data types for the walkie-talkie system.

### UI & Interface

User interface screens and menus.

- **`unmenu`**: Main menu screen.
- **`unmenusettings`**: Settings menu implementation.
- **`unmaphub`**: Map selection hub.
- **`untruck`**: In-game mission hub (Truck UI), CCTV, journal, loadout.
- **`uncoremenu`**: Shared UI components, templates, and styles.

### Infrastructure & Tools

Utilities and external tools.

- **`unstd`**: Shared utilities (rendering helpers, manual UI, materials).
- **`untmxmap`**: Tiled map (`.tmx`) parsing and loading into Bevy.
- **`unsettings`**: Persistent settings schema and serialization.
- **`crates/tools/`**:
  - `ghost_radio`: CLI tool for testing ghost communication/phrasebooks.
  - `ghost_list`: Tool for managing/listing ghost definitions.
  - `text_to_speech`: Tool for generating TTS audio assets.
