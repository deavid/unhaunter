This is an index of all files in the project for reference. So if we need to know where's something, this should be enough
to guide us on where to look it.

In this guide we talk about a hypothetical "Feature X": This refers to, if we wanted to add a new feature that modifies that
file, what it could be. This gives ideas when finding what files are relevant for modification when we need to do something.
It doesn't exist, and it's just for example purposes.

This file is meant for AI Coding assistants, specially those integrated with the editor such as "GitHub Copilot" or
"Gemini Code Assist" so they can see the whole project structure.


**General Project Files**

*   `.cargo/config.toml`:
    *   **Purpose:** Configuration file for Cargo (Rust's build system and package manager).  Specifically, this file sets target-specific compiler flags and linker options.
    *   **Interactions:**  Interacts directly with the Rust compiler and linker.
    *   **Feature X Relevance:**  Would likely be touched if we needed to:
        *   Change compilation targets (e.g., add support for a new architecture).
        *   Optimize for specific CPU features.
        *   Change linker options (e.g., use a different linker for faster builds).
        *   Add or modify compilation flags.
        * Change the WASM runner.

*   `CHANGELOG.md`:
    *   **Purpose:**  Documents the project's version history, including new features, bug fixes, and changes.
    *   **Interactions:**  Manually updated by developers.  No direct interaction with code.
    *   **Feature X Relevance:**  Would be touched after implementing *any* feature, bug fix, or significant change.  Essential for tracking project progress.

*   `Cargo.toml`:
    *   **Purpose:**  The main manifest file for the Rust project.  Defines project metadata, dependencies, build configurations, workspace members, and more.
    *   **Interactions:**  Read by Cargo.  Manually edited to manage dependencies, build settings, and crate information.
    *   **Feature X Relevance:**  Would be touched if we needed to:
        *   Add or remove dependencies (e.g., add a new library for sound effects).
        *   Change the project's version number.
        *   Modify build profiles (e.g., change optimization levels).
        *   Add or remove workspace members (sub-crates).
        *   Add new bin targets
        *   Change the `default-run`

*   `GHOST_COMMUNICATION.md`:
    *   **Purpose:**  A design document outlining the vision, goals, technical implementation, and future plans for the ghost communication system.
    *   **Interactions:**  This is a design document, so it doesn't directly interact with code.  It *guides* the implementation of the ghost communication system.
    *   **Feature X Relevance:**  Highly relevant if the feature involves:
        *   Adding new ways for the player to communicate with the ghost.
        *   Changing how the ghost responds to player input.
        *   Implementing a more sophisticated ghost AI or mood system.
        *   Adding voice acting or other audio enhancements to ghost communication.
    *   **Key Takeaways from this Document:** This is a critical document. It describes a data-driven, emergent communication system using:
        *   **FastText Embeddings:** For semantic understanding of player phrases and ghost moods.
        *   **Hierarchical Tagging:** For categorizing phrases and triggering responses.
        *   **Ghost Mood Model:** A dynamic emotional state for the ghost that influences its behavior.
        *   **Response Templates:**  YAML-based templates for generating ghost responses.

*   `NOTES.md`:
    *   **Purpose:**  A scratchpad for notes, ideas, research, and TODOs.
    *   **Interactions:**  Manually edited.  No direct interaction with code.
    *   **Feature X Relevance:**  A good place to *start* thinking about any new feature.  Might contain relevant notes or TODOs.

*   `README.md`:
    *   **Purpose:**  Provides an overview of the project, including gameplay description, build instructions, community links, and WASM deployment information.
    *   **Interactions:**  Manually edited.  No direct interaction with code.
    *   **Feature X Relevance:**  Would be updated to document any new major features, changes to build instructions, or updates to community links.

*   `build.rs`:
    *   **Purpose:**  A build script that runs before the main project is compiled.  Its primary purpose is to generate `*.assetidx` files, which list the assets in specific folders. This is critical for WASM builds, where directly listing directory contents is not possible.
    *   **Interactions:**  Executed by Cargo during the build process.  Reads the contents of the `assets/` directory. Writes `*.assetidx` files.
    *   **Feature X Relevance:**  Would be touched if:
        *   The structure of the `assets/` folder changes significantly (e.g., new asset types are added, or folders are reorganized).
        *   The logic for generating asset lists needs to be modified.
        * We add a new directory to `get_asset_types`.

*   `clippy.toml`:
    *   **Purpose:** Configuration file for Clippy, a Rust linter that helps enforce code style and catch common mistakes.
    *   **Interactions:**  Read by Clippy during code analysis.
    *   **Feature X Relevance:**  Might be touched to adjust linting rules or disable specific warnings.

*   `index.html`:
    *   **Purpose:**  The main HTML file for the WASM version of the game.  Loads the compiled WASM module and provides the necessary HTML structure.
    *   **Interactions:**  Loaded by web browsers when accessing the WASM version.
    *   **Feature X Relevance:**  Would likely be touched if:
        *   The WASM loading process needs to be changed.
        *   The HTML structure needs to be modified (e.g., to add UI elements or change the canvas size).
        *   You wanted to add external Javascript libraries

*   `pkg/.gitignore`:
    *   **Purpose:**  Specifies files and folders within the `pkg/` directory that should be ignored by Git. This is typically used to exclude generated files from the WASM build.
    *   **Interactions:**  Used by Git.
    *   **Feature X Relevance:**  Likely not relevant unless the WASM build process changes significantly.

*   `catall.sh`:
    * **Purpose:** A simple shell script that's likely a convenience script to list every file on the project. It doesn't have an impact on the builds, it's just for visualization purposes.
    * **Interactions:** Executed manually as a shell script by the user.
    * **Feature X Relevance:** Very little impact, unless we decide to add a new file and want to make it appear, but the script is so simple that this action is not even needed.

* `unhaunter/src/bin/unhaunter.rs`
    *   **Purpose:**  This is the main entry point for the *native* (non-WASM) build of the game. It's a very small file that simply calls `app_run()`.
    *   **Interactions:**  Executed by Cargo when running the game natively.
    *   **Feature X Relevance:**  Very unlikely to be touched, unless you need to change how the application is initialized.

* `unhaunter/src/lib.rs`
    *   **Purpose:**  The main library file for the `unhaunter` crate. This file declares the `app_run` function, which is used by both the native and WASM entry points. It also contains the `wasm_load` function, which is the entry point for WASM builds.
    *   **Interactions:**  Linked into both the native and WASM builds.
    *   **Feature X Relevance:**  Might be touched if you need to change how the WASM version is initialized.

* `unhaunter/src/app.rs`:
     * **Purpose**: Sets up the main Bevy application, including window settings, resources, plugins, and core systems.
     * **Interactions:**
        *   Initializes Bevy's default plugins and configures the window.
        *   Adds various custom plugins that define the game's functionality.
        *   Registers resource (`CurrentDifficulty`, `ObjectInteractionConfig`, ...).
     * **Feature X Relevance**:  This file is the heart of the application setup.  It would likely be touched if:
        *   A new plugin needs to be added to the game.
        *   A new global resource needed to be initialized.
        *   The basic application setup (window, frame rate, etc.) needs to be changed.

* `unhaunter/src/report_timer.rs`
    *   **Purpose:** Implements a system for reporting performance metrics (frame times) to the console.  Used for debugging and profiling.
    *   **Interactions:**  Uses Bevy's `DiagnosticsStore` to access performance data.
    *   **Feature X Relevance:**  Could be modified to:
        *   Change the reporting frequency.
        *   Report different metrics.
        *   Output the data in a different format (e.g., to a file).

**Data and Assets**

*   `assets/`:
    *   **Purpose:** Contains all of the game's assets, organized into subfolders.  This includes images, fonts, maps, sounds, music, shaders, and more.
    *   **Interactions:**  Assets are loaded by the Bevy asset server and used by various game systems.
    *   **Feature X Relevance:**  Almost any feature that adds new visual or audio elements will involve adding assets to this folder.  Changes to existing game elements might also involve modifying assets here.
    * **Sub-directories Description**
        * `assets/fonts/`: Stores font files in various formats (.ttf).  Different fonts are organized into subfolders.
        * `assets/img/`:  Contains image files, primarily in PNG format, used for sprites, backgrounds, UI elements, and textures.  Organized into subfolders based on asset type (e.g., `base-tiles`, `characters`, `decor`).
        * `assets/maps/`:  Holds TMX (Tiled Map Editor) and TSX (Tileset) files defining the game's levels and tilesets.  Also includes related files like `.ron` configuration files and a Tiled project file.
        * `assets/music/`:  Contains music files in OGG format.
        * `assets/sounds/`:  Stores sound effect files in OGG format.
        * `assets/manual/`:  Contains images and Markdown files used to create the in-game manual. Organized into chapters.
        * `assets/phrasebooks/`:  YAML files defining phrases used for player and ghost communication. Organised for player and ghost and also for different styles.
        * `assets/sample_ghosts/`: YAML files that define ghost types.
        * `assets/shaders/`:  WGSL shader files for custom rendering effects.
        * `assets/index`: It contains automatically generated `.assetidx` files that are used by the build.rs script to make those assets visible to Bevy.

*   `assets/ghost_responses.yaml`:
    *   **Purpose:**  Defines the response templates for ghosts, including trigger conditions and response types (text, actions, events, sounds, silence).  A core part of the ghost communication system.
    *   **Interactions:**  Used by the ghost AI to select appropriate responses based on player actions, ghost mood, and context.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new ghost responses.
        *   Change the conditions under which responses are triggered.
        *   Modify the types of responses available (e.g., add new actions or events).

*   `assets/phrasebooks/player.yaml`, `assets/phrasebooks/ghost.yaml`, etc.:
    *   **Purpose:**  YAML files defining the phrases that the player and ghost can use.  They are organized hierarchically using tags.
    *   **Interactions:**  Loaded by the ghost communication system.  Used to populate the player's phrasebook and to interpret player input.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new player phrases.
        *   Modify existing phrases or their associated tags.
        *   Add new ghost responses or change the logic for selecting responses.

*   `assets/sample_ghosts/`:
    *   **Purpose:** Contains YAML files, each defining a specific type of ghost (e.g., "poltergeist.yaml").  These files specify the ghost's name, type, and initial mood.
    *   **Interactions:**  Loaded by the ghost AI system.  Used to initialize ghost entities.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new ghost types.
        *   Change the characteristics of existing ghost types.

*   `tools/ghost_radio/`:
    *   **Purpose:** A command-line tool for prototyping and testing the ghost communication system *outside* of the main game.  It allows developers to experiment with phrase matching, mood modeling, and response selection in a simplified environment.
    *   **Interactions:**  Standalone tool.  Doesn't directly interact with the main game.  Shares data files (YAML phrasebooks) with the main game.
    *   **Feature X Relevance:**  Useful for:
        *   Developing and testing new ghost communication features before integrating them into the main game.
        *   Experimenting with different phrasebook structures and response selection algorithms.
        *   Debugging issues related to ghost communication.
    * **Files inside tools/ghost_radio**
        * `src/console_ui.rs`: Handles the command-line interface for the tool.  Displays ghost options, gets player input, and displays ghost responses.
        * `src/data.rs`: Defines the data structures for player phrases, ghost responses, ghost metadata, and emotional signatures.  These structures are used for both the tool and the main game.
        * `src/ghost_ai.rs`: Contains the core logic for the ghost AI, including functions for scoring responses based on player input, ghost mood, and semantic similarity.
        * `src/main.rs`: The main entry point for the tool.  Loads data, manages the communication loop, and interacts with the console UI.

**Crate Structure**

The `unhaunter` project is organized into a workspace with multiple crates (sub-projects).  This is a good practice for larger projects as it promotes modularity, code reuse, and faster compilation.

Here's a breakdown of each crate and its responsibilities:

*   `uncore/`:  Contains core game logic, data structures, and systems that are shared across multiple parts of the game.  This includes:
    *   Components (data attached to entities).
    *   Resources (global data accessible throughout the game).
    *   Events (for communication between systems).
    *   Systems (the logic that operates on components and resources).
    *   Types (custom data types).
    *   Traits (shared behavior).
    *   Utilities (helper functions).
    *   Asset loading.

*   `unfog/`: Handles the miasma (fog) effect, including its visual representation, movement, and interaction with the environment.

*   `ungame/`: Contains game-specific logic that's not part of the core engine, such as:
    *   UI setup for the main game screen.
    *   Object charge management.
    *   Room change events.
    *   Gear and pause UI.

*   `ungear/`: Defines the player's inventory and gear systems.  Handles equip/unequip logic, gear deployment, and interactions with gear items.

*   `ungearitems/`:  Implements the specific behavior of individual gear items (e.g., EMF meter, flashlight, thermometer).

*   `unghost/`:  Manages the ghost entity, its behavior, AI, and interactions with the player and environment.

*   `unlight/`:  Handles the lighting system, calculating visibility and applying lighting effects to the map.

*   `unmaphub/`:  Implements the map selection and difficulty selection screens.

*   `unmenu/`:  Manages the main menu and its options.

*   `unmenusettings/`: Handles the in-game settings menu, allowing players to adjust gameplay, audio, and video options.

*   `unnpc/`:  Manages Non-Player Characters (NPCs), including their dialog and interaction with the player.

*   `unplayer/`:  Contains player-specific logic, including movement, interaction, hiding, and sanity/health management.

*   `unsettings/`:  Defines the structure for storing and loading game settings (audio, video, gameplay).

*   `unstd/`:  Contains a collection of utilities, data structures, and systems that are used by multiple crates.  This likely includes:
    *   Board-related functionality (e.g., tile data, sprite management).
    *   UI elements for the manual.
    *   Custom materials.
    *   Map loading tools.

*   `untmxmap/`:  Handles loading TMX map files and converting them into Bevy-compatible data structures.

*   `untruck/`:  Implements the truck UI, including:
    *   Loadout management.
    *   Evidence tracking and ghost guessing.
    *   Sanity and sensor displays.
    *   Repellent crafting.

**Crate-Specific File Descriptions**

Detailed breakdown of each file per crate:

**`uncore/`**

This crate contains the basics and it is meant for most other crates to depend on it, while having minimal dependencies.

All structs/events/components/etc that need to be accessed by multiple crates should go here, unless they require
dependencies, then they would go in the `unstd` crate.

*   `uncore/src/lib.rs`: The main library file for the `uncore` crate. Defines constants, re-exports modules, and contains helper functions.
    * **Purpose:** This crate serves as the engine for the game.
    * **Interactions:** The "glue" for all of `uncore`.
    * **Feature X Relevance:** This is the lowest dependency in the project. If there is a feature that should be used in several parts of the game, the core should be touched. For example, if we wanted to add a new global variable to check for an existing "CHEAT_MODE", this is a nice place.

*   `uncore/src/assets/mod.rs`, `uncore/src/assets/index.rs`, `uncore/src/assets/tmxmap.rs`, `uncore/src/assets/tsxsheet.rs`:  Handles loading of assets, particularly TMX maps and TSX tilesets.  Includes custom asset loaders for Bevy.
    *   **Purpose:**  Provides a way to load TMX and TSX files as Bevy assets, making them accessible to the rest of the game.  The `index.rs` file enables loading `assetidx` files, which is essential for WASM builds.
    *   **Interactions:**  Interacts with Bevy's asset server and asset loading system.
    *   **Feature X Relevance:**  Would be touched if:
        *   New map or tileset formats need to be supported.
        *   The way assets are loaded or indexed needs to be changed.

*   `uncore/src/behavior/mod.rs`, `uncore/src/behavior/component.rs`:  Defines the `Behavior` component and related data structures, which determine how objects in the game world behave.
    *   **Purpose:**  Separates an object's visual representation (sprite) from its logical behavior.  Allows for defining complex interactions and states.
    *   **Interactions:**  Used by systems that handle object interactions, collisions, lighting, and AI.  Data is loaded from Tiled map properties.
    *   **Feature X Relevance:**  Would be touched if:
        *   New types of object behavior need to be added (e.g., a new type of interactive object).
        *   Existing behavior needs to be modified (e.g., changing how doors work).
        *   New properties need to be added to objects (e.g., adding a "fragility" property to breakable objects).

*   `uncore/src/colors.rs`:  Defines constant color values used throughout the game.
    *   **Purpose:**  Provides a central location for managing colors, making it easier to maintain consistency and make global changes.
    *   **Interactions:**  Used by any system or component that needs to set or manipulate colors.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new colors.
        *   Change existing color values.
        *   Reorganize color definitions.

*   `uncore/src/components/mod.rs`:  Declares the various component modules used in the game.
    *   **Purpose:**  Organizes components into logical groups.
    *   **Interactions:**  Other parts of the code import components from this module.
    *   **Feature X Relevance:**  Would be touched to add or remove component modules.

* `uncore/src/components/animation.rs`
    *   **Purpose:** The player, ghost, and any other animated character need sprites that change with time.
    *   **Interactions:** Uses `CharacterAnimationDirection` to determine in which direction to show the sprite. Uses `CharacterAnimationState` to determine if Standing or Walking. Uses `AnimationTimer` for a timer.
    *   **Feature X Relevance:** This file would be relevant if you needed to have more character states like attacking or talking and those needed animations.

*   `uncore/src/components/board/mod.rs`, `uncore/src/components/board/boardposition.rs`, `uncore/src/components/board/direction.rs`, `uncore/src/components/board/mapcolor.rs`, `uncore/src/components/board/position.rs`, `uncore/src/components/board/chunk.rs`:  Defines components and data structures related to the game board (grid), including positions, directions, and colors.
    *   **Purpose:**  Provides a way to represent and manipulate the 2D isometric grid on which the game takes place.  Handles coordinate conversions, distance calculations, and neighbor finding.
    *   **Interactions:**  Used by systems that handle movement, collision detection, lighting, and AI.
    *   **Feature X Relevance:**  Would be touched if:
        *   The grid system needs to be fundamentally changed (e.g., switching to a non-isometric projection).
        *   New types of board-related calculations are needed (e.g., pathfinding).
        *   New map properties like height need to be added (although `z` exists, it is not really used other than for rendering the correct order).

*   `uncore/src/components/game.rs`, `uncore/src/components/game_config.rs`, `uncore/src/components/game_ui.rs`:  Defines components related to general game state, configuration, and UI elements within the game world.
    *   **Purpose:**  Stores data about the game state (e.g., camera, player ID, UI elements), and configuration.
    *   **Interactions:**  Used by various game systems.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new game-level configuration options.
        *   Modify existing game state or UI elements.
    * Key components:
      * `GCameraArena`
      * `GameSprite`
      * `GameConfig`
      * `DamageBackground`: It represents a health visualization and the camera exposure.

*   `uncore/src/components/ghost_breach.rs`, `uncore/src/components/ghost_influence.rs`, `uncore/src/components/ghost_sprite.rs`:  Defines components related to the ghost entity, its spawn point (breach), and its interaction with objects.
    *   **Purpose:**  Represents the ghost and its properties.  Handles spawning, movement, rage, hunting, and interactions with objects that attract or repel it.
    *   **Interactions:**  Used by systems that handle ghost AI, movement, interaction, and rendering.
    *   **Feature X Relevance:**  Would be touched to:
        *   Modify ghost behavior (e.g., movement patterns, hunting logic).
        *   Add new ghost abilities or interactions.
        *   Change how the ghost interacts with objects or the environment.

*   `uncore/src/components/player.rs`, `uncore/src/components/player_inventory.rs`, `uncore/src/components/player_sprite.rs`:  Defines components related to the player character, including inventory, held objects, movement, and stats.
    *   **Purpose:**  Represents the player and their state.
    *   **Interactions:**  Used by systems that handle player input, movement, inventory management, interaction, and UI updates.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new player abilities or actions.
        *   Change player stats or inventory.
        *   Modify how the player interacts with objects or the environment.

*   `uncore/src/components/sprite_type.rs`: Defines an enum for categorizing different types of sprites (e.g., Ghost, Player, Miasma).
    *   **Purpose:**  Provides a way to differentiate between different sprite types for rendering and interaction purposes.
    *   **Interactions:** Used where needed for specific sprite types.
    *   **Feature X Relevance:**  Would be touched if:
        *   New types of sprites are added to the game.
        *   The categorization of existing sprites needs to be changed.

*   `uncore/src/components/summary_ui.rs`, `uncore/src/components/truck.rs`, `uncore/src/components/truck_ui.rs`, `uncore/src/components/truck_ui_button.rs`:  Defines components related to UI elements in the truck and the summary screen.
    *   **Purpose:**  Represents UI elements and their properties.
    *   **Interactions:**  Used by systems that handle UI rendering, updates, and interactions.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new UI elements to the truck or summary screen.
        *   Modify the appearance or behavior of existing UI elements.
        *   Add new functionality to truck UI buttons.

* `uncore/src/controlkeys.rs`:
    *   **Purpose:** Defines the keyboard controls for the player.
    *   **Interactions:** Used by the systems that manage the player's input
    *   **Feature X Relevance:** Would be touched if we wanted to allow more players by adding new control schemes, or to change player controls to make it configurable for the player.

*   `uncore/src/difficulty.rs`:
    *   **Purpose:** Defines an enum with all difficulties for the player, and a struct with a function to create one difficulty at a time.
    *   **Interactions:** Used by the `CurrentDifficulty` resource
    *   **Feature X Relevance:** It would be touched if we wanted to add a new difficulty or modify the current ones.

*   `uncore/src/events/mod.rs`, `uncore/src/events/board_data_rebuild.rs`, `uncore/src/events/loadlevel.rs`, `uncore/src/events/map_selected.rs`, `uncore/src/events/npc_help.rs`, `uncore/src/events/roomchanged.rs`, `uncore/src/events/sound.rs`, `uncore/src/events/truck.rs`: Defines custom Bevy events used for communication between systems.
    *   **Purpose:**  Provides a way for systems to communicate with each other without direct coupling.  Events are sent and received by systems, allowing for a more modular and flexible architecture.
    *   **Interactions:**  Systems send and receive events.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new events to trigger specific actions or communicate information between systems.
        *   Modify existing events or their data.
        *   Change how systems respond to events.

*   `uncore/src/metric_recorder.rs`:
    *   **Purpose:**  Provides a system for recording and reporting performance metrics.
    *   **Interactions:**  Uses Bevy's diagnostic system.
    *   **Feature X Relevance:**  Would be modified to:
        *   Track new metrics.
        *   Change the reporting frequency or format.

*   `uncore/src/platform.rs`:
    *   **Purpose:**  Defines platform-specific constants and configurations (e.g., WASM vs. native).
    *   **Interactions:**  Used by other parts of the code to adjust behavior based on the target platform.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add support for new platforms.
        *   Change platform-specific settings.

*   `uncore/src/plugin.rs`:
    *   **Purpose:**  Defines the core Bevy plugin for the `uncore` crate.  Registers systems and resources.
    *   **Interactions:**  Added to the Bevy `App` in `unhaunter/src/app.rs`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources to the core plugin.
        *   Modify the plugin's initialization logic.

*   `uncore/src/resources/mod.rs`, `uncore/src/resources/board_data.rs`, `uncore/src/resources/difficulty_state.rs`, `uncore/src/resources/ghost_guess.rs`, `uncore/src/resources/manual.rs`, `uncore/src/resources/maps.rs`, `uncore/src/resources/object_interaction.rs`, `uncore/src/resources/roomdb.rs`, `uncore/src/resources/summary_data.rs`, `uncore/src/resources/visibility_data.rs`: Defines Bevy resources, which are globally accessible data structures.
    *   **Purpose:**  Provides a way to store and access data that is shared across multiple systems.
    *   **Interactions:**  Resources are accessed by systems using the `Res` or `ResMut` types.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new global resources.
        *   Modify the structure of existing resources.
        *   Change how resources are initialized or updated.

* `uncore/src/states.rs`
    *   **Purpose:** Declares the App's State
    *   **Interactions:** Used by the systems using run_if
    *   **Feature X Relevance:** Would be touched if a new state needs to be added.

*   `uncore/src/systemparam/mod.rs`, `uncore/src/systemparam/collision_handler.rs`, `uncore/src/systemparam/gear_stuff.rs`: Defines custom `SystemParam` structs, which provide convenient access to multiple resources and queries within systems.
    *   **Purpose:**  Reduces boilerplate code and improves readability by grouping related data access patterns.
    *   **Interactions:**  Used as parameters in system functions.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new system parameters to provide access to different data combinations.
        *   Modify existing system parameters to include new resources or queries.

*   `uncore/src/systems/mod.rs`, `uncore/src/systems/animation.rs`: Contains system implementations (the logic of the game).
    *   **Purpose:**  Defines the functions that implement the game's behavior. Systems operate on components and resources.
    *   **Interactions:** Systems interact with each other through events and shared resources.
    *   **Feature X Relevance:**  Almost any feature that involves changing game logic will involve modifying existing systems or adding new ones.

*   `uncore/src/traits/mod.rs`, `uncore/src/traits/gear_usable.rs`: Defines traits, which are interfaces that can be implemented by multiple types.  This allows for polymorphism and code reuse.
    *   **Purpose:**  Defines shared behavior that can be implemented by different types of gear.
    *   **Interactions:**  Implemented by gear item structs.  Used by systems that interact with gear.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new methods to the `GearUsable` trait, extending the functionality of all gear items.
        *   Modify existing trait methods.

*   `uncore/src/types/mod.rs`, `uncore/src/types/board/mod.rs`, `uncore/src/types/board/cached_board_pos.rs`, `uncore/src/types/board/fielddata.rs`, `uncore/src/types/board/light.rs`, `uncore/src/types/board/light_field_sector.rs`, `uncore/src/types/evidence.rs`, `uncore/src/types/evidence_status.rs`, `uncore/src/types/game.rs`, `uncore/src/types/gear/mod.rs`, `uncore/src/types/gear/equipmentposition.rs`, `uncore/src/types/gear/spriteid.rs`, `uncore/src/types/gear/utils.rs`, `uncore/src/types/ghost/mod.rs`, `uncore/src/types/ghost/definitions.rs`, `uncore/src/types/ghost/types.rs`, `uncore/src/types/manual.rs`, `uncore/src/types/miasma.rs`, `uncore/src/types/quadcc.rs`, `uncore/src/types/root/mod.rs`, `uncore/src/types/root/anchors.rs`, `uncore/src/types/root/font_assets.rs`, `uncore/src/types/root/game_assets.rs`, `uncore/src/types/root/image_assets.rs`, `uncore/src/types/root/map.rs`, `uncore/src/types/tiledmap/mod.rs`, `uncore/src/types/tiledmap/map.rs`, `uncore/src/types/truck_button.rs`: Defines custom data types used throughout the game.
    *   **Purpose:**  Provides well-defined structures and enums for representing game data, improving code clarity and maintainability.
    *   **Interactions:**  Used by components, resources, systems, and other types.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new data types to represent new game concepts.
        *   Modify existing data types to add new fields or change existing ones.

*   `uncore/src/utils/mod.rs`, `uncore/src/utils/ghost_setfinder.rs`, `uncore/src/utils/light.rs`, `uncore/src/utils/mean.rs`, `uncore/src/utils/time.rs`: Contains utility functions and data structures used in various parts of the game.
    *   **Purpose:**  Provides reusable helper functions for common tasks, such as mathematical calculations, string formatting, and time management.
    *   **Interactions:**  Used by various systems and components.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new utility functions.
        *   Modify existing utility functions.
        *   Add new utility data structures.


**`unfog/`**

This crate appears to be responsible for the "miasma" effect, which I'm assuming is a visual fog or particle system that might obscure vision or indicate ghost presence.

*   `unfog/src/lib.rs`: The main library file for the `unfog` crate. Declares the modules.
    *   **Purpose:** Defines the crate and its public interface.
    *   **Interactions:**  Links other modules within the crate.
    *   **Feature X Relevance:**  Would be touched if you add, remove, or rename modules within this crate.

*   `unfog/src/components.rs`: Defines the `MiasmaSprite` component, which is attached to individual miasma particles.
    *   **Purpose:**  Stores data about each miasma particle, including its position, radius, speed, noise offsets, and visibility.  Also includes a `despawn` flag and a `life` value for controlling particle lifetime.  The `direction` and `vel_speed` fields likely control particle movement within a flow field.
    *   **Interactions:**  Used by systems that spawn, animate, update, and despawn miasma particles.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the properties of miasma particles (e.g., add color, size variation).
        *   Modify the behavior of miasma particles (e.g., how they respond to forces, how their visibility changes).
        *   Add new effects to the miasma (e.g., swirling patterns).

*   `unfog/src/metrics.rs`:  Defines constants for performance metrics related to the miasma system.
    *   **Purpose:**  Provides named constants for tracking the execution time of specific systems, used for profiling and optimization.
    *   **Interactions:**  Used by systems that are being profiled.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new metrics to track.
        *   Rename or remove existing metrics.

*   `unfog/src/plugin.rs`: Defines the `UnhaunterFogPlugin`, which integrates the miasma system into the Bevy application.
    *   **Purpose:**  Registers resources and systems related to the miasma effect.
    *   **Interactions:**  Added to the Bevy `App` in `unhaunter/src/app.rs`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources related to the miasma.
        *   Change the conditions under which miasma systems run (e.g., tie them to a specific game state).
        *   Modify the initialization of miasma resources.

*   `unfog/src/resources.rs`: Defines resources used by the miasma system.
    *   **Purpose:**  Stores global data related to the miasma effect.
    *   **Interactions:**  Accessed by systems that need to read or modify miasma properties.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new global parameters for controlling the miasma (e.g., density, color variation).
        *   Change the default values of existing parameters.
    * Key Resources:
      *  `MiasmaConfig`: Parameters that control the overall behavior, diffusion, velocity scale and initial pressure
      *  `PerlinNoiseTable`: Stores precomputed Perlin noise for use, to make the miasma feel more organic.

*   `unfog/src/systems.rs`: Contains the systems that implement the miasma effect.
    *   **Purpose:**  Handles spawning, animating, updating, and initializing the miasma.
    *   **Interactions:**  Operates on `MiasmaSprite` components, `BoardData`, and `VisibilityData` resources.  Uses `PerlinNoiseTable` for movement.
    *   **Feature X Relevance:**  This is where most changes to the miasma's behavior and appearance would be made.  Relevant if you need to:
        *   Change how the miasma is spawned or distributed.
        *   Modify the animation of the miasma particles.
        *   Change how the miasma interacts with the environment (e.g., collision, light).
        *   Add new visual effects to the miasma.
    *   Key Systems:
        *   `initialize_miasma`: Sets up the initial state of the miasma grid when a new level is loaded.  Calculates initial pressure values based on room properties.
        *   `spawn_miasma`:  Spawns new miasma particles based on visibility, pressure, and a random distribution.
        *   `animate_miasma_sprites`:  Updates the position of miasma particles, applying circular motion and Perlin noise-based offsets.  Also handles particle despawning.
        *   `update_miasma`: Calculates pressure changes and velocity updates in the miasma grid, simulating fluid-like behavior.

**`ungame/`**

This crate seems to contain higher-level game logic built on top of `uncore`.

*   `ungame/src/lib.rs`:  The main library file for the `ungame` crate, declaring modules and any public interface.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:**  Minimal. Would be touched if modules are added/removed.

*   `ungame/src/gear_ui.rs`:  Handles the setup and management of the in-game UI related to player gear and inventory.
    *   **Purpose:**  Creates the UI elements for displaying the player's equipped gear, inventory slots, and item information.
    *   **Interactions:**  Uses `uncore` components and resources (e.g., `Inventory`, `PlayerGear`).
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the layout or appearance of the gear UI.
        *   Add new UI elements related to gear (e.g., a durability display).
        *   Modify how gear information is displayed.

*   `ungame/src/level.rs`:  Handles the loading and setup of game levels.
    *   **Purpose:**  Loads map data from TMX files, spawns entities for tiles, players, ghosts, and other game objects, and initializes their components.  Manages room-related events.
    *   **Interactions:**  Uses `untmxmap` for TMX loading, `uncore` for components, resources, and events, and `unstd` for sprite and material management.
    *   **Feature X Relevance:**  This is a crucial file for any feature that involves:
        *   Changes to the level loading process.
        *   Adding new types of entities to the game world.
        *   Modifying the initialization of existing entities.
        *   Changing how rooms are defined or managed.

*   `ungame/src/object_charge.rs`: Implements the object charge system, which influences ghost behavior based on object properties.
    * **Purpose:** Manages the accumulation and discharge of "charge" on objects, which affects the ghost's rage and movement.
    * **Interactions:**  Uses `uncore` components (e.g., `GhostInfluence`, `GhostSprite`) and resources (`ObjectInteractionConfig`).
    * **Feature X Relevance:**  Would be modified to:
        *   Change how object charge affects ghost behavior.
        *   Add new types of object influence.
        *   Adjust the parameters of the charge accumulation and discharge.

*   `ungame/src/pause_ui.rs`: Sets up and manages the pause menu UI.
    *   **Purpose:**  Creates and displays the pause menu when the game is paused.
    *   **Interactions:**  Responds to keyboard input to resume the game or quit.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the appearance or layout of the pause menu.
        *   Add new options to the pause menu.

*   `ungame/src/plugin.rs`: Defines the `UnhaunterGamePlugin`, which integrates the `ungame` crate's functionality into the Bevy application.
    *   **Purpose:**  Registers systems and resources related to the game logic.
    *   **Interactions:**  Added to the Bevy `App` in `unhaunter/src/app.rs`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources to the game plugin.
        *   Modify the conditions under which systems run.

*   `ungame/src/roomchanged.rs`:  Handles the `RoomChangedEvent`, which is triggered when the player enters a new room or a significant room-related change occurs.
    *   **Purpose:**  Updates the state of interactive objects based on the room's state and potentially opens the van UI.
    *   **Interactions:**  Uses `uncore` components, resources, and events.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change how interactive objects respond to room changes.
        *   Add new actions triggered by the `RoomChangedEvent`.

*   `ungame/src/systems.rs`: Contains systems for setting up the game, cleaning up game entities, and handling keyboard input for camera movement.
    *   **Purpose:**  Initializes the game scene, handles camera control, and manages game state transitions.
    *   **Interactions:**  Uses Bevy's ECS and input systems.
    *   **Feature X Relevance:**  Would be touched to:
        *   Change the initial camera setup.
        *   Modify camera movement controls.
        *   Add new game setup or cleanup logic.

*   `ungame/src/ui.rs`:  Handles the setup and management of the in-game UI, including the health/sanity display, inventory display, and held object information.
    *   **Purpose:**  Creates and updates the UI elements that provide information to the player during gameplay.
    *   **Interactions:**  Uses `uncore` components and resources, as well as `ungear` components for inventory management.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the layout or appearance of the in-game UI.
        *   Add new UI elements to display additional information.
        *   Modify how existing UI elements are updated.

**`ungear/`**

This crate focuses on the player's gear and inventory.

*   `ungear/src/lib.rs`:  Main library file, declares modules, and re-exports `GearSpriteID`.
    *   **Purpose:**  Defines the crate and its public interface.
    *   **Interactions:**  Links other modules within the crate.
    *   **Feature X Relevance:**  Minimal. Would be touched if modules are added/removed.

*   `ungear/src/components/mod.rs`, `ungear/src/components/deployedgear.rs`, `ungear/src/components/playergear.rs`: Defines components related to gear, including `PlayerGear` (the player's inventory), `DeployedGear` (gear placed in the world), and related data structures.
    *   **Purpose:**  Represents the state of gear items, whether they are held by the player, placed in the world, or stored in the inventory.
    *   **Interactions:**  Used by systems that handle inventory management, gear deployment, and gear usage.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the structure of the player's inventory.
        *   Add new properties to gear items (e.g., durability).
        *   Change how gear is deployed or retrieved.
    * Key Components:
      * `PlayerGear`: Contains the player's `left_hand`, `right_hand` inventory and `held_item`
      * `Inventory`, `InventoryNext` and `InventoryStats`: These components control the inventory display and selection
      * `DeployedGear` and `DeployedGearData`: Represent gear that has been placed in the game world.

*   `ungear/src/plugin.rs`: Defines the `UnhaunterGearPlugin`, integrating the gear system into the Bevy application.
    *   **Purpose:**  Registers systems and resources related to gear.
    *   **Interactions:**  Added to the Bevy `App` in `unhaunter/src/app.rs`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources related to gear.
        *   Modify the conditions under which gear systems run.

*   `ungear/src/systems.rs`: Contains systems for updating gear state, handling player input related to gear, and managing the UI related to gear.
    *   **Purpose:** Implements the logic for using, equipping, deploying, and retrieving gear.
    *   **Interactions:**  Uses `uncore` components, resources, and events, as well as `ungear` components.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change how gear is used or interacted with.
        *   Add new gear-related actions.
        *   Modify the UI for gear and inventory.

*   `ungear/src/types/mod.rs`, `ungear/src/types/gear.rs`: Defines the `Gear` struct, which wraps a `GearKind` enum and holds a `Box<dyn GearUsable>`. This provides a common interface for all gear items.
    *   **Purpose:**  Provides a way to represent different types of gear in a uniform way.
    *   **Interactions:**  Used throughout the gear system.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new types of gear.
        *   Modify the properties or behavior of the `Gear` struct.

**`ungearitems/`**

This crate implements the specific behavior of each individual gear item.

*   `ungearitems/src/lib.rs`:  Main library file, declares modules and re-exports.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:** Minimal. Would be touched if modules are added/removed.

*   `ungearitems/src/components/mod.rs`:  Declares the component modules for each gear item.  This file also re-exports all of the gear item structs, making them easily accessible.
    *   **Purpose:**  Organizes the components for different gear items.
    *   **Interactions:**  Other parts of the code import components from this module.
    *   **Feature X Relevance:**  Would be touched to add or remove gear item components.

*   `ungearitems/src/components/`: Contains a separate file for each gear item, defining its specific component and implementing the `GearUsable` trait.  (e.g., `compass.rs`, `emfmeter.rs`, `flashlight.rs`, etc.)
    *   **Purpose:**  Implements the unique behavior of each gear item.  This includes how it responds to being triggered, how it updates over time, and how it interacts with the environment.
    *   **Interactions:**  Uses `uncore` components, resources, and system parameters (e.g., `GearStuff`).
    *   **Feature X Relevance:**  These files are the *core* of the gear item implementation.  They would be modified to:
        *   Change the behavior of a specific gear item.
        *   Add new functionality to a gear item.
        *   Modify the UI representation of a gear item's status.
    * Key Files to Understand
      * `flashlight.rs`: It has the logic to turn on/off, change states, manage battery and implement an overheat mechanism.
      * `emfmeter.rs`: It has the logic to take measures, filter them, and determine the evidence status.
      * `thermometer.rs`: Similar to the EMF, it takes values from the environment and smooth them.
      * `recorder.rs`: Implements the EVP functionality.
      * `repellentflask.rs`: Manages the state of the repellent, spawning particles when active.
      * `salt.rs`: Implements the `SaltData` struct, as well as how salt is spawned and the trail/particle system.
      * `sage.rs`: Implements `SageBundleData`, managing the burning state and spawning smoke particles.
      * `quartz.rs`: Implements `QuartzStoneData`, and manages the absorption of energy.

*   `ungearitems/src/from_gearkind.rs`:  Implements the `FromGearKind` trait for `Gear`, allowing easy creation of `Gear` instances from `GearKind` enums.  Also implements `FromPlayerGearKind` for creating a `PlayerGear` from a `PlayerGearKind`.
    *   **Purpose:**  Provides a convenient way to create gear items.
    *   **Interactions:**  Used wherever gear items need to be created.
    *   **Feature X Relevance:**  Would be modified to add support for new gear types.

*   `ungearitems/src/plugin.rs`: Defines the `UnhaunterGearItemsPlugin`, integrating the gear item logic into the Bevy application.
    *   **Purpose:**  Registers systems related to specific gear item behavior.
    *   **Interactions:**  Added to the Bevy `App`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems for gear item behavior.
        *   Modify the conditions under which gear item systems run.

*   `ungearitems/src/prelude.rs`:  Re-exports all of the gear item components for easy access.
    *   **Purpose:**  Simplifies importing gear item components in other files.
    *   **Interactions:**  Used by other parts of the code that need to access gear item components.
    *   **Feature X Relevance:** Would be modified when adding new gear types

* `ungearitems/src/metrics.rs`
    *  **Purpose:** Defines DiagnosticPath constants for performance metrics related to systems within the `ungearitems` crate. These metrics help track the execution time of various systems, aiding in performance profiling and optimization.
    *  **Interactions:** Used by the `time_measure()` methods within the systems.
    * **Feature X Relevance:** This would be directly relevant when adding a new system and you want to measure the execution time.


**`unghost/`**

This crate handles the core logic for the ghost entity, including its behavior, events, and interactions.

*   `unghost/src/lib.rs`:  The main library file, declaring the modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:**  Minimal.

*   `unghost/src/ghost.rs`: Contains the core logic for ghost movement, hunting behavior, rage management, and interaction with repellents.
    *   **Purpose:**  Implements the ghost's AI and behavior.
    *   **Interactions:**  Uses `uncore` components (e.g., `GhostSprite`, `Position`, `Direction`), resources (e.g., `BoardData`, `RoomDB`), and system parameters (e.g., `GearStuff`).
    *   **Feature X Relevance:**  This is a central file for any changes to ghost behavior.  Would be modified to:
        *   Change how the ghost moves (e.g., add new movement patterns, change speed).
        *   Modify the ghost's hunting logic (e.g., change how it targets players, how it reacts to hiding).
        *   Adjust the ghost's rage and hunting thresholds.
        *   Add new ghost abilities or interactions.
    *   Key Systems:
        *   `ghost_movement`: Handles the ghost's movement logic, including pathfinding, warping, and hunting.
        *   `ghost_enrage`:  Manages the ghost's rage level, triggering hunts and handling player damage during hunts. Also, plays roar sounds.
        * `ghost_fade_out_system`: It manages the final ghost despawn.
    *   Key Components:
        *   `GhostSprite`:  Stores the ghost's data, including its type, spawn point, target, rage, hunting state, and more.
        *   `FadeOut`:  A component added to the ghost when it's being expelled, controlling the fade-out effect.

*   `unghost/src/ghost_events.rs`: Handles ghost-triggered events, such as door slams and light flickers.
    *   **Purpose:**  Implements environmental events caused by the ghost, adding to the atmosphere and providing clues to the player.
    *   **Interactions:**  Uses `uncore` components, resources, and events (e.g., `RoomChangedEvent`, `BoardDataToRebuild`).  Interacts with `Interactive` and `RoomState` components on objects.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new types of ghost events.
        *   Change the conditions under which events are triggered.
        *   Modify the effects of existing events.
    *   Key Events:
        *   `GhostEvent`:  An enum representing the different types of events.

* `unghost/src/metrics.rs`:
    *   **Purpose:**  Defines constants for performance metrics related to ghost systems.
    *   **Interactions:** Used by systems that are being profiled.
    *   **Feature X Relevance:** Would be touched to add or modify metrics.

*   `unghost/src/plugin.rs`: Defines the `UnhaunterGhostPlugin`, integrating the ghost systems into the Bevy application.
    *   **Purpose:** Registers the systems defined in the `ghost` and `ghost_events` modules.
    *   **Interactions:**  Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems related to ghost behavior.
        *   Modify the conditions under which ghost systems run.

**`unlight/`**

This crate handles the game's lighting system, including visibility calculations and applying lighting effects to sprites.

*   `unlight/src/lib.rs`:  The main library file.  Declares modules.
    *   **Purpose:**  Defines the crate.
    *   **Interactions:** Links modules.
    *   **Feature X Relevance:**  Minimal.

*   `unlight/src/maplight.rs`:  Contains the core logic for lighting calculations and application.
    *   **Purpose:**  Calculates visibility, applies lighting to tiles and sprites, and manages ambient sound levels based on visibility. This is one of the most complex files on the project.
    *   **Interactions:**  Uses a wide range of `uncore` components, resources, and types. Interacts with `Sprite`, `Transform`, and `MapColor` components.  Accesses `BoardData`, `VisibilityData`, `RoomDB`, and more.
    *   **Feature X Relevance:**  This file is central to the game's visual appearance and atmosphere.  Would be modified to:
        *   Change how visibility is calculated.
        *   Modify lighting parameters (e.g., intensity, color, falloff).
        *   Add new light sources or effects.
        *   Change how lighting interacts with different materials or object types.
        *   Optimize the lighting calculations.
    *   Key Systems:
        *   `compute_visibility`: Calculates the player's field of view, taking into account walls and obstacles.
        *   `player_visibility_system`: Updates the `VisibilityData` resource with the computed visibility field.
        *   `apply_lighting`:  Applies lighting effects to map tiles and sprites, adjusting colors based on light sources, visibility, and exposure.
        *   `ambient_sound_system`:  Adjusts the volume of ambient sounds based on overall visibility.

*   `unlight/src/metrics.rs`: Defines constants for performance metrics related to the lighting systems.
    * **Purpose:** Used for performance profiling
    * **Interactions:** The constants are used in the `apply_lighting` and `compute_visibility` systems in `unlight::maplight`

*   `unlight/src/plugin.rs`: Defines the `UnhaunterLightPlugin`, integrating the lighting system into the Bevy application.
    *   **Purpose:** Registers the systems defined in the `maplight` module.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems related to lighting.
        *   Modify the conditions under which lighting systems run.

**`unmaphub/`**

This crate handles the map hub UI, where players select the map and difficulty before starting a game.

*   `unmaphub/src/lib.rs`:  The main library file.  Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:** Links modules.
    *   **Feature X Relevance:**  Minimal.

*   `unmaphub/src/map_selection.rs`:  Implements the map selection screen.
    *   **Purpose:**  Displays a list of available maps and allows the player to select one.
    *   **Interactions:**  Uses `uncore` resources (e.g., `Maps`), components, and events (e.g., `MapSelectedEvent`).
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the appearance or layout of the map selection screen.
        *   Add new maps or change how maps are displayed.
        *   Modify the logic for selecting a map.
    * Key Components:
        * `MapSelectionUI`: A marker component of the map selection
        * `MapSelectionItem`: It stores the `map_idx`
        * `MapSelectionState`: It contains the currently selected map
    * Key Events:
        * `MapSelectedEvent`: Triggered when the player selects a new map

*   `unmaphub/src/difficulty_selection.rs`: Implements the difficulty selection screen.
    *   **Purpose:**  Displays a list of difficulty levels and allows the player to select one.
    *   **Interactions:**  Uses `uncore` resources (e.g., `Difficulty`, `CurrentDifficulty`), components, and events.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the appearance or layout of the difficulty selection screen.
        *   Add new difficulty levels or change the effects of existing ones.
        *   Modify the logic for selecting a difficulty.
    * Key Components:
        * `DifficultySelectionUI`: A marker component for this UI
        * `DifficultyDescriptionUI`: A marker component to reference the `Text` with the description
        * `DifficultySelectionItem`: Represents one of the difficulty levels available.

*   `unmaphub/src/plugin.rs`: Defines the `UnhaunterMapHubPlugin`, integrating the map hub functionality into the Bevy application.
    *   **Purpose:** Registers systems and resources related to the map hub.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems or resources related to the map hub.
        *   Modify the conditions under which map hub systems run.
        *   Change the state transitions related to the map hub.

**`unmenu/`**

This crate manages the main menu of the game.

*   `unmenu/src/lib.rs`:  The main library file. Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:**  Minimal.

*   `unmenu/src/mainmenu.rs`: Implements the main menu UI and logic.
    *   **Purpose:**  Displays the main menu options (New Game, Settings, Quit) and handles player input.
    *   **Interactions:**  Uses `uncore` resources and components.  Sends `MenuEvent` events to trigger state transitions.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the appearance or layout of the main menu.
        *   Add new menu options.
        *   Modify the behavior of existing menu options.
    * Key Components:
        * `Menu`:  Represents the state of the menu, including the currently selected item.
        * `MenuItem`: Represents a single menu item, including its identifier and highlight state.
        * `MCamera`:  A marker component for the menu's camera.
        * `MenuUI`:  A marker component for the menu's UI elements.

*   `unmenu/src/plugin.rs`: Defines the `UnhaunterMenuPlugin`, integrating the main menu functionality into the Bevy application.
    *   **Purpose:** Registers systems and resources related to the main menu.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems or resources related to the main menu.
        *   Modify the conditions under which main menu systems run.

**`unmenusettings/`**

This crate manages the in-game settings menu.

*   `unmenusettings/src/lib.rs`:  The main library file.  Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:**  Minimal.

*   `unmenusettings/src/components.rs`: Defines components and events used by the settings menu.
    *   **Purpose:**  Provides data structures for representing menu items, settings, and navigation actions.
    *   **Interactions:**  Used by systems that handle settings menu logic and UI updates.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new components or events related to settings management.
        *   Modify the structure of existing components or events.
    * Key Components:
        * `SettingsMenu`: Marks the root UI entity.
        * `SettingsState`: Holds the game state for the setting menu.
        * `MenuItem`: Represents a single option.
        * `MenuEvent`: An enum with all the possible changes and state transitions.

*   `unmenusettings/src/menu_ui.rs`: Implements the UI setup for the settings menu.
    *   **Purpose:** Creates the visual elements of the settings menu and defines their layout.
    *   **Interactions:** Uses Bevy's UI system.
    *   **Feature X Relevance:** Would be touched to:
        *   Change the appearance or layout of the settings menu.
        *   Add new UI elements to the settings menu.
        *   This file contains `setup_ui_main_cat_system` and `setup_ui_main_cat` that are generic.

*   `unmenusettings/src/menus.rs`: Defines enums and structs for organizing the different settings categories and options.
    *   **Purpose:** Provides a structured way to represent the hierarchy of settings (e.g., Gameplay, Video, Audio) and their individual values.
    *   **Interactions:** Used by systems that handle settings changes and UI updates.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new settings categories or options.
        *   Modify the structure of existing settings.
        *   Change the available values for settings.
    * Key enums
        * `MenuSettingsLevel1`: The first level of menu, contains the different main settings files.
        * `AudioSettingsMenu`: Contains all the audio settings inside that file.
        * `GameplaySettingsMenu`: Contains all the gameplay settings inside that file.

*   `unmenusettings/src/plugin.rs`: Defines the `UnhaunterMenuSettingsPlugin`, integrating the settings menu functionality into the Bevy application.
    *   **Purpose:** Registers systems and resources related to the settings menu.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems or resources related to the settings menu.
        *   Modify the conditions under which settings menu systems run.

*   `unmenusettings/src/systems.rs`: Implements the systems that handle user input, settings changes, and UI updates for the settings menu.
    *   **Purpose:** Processes keyboard input, handles menu navigation, saves settings changes, and updates the UI to reflect the current settings.
    *   **Interactions:** Uses `uncore` and `unsettings` components, resources, and events.  Interacts with Bevy's UI system.
    *   **Feature X Relevance:** This is a key file for modifying settings menu behavior.  Would be touched to:
        *   Change how user input is handled.
        *   Modify the logic for saving or loading settings.
        *   Add new functionality to the settings menu.

* `unmenusettings/TODO.md`:
    * **Purpose:** TODO markdown file
    * **Interactions:** Only read by humans.
    * **Feature X Relevance:** It has some hints of future features to add.

**`unnpc/`**

This crate handles Non-Player Characters (NPCs), focusing on their dialog and interactions.

*   `unnpc/src/lib.rs`:  The main library file. Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:** Minimal.

*   `unnpc/src/npchelp.rs`: Implements the NPC help system, which provides tutorial-like dialog to the player.
    *   **Purpose:**  Manages NPC dialog, including triggering dialog based on proximity and handling player interaction with NPCs.
    *   **Interactions:**  Uses `uncore` components (e.g., `NpcHelpDialog`, `Interactive`, `Position`, `PlayerSprite`), resources, and events.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the content of NPC dialog.
        *   Modify the conditions under which dialog is triggered.
        *   Add new NPCs or change their behavior.
        *   Change the UI for displaying NPC dialog.
    * Key Components:
        * `NpcHelpDialog`:  Stores the dialog text, a flag indicating whether the dialog has been seen, and a trigger distance.
        * `NpcUI`: A marker component for the UI elements of the NPC help system.
        * `NpcDialogText`: A marker component for the `Text` entity that displays the actual dialog.
    * Key Events:
        * `NpcHelpEvent`: Triggered when the player is near an NPC that hasn't spoken yet.
    * Key Systems:
        * `setup_ui`: Creates the UI element
        * `keyboard`: Allows closing of the dialog with keys.
        * `npchelp_event`: Receives the event of being close to the NPC, so it starts the dialog.
        * `auto_call_npchelp`: Automatically triggers the dialog on proximity and calls for the event.

*   `unnpc/src/plugin.rs`: Defines the `UnhaunterNPCPlugin`, integrating the NPC system into the Bevy application.
    *   **Purpose:** Registers systems and resources related to NPCs.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources related to NPCs.
        *   Modify the conditions under which NPC systems run.

**`unplayer/`**

This crate focuses on player-specific logic, including movement, interactions, hiding, and health/sanity management.

*   `unplayer/src/lib.rs`:  The main library file.  Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:** Links other modules.
    *   **Feature X Relevance:** Minimal.

*   `unplayer/src/plugin.rs`: Defines the `UnhaunterPlayerPlugin`, integrating the player systems into the Bevy application.
    *   **Purpose:** Registers systems related to player control, movement, interaction, and status effects.
    *   **Interactions:** Added to the Bevy `App`.
    *   **Feature X Relevance:** Would be touched to:
        *   Add new systems related to player behavior.
        *   Modify the conditions under which player systems run.

*   `unplayer/src/systems/mod.rs`:  Declares the modules containing the player-related systems.
    *   **Purpose:**  Organizes systems into logical groups.
    *   **Interactions:**  Other parts of the code import systems from these modules.
    *   **Feature X Relevance:**  Would be touched to add or remove system modules.

*  `unplayer/src/systems/grabdrop.rs`: Implements systems for grabbing, dropping, deploying, and retrieving gear.
    *  **Purpose**: Manages the player's interaction with objects in the world, including:
        *  Grabbing pickable objects.
        *  Dropping held objects.
        *  Deploying gear (placing it on the ground).
        *  Retrieving deployed gear.
    *   **Interactions:**  Uses `uncore` components (`Position`, `Behavior`, `Interactive`), resources (`BoardData`), and system parameters (`GearStuff`).  Also uses `ungear` components (`PlayerGear`, `DeployedGear`).
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the mechanics of grabbing or dropping objects.
        *   Add new types of objects that can be interacted with.
        *   Modify the rules for where objects can be placed.
    * Key Systems:
        * `grab_object`: Handles grabbing pickable objects
        * `drop_object`: Handles dropping
        * `deploy_gear`: Handles deploying gear
        * `retrieve_gear`: Handles retrieving the deployed gear
        * `update_held_object_position`: Adjusts the Z of the held object and plays a sound.

* `unplayer/src/systems/hide.rs`:
    * **Purpose**: Implements the player hiding mechanic. Allows the player to hide behind certain objects.
    * **Interactions:**  Uses `uncore` components (e.g., `Hiding`, `PlayerSprite`, `Behavior`), resources, and events.
    * **Feature X Relevance**: Would be modified to change the hiding mechanic:
        * Change the keys to hide and unhide.
        * Change what objects can be hidden behind.
        * Add animations or visual effects to hiding.
    * Key Systems:
        * `hide_player`: Called when the player triggers the hide action.
        * `unhide_player`: Called when the player stops hiding.

*   `unplayer/src/systems/keyboard.rs`:  Handles player movement and interaction input from the keyboard.
    *   **Purpose:**  Processes key presses and translates them into player actions, such as movement, interaction, and animation updates.
    *   **Interactions:**  Uses `uncore` components (`Position`, `Direction`, `PlayerSprite`, `AnimationTimer`, `ControlKeys`), resources, and system parameters (`CollisionHandler`).
    *   **Feature X Relevance:**  Would be modified to:
        *   Change the key bindings for player actions.
        *   Add new player actions triggered by keyboard input.
        *   Modify the player's movement speed or behavior.
        *   Adjust camera movement
        * This file seems to manage the camera as well!

*   `unplayer/src/systems/sanityhealth.rs`:  Manages the player's sanity and health systems.
    *   **Purpose:**  Updates sanity and health based on environmental factors, ghost presence, and player actions.  Also handles visual feedback for health and sanity levels.
    *   **Interactions:**  Uses `uncore` components (e.g., `PlayerSprite`), resources (e.g., `BoardData`, `RoomDB`), and events.
    *   **Feature X Relevance:**  Would be modified to:
        *   Change how sanity and health are calculated.
        *   Add new factors that affect sanity or health.
        *   Modify the visual effects associated with low health or sanity.

**`unsettings/`**

This crate handles persistent game settings, such as audio and video options, and profile settings.

*   `unsettings/src/lib.rs`:  The main library file.  Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:**  Links other modules.
    *   **Feature X Relevance:**  Minimal.

*   `unsettings/src/audio.rs`: Defines the `AudioSettings` struct and related enums (`AudioLevel`, `SoundOutput`, `AudioPositioning`, `FeedbackDelay`, `FeedbackEQ`).
    *   **Purpose:**  Stores audio-related settings, such as volume levels, sound output mode, and audio positioning options.
    *   **Interactions:**  Used by systems that need to access or modify audio settings.  Serialized and deserialized to save/load settings.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new audio settings.
        *   Change the available options for existing settings.
        *   Modify the default values for settings.

*   `unsettings/src/game.rs`: Defines the `GameplaySettings` struct and related enums (`MovementStyle`, `CameraControls`, `CharacterControls`).
    *   **Purpose:** Stores gameplay-related settings, such as movement style, camera controls, and character controls.
    *   **Interactions:** Used by systems that need to access or modify gameplay settings. Serialized and deserialized to save/load settings.
    *   **Feature X Relevance:** Would be modified to:
        *   Add new gameplay settings.
        *   Change the available options for existing settings.
        *   Modify the default values for settings.

*   `unsettings/src/plugin.rs`: Defines the `UnhaunterSettingsPlugin`, integrating the settings system into the Bevy application.
    *   **Purpose:**  Initializes resources for persistent settings.
    *   **Interactions:**  Added to the Bevy `App`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new settings categories or options.
        *   Change how settings are loaded or saved.

*   `unsettings/src/profile.rs`: Defines the `ProfileSettings` struct and related enums.
    *   **Purpose:**  Stores player profile-related settings, such as display name and color preference.
    *   **Interactions:**  Used by systems that need to access or modify profile settings. Serialized and deserialized.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new profile settings.
        *   Change the available options for existing settings.

*   `unsettings/src/video.rs`: Defines the `VideoSettings` struct and related enums (`WindowSize`, `AspectRatio`, `Scale`).
    *   **Purpose:**  Stores video-related settings, such as window size, aspect ratio, and UI/font scaling.
    *   **Interactions:**  Used by systems that need to access or modify video settings.  Serialized and deserialized.
    *   **Feature X Relevance:**  Would be modified to:
        *   Add new video settings.
        *   Change the available options for existing settings.
        *   Modify the default values for settings.

**`unstd/`**

This crate is a collection of standard utilities and data structures used across multiple other crates in the `unhaunter` project.
It's a good place to put code that is reused in several places, promoting DRY (Don't Repeat Yourself) principles.

It is an expansion on top of `uncore` for these things that need more dependencies.

*   `unstd/src/lib.rs`: The main library file.  Declares modules.
    *   **Purpose:** Defines the crate.
    *   **Interactions:** Links other modules.
    *   **Feature X Relevance:** Minimal.

*   `unstd/src/board/mod.rs`, `unstd/src/board/spritedb.rs`, `unstd/src/board/tiledata.rs`:  Defines data structures and utilities related to the game board (tile grid).  This includes a `SpriteDB` for efficient sprite management and `MapTileComponents` for storing pre-built Bevy components.
    *   **Purpose:**  Provides tools for managing the tile grid, storing tile data, and optimizing sprite rendering.
    *   **Interactions:**  Used by systems that interact with the game board, such as map loading, collision detection, and rendering.
    *   **Feature X Relevance:**  Would be touched to:
        *   Change how the game board is represented or managed.
        *   Modify the `SpriteDB` to handle new sprite types or properties.
        *   Optimize the rendering of map tiles.
    * Key files:
        * `spritedb.rs`: Defines `SpriteDB` to cache all `MapTileComponents`
        * `tiledata.rs`: Defines `MapTileComponents`

*   `unstd/src/manual/mod.rs`, `unstd/src/manual/preplay_manual_ui.rs`, `unstd/src/manual/user_manual_ui.rs`, `unstd/src/manual/utils.rs`, `unstd/src/manual/chapter1/mod.rs`, `unstd/src/manual/chapter1/page1.rs`, `unstd/src/manual/chapter1/page2.rs`, `unstd/src/manual/chapter1/page3.rs`, `unstd/src/manual/chapter2/mod.rs`, `unstd/src/manual/chapter2/page1.rs`, `unstd/src/manual/chapter2/page2.rs`, `unstd/src/manual/chapter3/mod.rs`, `unstd/src/manual/chapter3/page1.rs`, `unstd/src/manual/chapter4/mod.rs`, `unstd/src/manual/chapter4/page1.rs`, `unstd/src/manual/chapter5/mod.rs`, `unstd/src/manual/chapter5/page1.rs`:  Implements the in-game manual, including both the pre-play tutorial and the user-accessible manual.
    *   **Purpose:**  Provides a structured way to present information to the player about game mechanics, controls, and strategies.
    *   **Interactions:**  Uses Bevy's UI system.  Uses `uncore` resources and components.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new chapters or pages to the manual.
        *   Modify the content of existing manual pages.
        *   Change the appearance or layout of the manual UI.
        *   Add new UI elements or interactions to the manual.
    * The Chapter and pages structure is very well done. The most relevant parts here are:
        * `utils.rs`: Contains several functions that help create a common style in the manual, like headers, paragraphs, and grid layouts.
        * `preplay_manual_ui.rs`: It contains all that is needed for the pre-play user manual, including the state transitions.
        * `user_manual_ui.rs`:  It contains all that is needed for the user manual, including the state transitions.

*   `unstd/src/materials.rs`: Defines custom Bevy materials used for rendering.
    *   **Purpose:**  Creates specialized shaders and materials for visual effects.
    *   **Interactions:**  Used by the rendering system.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new visual effects.
        *   Modify existing shaders.
        *   Change how materials are applied to objects.

*   `unstd/src/plugins/mod.rs`, `unstd/src/plugins/board.rs`, `unstd/src/plugins/manual.rs`, `unstd/src/plugins/root.rs`, `unstd/src/plugins/summary.rs`: Defines Bevy plugins that group related systems and resources.
    *   **Purpose:**  Organizes functionality into reusable modules.
    *   **Interactions:**  Added to the Bevy `App`.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new systems or resources to a plugin.
        *   Modify the conditions under which systems run.
        *   Reorganize the plugin structure.

*   `unstd/src/systemparam/mod.rs`, `unstd/src/systemparam/interactivestuff.rs`:  Defines custom `SystemParam` structs for convenient access to related data.
    *   **Purpose:**  Reduces boilerplate code and improves readability in systems.
    *   **Interactions:**  Used as parameters in system functions.
    *   **Feature X Relevance:**  Would be touched to:
        *   Add new system parameters.
        *   Modify existing system parameters.

*   `unstd/src/tiledmap.rs`:  Provides utilities for working with Tiled map data.
    *   **Purpose:**  Simplifies interacting with data loaded from TMX and TSX files.
    *   **Interactions:**  Used by map loading and tile management systems.
    *   **Feature X Relevance:**  Would be touched if:
        *   The way Tiled map data is handled needs to be changed.
        *   New features or properties from Tiled need to be supported.

**Non-Source Code Files (Concise Summary)**

**1. Project Metadata and Configuration (already covered, but included for completeness):**

*   `.cargo/config.toml`: Cargo target-specific settings (compiler flags, linker).
*   `Cargo.toml`: Main project manifest (dependencies, workspace, etc.).
*   `Cargo.lock`: Records exact versions of dependencies.
*   `clippy.toml`: Configuration for the Clippy linter.
*   `index.html`: Entry point for the WASM build.
*   `pkg/.gitignore`:  Files/folders to ignore in the WASM build output.

**2. Documentation and Notes:**

*   `CHANGELOG.md`: Project version history.
*   `GHOST_COMMUNICATION.md`: Design document for the ghost communication system.
*   `NOTES.md`: Scratchpad for notes, ideas, and TODOs.
*   `README.md`: Project overview, build instructions, and community links.

**3. Assets:**

*   `assets/fonts/`: Font files (`.ttf`), organized into subfolders for each font family (e.g., `chakra_petch`, `kode_mono`, `londrina_solid`, `overlock`, `syne`, `titillium_web`, `victor_mono`). Each folder contains variations (Bold, Italic, etc).
*   `assets/img/`: Image files (mostly `.png`, some `.aseprite`, `.xcf`, and one `.svg`), organized into subfolders:
    *   `assets/img/base-tiles/`:  Tiles for floors, walls, frames, etc.  Subdivided into categories like `base`, `bath`, `decor`, `furniture`.
    *   `assets/img/base-tiles/base/`: Basic structural tiles (floor, wall, etc)
    *   `assets/img/base-tiles/bath`: Bathroom tiles
    *   `assets/img/base-tiles/characters`: player characters, ghost
    *   `assets/img/base-tiles/decor`: Decorations
    *   `assets/img/base-tiles/furniture`: Furniture
    *   `assets/img/`:  General game images (breach, character, floor, ghost, light, miasma, etc.).
    *   `assets/img/src`: Source files for some images, often in GIMP's `.xcf` format.
    * There are a lot of `aseprite` files. Aseprite is a specialized editor for Pixel Art.

*   `assets/maps/`: Map files (`.tmx`, `.tsx`), metadata (`.ron`), and related files.
    *   `.tmx`:  Tiled Map Editor files, defining the layout of each level.
    *   `.tsx`:  Tiled Tileset files, defining the properties of tiles used in the maps.
    *   `metadata/tileset.ron`:  Likely a RON file containing additional metadata about the tilesets.
    *   `parse_test.py`: Probably a Python script used for testing or parsing map data.

*   `assets/music/`:  Music files (`.ogg`).
    * `unhaunter_intro.ogg`

*   `assets/sounds/`: Sound effect files (`.ogg`). Organized into subfolders for different types of sounds (ambient, effects, ghost, items).
    *  `ambient-clean.ogg`, `background-noise-house-1.ogg`, `birds-clean.ogg` are background sounds.
    *  `door-close.ogg`, `door-open.ogg`, `hide-rustle.ogg` are sound effects of interactions
    * `effects-*.ogg`: Short sounds (chirps, dings) probably used for UI or gear feedback.
    * `ghost-*.ogg`: sounds the ghost does (roar, snore)
    * `item-*.ogg`: interaction with items (drop, move, pickup)
    * `quartz_crack.ogg`, `sage_activation.ogg`, `salt_drop.ogg`: Sounds for consumables
    * `switch-on-1.ogg`, `switch-off-1.ogg`: Sound for light switches.

*   `assets/shaders/`:  Shader files (`.wgsl`).
    *   `custom_material1.wgsl`, `custom_material2.wgsl`:  Shaders for map tiles and sprites.
    *   `uipanel_material.wgsl`: Shader for UI panels.

* `assets/manual/images/`: Images used in the in-game manual
    *   Organized by chapter (`chapter1`, `chapter2`, etc.).
    *   Each chapter folder contains PNG images used in that chapter.
    * `chapter1/README.md`: Contains a list of images per page on chapter1

*   `assets/phrasebooks/`: YAML files defining phrases used in the ghost communication system.
    *   `ghost.yaml`, `ghost_format.yaml`:  Define ghost responses and their structure.
    *   `player.yaml`, `player_format.yaml`: Define player phrases and their structure.
    * `assets/phrasebooks/player/standard_phrases`: Grouped by intent and tone (farewells, greetings, questions, etc.).
    * `assets/phrasebooks/player/extra_phrases`: Contains extra phrases in many categories.

*  `assets/sample_ghosts/`: YAML files defining sample ghosts (e.g., `poltergeist.yaml`, `shade.yaml`).
    * `ghost_metadata_format.yaml`: Describes the structure of the ghost data.

*   `assets/index`: It contains `.assetidx` files, crucial for wasm builds. These files are generated by the `build.rs`
    * `fonts-ttf.assetidx`
    * `img-png.assetidx`
    * `manual-png.assetidx`
    * `maps-tmx.assetidx`
    * `maps-tsx.assetidx`
    * `music-ogg.assetidx`
    * `phrasebooks-yaml.assetidx`
    * `sounds-ogg.assetidx`

**4. Build Artifacts and Temporary Files:**

*   `.gitignore`:  Specifies intentionally untracked files that Git should ignore.
*   `screenshots/`: Contains screenshots of the game.

**5. Tools**
* `tools/ghost_radio`: A console-based tool for testing the ghost communication.

