# Crate Analysis: `uncore-foundation`

## Core Responsibilities

This crate serves as the foundational data dictionary for the entire project. It defines the core, stable, and
universally understood concepts of the game world. It is primarily composed of data-only structs and enums with minimal
logic.

## Public API

The crate exposes several key modules and types:

- **`colors`**: Defines the application's color palette.
- **`platform`**: Provides platform-specific helpers (e.g., for web vs. native).
- **`random_seed`**: Utilities for seeding random number generators.
- **`difficulty`**:
  - `Difficulty` enum: Defines game difficulty levels (e.g., `TutorialChapter1`, `MasterChallenge`).
  - `DifficultyStruct` (Bevy Resource): A comprehensive struct holding all gameplay parameters that change with
    difficulty (e.g., `ghost_speed`, `sanity_drain_rate`).
  - `CurrentDifficulty` (Bevy Resource): Holds the active `DifficultyStruct` for the current game session.
- **`types` module**:
  - **`Evidence` enum**: Defines the types of evidence players can find (e.g., `FreezingTemp`, `EMFLevel5`).
  - **`Grade` enum**: Defines the A-F performance grades for scoring.
  - **`ghost` module**:
    - `GhostType` enum: The master list of all ghost types.
    - `GhostPersonality` struct: Defines behavioral patterns (e.g., `aggressive`, `subtle`) with rates for various
      interactions. Each `GhostType` is mapped to a personality.
    - `GhostSet` enum: Pre-defined collections of ghosts for specific scenarios.

## Dependencies

- `bevy_prelude`
- `serde`
- `enum_iterator`
- `thiserror`
- `uncore-components` (via `difficulty.rs`)
- `uncore-types` (via `difficulty.rs`)

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** This crate generally adheres well to SRP. The concepts are well-delineated
  into modules (`difficulty`, `ghost`, `evidence`). It focuses strictly on defining core data types, leaving their
  manipulation and logic to other crates.
- **Open/Closed Principle:** The use of enums and structs allows for extension. For example, adding a new `GhostType`
  requires changes here, but the systems using `GhostType` elsewhere might not need modification if they are programmed
  against the abstraction.

### 2D/3D Coupling

- **Conclusion:** **Very low to no coupling.**
- **Reasoning:** This crate consists almost entirely of abstract data and game concepts. It does not contain any
  rendering logic, 2D vector math, or sprite/mesh information. This is a huge advantage for a future 3D pivot, as this
  foundational layer will likely require minimal changes.

### Game Logic vs. Engine Logic

- This crate contains a mix of both:
  - **Game-specific Logic:** `GhostType` (the specific list of ghosts), `GhostPersonality` (the specific behaviors), and
    `GhostSet` are all highly specific to the _Unhaunter_ game.
  - **Engine-level Concepts:** The ideas of `Difficulty`, `Evidence`, `Grade`, and even a generic `Ghost` concept could
    be considered reusable "engine" components for other paranormal investigation games. This is a key area to consider
    when refactoring towards a more generic engine.

### Other Notes

- The `difficulty.rs` file is not publicly exported in `lib.rs`. This appears to be an intentional architectural choice
  to prevent a circular dependency, as `difficulty.rs` depends on `uncore-types` and `uncore-components`. This is a
  crucial piece of information for understanding inter-crate coupling.
