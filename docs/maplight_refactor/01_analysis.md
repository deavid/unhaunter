# Maplight Refactor Analysis: `apply_lighting`

## 1. Current State Assessment

The `apply_lighting` function in [crates/unlight-plugin/src/maplight.rs](../../crates/unlight-plugin/src/maplight.rs) is
the primary "Perceptual Engine" of Unhaunter. It is a 1300+ line Bevy system that handles:

1. **Light Gathering**: Collecting emitters from gear, players, and world.
2. **Shadow Casting**: Computing visibility fields for every light source.
3. **Perceptual Adaptation**: Simulating eye exposure via a 240-frame Hann window.
4. **Stochastic Scheduling**: Deciding which 15% of the world to update this frame.
5. **Entity-Specific Logic**: Handing different rendering math for Tiles, Ghosts, Breaches, and Miasma.
6. **Tonemapping & Spectral Math**: The core "look" including UV/IR interactions.

### The "Tangle"

The main issue is **Variable Capturing**. The function defines deep closures (`fpos_gamma_color`,
`fpos_sampling_corner`) that capture 20+ local variables. This makes it impossible to extract these closures into
functions without creating massive "Context" structs, creating a circular dependency of complexity.

---

## 2. Refactoring Opportunities

### A. The "Sampler" Context Pattern

Instead of closures, we should define a `LightingContext` (or `Perceiver`) struct.

- **Proposal**: Create a `LightingContext<'a>` that holds references to all necessary resources (`LightGrid`,
  `VisibilityData`, the pre-computed `Flashlight` list).
- **Benefit**: Methods like `context.sample_at(pos)` can then be tested in isolation with mock data.

### B. Separation of Concerns: The "Pipeline" Model

The logic currently flows in a single linear path. We can break this into a pipeline of specialized systems:

1. **EmitterSystem**: (Pre-process) Aggregates all `LightEmitter` and `Toggleable` states into a compact `ActiveLights`
   resource.
2. **ExposureSystem**: (Pre-process) Updates the `ExposureHistory` resource. This logic is purely mathematical and
   doesn't need to know about individual entity materials.
3. **StochasticSelectionSystem**: (Pre-process) Populates a `PriorityUpdateQueue` with entities that _need_ to be
   updated (moving ghosts, hover states, nearby items).
4. **LightingApplicationSystem**: (Main) The only system that actually touches `MeshMaterial2d`. It consumes the outputs
   of the previous three systems.

### C. Component-Driven Visual Strategy

Currently, `apply_lighting` has a giant block of code for `if let Some(ethereal) = o_ethereal`. This is repeated for
`ecto_vis`, `miasma`, etc.

- **Proposal**: Move this logic into **Processor Traits**.
  ```rust
  trait VisualProcessor {
      fn apply(&self, ctx: &LightingContext, material: &mut CustomMaterial1);
  }
  ```
  Each component (`Ethereal`, `SpectralInfluence`) would implement logic to modify the material parameters based on the
  lighting context.

### D. Data-Driven Spectral Curves

The "magic numbers" for UV power, Red decay, and IR thresholds are hardcoded in the middle of loops.

- **Proposal**: Move these into a `SpectralSettings` asset.
- **Benefit**: Allows for faster tuning (or even runtime difficulty adjustment) without navigating the "God Function."

---

## 3. Advanced/Diverse Ideas

### E. The "Material Buffer" Approach (GPU Scaling)

If the number of entities grows, the "cloning and updating materials" pattern becomes a bottleneck.

- **Proposal**: Instead of `MeshMaterial2d`, could we use a single `StorageBuffer` or `Texture` that represents the
  "Light Condition Overlay"?
- **Refactor Path**: Even without switching to a full GPU buffer, refactoring the system to treat material updates as
  "writes to a buffer" prepares the codebase for future high-performance rendering.

### F. Functional Pureness for Tonemapping

The tonemapping logic (`tonemap(r)`, `calc_gamma`, `calc_rgba`) is currently mixed with coordinate sampling.

- **Proposal**: Extract the tonemapping into a separate module: `unlight-core::tonemapping`.
- **Benefit**: This allows us to unit-test the "visual feel" (e.g., "In a pitch black room with 0.1 lux, does the color
  result in the correct RGB value?") without initializing a Bevy App.

### G. Throughput-Oriented System Parallelism

We avoid internal parallelization of loops (e.g., `par_iter`) within any single system, as it introduces significant
complexity and potential for race conditions or material-asset contention.

- **Proposal**: Break the logic into several independent systems. Bevy's scheduler will naturally run these in parallel
  on different threads if they do not have access conflicts (i.e., they don't both require
  `ResMut<Assets<CustomMaterial1>>` at the same time).
- **Latency Acceptance**: Most lighting effects (exposure, spectral charging, tile colors) can tolerate a 3–5 frame
  delay without breaking immersion. We prioritize aggregate throughput (FPS) over frame-perfect latency. If a system
  calculates light data in frame $N$ and the material update system doesn't see it until frame $N+1$, that is perfectly
  acceptable if it enables smoother overall performance.

### H. Componentized Stochastic Scheduler

The logic that decides which tiles to skip (`min_threshold * dist / 9 > ...`) is buried in the middle of the render
loop.

- **Proposal**: Introduce a `UpdatePriority` component.
- **Refactor Path**: A separate system calculates priorities based on player distance and "interest" (hover, spectral
  activity). The lighting system then simply queries for `Query<..., With<HighPriority>>`.
- **Benefit**: Decouples the "Rendering Logic" from the "Performance Optimization Logic."

---

## 4. Proposed Logical Map

| Current Section                | Proposed Home                  | Type                            |
| :----------------------------- | :----------------------------- | :------------------------------ |
| Flashlight collection          | `unlight-plugin::emitters`     | System                          |
| Hann-filter Exposure           | `unlight-plugin::perceptual`   | System                          |
| Coordinate rotation/projection | `unspatial-core::projection`   | Pure Functions                  |
| Spectral charge/decay          | `unlight-core::spectral`       | System                          |
| **Material Update Loop**       | **`unlight-plugin::maplight`** | **The (new, thin) Main System** |

---

## 5. Risk Assessment & Critical considerations

### A. The "Query Factor" (ECS Performance)

Currently, `apply_lighting` iterates once over the relevant entities.

- **Risk**: Splitting into multiple systems (e.g., `update_ghost_visuals`, `update_tile_visuals`) might require
  iterating the same archetype tables multiple times.
- **Mitigation**: Bevy is fast at iteration, but we must ensure we don't have $N$ systems checking the same $M$ entities
  just to decide they do nothing. The "Stochastic Scheduler" (3.C) becomes critical here to filter the list _before_
  heavy processing.

### B. Throughput vs. Lag (The "Cohesive Result" Strategy)

The primary goal is maintaining a high framerate (Throughput).

- **The 5-Frame Window**: We explicitly accept up to a 5-frame delay for lighting updates. It is better to have
  "slightly delayed" but smooth lighting than "frame-perfect" but stuttering performance.
- **Non-Blocking Updates**: This allows us to run expensive computations (like shadow-casting or spectral decay) in
  systems that don't need to block the main material rendering loop. Bevy can schedule these across available CPU cores,
  and we accept the results whenever they arrive.
- **Requirement**: Inner loops (within a system) must **never** be parallelized. Parallelism is strictly a system-level
  orchestration handled by the Bevy scheduler.

### C. The `dyn Trait` Trap in ECS

The "VisualProcessor" trait (2.C) sounds nice but is hard to implement efficiently in Bevy. You cannot easily query
`Query<&mut dyn VisualProcessor>`.

- **Correction**: Instead of runtime polymorphism, use **Static Dispatch via SystemParams**. Create a helper struct:
  ```rust
  #[derive(SystemParam)]
  struct SpectralEntity<'w, 's> {
      influence: Query<'w, 's, &'static mut SpectralInfluence>,
      // ...
  }
  ```
  And passing this to a specific handler function is better than trying to make a generic trait.

### D. Visibility Hysteresis

The function uses a `Local<HashSet<Entity>>` to detect when an entity _stops_ being visible (to reset its state).

- **Risk**: Splitting systems makes this local state tracking harder. If System A decides to hide an entity, does System
  B know?
- **Mitigation**: The `Visibility` component is the source of truth. The `Local` tracker only exists to optimize the
  `insert/remove` calls. This logic should probably remain in the final "Application" system.

---

## 6. Refined Implementation Strategy

### Phase 1: Pure Logic Extraction (Low Risk)

Extract mathematical models into `unlight-core`.

1. Move `tonemap`, `f_gamma`, `calc_rgba` into a `tonemapping.rs` module.
2. Move the Hann Window / IIR filter logic into a `ExposureModel` struct with standard methods (`.add_sample()`,
   `.update(dt)`).
3. **Outcome**: `apply_lighting` shrinks by ~300 lines; no ECS changes.

### Phase 2: Context Struct & Closure Elimination (Medium Risk)

Replace the 20-variable closures with a `LightingSampler` struct.

1. Create `struct LightingSampler<'a> { lg: &'a LightGrid, flashlights: ... }`.
2. Implement `.sample_light_at(pos)` on this struct.
3. Pass this struct into the rendering loop. **Outcome**: Easier verification of lighting math; closures disappear.

### Phase 3: System Migration (High Risk)

Break the monolithic system.

1. **Pull out `update_exposure_system`**: Run this `before(apply_lighting)`.
2. **Pull out `gather_flashlights_system`**: Run this `before(apply_lighting)`. Store result in a temporary resource (or
   use system piping).
3. **Refactor `apply_lighting`**: Now it only focuses on iterating entities and applying the math from Phase 1 using the
   Context from Phase 2.

---

## 7. The Long Game: GPU Compute

We must acknowledge that `apply_lighting` is essentially a **Software Pixel Shader** running on the CPU for a grid of
tiles.

- **Observation**: The `LightGrid` and `VisibilityField` are 3D Arrays. This is ideal for a Compute Shader or Fragment
  Shader.
- **Architecture Goal**: The "Context" struct from Phase 2 should effectively mirror what a future Uniform Buffer would
  look like. By cleaning up the CPU logic now, we define the "Interface" for a future GPU implementation.

---

## 8. Current Implementation Status (January 2026)

### Summary of Progress

The refactor is in the **early stages**. While some structural foundations have been laid, the "Tangle" in
`apply_lighting` remains largely intact. The function has actually grown to **~1500 lines**.

### Phase 1: Pure Logic Extraction

- **Done**: `compute_color_exposure` has been moved to
  [crates/unrender-std/src/utils/light.rs](../../crates/unrender-std/src/utils/light.rs).
- **Done**: Hann Window weight initialization has been moved to `init_light_grid` in
  [crates/unlight-plugin/src/lighting_sim/systems.rs](../../crates/unlight-plugin/src/lighting_sim/systems.rs).
- **Done**: Artistic tonemapping (`tonemap`), gamma calculations, and the per-frame exposure update logic have been
  moved to `unlight-core::tonemapping` and `unlight-core::exposure`.
- **Done**: `unlight-core::tonemapping` and `ExposureModel` struct are implemented and used.

### Phase 2: Context Struct & Closure Elimination

- **Done**: `fpos_gamma_color`, `fpos_sampling_corner`, `f_vis`, `calc_gamma`, and `calc_rgba` have been refactored into
  a `LightingSampler` struct.
- **Done**: Capture of local variables in `apply_lighting` has been significantly reduced.
- **Outcome**: `apply_lighting` is more readable and logic is more encapsulated.

### Phase 3: System Migration

- **In Progress**: Flashlight collection and exposure updates have been moved to independent systems
  (`gather_flashlights_system` and `update_exposure_system`).
- **Outcome**: `apply_lighting` now only handles scene reconstruction (material updates). It communicates with other
  lighting systems via the `ActiveFlashlights` resource.
- **Next Steps**: Further decompose `apply_lighting` to separate tile updates from sprite updates to improve performance
  and readability.
- **Note**: A significant new module
  [crates/unlight-plugin/src/lighting_sim/](../../crates/unlight-plugin/src/lighting_sim/) has been added. It handles
  prebaked lighting propagation and populates the `LightGrid`, which is a prerequisite for the refactor but doesn't yet
  break up the main system.

### Structural Observations

- **`unlight-core`**: Now contains the `LightGrid` and `LightData` definitions, establishing a "canonical path" for
  lighting data, though some duplication/re-exports still exist in `unlight-plugin`.
- **`unspatial-core`**: `Position` now includes `unrotate_by_dir`, partially addressing coordinate projection needs, but
  no dedicated `projection` module exists.
