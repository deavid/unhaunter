# unboard-core Debloating: Phase 2 Analysis

## Current Status

We have successfully extracted the "Vocabulary" (`Class`, `TileState`, `Behavior` struct) into `unbehavior` and
`unspatial-core`. However, `unboard-core` remains the "God Crate" because it still holds:

1. **The Factory Logic:** The code that decides _what_ a "Door" is (components, sounds, etc.).
2. **The Simulation State:** `BoardData` still holds the state for Light, Heat, and Miasma.
3. **Tiled Dependencies:** Wrappers around `tiled` crate types are still present.

## Objective

The goal of this phase is to strictly enforce `unboard-core` as a **Spatial Topology Container**. It should only know
about the grid, chunks, and collision. It should _not_ know how to load a map from Tiled, nor should it store simulation
data for other systems.

## Analysis of Remaining Debt

### 1. The Map Factory (`behavior/mod.rs`)

The function `apply_components_to_entity` acts as the translator between high-level `Class` definitions and low-level
Bevy components (including assets).

- **Problem:** It hardcodes asset paths (strings) and couples the Board to the Map Loader logic. It also forces a
  dependency on `tiled` types.
- **Solution:** Move this logic to `unmapload-plugin` (or `untmxmap-plugin`).
  - The Board should define components like `Door`, `Switch`, `Wall`.
  - The Map Loader should _insert_ these components when it sees definitions it recognizes.

### 2. Tiled Wrappers (`types/tiledmap/*`)

`unboard-core` currently defines `MapLayer` and `MapTile` which are wrappers around `tiled` crate types.

- **Problem:** This implies `unboard-core` knows about the file format used to author levels.
- **Solution:** Move these strictly to the crate responsible for loading Tiled maps (`untmxmap-plugin` or
  `unmapload-plugin`).

### 3. The "God Resource" `BoardData` (`resources/board_data.rs`)

`BoardData` is a catch-all struct for all grid-based data.

- **Problem:** It creates circular dependencies. `unlight-plugin` needs `BoardData` to read the light grid, but
  `BoardData` defines the light grid, so `unboard-core` arguably shouldn't "own" it if `unlight-plugin` is the domain
  expert.
- **Solution:** Split `BoardData` into focused resources:
  - `BoardTopology` (remains in `unboard-core`): `collision_field`, `map_size`, `origin`, `floor_mapping`.
  - `LightGrid` (move to `unlight-plugin` or `unlight`): `light_field`, `prebaked_*`.
  - `ThermalGrid` (move to `unthermal`/`unclimate`): `temperature_field`.
  - `MiasmaGrid` (move to `unfog`): `miasma`.

### 4. Logic & Type Leaks

- **Render Logic:** `types/quadcc.rs` generates a `Mesh`. This is `unrender` logic.
- **Simulation Types:** `types/light.rs`, `types/miasma.rs`, `types/prebaked_lighting_data.rs` are domain-specific data
  structures living in the generic container.

## Execution Plan: The "Great Separation"

We will execute this in a specific order to maintain build stability.

### Step 1: Evict the Factory (The "Head" Cut)

We will move the logic that _creates_ entities out of `unboard-core`.

1. **Move `apply_components_to_entity`** to `untmxmap-plugin` (or `unmapload-plugin`).
   - This will likely require exposing some components from `unboard-core` as `pub` so the loader can attach them.
2. **Move `types/tiledmap`** to `untmxmap-plugin`.
3. **Action:** Remove `tiled` dependency from `unboard-core`.

### Step 2: Evict The Data (The "Body" Split)

We will decentralize the data storage.

1. **Light Data:**

   - Move `types/light.rs` and `types/prebaked_lighting_data.rs` to `unlight-plugin`.
   - Create `LightGrid` resource in `unlight-plugin`.
   - Refactor `unlight-plugin` to use `LightGrid`.
   - Refactor `unrender` to get light data from `LightGrid`.
   - Remove light fields from `BoardData`.

2. **Miasma Data:**

   - Move `types/miasma.rs` to `unfog-plugin`.
   - Create `MiasmaGrid` resource in `unfog-plugin`.
   - Refactor uses.

3. **Render Helpers:**
   - Move `types/quadcc.rs` to `unrender-std` (or similar shared render crate).

### Step 3: Cleanup

1. Rename `BoardData` to `BoardTopology` (optional, but marks the transition).
2. Verify `unboard-core` dependency tree is clean of `tiled` and domain-specific plugins.

## Risks & Mitigation

- **Circular Dependencies:** When moving `apply_components_to_entity`, we must ensure `unmapload` can see all the
  components it needs to attach (`Interactive`, `Collision`, etc.). These components should likely stay in
  `unboard-core` (or `uninteraction`, `unphysics` etc), which is fine. The _dependency direction_ is
  `Loader -> Components`, which is correct.
- **Performance:** Splitting one big struct into multiple Resources might have minor cache implications, but Bevy's ECS
  handles separate Resources well. Parallel access might actually improve.

## Outcome

`unboard-core` becomes a dumb container for "Where things are" (Topology) and "What stops things" (Collision), without
knowing "What things mean" (Game Logic/Loading).
