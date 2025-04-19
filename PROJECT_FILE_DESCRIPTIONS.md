**Project Index for AI Assistants**

*   **Purpose:** Outline file/directory roles in the `unhaunter` project. Aids code navigation and modification understanding.
*   **"Feature X Relevance":** Indicates files likely touched when adding features related to their described purpose.

**General Project Files**

*   `.cargo/config.toml`: Cargo target-specific build flags (CPU features, linker, WASM runner). Modify for target/build opt.
*   `CHANGELOG.md`: Version history (features, fixes). Update after *any* change.
*   `Cargo.toml`: Main Rust manifest (metadata, dependencies, workspace, build profiles). Modify for deps, version, build cfg, crates.
*   `NOTES.md`: Dev scratchpad (ideas, TODOs). Check for context/plans.
*   `README.md`: Project overview, gameplay, build instructions, links. Update for major features/build changes.
*   `build.rs`: Build script generating `*.assetidx` files (asset lists for WASM). Modify for asset structure/indexing changes.
*   `clippy.toml`: Rust linter (Clippy) config. Modify for lint rules.
*   `index.html`: WASM entry HTML (loads WASM module). Modify for WASM load process, HTML structure, JS libs.
*   `pkg/.gitignore`: Git ignore rules for WASM build output (`pkg/`). Rarely modified.
*   `catall.sh`: Helper script to list all project files. (Dev utility).
*   `unhaunter/src/bin/unhaunter.rs`: Native build entry point. Calls `app_run()`. Rarely modified.
*   `unhaunter/src/lib.rs`: Main lib (`app_run()`), WASM entry (`wasm_load`). Modify for WASM init changes.
*   `unhaunter/src/app.rs`: **[CORE SETUP]** Main Bevy App setup (window, resources, plugins, systems). Modify for adding plugins, global resources, core app config.
*   `unhaunter/src/report_timer.rs`: Simple performance metric console reporter. Modify reporting logic/format.

**Data and Assets (`assets/`)**

*   **Purpose:** Contains all game assets (images, fonts, maps, audio, shaders, data files). Add/modify assets for features.
*   **Sub-dirs:**
    *   `fonts/`: `.ttf` font files, organized by family.
    *   `img/`: `.png`, `.aseprite`, `.xcf` images (sprites, UI, tiles). Inc. `base-tiles/` (structure, decor, furniture), `src/` (source files).
    *   `maps/`: `.tmx`, `.tsx` map/tileset files, `.ron` metadata, Tiled project.
    *   `music/`: `.ogg` music files.
    *   `sounds/`: `.ogg` sound effect files.
    *   `manual/`: Images/markdown for in-game manual, by chapter.
    *   `phrasebooks/`: `.yaml` files for player/ghost communication system (tagged phrases).
    *   `sample_ghosts/`: `.yaml` ghost type definitions.
    *   `shaders/`: `.wgsl` custom shader files.
    *   `index/`: **(Generated)** `.assetidx` asset list files (used by `build.rs`).
*   `assets/ghost_responses.yaml`: Ghost response templates (YAML - trigger conditions, response types). Modify for ghost AI response logic.
*   `assets/phrasebooks/*.yaml`: Player/ghost phrase definitions (YAML - tagged). Modify for communication content/logic.
*   `assets/sample_ghosts/*.yaml`: Ghost type definitions (YAML - name, type, mood). Modify for new/changed ghost types.
*   `tools/ghost_radio/`: **[DEV TOOL]** Console tool for testing ghost communication system offline (uses phrasebooks). `console_ui.rs`, `data.rs`, `ghost_ai.rs`, `main.rs`.

**Crate Structure (Workspace)**

Modular design for code reuse and compilation speed. Crates listed below.

*   **`uncore/`:** **[CORE ENGINE]** Shared core logic (components, resources, events, types, base systems, asset loading). Low-level dependency.
*   **`unfog/`:** Miasma (fog) effect system.
*   **`ungame/`:** Higher-level game logic (in-game UI, level loading, object charge, pause).
*   **`ungear/`:** Player gear & inventory system (equip, deploy).
*   **`ungearitems/`:** Specific logic for *each* gear item (implements `GearUsable` trait).
*   **`unghost/`:** Ghost entity logic (AI, behavior, movement, events, hunting).
*   **`unlight/`:** Lighting & visibility system.
*   **`unmaphub/`:** Map & difficulty selection UI screens.
*   **`unmenu/`:** Main menu UI & logic.
*   **`unmenusettings/`:** In-game settings menu UI & logic.
*   **`unnpc/`:** NPC dialog & interaction system.
*   **`unplayer/`:** Player-specific logic (movement, interaction, hiding, stats).
*   **`unsettings/`:** Persistent settings definitions & loading/saving structure.
*   **`unstd/`:** **[SHARED UTILS]** Shared utilities beyond `uncore` (board helpers, manual UI, materials, tiled utils). Requires more deps than `uncore`.
*   **`untmxmap/`:** Tiled TMX/TSX map loading into Bevy.
*   **`untruck/`:** Truck UI implementation (loadout, journal, sensors, crafting).
*   **`unwalkie/`:** Walkie-talkie NPC hint system.
*   **`uncoremenu/`:** Reusable core UI components/systems for menus.

**Key Crate Files (Highly Condensed)**

*   **`uncore/`**:
    *   `assets/`: Custom Bevy asset loaders (TMX, TSX, assetidx). Modify for asset formats.
    *   `behavior/`: `Behavior` component (from Tiled props) defining object logic/state. Modify for new object behaviors.
    *   `components/`: Core Bevy components (board position, player, ghost, UI markers, etc.). Add/modify core data attached to entities.
    *   `events/`: Custom Bevy events for system communication. Add/modify events for new interactions.
    *   `resources/`: Global Bevy resources (BoardData, RoomDB, Difficulty, Maps, etc.). Add/modify global game state.
    *   `systemparam/`: Custom `SystemParam`s for cleaner system function signatures.
    *   `traits/`: Shared traits like `GearUsable`. Define/modify shared interfaces.
    *   `types/`: Core data structures (Evidence, GearKind, GhostType, Position, LightData etc.). Add/modify game data models.
    *   `utils/`: General utility functions (math, time, etc.).
*   **`unfog/`**: Miasma effect. `components.rs` (particle data), `resources.rs` (config, noise), `systems.rs` (spawning, animation, simulation). Modify for miasma appearance/behavior.
*   **`ungame/`**: Higher-level gameplay. `level.rs` (level loading/spawning), `ui.rs`/`gear_ui.rs` (in-game UI setup), `object_charge.rs` (ghost interaction system), `pause_ui.rs`. Modify for game flow, scene setup, core UI.
*   **`ungear/`**: Inventory/Gear core. `components/playergear.rs` (player inventory struct), `components/deployedgear.rs` (deployed gear). `systems.rs` (equip/deploy/UI update logic). Modify for inventory mechanics.
*   **`ungearitems/`**: **[CORE GEAR LOGIC]** `components/*.rs`: *Individual logic for each gear item* (Flashlight, EMF, Salt, etc.). Implements `GearUsable`. Modify files here for specific gear behavior changes. `from_gearkind.rs`: Maps `GearKind` enum to concrete types.
*   **`unghost/`**: Ghost logic. `ghost.rs` (movement, hunting, rage AI), `ghost_events.rs` (door slams, light flickers). Modify for ghost behavior/abilities.
*   **`unlight/`**: Lighting/Visibility. `maplight.rs` (visibility calc, light application), `lighting.rs` (light field rebuilding), `prebake.rs` (static light optimization). Modify for visual appearance, lighting effects, performance.
*   **`unmaphub/`**: Map/Difficulty selection UI. `map_selection.rs`, `difficulty_selection.rs`. Modify for menu flow/appearance.
*   **`unmenu/`**: Main menu UI. `mainmenu.rs`. Modify for main menu options/layout.
*   **`unmenusettings/`**: Settings menu. `components.rs` (state/events), `menu_ui.rs` (generic UI setup), `menus.rs` (setting definitions), `systems.rs` (input handling, saving). Modify for adding/changing settings options.
*   **`unnpc/`**: NPC Help/Dialog. `npchelp.rs`. Modify for NPC interactions/dialog content.
*   **`unplayer/`**: Player logic. `systems/keyboard.rs` (movement, camera), `systems/grabdrop.rs` (item interaction), `systems/hide.rs` (hiding), `systems/sanityhealth.rs` (stats). Modify for player actions/controls.
*   **`unsettings/`**: Settings data structures (`audio.rs`, `game.rs`, etc.). Defines persistent settings schema. Modify to add/change savable settings fields. `plugin.rs` handles loading/saving setup.
*   **`unstd/`**: Shared utilities. `board/` (SpriteDB caching, tile data), `manual/` (in-game manual UI/content), `materials.rs` (custom shaders/materials), `systemparam/` (custom system params). Modify for shared tools, manual content, rendering effects.
*   **`untmxmap/`**: Tiled map loading. `bevy.rs` (Bevy integration), `load.rs` (core Tiled parsing), `map_loader.rs` (custom loader using Bevy assets). Modify for Tiled data handling changes.
*   **`untruck/`**: Truck UI. `ui.rs` (main layout, tabs), `journalui.rs` / `journal.rs` (evidence/ghost guess), `loadoutui.rs` (gear selection), `sanity.rs` etc. (specific panels). Modify for truck screen features/layout.
*   **`unwalkie/`**: Walkie-talkie hint system. `walkie_play.rs`. Modify for hint logic/timing/content.
*   **`uncoremenu/`**: Reusable menu UI building blocks/systems (`templates.rs`, `systems.rs`). Modify for shared menu component behavior.
