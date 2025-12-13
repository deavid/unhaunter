# Crate Analysis: `unfog`

## Core Responsibilities

`unfog` implements the "Miasma" system, which is a dynamic, grid-based fluid simulation used to represent fog, gas, or
supernatural presence. It manages the simulation of miasma pressure, velocity, and diffusion across the game board, as
well as the visual representation of this fog using sprites.

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterFogPlugin`**: Initializes the `MiasmaConfig` resource and registers the simulation systems.

- **`resources` Module**:

  - **`MiasmaConfig`**: Configuration parameters for the fluid simulation (diffusion rate, friction, etc.).

- **`components` Module**:

  - **`MiasmaSprite`**: Marker component for the visual entities representing the fog.

- **`systems` Module**:
  - **`initialize_miasma`**: Sets up the initial state of the miasma field when a level is loaded.
  - **Simulation Systems**: (Inferred from `systems.rs` imports and context) Systems that update the `BoardData` miasma
    fields based on fluid dynamics equations.

## Dependencies

- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `uncore-systems`
- `unstd`
- `bevy`
- `bevy_platform`
- `rand`
- `ndarray`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is well-scoped to "Miasma Simulation". It handles both the
  physics (fluid dynamics) and the rendering (sprite management) of this specific feature.
- **Performance:** The use of `ndarray` indicates a focus on performance for the grid-based simulation, which is
  computationally intensive.

### 2D/3D Coupling

- **Conclusion:** **High Coupling.**
- **Reasoning:**
  - **Grid-Based:** The simulation runs on the `BoardData` grid, which is tied to the 2D tile layout.
  - **Sprite Rendering:** It uses `MiasmaSprite` and likely `SpriteBundle` (via `GameSprite`) to render the fog. In a 3D
    game, volumetric fog or particle systems would be the standard approach, not a grid of 2D sprites.
  - **Isometric Projection:** The visual positioning of the fog sprites relies on the same 2D isometric projection as
    the rest of the board.

### Game Logic vs. Engine Logic

- **Conclusion:** **Engine Feature (Physics/VFX)**.
- **Reasoning:** While the _existence_ of Miasma is a game design choice, the implementation is a generic fluid
  simulation on a grid. It could be reused for smoke, water, or other gas effects in a different 2D tile-based game.

### Other Notes

- **Simulation Complexity:** The `MiasmaConfig` exposes parameters like `inertia_factor` and `friction`, suggesting a
  relatively sophisticated simulation model for a 2D game.
