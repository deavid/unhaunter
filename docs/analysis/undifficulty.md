# Crate Analysis: `undifficulty`

## Core Responsibilities

`undifficulty` is the central repository for game balance and difficulty configuration. It defines the
`DifficultySettings` trait and implements it for the `Difficulty` enum (defined in `uncore-types`). It acts as a
"Configuration Provider" that translates a high-level difficulty selection (e.g., "Master Challenge") into concrete
gameplay values (e.g., "Ghost Speed = 2.5").

## Public API

The crate exposes:

- **`difficulty_settings` Module**:

  - **`DifficultySettings` Trait**: Defines the interface for accessing all tunable game parameters (ghost speed, sanity
    drain, gear availability, etc.).
  - **`DifficultyStruct`**: A struct that holds a snapshot of these settings, which is used as the `CurrentDifficulty`
    resource.

- **`current_difficulty` Module**:

  - **`CurrentDifficulty` Resource**: A Bevy `Resource` that holds the active configuration for the current game
    session.

- **`difficulty_impl` Module**:
  - **`impl DifficultySettings for Difficulty`**: The massive `match` statements that return specific values for each
    parameter for each difficulty level. This is where the actual game balance data lives.

## Dependencies

- `uncore-foundation`
- `uncore-types`
- `uncore-components`
- `bevy`
- `serde`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate has one job: "Define Game Balance Values". It separates the
  _definition_ of difficulty (the enum in `uncore-types`) from the _implementation_ of what that difficulty means (the
  values here).
- **Open/Closed Principle:** Adding a new difficulty level requires updating the `match` arms in `difficulty_impl.rs`,
  but adding a new _parameter_ (e.g., `ghost_invisibility_duration`) involves updating the trait and all
  implementations. This is a trade-off for having a centralized config.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **Data-Centric:** It returns `f32`, `u32`, and `bool` values.
  - **Abstract:** Concepts like "Ghost Speed" or "Sanity Drain" are applicable to both 2D and 3D games.
  - **Minor Coupling:** Some parameters might be tuned for the specific scale of the 2D world (e.g.,
    `hunt_provocation_radius`), but the _concept_ is generic.

### Game Logic vs. Engine Logic

- **Conclusion:** **Pure Game Logic**.
- **Reasoning:** This is the definition of the game's rules and balance. It dictates exactly how hard the game is and
  how mechanics behave.

### Other Notes

- **Centralized Tuning:** Having all these magic numbers in one place (`difficulty_impl.rs`) makes balancing the game
  much easier than having them scattered across various systems.
- **Resource Pattern:** The use of `CurrentDifficulty` as a resource allows any system to easily query "how fast should
  the ghost be right now?" without needing to know _which_ difficulty is selected.
