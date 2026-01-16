# Consolidated Architectural Audit: Unhaunter Engine

## 1. Executive Summary

The `unhaunter` codebase is a highly sophisticated discrete spatial simulation. It successfully implements complex
"Mechanisms" (Field solvers, personality-driven AI, isometric normalization). However, the project currently suffers
from **Semantic Leakage**, where the "Meaning" of the game (Ghosts, Investigation rules) has bled into the "Mechanisms"
of the engine (Physics, Map Loading).

The engine is approximately **75% ready** for complete decoupling into a generic "Ghost Simulation Platform."

---

## 2. Bounded Context Analysis (Batch Summaries)

### Batch 1: Foundations (`unfoundation`, `unspatial`, `untags`, `unevents`)

- **The Win:** `unspatial-core` and `untags-core` provide a robust, grid-sovereign foundation. The "Discrete 3D Grid"
  logic is the engine's strongest asset.
- **The Leak:** `unfoundation-core` contains specific UI theme colors and 2D sprite indices. `uninteraction-core`
  contains logic that directly manipulates Bevy Materials, preventing headless simulation.
- **Verdict:** 🟢 **Stable**, but requires extraction of visual constants.

### Batch 2: World & Simulation (`unboard`, `unlight`, `unthermal`, `unfog`, `unsound`)

- **The Win:** Sophisticated field propagation (Thermal, Vector-Sound, Miasma-Pressure). `unboard-core` is a perfect
  engine primitive.
- **The Leak:** **"Antagonist Leakage."** Thermal and Fog systems explicitly query for `GhostSprite`. The physics
  systems "know" the ghost exists.
- **The "God Plugin":** `unlight-plugin` acts as the final assembly point for all game visuals, creating a massive
  maintenance bottleneck when adding new evidence types.
- **Verdict:** 🟡 **High Performance**, but coupled to the "Ghost" concept.

### Batch 3: Actors & Locomotion (`unplayer`, `unghost`, `unnavigation`, `unnpc`)

- **The Win:** `unghost-plugin` features a brilliant "DNA" system using Perlin noise to drive emergent personality. The
  Ghost Interaction System (GIS) is a clean, event-based subsystem.
- **The Leak:** The Player is a "Data Sink." Sanity logic depends on every physical field (Light, Sound, Thermal).
  Pathfinding logic (A\*) is stranded in the Player plugin rather than living in the Navigation domain.
- **Verdict:** 🟡 **Functional**, but couplings make it hard to add new fields or entities.

### Batch 4: Tools & Equipment (`ungear`, `ungearitems`)

- **The Win:** **"Illiterate Tools."** The EMF and Geiger math is phenomenal; tools sample the environment's "noise"
  rather than querying the ghost's position. This is the project's design "North Star."
- **The Leak:** `GearStuff` is a God-Parameter that creates high dependency weight. The Quartz Stone directly modifies
  the Ghost's AI state, violating DDD boundaries.
- **Verdict:** 🟢 **Design Excellence**, but requires better data-driven sensor abstractions.

### Batch 5: Interface & Meta (`unui`, `untruck`, `unmenu`, `unmanual`)

- **The Win:** The Menu Template system (`unmenu-core`) is a highly reusable "UI Engine."
- **The Leak:** Truck UI styling is hard-coded into data structs. "End Mission" orchestration logic is trapped in the
  Truck UI rather than a dedicated Mission Manager.
- **Verdict:** 🟡 **Polished**, but UI is currently doubling as a Game Master.

### Batch 6: Infrastructure (`unassets`, `unmapload`, `untmxmap`, `ungame`)

- **The Win:** Naive TMX parsing allows for near-instant map metadata scanning. Contiguous Z-mapping normalizes Tiled's
  floor complexity.
- **The Leak:** The Map Loader decides which ghost and evidence to spawn. This couples the "Simulation Setup" to the
  "Classic Mode Rules."
- **Verdict:** 🔴 **Rule Coupling.** The loader is too tied to Unhaunter's specific "Identify-the-Ghost" loop.

---

## 3. Key Architectural Debt

### 3.1 The "Antagonist" Leak

The engine's simulation layer (Physics) frequently queries for the `Ghost` entity.

- _Impact:_ You cannot easily create a game mode where the "Haunting" comes from the house itself or multiple entities
  without rewriting the physics solvers.

### 3.2 Visual/Representation Coupling

Sprite IDs and Isometric Math are embedded in `-core` crates.

- _Impact:_ Pivoting to a 3D-mesh engine or a Top-Down view would require touching 50% of the crates in the workspace.

### 3.3 The God-Parameter Bottleneck (`GearStuff`)

A single `SystemParam` carries the weight of the entire world state.

- _Impact:_ High compile times and extreme difficulty in unit-testing specific tool behaviors in isolation.

---

## 4. Engine Readiness: The Refactoring Blueprint

### Phase 1: Mechanism vs. Meaning Split (High Priority)

1. **Componentize Emitters:** Create generic `ThermalEmitter`, `SoundEmitter`, and `FluidEmitter`. Solvers should
   iterate over these, not "Ghosts."
2. **Generic Locomotion:** Movement should operate on a `Velocity` component. External factors (Miasma, Salt) should
   modify `Velocity` rather than being hard-coded in `movement.rs`.
3. **Spawn Point Abstraction:** The Map Loader should spawn `GenericSpawnPoint` entities. A separate `GameModePlugin`
   should listen for map completion and spawn the specific Antagonist.

### Phase 2: Orchestration Refactor (Medium Priority)

1. **Extract `unnavigation-plugin`:** Consolidate A\* pathfinding and collision math into a dedicated crate usable by
   both Players and AI.
2. **Sensory Receiver Pattern:** Give the Player a `SensoryStress` component. Fields apply stress; Sanity logic reads
   stress. This decouples the Player from specific Field implementations.
3. **Mission Controller:** Extract "End Mission" logic from `untruck-plugin` into a standalone `unmission-plugin`.

### Phase 3: Visual Decoupling (Long Term)

1. **Semantic Visual Keys:** Replace `GearSpriteID` and other sprite indices with string keys (e.g.,
   `"item.emf_meter"`).
2. **Theme Resources:** Move color constants into a `Theme` resource.
3. **Projection Mapping:** Move `to_screen_coord` out of `Position` and into a `CameraProjection` utility.

---

## 5. Conclusion

The Unhaunter Engine is a powerhouse of discrete simulation logic. By moving from a "Hard-coded Ghost" model to a
"Generic Emitter/Receiver" model, the codebase will achieve true Engine-Mod separation. The current "Illiterate Tools"
implementation proves that the project has a clear and viable design philosophy that is ready to be scaled into a
platform.
