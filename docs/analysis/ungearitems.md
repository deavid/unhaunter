# Crate Analysis: `ungearitems`

## Core Responsibilities

`ungearitems` contains the concrete implementations of all the equipment (gear) in the game. It implements the
`GearUsable` trait (from `ungear`) for each specific item, defining its unique behavior, visuals, and interactions.

## Public API

The crate exposes:

- **`components` Module**:

  - Contains a submodule for each gear type (e.g., `flashlight`, `emfmeter`, `thermometer`).
  - Each submodule defines a struct (e.g., `Flashlight`) that implements `GearUsable`.

- **`from_gearkind` Module**:
  - **`FromGearKind` Trait**: A factory pattern implementation that converts a `GearKind` enum variant into a fully
    initialized `Gear` object containing the correct `GearUsable` implementation.

## Dependencies

- `undifficulty`
- `uncore-foundation`
- `uncore-board`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `uncore-systems`
- `ungear`
- `bevy`
- `bevy_platform`
- `rand`
- `ndarray`
- `enum-iterator`
- `fastapprox`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** Each file in `components/` is responsible for exactly one piece of gear.
  This makes the codebase highly navigable and maintainable.
- **Open/Closed Principle:** Adding a new item requires adding a new file in `components/` and one line in the `match`
  statement in `from_gearkind.rs`. The rest of the game doesn't need to change.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **Logic-Driven:** Most gear logic is about reading values (temperature, EMF) or toggling states (on/off).
  - **Visual Abstraction:** While they return `GearSpriteID`s, the logic itself is mostly independent of the rendering
    dimension.
  - **Spatial Interaction:** Some items (like `Flashlight` or `MotionSensor`) might have spatial logic (cones of light,
    proximity checks) that currently rely on 2D vectors or grid positions, but this is encapsulated within the item's
    implementation.

### Game Logic vs. Engine Logic

- **Conclusion:** **Pure Game Content**.
- **Reasoning:** These are the specific "toys" of _Unhaunter_. They define the gameplay tools available to the player.

### Other Notes

- **Factory Pattern:** The `FromGearKind` implementation acts as a central factory for instantiating gear, decoupling
  the creation logic from the usage logic.
