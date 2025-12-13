# Crate Analysis: `ungame`

## Core Responsibilities

`ungame` is a collection of high-level gameplay systems and UI logic. It implements specific game mechanics that don't
fit into lower-level "engine" crates. Its responsibilities include:

- **Gameplay Mechanics:** Implementing rules like the fuse box overload system (`environmental_mechanics`) and evidence
  perception (`evidence_perception`).
- **UI Management:** Handling the in-game UI (`game_ui`), gear selection UI (`gear_ui`), and pause menu (`pause_ui`).
- **State Updates:** Updating the board state based on game events (`boardfield_update`, `roomchanged`).
- **Player Interaction:** Managing how the player interacts with objects (`object_charge`) and views gear
  (`looking_gear`).

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterGamePlugin`**: Registers all the gameplay systems and initializes the `GameConfig` resource.

- **`systems` Module**:

  - **`app_setup`**: Likely registers the core gameplay loop systems.

- **Specific Mechanics Modules**:
  - **`environmental_mechanics`**: Contains the `fuse_box_overload_system`.
  - **`evidence_perception`**: Updates `CurrentEvidenceReadings` based on what the player is holding and seeing.
  - **`boardfield_update`**: Updates the board's logical state (e.g., temperature, lighting) based on entity positions.

## Dependencies

- `undifficulty`
- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `unstd`
- `ungear`
- `unsettings`
- `unlight`
- `unprofile`
- `bevy`
- `bevy_platform`
- `bevy-persistent`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is a bit of a "Gameplay Logic" aggregator. While individual
  modules (like `evidence_perception`) have clear responsibilities, the crate as a whole is a collection of loosely
  related gameplay features. This is common for a "game" crate in an engine-game split.
- **Coupling:** It depends on almost everything else (`ungear`, `unlight`, `unprofile`, etc.), which is expected for the
  top-level gameplay logic.

### 2D/3D Coupling

- **Conclusion:** **Medium Coupling.**
- **Reasoning:**
  - **Logic-Centric:** Many systems (like fuse box overload or evidence perception) are based on logical states
    (counters, timers, enums) rather than spatial geometry.
  - **Board Dependence:** Systems like `boardfield_update` and `roomchanged` likely interact with the `BoardData` grid,
    which is 2D-based.
  - **UI:** The UI modules are 2D but agnostic to the game world's dimension.

### Game Logic vs. Engine Logic

- **Conclusion:** **Pure Game Logic**.
- **Reasoning:** This crate defines the specific rules of _Unhaunter_. The fact that lights trip the breaker or that
  specific gear reveals specific evidence is unique to this game.

### Other Notes

- **Mechanics Implementation:** This is where the "fun" is implemented. It translates the data from `undifficulty` and
  the state from `uncore-components` into actual player experiences.
