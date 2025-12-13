# Crate Analysis: `uncore-systems`

## Core Responsibilities

This crate provides a collection of core Bevy `System`s and a `Plugin` that ties them together. Systems contain the main logic of the game, operating on the components and resources defined in other crates and communicating via events. This crate is where the "verbs" of the game are implemented, processing data to create behavior.

## Public API

The crate exposes several modules containing systems and utilities:

-   **`plugin` Module**:
    -   **`UnhaunterCorePlugin`**: The main plugin for this crate. Its `build` function is responsible for adding all the core systems and initializing necessary resources into the Bevy `App`. This acts as the central setup for the crate's logic.

-   **`systems` Module**:
    -   **`ambient_sound_mute`**: Contains the `process_ambient_mute_events` system, which listens for `AmbientSoundMuteEvent`s and manages the `AmbientMuteController` resource to control sound levels.
    -   **`animation`**: Contains the `animate_sprite` system, which drives 2D sprite animations by updating the `index` of `TextureAtlas` components based on `AnimationTimer`s. **(High 2D Coupling)**
    -   **`board`**: Contains the `sync_map_entity_field` system, an optimization that keeps the spatial lookup grid in the `BoardData` resource synchronized with the actual `Position` of entities.
    -   **`evidence_decay`**: Contains the `decay_evidence_clarity_system`, which gradually reduces the `clarity` of evidence readings over time if they are not actively being refreshed.

-   **Supporting Modules**:
    -   **`metric_recorder`**: A system for recording and sending performance metrics and diagnostics via a global `mpsc` channel to Bevy's `DiagnosticsStore`.
    -   **`noise`**: Defines a `PerlinNoise` resource that precomputes a large lookup table for Perlin noise values, a performance optimization to avoid expensive calculations at runtime.
    -   **`platform`**: Uses conditional compilation (`#[cfg]`) to provide platform-specific constants (e.g., `IS_WASM`, `ASPECT_RATIO`).
    -   **Phantom Modules**: `systemparam`, `traits`, and `utils` are declared in `lib.rs` but do not have corresponding files, likely indicating they are remnants of a past refactor.

## Dependencies

-   `uncore-foundation`
-   `uncore-board`
-   `uncore-events`
-   `uncore-resources`
-   `uncore-components`
-   `bevy`
-   `bevy_platform`
-   `enum-iterator`
-   `itertools`
-   `noise`

## Architectural Analysis & Notes

### SOLID Principles
-   **Single Responsibility Principle (SRP):** This crate has a reasonably well-defined responsibility: to house common, core systems. However, like `uncore-components` and `uncore-resources`, it's a "grab bag" of systems that could potentially be broken down further. For example, `animation` could be its own crate. The `UnhaunterCorePlugin` aggregates these systems, which is a good pattern.
-   The use of `app_setup` functions within each system module to encapsulate its own setup logic is a good practice that improves modularity within the crate.

### 2D/3D Coupling
-   **Conclusion:** **High Coupling.**
-   **Reasoning:**
    -   The `animation` system is explicitly designed for 2D sprite sheet animation, as it directly manipulates the `index` of a `TextureAtlas`. This would be entirely replaced in a 3D context with a different animation system (e.g., for skeletal animation).
    -   The `board` system, by operating on `BoardData`, `BoardPosition`, and `Position`, inherits the very high 2D isometric coupling from `uncore-board`.
    -   While systems like `evidence_decay` and `ambient_sound_mute` are logic-based and not directly coupled, they operate within a game architecture that is fundamentally 2D. The `platform` module's `ASPECT_RATIO` constant is also a 2D-centric concern.
-   The logic in this crate, especially for anything visual or spatial, would need significant rewrites for a 3D pivot.

### Game Logic vs. Engine Logic
-   **Conclusion:** A mix, but with a clearer separation than in some other crates.
-   **Reasoning:**
    -   **Engine-level:** The patterns and systems here are quite foundational. An animation system, a board synchronization system, a metrics recorder, and a noise generator are all common engine components.
    -   **Game-specific:** The `evidence_decay` system is highly specific to *Unhaunter*'s evidence mechanics. The specific implementation details of the `animation` and `board` systems are also tied to game-specific components (e.g., how characters animate, how the board is structured).
-   To evolve into an engine, the generic systems (`animation`, `board` synchronization) would need to be made more abstract to operate on generic "character" or "spatial" components, while game-specific logic like `evidence_decay` would be moved out into a `unhaunter-game-systems` crate.

### Other Notes
-   **Dependency Hub:** This crate depends on nearly all the other `uncore-*` crates (`foundation`, `board`, `events`, `resources`, `components`), which is expected for a systems crate. It sits at a high level in the dependency graph, consuming data and components to produce behavior.
-   **Performance Optimizations:** The `PerlinNoise` resource with its precomputed lookup table is another example (along with `uncore-assets`'s naive parsers) of performance-conscious design in the codebase.
-   **Incomplete Refactoring:** The presence of "phantom modules" (`systemparam`, `traits`, `utils`) in `lib.rs` provides strong evidence of past refactoring efforts and suggests that the codebase could benefit from further cleanup and organization.
