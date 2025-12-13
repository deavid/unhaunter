# Crate Analysis: `unlight`

## Core Responsibilities

`unlight` is responsible for the game's lighting and visibility systems. It handles:

- **Lighting Simulation:** Calculating how light propagates through the game world from various sources (ambient,
  flashlights, lamps, ghost effects).
- **Visibility Calculation:** Determining what the player can see (Line of Sight) and applying "Fog of War" or darkness.
- **Visual Application:** Applying the calculated lighting values to the sprites and materials (`CustomMaterial1`) to
  render the scene correctly.
- **Audio Integration:** It seems to handle some audio logic (`audio` module), possibly related to ambient sounds or
  sound propagation which might share similar logic to light/visibility.

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterLightPlugin`**: Registers the lighting and audio systems.

- **`maplight` Module**:

  - **`app_setup`**: Registers the main lighting update systems.
  - **Logic**: Contains the heavy lifting for updating sprite colors and visibility based on the calculated light field.

- **`lighting` Module**:
  - **`rebuild_lighting_field`**: The core algorithm that updates the `LightFieldData` in `BoardData`. It uses a wave
    propagation algorithm (and prebaked data) to simulate light spreading.

## Dependencies

- `undifficulty`
- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `uncore-systems`
- `unstd`
- `unfog`
- `ungear`
- `ungearitems`
- `unsettings`
- `bevy`
- `bevy_platform`
- `rand`
- `ndarray`
- `fastapprox`
- `bevy-persistent`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate focuses on "Light and Visibility". The inclusion of `audio` is a
  bit suspicious but might be justified if it uses the same occlusion/propagation logic as light.
- **Performance:** The use of `ndarray`, `fastapprox`, and "prebaked" lighting contributions (`prebake` module)
  indicates that this is a performance-critical system. Lighting calculation on a CPU grid is expensive.

### 2D/3D Coupling

- **Conclusion:** **Extreme Coupling.**
- **Reasoning:**
  - **Grid-Based Propagation:** The entire lighting engine (`lighting.rs`) is built around propagating values through
    the `BoardData` 3D grid (which is really a stack of 2D grids).
  - **Sprite Coloring:** The output of the system is directly applied to `CustomMaterial1` and sprite colors
    (`maplight.rs`). In a modern 3D engine, lighting is handled by the renderer (shaders, point lights, ambient
    occlusion), not by manually tinting sprites on the CPU.
  - **Custom Physics:** It implements its own light physics (transmission, absorption) tailored for the tile grid.

### Game Logic vs. Engine Logic

- **Conclusion:** **Engine Implementation (Custom Rendering)**.
- **Reasoning:** This is a custom lighting engine built on top of Bevy. It's not "gameplay" (rules), but "tech" (how to
  make things look lit).

### Other Notes

- **Pivot Implication:** This entire crate is likely obsolete in a full 3D pivot. Bevy's PBR (Physically Based
  Rendering) pipeline handles lights, shadows, and occlusion automatically. We would replace this custom grid
  propagation with standard Bevy `PointLight`, `SpotLight`, and `DirectionalLight` components.
