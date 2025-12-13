# Crate Analysis: `unplayer`

## Core Responsibilities

The `unplayer` crate is the central hub for player character logic. It manages:

- **Input Handling**: Processing keyboard and mouse inputs to control the player.
- **Movement**: Implementing movement physics, collision detection, and navigation (both direct control and
  waypoint-based).
- **State Management**: Tracking player health, sanity, stamina, and hiding status.
- **Interaction**: Handling interactions with the environment (grabbing, dropping, hiding).
- **Visuals**: Updating player animations and sprites based on state and movement.

## Architectural Analysis

- **Type**: Feature Crate (Player Logic)
- **Role**: Implements the "Controller" and "Model" aspects of the player character.
- **Dependencies**:
  - `uncore_board`: For position and environment data.
  - `uncore_components`: For player components (`PlayerSprite`, `Stamina`, `Hiding`).
  - `uncore_resources`: For game state and input resources.
  - `uncore_systems`: For utility systems.
  - `undifficulty`: For difficulty scaling (sanity loss).
  - `ungear`: For inventory management.
  - `unprofile`: For persistent player data.
  - `unstd`: For interaction utilities.

## 2D/3D Coupling

- **High Coupling**:
  - **Movement**: `movement.rs` relies heavily on `Position` (2D grid coordinates) and `Direction` (2D cardinal/ordinal
    directions).
  - **Environment Interaction**: `sanityhealth.rs` accesses `BoardData` fields (`light_field`, `temperature_field`)
    which are 2D grid-based.
  - **Visuals**: `PlayerSprite` and `AnimationTimer` are designed for 2D sprite animation.
  - **Pathfinding**: `pathfinding.rs` (implied) and `waypoint.rs` likely operate on the 2D grid.

## Migration Risks

- **Movement Logic**: The movement system is tightly coupled to the 2D grid and will need significant refactoring for 3D
  (e.g., using `Transform` instead of `Position`, continuous movement instead of grid-snapped or grid-aware movement).
- **Environment Querying**: Accessing environmental data (light, temp) from `BoardData` will need to be adapted to query
  the 3D environment (e.g., using raycasts or 3D spatial partitioning).
- **Input Handling**: Mouse interaction (click-to-move, click-to-interact) will need to be updated to handle 3D
  raycasting from the camera.

## Key Files

- `src/systems/movement.rs`: Core movement logic.
- `src/systems/sanityhealth.rs`: Sanity and health mechanics.
- `src/systems/waypoint.rs`: Click-to-move navigation.
- `src/systems/input/`: Input processing (keyboard, mouse).
