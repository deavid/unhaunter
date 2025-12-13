# Crate Analysis: `unghost`

## Core Responsibilities

`unghost` implements the Artificial Intelligence (AI) and behavior logic for the ghost entity. It is the "brain" of the
antagonist. Its responsibilities include:

- **State Machine:** Managing the ghost's current state (Idle, Hunting, Roaming, Interacting).
- **Pathfinding & Movement:** Calculating where the ghost should move next, avoiding obstacles and targeting the player.
- **Procedural Behavior:** Using Perlin noise to dynamically vary the ghost's aggression, visibility, and interaction
  frequency over time (`dynamic_behavior_update`).
- **Interaction System:** Deciding when and how to interact with the environment (flickering lights, throwing objects)
  via the `gis` (Ghost Interaction System).
- **Visuals:** Managing the ghost's sprite, visibility (fading in/out), and special effects (orbs, focus rings).

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterGhostPlugin`**: Registers the ghost AI systems.

- **`ghost` Module**:

  - **`GhostSprite`**: (Re-exported or extended) The main component holding ghost state.
  - **Logic**: The core `ghost_cycle` (likely in `ghost.rs`) that drives the frame-by-frame decision making.

- **`systems` Module**:
  - **`dynamic_behavior_update`**: Updates the `GhostBehaviorDynamics` component to create organic, unpredictable
    behavior.
  - **`gis`**: Sub-systems for handling specific interactions.

## Dependencies

- `undifficulty`
- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-resources`
- `uncore-components`
- `uncore-systems`
- `unstd`
- `ungear`
- `ungearitems`
- `bevy`
- `rand`
- `ordered-float`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate focuses on "Ghost Logic". However, `ghost.rs` being over 1000
  lines suggests it might be a "God Object" for the ghost's internal logic. It could potentially be refactored into
  smaller state handlers.
- **Coupling:** It is highly coupled to the board (`uncore-board`) for movement and the player (`uncore-components`) for
  targeting. This is expected for an AI agent.

### 2D/3D Coupling

- **Conclusion:** **High Coupling.**
- **Reasoning:**
  - **Movement:** The pathfinding and movement logic likely relies on `BoardPosition` (grid) and `Position` (2D vector
    with Z-sorting).
  - **Visibility:** Logic for "line of sight" or "hiding" is often simplified in 2D (e.g., checking grid cells) compared
    to 3D (raycasting against meshes).
  - **Visuals:** The ghost is rendered as a `Sprite`, and effects like `FocusRing` are 2D.

### Game Logic vs. Engine Logic

- **Conclusion:** **Pure Game Logic**.
- **Reasoning:** This is the specific AI for the _Unhaunter_ ghost. It defines the core gameplay loop of "Cat and
  Mouse".

### Other Notes

- **Procedural Generation:** The use of Perlin noise for behavior (`dynamic_behavior_update`) is a sophisticated touch
  that adds depth to the gameplay, making the ghost feel "alive" rather than just a state machine.
