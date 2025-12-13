# Detailed Refactoring Plan: The "Unhaunter" Engine

**Status:** Draft **Target Audience:** Engineering Team **Goal:** Decouple the codebase to enable a "Ghost Engine"
architecture, improve compile times, and enforce strict domain boundaries.

---

## Context: The Problem

Currently, `unhaunter` suffers from a "Distributed Monolith" architecture.

1. **Global State Dumps:** Crates like `uncore-components` hold unrelated data (UI, Physics, AI), forcing the entire
   game to recompile when one small thing changes.
2. **Circular Dependencies:** High-level logic depends on low-level components, which depend back on high-level logic
   (often solved by dumping everything into `uncore-*`).
3. **Infrastructure Leakage:** Core game rules know about "Sprites" and "Assets," making it impossible to test logic
   without a GPU/Window context.

This plan breaks the refactor into 6 safe, verifiable phases.

---

## Phase 1: The Spatial Foundation (`unspatial`)

**Objective:** Create a zero-dependency "Switzerland" for spatial types. This breaks the dependency cycle at the lowest
level.

### Steps

1. **Create Crate `unspatial`:**
   - This crate must have **ZERO** dependencies on other `un*` crates.
   - It may depend on `bevy_math` (for `Vec3`), but avoid full `bevy`.
2. **Migrate Primitives:**
   - Move `Position` (logical grid coordinates) from `uncore-board` to `unspatial`.
   - Move `Direction` (enum) from `uncore-board` to `unspatial`.
   - Move `BoardPosition` (3D grid coordinates) from `uncore-board` to `unspatial`.
3. **Retarget Dependencies:**
   - Update `uncore-board` to depend on `unspatial`.
   - Update `uncore-components` to depend on `unspatial`.
   - **Crucial:** Remove the `pub use uncore_board::components::position::Position` re-export in `uncore-components`.
     All code must import directly from `unspatial`.

**Success Criteria:**

- `unspatial` compiles in isolation.
- `grep "struct Position" crates/uncore-board` returns nothing (it's moved).

---

## Phase 2: Shattering the Monolith & Managing Cycles

**Objective:** Empty `uncore-components` and `uncore-resources` without causing "Crate A needs B, B needs A" compile
errors.

### The Strategy: "Tags & Targets"

To avoid cycles (e.g., Ghost needs Player position, Player needs Ghost status), we will use a **Shared Interaction
Layer**.

1. **Create Crate `untags` (or repurpose `uncore-types`):**
   - This crate holds lightweight "Marker Components" and "Shared Data Enums".
   - Examples: `PlayerTag`, `GhostTag`, `InteractableTag`.
   - **Rule:** No logic, just data definitions.
2. **Create Crate `uninteraction` (Optional but recommended):**
   - Holds generic components like `Target` (entity wrapper).
   - Allows `unghost` to hunt a `Target` without knowing it's a `Player`.

### Steps

1. **Audit & Move (Iterative):**
   - **Ghost Domain:** Move `GhostSprite`, `GhostBreach` -> `unghost`.
   - **Player Domain:** Move `PlayerSprite`, `Stamina` -> `unplayer`.
   - **Truck Domain:** Move `TruckUI` -> `untruck`.
2. **Fixing The Cycle (The Ghost-Player Example):**
   - _Problem:_ `unghost` AI needs to find the player.
   - _Old Way:_ `Query<&Position, With<PlayerSprite>>` (Requires importing `PlayerSprite`).
   - _New Way:_
     - Define `PlayerTag` in `untags`.
     - `unplayer` adds `PlayerTag` to the player entity.
     - `unghost` imports `untags`.
     - `unghost` queries `Query<&Position, With<PlayerTag>>`.
   - _Result:_ `unghost` does **not** depend on `unplayer`.
3. **Decommission:**
   - Delete `uncore-components` and `uncore-resources` when empty.

**Success Criteria:**

- `unghost` does not depend on `unplayer`.
- `unplayer` does not depend on `unghost`.
- Both depend on `untags` / `unspatial`.

---

## Phase 3: Clean Architecture (The View Synchronizer)

**Objective:** Decouple Game Logic from Rendering. Logic shouldn't know about "Sprites".

### Steps

1. **Abstract Assets:**
   - In `uncore-types`, replace `Handle<Image>` with a logical enum `AssetKey` (e.g., `AssetKey::Character1`,
     `AssetKey::EMFReader`).
2. **Abstract Gear Visuals:**
   - Remove `GearSpriteID` from `ungear`. Use `GearKind` or `GearID`.
3. **Implement "View Synchronizer":**
   - Create a new system (e.g., in `unassets` or `unrender`).
   - **Logic:** `Query<Entity, (Added<GearID>, Without<Handle<Image>>)>`.
   - **Action:** When a logical `GearID` is added to an entity, this system looks up the correct sprite in an
     `AssetLibrary` resource and adds the `SpriteBundle`.
   - _Benefit:_ The logic crate (`ungear`) never imports `bevy::sprite`.

**Success Criteria:**

- Core logic crates can compile without `bevy_sprite` or `bevy_render` features enabled.

---

## Phase 4: Interface Segregation (Refactoring Traits)

**Objective:** Stop forcing invisible items to implement rendering methods.

### Steps

1. **Split `GearUsable`:**
   - **`GearLogic` Trait:** `update()`, `trigger()`, `get_state()`. (Lives in `ungear`).
   - **`GearPresentation` Trait:** `get_display_name()`, `get_icon()`. (Lives in `unui` or `ungear_view`).
2. **Refactor Implementations:**
   - Items implement `GearLogic`.
   - UI systems query for `dyn GearPresentation` (or a component wrapper) only when needed.

---

## Phase 5: The Generic Engine (Registry Pattern)

**Objective:** `unmapload` should load _any_ entity, not just the ones hardcoded in `unhaunter`.

### Steps

1. **Define `MapEntitySpawner` Trait:**
   - Method: `spawn(commands, position, tiled_properties) -> Option<Entity>`.
2. **Create Spawner Registry:**
   - A Bevy Resource: `Resource<Vec<Box<dyn MapEntitySpawner>>>`.
3. **Register Spawners:**
   - `unghost` plugin registers a `GhostSpawner`.
   - `unplayer` plugin registers a `PlayerSpawner`.
4. **Refactor `unmapload`:**
   - Iterate through the registry.
   - Ask each spawner: "Do you handle Tiled Class 'Ghost'?"
   - If yes, execute.
   - _Result:_ `unmapload` no longer imports `unghost` or `unplayer`.

**Success Criteria:**

- Adding a new enemy type requires **zero** changes to `unmapload`.

---

## Phase 6: Verification & Polish

**Objective:** Prove it works.

1. **Dependency Graph Check:** Ensure `unhaunter` (the main app) is the only crate that ties everything together.
2. **Headless Test:** Write a test in `unghost` that simulates a ghost moving towards a target position using
   `unspatial` types, running in a pure Rust environment (no Bevy App needed, or minimal Bevy App).

---
