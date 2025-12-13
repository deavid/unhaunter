# Crate Analysis: `uncore-components`

## Core Responsibilities

This crate defines the shared ECS components used throughout the game. These components attach data and behaviors to
entities in the Bevy ECS, bridging the gap between raw data types (from `uncore-foundation` and `uncore-types`) and the
systems that operate on them. It covers a wide range of game elements from player and ghost states to UI elements and
board mechanics.

## Public API

The crate exposes a large number of components, grouped by theme:

- **Animation & Visuals:**

  - `AnimationTimer`: Manages sprite sheet animation, tied to specific frame ranges.
  - `CharacterAnimationDirection`, `CharacterAnimationState`, `CharacterAnimation`: Define character animation states
    and directions, converting them to sprite sheet indices.
  - `FocusRing`: Component for visual effects like pulsing rings around entities.
  - `SpriteType`: Enum to categorize different types of sprites (e.g., `Ghost`, `Player`, `Miasma`).
  - `GhostBreach`: Marker component for the ghost's visual spawn effect.
  - `GhostOrbParticle`: Component defining properties of floating ghost orbs, including life, amplitude, frequency, and
    base position.
  - `RepellentParticle`: Component for visual effects of repellent, including life, direction, and hit status.

- **Game Entities (Player/Ghost):**

  - `PlayerSprite`: Contains player-specific data like ID, controls, sanity, health, and spawn position. Depends on
    `uncore_board::components::direction::Direction` and `uncore_board::components::position::Position`.
  - `GhostSprite`: Contains ghost-specific data like type, spawn point, target, rage, hunting state, and interaction
    counters. Depends on `uncore_foundation::types::ghost::types::GhostType` and
    `uncore_board::components::boardposition::BoardPosition`.
  - `HeldObject`: Marker for an object currently held by the player.
  - `Hiding`: Marks a player entity as hiding.
  - `Stamina`: Manages player stamina, running, and recovery.

- **Board & Spatial:**

  - `BoardPosition`: (i64 x, y, z) grid coordinates, including tile dimensions, conversion to `Position`, and neighbor
    iteration.
  - `ChunkIterator`, `CellIterator`: Utilities for iterating over chunks and cells of the map.
  - `Direction`: (f32 dx, dy, dz) vector, capable of converting to 2D screen coordinates using isometric projection
    constants.
  - `MapColor`: Component for coloring map tiles.
  - `Position`: (f32 x, y, z, global_z) floating-point logical position, also capable of converting to 2D screen
    coordinates using isometric projection constants.
  - `MapTileSprite`: Marker for map tiles.
  - `MoveToTarget`: Component to define an entity's target position and optional interaction.
  - `Waypoint`, `WaypointQueue`, `WaypointOwner`: Components for pathfinding and AI movement.

- **Lighting:**

  - `LightSource`: Marker component for light emitters.
  - `LightLevel`: Component to track light levels (lux) at a given position.

- **UI & Configuration:**

  - `GameConfig`: Resource for basic game configuration (e.g., `player_id`).
  - `GameUI`, `EvidenceUI`, `HeldObjectUI`, `RightSideGearUI`, `WalkieTextUIRoot`, `WalkieText`: Marker components for
    various UI elements.
  - `DamageBackground`: Component for visual damage feedback.
  - `HintBoxUIRoot`, `HintBoxText`: Marker components for hint UI.
  - `InventoryNext`, `Inventory`, `InventoryStats`: Components for player inventory management.
  - `SummaryUI`, `SCamera`, `SummaryUIType`: Components for the end-of-mission summary screen.
  - `TabState`, `TabContents`, `TruckTab`: Components for managing truck UI tabs.
  - `TruckUI`, `TruckUIGhostGuess`: Marker components for the truck UI and ghost guess elements.
  - `TruckUIButton`: Component for interactive buttons within the truck UI, handling state, type, and visual feedback.

- **Ghost Mechanics:**
  - `GhostBehaviorDynamics`: Stores dynamic behavioral parameters for ghosts, including per-evidence clarity and noise
    offsets, contributing to unique ghost behaviors.
  - `GhostInfluence`: Defines how objects (attractive/repulsive) affect ghost behavior.

## Dependencies

- `uncore-foundation`
- `uncore-board`
- `uncore-types`
- `uncore-events`
- `bevy`
- `rand`
- `serde`
- `fastapprox`
- `unsettings`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** This crate contains a very wide array of components. While individual
  components might be cohesive, the crate itself has low cohesion. It aggregates components for _all_ game systems
  (player, ghost, UI, animation, board, lighting). This makes it a potential "God Component" crate, which could indicate
  a violation of SRP at the crate level. For a "ghost game engine" vision, these components would need to be
  re-evaluated and potentially re-distributed among more focused crates (e.g., `uncore-player-components`,
  `uncore-ghost-components`, `uncore-ui-components`).

### 2D/3D Coupling

- **Conclusion:** **Very High Coupling.**
- **Reasoning:** This crate exhibits significant coupling to the 2D isometric rendering pipeline:
  - **Explicit Isometric Projection:** `Direction` and `Position` components (and their related helper functions)
    directly incorporate isometric projection constants (`PERSPECTIVE_X`, `PERSPECTIVE_Y`, `PERSPECTIVE_Z`) to convert
    3D logical coordinates into 2D screen coordinates. This is a hard architectural dependency on 2D isometric.
  - **Sprite-based Animation:** `CharacterAnimation` and `AnimationTimer` are designed around sprite sheets and indexed
    frames, which is typical for 2D animation.
  - **`MapTileSprite` and UI Components:** Components related to map tiles and various UI elements are inherently tied
    to 2D screen space and sprite rendering.
  - **`GhostOrbParticle` and `RepellentParticle`:** While their `Vec3` amplitude/frequency might hint at 3D, their
    eventual rendering would rely on the current 2D sprite system.
- Refactoring for a 3D pivot would require a complete overhaul of `Direction::to_screen_coord` and
  `Position::to_screen_coord`, along with replacing sprite-based assets and animation logic with 3D model-based
  equivalents.

### Game Logic vs. Engine Logic

- This crate has a strong mix, with a tendency towards **Game-specific Logic** due to its fine-grained components for
  _Unhaunter_'s specific mechanics.
  - **Game-specific:** `GhostSprite`, `PlayerSprite` (with `crazyness`, `mean_sound`), `GhostBehaviorDynamics`,
    `TruckUI`, `SummaryUI`, `HintBoxUIRoot`, `EvidenceUI`, `HeldObject`. These are clearly tailored to the current game.
  - **Engine-level Concepts:** `AnimationTimer`, generic `LightSource`, `MoveToTarget`, `Waypoint`, and basic
    `Position`/`Direction` could be generalized for an engine. However, even `Position` and `Direction` are currently
    heavily polluted with 2D isometric projection logic, making them game-specific in their current form.
- To become more "engine-like," many of these components would need to be abstracted or separated. For example, a
  generic `Character` component could exist at the engine level, with `UnhaunterPlayer` or `UnhaunterGhost` inheriting
  or composing it at the game level.

### Other Notes

- **Dependency on `uncore-board`:** The `Position` and `Direction` components directly use definitions from
  `uncore-board`, reinforcing the tightly coupled nature of the foundational/spatial crates.
- **`unsettings` dependency:** `PlayerSprite` directly depends on `unsettings::controls::ControlKeys`, embedding control
  scheme logic directly into a player component, which might be a candidate for externalization or a more abstract
  event-driven approach in an engine context.
