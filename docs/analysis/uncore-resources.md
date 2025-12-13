# Crate Analysis: `uncore-resources`

## Core Responsibilities

This crate defines Bevy `Resource`s and `State`s, which act as singletons holding global mutable game state and controlling the high-level flow of the application. It manages everything from game configuration and UI state to real-time simulation data (like board conditions) and mission progress.

## Public API

The crate exposes two main modules:

### `resources` Module
This module defines various Bevy `Resource`s, which are global singletons:

-   **`AmbientMuteController`**: Manages active ambient sound mute effects and calculates the current global mute multiplier. Depends on `uncore-events`.
-   **`BoardData`**: **(Very High 2D/3D Coupling)** A massive resource holding the entire game board's dynamic state, including `light_field` (`Array3<LightFieldData>`), `collision_field` (`Array3<CollisionFieldData>`), `temperature_field` (`Array3<f32>`), `sound_field`, `map_entity_field` (`Array3<Vec<Entity>>`), `miasma` (`MiasmaGrid`), and prebaked lighting data. It's a central hub for environmental simulation and heavily uses `ndarray` for 3D grids. It has `BoardPosition` and `Position` in its types.
-   **`CliOptions`**: Stores command-line options like `include_draft_maps`.
-   **`CurrentEvidenceReadings`**: Tracks clarity of evidence readings in real-time. Depends on `uncore-foundation`.
-   **`DifficultySelectionState`**: Manages the currently selected difficulty and map index. Depends on `uncore-types`.
-   **`GhostGuess`**: Stores player's current ghost type and evidence guesses. Depends on `uncore-foundation`.
-   **`HintUiState`**: Manages the state and animation of on-screen hint UI.
-   **`LookingGear`**: Tracks which gear item (left/right hand) the player is currently focusing on. Depends on `uncore-types`.
-   **`Manual`**: Holds the content of the in-game manual. Depends on `uncore-types`.
-   **`Maps`**: Stores loaded map data (`Vec<Map>`) and sheets (`Vec<Sheet>`). Depends on `uncore-assets`.
-   **`MissionSelectMode`**: Tracks whether the game is in Campaign or Custom mission selection.
-   **`MouseVisibility`**: Controls the visibility of the mouse cursor.
-   **`ObjectInteractionConfig`**: Global configuration for ghost-object interaction parameters (charge rates, influence multipliers).
-   **`PlayerInput`**: Acts as a virtual joystick, holding player's desired movement direction and optional click-to-move target. **(Moderate 2D Coupling)**: `target_position: Option<Vec2>` is 2D screen coordinate.
-   **`PotentialIDTimer`**: Manages the timer for potential ghost identification based on evidence. Depends on `uncore-foundation`.
-   **`RoomDB`**: Maps `BoardPosition` to room names and tracks room-specific states. Depends on `uncore-board`.
-   **`SummaryData`**: Stores all data related to the end-of-mission summary and scoring. Depends on `uncore-foundation` and `undifficulty`.
-   **`VisibilityData`**: Holds a 3D `Array3<f32>` for visibility field data.
-   **`WalkiePlay`**: Manages walkie-talkie events, their playback stats, and hinted evidence. Depends on `unwalkiecore` and `uncore-foundation`.

### `states` Module
This module defines Bevy `States` enums to manage the application's high-level flow:

-   **`AppState`**: Top-level game states (`Loading`, `MainMenu`, `InGame`, `Summary`, `MapHub`, `UserManual`, `PreplayManual`, `MissionSelect`).
-   **`GameState`**: Sub-states within the main `InGame` state (`None`, `Truck`, `Pause`, `NpcHelp`).
-   **`MapHubState`**: States for the map selection hub (`DifficultySelection`, `None`).

## Dependencies

-   `uncore-board`
-   `uncore-foundation`
-   `uncore-events`
-   `uncore-types`
-   `uncore-assets` (indirectly via `Maps`)
-   `undifficulty` (indirectly via `SummaryData`)
-   `unwalkiecore` (indirectly via `WalkiePlay`)
-   `bevy`
-   `bevy_platform`
-   `ndarray`
-   `enum-iterator`

## Architectural Analysis & Notes

### SOLID Principles
-   **Single Responsibility Principle (SRP):** This crate, like `uncore-components`, acts as a large "God Resource" crate. While each individual resource might adhere to SRP, the crate itself has very low cohesion, bundling together global state for almost every major game system (audio, board, UI, input, game logic, mission data, assets).
    -   **Refactoring Suggestion:** This crate is a prime candidate for decomposition. Resources could be grouped into more domain-specific crates (e.g., `uncore-ui-resources`, `uncore-spatial-resources`, `uncore-game-state-resources`). This would improve modularity and make the codebase easier to reason about.

### 2D/3D Coupling
-   **Conclusion:** **Very High Coupling.**
-   **Reasoning:**
    -   **`BoardData`**: Heavily relies on `BoardPosition` and `Position` (from `uncore-board`), inheriting their very strong 2D isometric coupling. The `Array3` structures themselves are abstract, but their interpretation as grid data in an isometric view is implied.
    -   **`Maps`**: Directly uses `Map` and `Sheet` from `uncore-assets`, which are part of the Tiled map system, strongly tied to 2D isometric representation.
    -   **`PlayerInput`**: `target_position: Option<Vec2>` implies 2D screen coordinates for click-to-move, rather than 3D world coordinates.
    -   **UI-related Resources**: Resources like `HintUiState`, `LookingGear`, `Manual`, `MissionSelectMode`, `MouseVisibility`, and `WalkiePlay` manage UI states that are typically rendered in 2D overlays. While the resources themselves aren't inherently 2D-rendering specific, they manage states for 2D UI elements.
-   Refactoring for 3D would require significant changes to `BoardData`'s interpretation of spatial data, `Maps`' handling of asset structures, and `PlayerInput`'s target positions.

### Game Logic vs. Engine Logic
-   **Conclusion:** A strong mix, leaning heavily towards **Game-specific Logic** but with some reusable patterns.
-   **Reasoning:**
    -   **Game-specific:** Resources like `CurrentEvidenceReadings`, `GhostGuess`, `SummaryData`, `WalkiePlay`, and `ObjectInteractionConfig` are very specific to *Unhaunter*'s core gameplay mechanics, ghost interactions, and scoring.
    -   **Engine-level Concepts:** `AmbientMuteController`, `CliOptions`, `MouseVisibility`, and the general `AppState`/`GameState` management patterns could be seen as engine-level. However, the specific enum variants within `AppState`, `GameState`, and `MapHubState` are tailored to *Unhaunter*'s flow.
-   To evolve into a "ghost game engine," the game-specific resources would need to be moved to a game-specific crate, leaving more generic global state management in a core `engine-resources` crate.

### Other Notes
-   **Dependency Sprawl:** This crate has a large number of dependencies on other core crates (`uncore-board`, `uncore-foundation`, `uncore-events`, `uncore-types`, `uncore-assets`, `undifficulty`, `unwalkiecore`). This is a symptom of its "God Resource" nature and indicates significant coupling across the codebase.
-   **`ndarray` usage:** The use of `ndarray` for `Array3` in `BoardData` and `VisibilityData` highlights the computational intensity of the spatial and environmental simulations.
