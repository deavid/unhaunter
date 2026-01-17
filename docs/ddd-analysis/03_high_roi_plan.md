# The 90% Readiness Plan: "Mechanism over Meaning"

This Plan of Action is designed to be a **High-ROI Refactor**. We will focus on breaking the "hard-coded dependencies"
that prevent the engine from being a generic platform, while ignoring minor aesthetic cleanups or deep-level
serialization changes that offer diminishing returns.

Our target is to move from **75% to 90% Engine Readiness.**

---

## Phase 1: The Physics Decoupling (High Return)

**Goal:** Solvers (`unthermal`, `unfog`, `unsound`) should not know the "Ghost" exists.

1. **Define Generic Emitters:** In `unboard-core` or a new `unphysics-core`, define generic components:
   - `ThermalEmitter { target_temp: f32, power: f32 }`
   - `FluidEmitter { pressure: f32 }`
   - `SoundEmitter { volume: f32 }`
2. **Refactor Solvers:** Update the systems in `unthermal-plugin`, `unfog-plugin`, and `unsound-plugin` to query for
   these components instead of `GhostSprite`.
3. **Update Ghost:** In `unghost-plugin`, simply attach these emitter components to the Ghost entity.
4. **The Win:** You can now have "Haunted Objects," "Leaking Pipes," or "Multiple Entities" affecting the physics fields
   without changing a single line of simulation code.

## Phase 2: The Orchestration Split (High Return)

**Goal:** The Map Loader should be a "Stage Hand," not a "Game Master."

1. **Spawn Point components:** Create `GhostSpawnPoint` and `PlayerSpawnPoint` components.
2. **Loader Refactor:** Modify `unmapload-plugin` to simply spawn these points and fire a `MapGeometryInitializedEvent`.
3. **Introduce the GM Plugin:** Create `unclassic-mode-plugin`. This plugin listens for the geometry event and _then_
   performs the logic of:
   - Picking the Ghost type.
   - Populating `haunt_state.evidences`.
   - Spawning the actual `GhostSprite` at one of the `GhostSpawnPoint` locations.
4. **The Win:** This is the "Mod Seam." You can now swap `unclassic-mode-plugin` for an `unescape-mode-plugin` and reuse
   the map loader perfectly.

## Phase 3: Generic Visual Representation (High Return)

**Goal:** Break the "God System" in `unlight-plugin`.

1. **Visual Interaction Traits:** Create components that describe _how_ an entity reacts to light, rather than _what_ it
   is.
   - `UVVisible { clarity: f32 }`
   - `RedLightVisible { clarity: f32 }`
   - `AlphaModulator { multiplier: f32 }` (for the Ghost's flickering alpha).
2. **Refactor `apply_lighting`:** Remove the `if SpriteType == Ghost` logic. Replace it with a query for these new
   components.
3. **The Win:** Adding a new evidence type (e.g., "Infrared Residue") no longer requires modifying the complex lighting
   shader logic. You just add a component to an entity.

## Phase 4: Structural Polish (Medium Return)

**Goal:** Fix the most egregious "Semantic Leaks" in the core data.

1. **Move Pathfinding:** Extract the A\* logic from `unplayer-plugin` and move it to `unnavigation-core`. Provide a
   generic `Pathfinder` SystemParam.
2. **De-Isometrize `unspatial`:**
   - Move the `PERSPECTIVE` constants and `to_screen_coord` math to a `unrender-std` utility.
   - Remove `global_z` from the core `Position` struct if possible, or rename it to something generic like
     `visual_priority`.
3. **Decompose `GearStuff`:** Break the `GearStuff` bundle into smaller, domain-specific traits (e.g.,
   `ThermodynamicsReader`, `SoundReader`).

---

# Impact Assessment

| Action                       | Difficulty | Return on Investment                                 |
| :--------------------------- | :--------- | :--------------------------------------------------- |
| **Emitter Components**       | Low        | **Extreme.** Decouples simulation from gameplay.     |
| **Orchestration Split**      | Medium     | **High.** Enables multiple game modes.               |
| **Visual Interaction Trait** | Medium     | **High.** Fixes the "God Plugin" maintenance burden. |
| **Pathfinding Move**         | Low        | **Medium.** Centralizes a core engine mechanism.     |
| **Isometric Decoupling**     | Low        | **Medium.** Prepares for 3D/Top-Down pivots.         |

---

## Conclusion: The "90% State"

After these four phases, your engine will be a **Platform**. A developer (or you, building Escape Mode) could:

1. Load a map (Stage Hand).
2. Define how things move (Navigation).
3. Define how the atmosphere behaves (Physics Solvers).
4. And finally, add "Meaning" by spawning entities that utilize the Emitters, Pathfinders, and Visual traits.

**This plan avoids "Refactor for Refactor's sake" and targets only the hard dependencies that currently lock
Unhaunter-the-game to Unhaunter-the-engine.**
