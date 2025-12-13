# Codebase Topology & System Map

## System Overview

The `unhaunter` project is a modular Rust application built on the Bevy engine. It employs a **Core-Feature**
architecture to separate foundational data/logic from gameplay implementations.

## Crate Hierarchy

### 1. The Core (Foundation & Data)

These crates provide the "Ubiquitous Language" and shared state of the application. They have few dependencies and are
used by almost every other crate.

- **`uncore-foundation`**: Primitive types, constants, and math utilities.
- **`uncore-types`**: Shared enums and structs (e.g., `Difficulty`, `Evidence`).
- **`uncore-board`**: The spatial model of the world.
  - **Key Component**: `Position` (3D logical coordinates: x, y, z).
  - **Key Resource**: `BoardData` (3D grid storing environment data).
- **`uncore-components`**: Reusable ECS components (`Health`, `Name`, `GhostSprite`).
- **`uncore-resources`**: Global state (`GameState`, `AppState`, `Time`).
- **`uncore-events`**: Event definitions for system communication.

### 2. The Simulation Layer (World Logic)

These crates implement the rules of the game world, operating primarily on the Core data.

- **`unlight`**: Simulates light propagation through the 3D grid (`BoardData`).
- **`unfog`**: Calculates visibility and fog-of-war based on player position.
- **`unmapload` / `untmxmap`**: Handles world generation and loading.
- **`unstd`**: Standard library of game-specific utilities and system parameters.

### 3. The Gameplay Layer (Features)

These crates implement specific game mechanics and entity behaviors.

- **`unplayer`**: Player controller, input handling, and interaction logic.
- **`unghost`**: Ghost AI, behavior trees, and pathfinding.
- **`unnpc`**: NPC interactions and simple behaviors.
- **`ungear` / `ungearitems`**: Inventory system and item definitions.
- **`unwalkie`**: Walkie-talkie mechanics and voice events.

### 4. The Presentation Layer (UI & Config)

These crates handle user interaction and configuration.

- **`untruck`**: The main "Base of Operations" UI.
- **`uncoremenu` / `unmenu`**: Main menu and in-game menu systems.
- **`unsummary`**: End-of-mission reporting.
- **`unsettings`**: Configuration management (Video, Audio, Controls).
- **`unprofile`**: Persistent player data (Save/Load).

## Architectural Patterns

### Entity-Component-System (ECS)

The project strictly follows ECS patterns:

- **Data**: Stored in Components (`Position`, `Behavior`) and Resources (`BoardData`).
- **Logic**: Implemented in Systems (functions in `src/systems/`).
- **Communication**: Decoupled via Events (`MessageWriter`/`MessageReader`).

### Plugin Architecture

Each crate exposes a `Plugin` (e.g., `UnplayerPlugin`) that registers its systems and resources. This adheres to the
**Open/Closed Principle**, allowing the main `App` to be extended easily.

### Spatial Model

The world is modeled as a **3D Grid**.

- **Logic**: Uses `Position` (f32 x, y, z) for continuous logical coordinates within the grid.
- **Storage**: Uses `BoardPosition` (i64 x, y, z) for discrete grid cell access (e.g., looking up light level in
  `BoardData`).
- **View**: Currently rendered via isometric projection, but the underlying model is 3D-ready.

## Key Data Flows

1. **Input**: `unplayer` reads `Input<KeyCode>` -> Updates `PlayerInput` resource.
2. **Movement**: `unplayer` reads `PlayerInput` -> Updates `Position` component.
3. **Simulation**: `unlight` / `unfog` observe `Position` changes -> Update `BoardData` / Visibility.
4. **Rendering**: Visual systems read `Position` -> Update `Transform` (Bevy's visual coordinates).

## Critical Technical Debt

- **Duplicated Types**: The `Position`, `BoardPosition`, and `Direction` structs are defined in **both** `uncore-board`
  and `uncore-components`.
  - `uncore-components` depends on `uncore-board`, so the definitions in `uncore-components` are redundant and
    potentially dangerous.
  - **Action**: Refactor to use `uncore-board` as the single source of truth for spatial types.

## Conclusion

The codebase is highly modular and well-organized. The separation of concerns is enforced by the crate structure,
preventing "spaghetti code". The central reliance on `uncore-board` provides a unified spatial truth for all systems.
