# TASK: "Initialization Cascade" and Thermal/Sound Extraction (Revised)

## Architectural Context

We are transitioning Unhaunter to an **Engine vs. Plugin** architecture. Currently, `unboard-core` holds simulation data
(Temperature, Sound) that belongs to specific domains, and `unmapload-plugin` hardcodes the initialization of every
single system.

**The Goal:**

1. Shrink `BoardTopology` so it is read-only during gameplay.
2. Create dedicated `unthermal` and `unsound` crates.
3. Implement the "Initialization Cascade": `unmapload` broadcasts a `MapGeometryInitializedEvent`, and plugins
   independently initialize their own grids.

---

## Phase 1: Event Infrastructure

**Goal:** Define the event that triggers the cascade.

1. **Modify `crates/unevents-core/src/events/loadlevel.rs`:**

   - Add a new event struct:

     ```rust
     #[derive(Event, Clone, Debug)]
     pub struct MapGeometryInitializedEvent {
         pub map_size: (usize, usize, usize),
         pub origin: (i32, i32, i32),
     }
     ```

## Phase 2: Thermal Extraction (`unthermal`)

**Goal:** Move heat simulation out of `unboard-core` and `ungearitems` while managing dependencies correctly.

1. **Create Crates:** Create `crates/unthermal-core` and `crates/unthermal-plugin` with standard `Cargo.toml`.

   - `unthermal-core`: Depends on `unspatial-core`, `unfoundation-core`, `bevy`, `ndarray`. **No dependency on
     `unboard-core`.**
   - `unthermal-plugin`: Depends on `unthermal-core`, `unboard-core`, `unspatial-core`, `unfoundation-core`, `bevy`,
     `ndarray`.

2. **Move Data (`unthermal-core/src/resources.rs`):**

   - Move `temperature_field`, `temperature_field_prev`, `temperature_activity`, `connectivity_scores`,
     `temp_diffusion_config`, and `ambient_temp` from `unboard-core/src/resources/board_topology.rs` into a new
     `ThermalGrid` resource.
   - Move `TemperatureDiffusionConfig` struct to this file.
   - **Do NOT move** logic that depends on collision data (`calculate_connectivity_score`).

3. **Move Logic (`unthermal-plugin/src/systems.rs` and `utils.rs`):**

   - **Step 3a: Port Connectivity Logic**
     - Copy `calculate_connectivity_score` and `precompute_connectivity_scores` logic from `unboard-core` to
       `unthermal-plugin` (create `utils.rs` or keep in systems if small).
     - **Refactor:** Change signatures from `&self` (BoardTopology) to independent functions accepting `pos`,
       `map_size`, and `&Array3<CollisionFieldData>`.
   - **Step 3b: Port Update System**
     - **Cut** `temperature_update` from `crates/ungearitems-plugin/src/components/thermometer.rs`.
     - **Paste** into `unthermal-plugin/src/systems.rs`. Update to use `ResMut<ThermalGrid>` and call the new
       connectivity functions/use cached scores.

4. **Split Initialization:**
   - **`init_thermal_grid_allocation` (System):** Listen to `MapGeometryInitializedEvent`.
     - Resize `ThermalGrid` arrays to match `event.map_size`.
     - Fill `temperature_field` with `ambient_temp`.
   - **`init_thermal_grid_content` (System):** Listen to `LevelReadyEvent` (runs _after_ `load_level_handler` finishes).
     - **Cut** randomization/smoothing logic from `unmapload-plugin/src/level_finalization.rs`.
     - **Paste** here. This ensures `RoomDB` and `HauntState` are populated for breach room detection.

## Phase 3: Sound Extraction (`unsound`)

**Goal:** Move audio propagation out of `unboard-core` and `ungearitems`.

1. **Create Crates:** Create `crates/unsound-core` and `crates/unsound-plugin`.
2. **Move Data (`unsound-core/src/resources.rs`):**
   - Move `sound_field: HashMap<BoardPosition, Vec<Vec2>>` from `BoardTopology` into a new `SoundGrid` resource.
3. **Move Logic (`unsound-plugin/src/systems.rs`):**
   - **Cut** the `sound_update` system from `crates/ungearitems-plugin/src/components/recorder.rs`.
   - **Paste** into `unsound-plugin/src/systems.rs`. Update to use `ResMut<SoundGrid>`.
   - Create `init_sound_grid` system listening to `MapGeometryInitializedEvent` to clear the `SoundGrid` HashMap.

## Phase 4: `unlight` Initialization

**Goal:** Make Light self-initializing.

1. **Modify `crates/unlight-plugin/src/systems.rs`:**
   - Create an `init_light_grid` system that listens for `MapGeometryInitializedEvent`.
   - Inside, resize `LightGrid.light_field` to match `event.map_size` and reset exposure values.

## Phase 4.5: `unfog` (Miasma) Initialization

**Goal:** Make Miasma self-initializing (previously overlooked).

1. **Modify `crates/unfog-plugin/src/systems.rs`:**
   - Create an `init_miasma_grid` system listening to `MapGeometryInitializedEvent`.
   - Resize `MiasmaGrid.pressure_field` and `velocity_field` to match `event.map_size`.

## Phase 5: The Map Loader Refactor

**Goal:** Convert the God Function to the Event Broadcaster.

1. **Modify `crates/unmapload-plugin/src/level_setup.rs`:**
   - In `load_level_handler`: Remove initialization lines for `temperature_field`, `light_field`, `miasma`, and
     `sound_field`.
   - Remove `LightGrid`, `MiasmaGrid` from `LoadLevelSystemParam`.
   - After initializing `BoardTopology`, emit the `MapGeometryInitializedEvent`.
   - Ensure `LevelReadyEvent` is still emitted at the very end to trigger content phases.

## Phase 6: Link Consumers

**Goal:** Update systems that previously read temp/sound from `BoardTopology`.

1. **Modify `crates/unplayer-plugin/src/systems/sanityhealth.rs`:**
   - In `lose_sanity`: Replace `bf.temperature_field` with `thermal_grid.temperature_field` and `bf.sound_field` with
     `sound_grid.sound_field`. Add `Res<ThermalGrid>` and `Res<SoundGrid>`.
2. **Modify `crates/ungearitems-plugin/src/components/thermometer.rs`:**
   - In `update_thermometer`: Read from `Res<ThermalGrid>`.
3. **Modify `crates/ungearitems-plugin/src/components/recorder.rs`:**
   - In `update_recorder`: Read from `Res<SoundGrid>`.
4. **Modify `crates/ungearitems-plugin/src/components/spiritbox.rs`:**
   - In `update_spiritbox`: Read from `Res<ThermalGrid>` and `Res<SoundGrid>`.

## Acceptance Criteria

- `BoardTopology` no longer contains temperature or sound data.
- `load_level_handler` no longer hardcodes grid initialization for light/temp/sound/miasma.
- `unthermal-core` does **not** depend on `unboard-core`.
- Miasma grid is correctly resized on map load.
- The game compiles and functions identically (No-Op Refactor).
