# Optimization plan for Temperature field

## Plan

### **Phase 1: Valid Tile Precomputation**

**Objective:** Eliminate the need to check `bcf.player_free` and `roomdb` lookups during the runtime loop.

1. **Modify `ThermalGrid` Resource (in `unthermal-core`):**
   - Add a new field `pub valid_tiles: Vec<(usize, usize, usize)>`. This will store the `ndidx` of every tile that
     participates in thermal sim.
   - Add a `pub iterator_index: usize` field to `ThermalGrid` to track the rolling window position across frames.
   - Add a `pub low_priority_queue: Vec<(usize, usize, usize)>` (or similar ring buffer) for the standard rolling
     internal update.
   - (Optional but cleaner) Add `pub high_priority_queue: Vec<(usize, usize, usize)>` for tiles that changed
     significantly last frame.

2. **Update `init_thermal_grid_content` (in `unthermal-plugin`):**
   - After the map is loaded, iterate over the entire `BoardCollisionField`.
   - If a tile is `see_through` OR `player_free`, add its `ndidx` to `valid_tiles`.
   - This happens once per level load.

### **Phase 2: Data Structure Cleanup & Prerequisite Fixes**

**Objective:** Ensure we have the right data accessible efficiently.

1. **Leverage `connectivity_scores`:**
   - Currently, `connectivity_scores` is computed but ignored. We will use it.
   - Verify `utils::precompute_connectivity_scores` logic assigns meaningful values (it currently uses
     `default_score: 16` for normal tiles, `stair_score`, etc.).
   - Update `precompute_connectivity_scores` if necessary to ensure it distinguishes "insulating wall" vs "conducting
     air" so we don't need runtime checks. _Self-correction: The current system calculates conductivity dynamically
     based on neighbors. We might need to stick to the dynamic check for neighbors inside the loop if
     `connectivity_scores` isn't granular enough, but we should at least use it for the self-tile property._

2. **Remove Obsolete Code:**
   - Delete the `energy_changes` used in `temperature_update`.
   - Delete the `old_temps` Vec allocation logic.

### **Phase 3: The New `temperature_update` Loop**

**Objective:** Implement the rolling window, in-place update.

1. **Iteration Logic:**
   - Calculate `budget`: `(valid_tiles.len() as f32 * quality_factor * SPEED_FACTOR).ceil() as usize`.
   - Loop `budget` times.
   - Inside the loop:
     - Get `self_idx = valid_tiles[iterator_index]`. Increment/wrap `iterator_index`.
     - Read `current_temp = thermal_grid.temperature_field[self_idx]`.

2. **Diffusion Math (Gauss-Seidel In-Place):**
   - Reconstruct neighbors on the fly (Left, Right, Top, Bottom, maybe Stairs).
   - For each neighbor:
     - Check if valid (is it in bounds? is it a wall?). _Optimization: `valid_tiles` only guarantees self is valid.
       Neighbors might be walls._
     - Read neighbor temp directly from `temperature_field`.
     - Compute energy exchange (simplified or cubic, keeping cubic for now to minimize behavior change).
   - Compute `new_temp` for self.
   - Write `new_temp` directly to `temperature_field[self_idx]`.

3. **Residual & Activity:**
   - Compute `delta = (new_temp - current_temp).abs()`.
   - Accumulate `thermal_grid.temperature_activity[self_idx] += delta`.
   - (Optional advanced step) If `delta > THRESHOLD`, add neighbors to a "high priority" list for next frame (future
     improvement, start with simple rolling window first).

### **Phase 4: Cleanup & Validation**

**Objective:** Remove unused dependencies and verify partial updates don't break the game.

1. **Deprecate/Remove Unused Fields:**
   - If `temperature_field_prev` becomes truly unused by any other system (e.g., rendering gradients), we can mark it
     for removal or repurpose it.

2. **Tuning:**
   - Adjust `SPEED_FACTOR` so that `quality_factor=0.1` updates the whole grid roughly every 1-2 seconds (approx. 60-120
     frames).
   - `valid_tiles.len()` ~2000. `budget` ~20-50 tiles/frame.

### **Risk Assessment / De-risking**

- **Risk:** Visual tearing or "checkerboard" updates because neighbors update out of sync.
  - _Mitigation:_ The diffusion is naturally smooth. Gauss-Seidel is stable. Visually, the `quality_factor` controls the
    refresh rate. At worst, heat spreads slightly slower.
- **Risk:** `connectivity_scores` might be wrong.
  - _Mitigation:_ Stick to the existing logic for _classifying_ conductivity (checking `bcf` flags) inside the new
    efficient loop first. We can optimize that micro-op later. The big win is skipping the 8000 empty tiles.
- **Risk:** Breaking stair vertical conduction.
  - _Mitigation:_ Ensure the neighbor collection logic explicitly includes the stair offset check, just like the
    original code.

This plan moves from O(GridSize) to O(ValidTiles \* Quality). It is strictly more efficient and scales correctly.
