# unboard-core Debloating: Analysis and De-Risked Plan

## Objective

The goal is to dissolve the current "monolith" status of `unboard-core`. Currently, it acts as a global container for
Game State (Lighting, Temperature, Gameplay Content), which causes circular usage and bloated compile times.

The target state is:

- **`unboard-core`**: Purely Board Topology (Grid, chunks, collision, floors).
- **`unbehavior`** (New): The Schema definition for game content (Classes, Properties).
- **Plugins**: Own their own simulation data (e.g., `unlight` owns the Light Grid).

## Analysis: The "Affinity Domains"

We categorize the current contents of `unboard-core` into these domains:

1. **🟦 Topology (Keep):** Discrete 3D grid, chunks, coordinate space.
2. **🟧 Factory/Schema (Move to `unbehavior`):** What _can_ be placed (Classes, Tiled props).
3. **🟪 Simulation (Move to Plugins):** Data for specific systems (Light, Heat, Miasma).

## De-Risked Execution Plan

This plan prioritizes stability. We will **skip** items that require deep logic rewrites if they threaten the stability
of the build, focusing on the high-value architectural split first.

### Phase 0: The "Linchpin" Types (Immediate & Safe) [DONE]

These moves resolve circular dependencies before they happen.

1. **Move `Orientation` to `unspatial-core`** [DONE]

   - _Why:_ `RoomState` (staying in board) needs it. `Behavior` (moving away) needs it.
   - _Risk:_ Low. Pure enum move.
   - _Action:_ Move `Orientation` enum to `unspatial-core`.

2. **Move `Class` and `TileState` to `unbehavior` (Prep)** [DONE]
   - _Why:_ These are pure data definitions required by almost everything.
   - _Risk:_ Low.

### Phase 1: The Behavior Extraction

We create the shared dictionary between Map Loading and Game Logic.

1. **Create `unbehavior` crate**
2. **Move `Behavior` component and `SpriteConfig`**
   - _Dependency Chain:_ `unbehavior` depends on `unspatial-core` and `unboard-core` (for Position/Topology).
   - _Refactor:_ `unmessage` and `unclass` dependencies need to be checked.
3. **Audit `Interactive` Component**
   - _Decision:_ If moving `Interactive` to `uninteraction` causes a circular dependency with `unbehavior` (because
     interactive objects check state), we will **DEFER** moving `Interactive` and keep it effectively "orphaned" in
     `unboard` or `unbehavior` for now.

### Phase 2: The Resource Migration ("Stop the World" avoidance)

We will not try to split `BoardData` in one go. We will use a **Duplicate -> Migrate -> Delete** strategy per domain.

#### 2.1 Lighting Data (High Value)

- **Step 1:** Create `LightGrid` resource in `unlight-plugin`.
- **Step 2:** Update `unlight` systems to write to `LightGrid` _instead_ of `BoardData.light_field`.
- **Step 3:** Update all readers (`unrender`) to read `LightGrid`.
- **Step 4:** Remove `light_field` from `BoardData`.

#### 2.2 Thermal Data (Medium Value)

- _Action:_ Move `temperature_field` to `unthermal` (or `unclimate`).
- _Note:_ If this is too entangled with generic Board logic, we might **SKIP** this for this pass.

#### 2.3 Miasma Data (Easy Win)

- _Action:_ Move `MiasmaGrid` to `unfog`.

## "Do Not Touch" List (Risks Accepted)

To ensure success, we will explicitly **NOT** move these items in this pass:

1. **`RoomDB`**: Keeps the Board "aware" of rooms. Moving this requires abstracting "Rooms" out of the Board, which is
   too complex for now.
2. **`FloorLevelMapping`**: Essential for the Board to understand the Z-axis.
3. **Legacy Collision Logic**: Unless it moves cleanly with components, we keep it in `unboard-core`.

---

## Success Criteria

- `unboard-core` no longer compiles `serde` or `tiled` logic related to specific game objects (like "Van" or "Ghost").
- `unlight` is self-contained.
- No circular dependencies created.
