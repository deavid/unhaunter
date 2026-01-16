# TASK: "Initialization Cascade" and Thermal/Sound Extraction

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

**Goal:** Move heat simulation out of `unboard-core` and `ungearitems`.

1. **Create Crates:** Create `crates/unthermal-core` and `crates/unthermal-plugin` with standard `Cargo.toml` files
   (depend on `unspatial-core`, `unfoundation-core`, `bevy`, `ndarray`).
2. **Move Data (`unthermal-core/src/resources.rs`):**
   - Move `temperature_field`, `temperature_field_prev`, `temperature_activity`, `connectivity_scores`,
     `temp_diffusion_config`, and `ambient_temp` from `unboard-core/src/resources/board_topology.rs` into a new
     `ThermalGrid` resource.
   - Move `TemperatureDiffusionConfig` struct and `impl BoardTopology` connectivity score methods to this file.
3. **Move Logic (`unthermal-plugin/src/systems.rs`):**
   - **Cut** the `temperature_update` system from `crates/ungearitems-plugin/src/components/thermometer.rs`.
   - **Paste** it into `unthermal-plugin/src/systems.rs`. Update it to use `ResMut<ThermalGrid>` instead of
     `BoardTopology`.
   - **Cut** the `after_level_ready` logic (specifically the temperature randomization and smoothing part) from
     `unmapload-plugin/src/level_finalization.rs` and make it a system `init_thermal_grid` that runs on
     `MapGeometryInitializedEvent`.

## Phase 3: Sound Extraction (`unsound`)

**Goal:** Move audio propagation out of `unboard-core` and `ungearitems`.

1. **Create Crates:** Create `crates/unsound-core` and `crates/unsound-plugin`.
2. **Move Data (`unsound-core/src/resources.rs`):**
   - Move `sound_field: HashMap<BoardPosition, Vec<Vec2>>` from `BoardTopology` into a new `SoundGrid` resource.
3. **Move Logic (`unsound-plugin/src/systems.rs`):**
   - **Cut** the `sound_update` system from `crates/ungearitems-plugin/src/components/recorder.rs`.
   - **Paste** into `unsound-plugin/src/systems.rs`. Update it to use `ResMut<SoundGrid>`.
   - Create an `init_sound_grid` system that listens to `MapGeometryInitializedEvent` and clears the HashMap.

## Phase 4: `unlight` Initialization

**Goal:** Make Light self-initializing.

1. **Modify `crates/unlight-plugin/src/systems.rs`:**
   - Create an `init_light_grid` system that listens for `MapGeometryInitializedEvent`.
   - Inside, resize `LightGrid.light_field` to match `event.map_size` and reset exposure values.

## Phase 5: The Map Loader Refactor

**Goal:** Convert the God Function to the Event Broadcaster.

1. **Modify `crates/unmapload-plugin/src/level_setup.rs`:**
   - In `load_level_handler`: Remove the lines initializing `temperature_field`, `light_field`, `miasma`, and
     `sound_field`.
   - Remove `LightGrid`, `MiasmaGrid` from `LoadLevelSystemParam`.
   - After initializing `BoardTopology`, emit the `MapGeometryInitializedEvent`.

## Phase 6: Link Consumers

**Goal:** Update systems that previously read temp/sound from `BoardTopology`.

1. **Modify `crates/unplayer-plugin/src/systems/sanityhealth.rs`:**
   - In `lose_sanity`: Replace `bf.temperature_field` with `thermal_grid.temperature_field` and `bf.sound_field` with
     `sound_grid.sound_field`. Add `Res<ThermalGrid>` and `Res<SoundGrid>` to params.
2. **Modify `crates/ungearitems-plugin/src/components/thermometer.rs`:**
   - In `update_thermometer`: Read from `Res<ThermalGrid>`.
3. **Modify `crates/ungearitems-plugin/src/components/recorder.rs`:**
   - In `update_recorder`: Read from `Res<SoundGrid>`.
4. **Modify `crates/ungearitems-plugin/src/components/spiritbox.rs`:**
   - In `update_spiritbox`: Read from `Res<ThermalGrid>` and `Res<SoundGrid>`.

## Acceptance Criteria

- `BoardTopology` no longer contains temperature or sound data.
- `load_level_handler` no longer hardcodes grid initialization for light/temp/sound/miasma.
- The game compiles and functions identically to before (No-Op Refactor).
- `unboard-core` dependencies in `Cargo.toml` are reduced.
