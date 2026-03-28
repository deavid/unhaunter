# Multi-Mission Dedicated Server: Feasibility Analysis

NOTE: Status as of 2026-02-22 - this is deemed as not wanted.

## 1. Executive Summary

This document analyzes what changes would be required for Unhaunter's dedicated server to simulate **multiple missions
in parallel** — i.e., a single Bevy `App` hosting N independent investigation rooms concurrently, each with its own
ghost, players, map, and game state.

**Conclusion:** No hard architectural blockers exist. The codebase is clean, has almost zero global statics, and many
subsystems already use per-entity components. However, the migration is a substantial refactoring effort because the
current design assumes **one mission at a time**: approximately 30 Bevy `Resource` types hold per-mission state that
would need to become per-mission (via a `MissionInstance` entity pattern or similar), and dozens of systems perform
unscoped entity queries that would cross-contaminate missions without filtering.

The most non-obvious blockers are:

1. **Entity handles stored inside Resources** — `LightGrid::prebaked_metadata` and `TruckGear` store raw `Entity` values
   that belong to a single mission; a stale handle from Mission A could be misinterpreted in Mission B.
2. **`Local<T>` system state leakage** — ~15 systems use `Local<T>` for timers, accumulators, or cached values that are
   per-system-instance, not per-mission.
3. **Nuclear cleanup model** — the current `despawn_recursive` cleanup in `headless_summary_reset_system` tears down
   _everything_, which would destroy all missions simultaneously.

---

## 2. Architecture Model

### 2.1 Current Dedicated Server

The dedicated server (`unhaunter_dedicated` / `--dedicated` flag) runs a Bevy app with `MinimalPlugins` and
`SingleThreaded` task executors. It uses `ScheduleRunnerPlugin` at 60 Hz. All game plugins are loaded in headless mode
(rendering, audio, and UI disabled). A TCP host accepts client connections and builds `SnapshotMsg` each tick to
replicate state.

Key entry point: `unhaunter/src/app.rs` lines 50–100.

### 2.2 Proposed Multi-Mission Model

Each concurrent mission is represented by a **`MissionInstance` marker entity** inserted at mission start. All
per-mission resources become components on that entity (or children of it). Systems filter queries by a mission marker
component that is propagated to every entity belonging to that mission (players, ghosts, gear, map tiles). Network
clients are bound to a `MissionId` so snapshots and inputs are routed to the correct mission context.

Spatial isolation is achieved via **component markers** (e.g., a `MissionId(u32)` component on every mission-scoped
entity), not coordinate offsets. Systems query entities filtered by their `MissionId`, and per-mission data is looked up
from the corresponding `MissionInstance` entity.

---

## 3. Resource Classification

Every `#[derive(Resource)]` type in the game crates was cataloged and classified into one of three categories:

| Category             | Count | Description                             |
| -------------------- | ----- | --------------------------------------- |
| **Per-Mission**      | ~30   | Must be duplicated per mission instance |
| **Shareable**        | ~12   | Safe to share across all missions       |
| **Server-Singleton** | ~5    | Inherently global (CLI, network conn)   |

### 3.1 Per-Mission Resources (Must Become Per-Mission)

These resources hold state specific to a single investigation and would need to exist once per concurrent mission. The
recommended approach is to store them as components on a `MissionInstance` entity, accessed via a query rather than
`Res<T>`.

| Resource                  | Crate               | Notes                                                            |
| ------------------------- | ------------------- | ---------------------------------------------------------------- |
| `BoardTopology`           | `unboard-core`      | Map geometry, floor mappings, ambient temp                       |
| `BoardEntityField`        | `unboard-core`      | `Array3<Vec<Entity>>` — **stores entity handles**                |
| `BoardCollisionField`     | `unboard-core`      | `Array3<CollisionFieldData>`                                     |
| `ThermalGrid`             | `unthermal-core`    | `Array3<f32>` temperature & activity fields                      |
| `LightGrid`               | `unlight-core`      | Light fields + `PrebakedMetadata` with **entity handles**        |
| `MiasmaGrid`              | `unfog-core`        | Pressure/velocity fluid simulation fields                        |
| `RoomDB`                  | `unbehavior`        | Room tile mappings and room states                               |
| `SoundGrid`               | `unsound-core`      | `HashMap<BoardPosition, Vec<Vec2>>`                              |
| `HauntState`              | `unghost-core`      | Evidence list, ghost dynamics, warning state                     |
| `GhostGuess`              | `unghost-core`      | Player's current ghost identification guess                      |
| `CurrentEvidenceReadings` | `unghost-core`      | Per-evidence clarity readings                                    |
| `PotentialIDTimer`        | `unghost-core`      | Ghost identification detection state                             |
| `CurrentDifficulty`       | `undifficulty-core` | Selected difficulty for this mission                             |
| `SummaryData`             | `unsummary-core`    | Scores, timings, grade, money                                    |
| `ActiveMissionEvaluator`  | `unsummary-core`    | `Box<dyn MissionEvaluator>`                                      |
| `TruckGear`               | `untruck-core`      | `Vec<Entity>` — **stores entity handles**                        |
| `RepellentCraftTracker`   | `untruck-core`      | Repellent crafting state                                         |
| `WalkiePlay`              | `unwalkie-core`     | NPC walkie-talkie dialog state & event history                   |
| `LevelLoadingStatus`      | `unmapload-plugin`  | Loading lifecycle enum                                           |
| `CurrentMapSeed`          | `unnet-core`        | `u64` seed for map generation                                    |
| `LobbyData`               | `unnet-core`        | Player list, selected map/difficulty — becomes per-lobby         |
| `RoomOwner`               | `unnet-core`        | `NetworkId` of room owner                                        |
| `MissionEndRequested`     | `unnet-core`        | `bool` flag                                                      |
| `ChangedTiles`            | `unnet-core`        | `Vec<MapTileState>` for delta sync                               |
| `LocalPlayer`             | `unnet-core`        | `Option<NetworkId>` — per-client concept                         |
| `GearSpawnerRegistry`     | `ungear-core`       | Builder closures per `GearKind` — could be shared if stateless   |
| `ObjectInteractionConfig` | `unghost-core`      | Game balance parameters — shareable if identical across missions |

### 3.2 Shareable Resources (Safe Across All Missions)

These are read-only asset caches, lookup tables, or configuration that does not change per mission.

| Resource                      | Crate               | Notes                                             |
| ----------------------------- | ------------------- | ------------------------------------------------- |
| `SpriteDB`                    | `unrender-std`      | Sprite metadata cache, populated once             |
| `MapTileSetDb`                | `untiled-core`      | Atlas data (uses `AtlasData::Headless` on server) |
| `PerlinNoise`                 | `unnoise-core`      | Read-only permutation lookup table                |
| `PlayerAssets`                | `unplayer-core`     | `AssetCollection` — handles only                  |
| `GhostAssets`                 | `unghost-core`      | `AssetCollection`                                 |
| `MapAssets`                   | `unmapload-core`    | `AssetCollection`                                 |
| `MissionAssets`               | `unmapload-core`    | `AssetCollection` — audio handles                 |
| `TruckAssets`                 | `untruck-core`      | `AssetCollection`                                 |
| `CustomSpritePickingSettings` | `unpicking-core`    | Global picking config (server may skip)           |
| `CliOptions`                  | `uncommon-app-core` | Command-line arguments, immutable at runtime      |
| `GearSpawnerRegistry`         | `ungear-core`       | If all builders are stateless, can be shared      |

### 3.3 Server-Singleton Resources

| Resource         | Crate               | Notes                                                           |
| ---------------- | ------------------- | --------------------------------------------------------------- |
| `NetworkConn`    | `unnet-plugin`      | TCP listeners + client connections; needs multi-mission routing |
| `PlayerRegistry` | `unnet-plugin`      | UUID → NetworkId mapping; needs mission association             |
| `CliOptions`     | `uncommon-app-core` | Read-only CLI config                                            |

---

## 4. Non-Obvious Findings

### 4.1 Entity Handles Inside Resources

Three resources store raw `Entity` values that are only valid for the mission that created them:

1. **`LightGrid::prebaked_metadata`** (`unlight-core`): `PrebakedMetadata` contains
   `light_sources: Vec<(Entity, (usize,usize,usize))>`, `doors: Vec<Entity>`, `breakers: Vec<Entity>`, and
   `light_source_ids: HashMap<Entity, u32>`. These entities are spawned during map loading for a specific mission. If
   two missions share the same `LightGrid` resource (which they must not), a door entity from Mission A could be looked
   up in Mission B's context, causing silent logic errors or panics.

2. **`BoardEntityField`** (`unboard-core`): `Array3<Vec<Entity>>` mapping board positions to the entities occupying
   them. Same concern as above.

3. **`TruckGear`** (`untruck-core`): `Vec<Entity>` listing the gear entities in the truck. Mission-scoped.

**Mitigation:** These must be per-mission. Making them components on the `MissionInstance` entity is sufficient. No
architectural change to the types themselves is needed, only to how they are stored and accessed.

### 4.2 `Local<T>` System State Leakage

Bevy's `Local<T>` is per-system-instance, not per-entity or per-mission. Approximately 15 systems use `Local<T>` for
state that is logically per-mission:

| System                  | File                             | `Local<T>` Usage                                                               |
| ----------------------- | -------------------------------- | ------------------------------------------------------------------------------ |
| `ghost_enrage`          | `unghost-plugin/.../enrage.rs`   | `Local<PrintingTimer>`, `Local<MeanValue>`, `Local<f32>` (last roar timestamp) |
| `ghost_movement`        | `unghost-plugin/.../movement.rs` | `Local<PrintingTimer>`                                                         |
| Various thermal systems | `unthermal-plugin`               | `Local<usize>` iterator indices                                                |
| Fog systems             | `unfog-plugin`                   | `Local<T>` accumulators                                                        |
| Lighting systems        | `unlight-plugin`                 | `Local<T>` iteration state                                                     |

In a single-mission model this works fine. In multi-mission, a `Local<f32>` for "last roar time" would be shared across
ALL ghosts from ALL concurrent missions, causing incorrect cooldown behavior.

**Mitigation options:**

- Move `Local<T>` state into components on the ghost/mission entity.
- Use a `HashMap<MissionId, T>` inside the `Local` to partition state.
- Restructure systems to be stateless where possible (preferred).

### 4.3 Nuclear Cleanup

`headless_summary_reset_system` in `unnet-plugin` currently transitions the global `AppState` from `Summary` back to
`Lobby` and despawns all mission entities. In a multi-mission server, this would tear down ALL missions.

The cleanup logic must be scoped: only entities with a matching `MissionId` component are despawned, and only the
per-mission resources (components on the `MissionInstance` entity) are removed.

### 4.4 Global State Machine

`AppState` and `GameState` are Bevy `States` resources — inherently singleton. The entire game flow (Loading → MainMenu
→ Lobby → InGame → Summary) is driven by these. A multi-mission server cannot use Bevy `States` for per-mission
lifecycle.

**Mitigation:** Replace per-mission state tracking with a `MissionPhase` component on the `MissionInstance` entity:

```rust
#[derive(Component, Clone, Copy, PartialEq, Eq)]
enum MissionPhase {
    Loading,
    Lobby,
    InGame,
    Summary,
    Teardown,
}
```

Systems use `Query<&MissionPhase>` instead of `in_state(AppState::InGame)`. The global `AppState` remains for
server-level lifecycle (startup, shutdown) but no longer drives mission flow.

### 4.5 Unscoped Entity Queries

Nearly every gameplay system iterates entities globally:

- `ghost_movement` queries `Query<&Ghost>` and `Query<&PlayerSprite>` without any mission filter.
- `update_ghost_warning_field` iterates all `q_ghost.iter()`.
- `host_build_snapshot` (via `HostSnapshotParams`) queries all players, ghosts, and gear globally to build a single
  `SnapshotMsg`.

All such queries must add a `With<MissionId>` filter or be parameterized by mission. This is the largest mechanical
change — it touches virtually every system file.

### 4.6 Network Protocol Changes

The current `NetworkMessage` enum and `SnapshotMsg` have no concept of mission identity. Required changes:

1. **`ClientConnection`** needs a `mission_id: MissionId` field indicating which mission this client belongs to.
2. **`SnapshotMsg`** needs a `mission_id` field so the client knows which mission the snapshot describes.
3. **`NetworkMessage` variants** that carry gameplay data (e.g., `InputAction`, `GearAction`, `ChangedTile`) need a
   `mission_id` to route to the correct mission context on the server.
4. **`PlayerRegistry`** needs to track which mission each player is in.

### 4.7 Shared `Time` Is Acceptable

All missions on the same server tick at the same rate (60 Hz via `ScheduleRunnerPlugin`). `Res<Time>` being shared is
correct and requires no changes.

### 4.8 No Dangerous Global Statics

The only `static` in game crates is `DIAGNOSTIC_CHANNEL` in `unmetrics-core` — a `LazyLock<StaticChannel>` used for
fire-and-forget diagnostics. It is thread-safe and mission-agnostic. The `thread_local` RNG in `uncommon-app-core` is
per-thread and safe.

---

## 5. System-Level Impact Assessment

### 5.1 Systems Requiring Per-Mission Scoping

This is not exhaustive but covers the highest-impact areas:

| Plugin                  | Key Systems                                                                                                   | Resources Accessed                                                                                 | Effort                                        |
| ----------------------- | ------------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------- | --------------------------------------------- |
| `unghost-plugin`        | `ghost_movement`, `ghost_enrage`, `ghost_hunt`, `update_ghost_warning_field`, `ghost_spawn`, `ghost_fade_out` | `RoomDB`, `BoardTopology`, `BoardCollisionField`, `HauntState`, `SummaryData`, `CurrentDifficulty` | High — many resources + `Local<T>`            |
| `unthermal-plugin`      | `thermal_diffusion`, `thermal_ghost_effect`                                                                   | `ThermalGrid`, `BoardTopology`                                                                     | Medium                                        |
| `unlight-plugin`        | `light_propagation`, `prebake_lighting`, `dynamic_light_update`                                               | `LightGrid`, `BoardTopology`                                                                       | Medium — entity handles in prebaked data      |
| `unfog-plugin`          | `miasma_simulation`, `fog_rendering`                                                                          | `MiasmaGrid`, `BoardTopology`                                                                      | Medium                                        |
| `unnet-plugin`          | `host_build_snapshot`, `host_apply_client_input`, `network_io_system`, `headless_summary_reset`               | `NetworkConn`, `PlayerRegistry`, all gameplay resources via `HostSnapshotParams`                   | High — routing + snapshot scoping             |
| `unclassic-mode-plugin` | `classic_mode_orchestrator`                                                                                   | `ClassicModeSystemParam` (bundles ~15 resources)                                                   | High — spawns player/ghost, inserts resources |
| `unmapload-plugin`      | Map loading systems                                                                                           | `LevelLoadingStatus`, `BoardTopology`, tile spawning                                               | Medium                                        |
| `untruck-plugin`        | Truck gear management                                                                                         | `TruckGear` (entity handles)                                                                       | Low–Medium                                    |
| `unsound-plugin`        | Sound emission                                                                                                | `SoundGrid`, `SoundEmitter`                                                                        | Low                                           |
| `unwalkie-plugin`       | NPC dialog                                                                                                    | `WalkiePlay`                                                                                       | Low                                           |

### 5.2 Systems That Are Already Safe

- **Asset loading** — `AssetCollection` resources are populated once and shared.
- **Rendering** — disabled on dedicated server (headless mode).
- **UI** — disabled on dedicated server.
- **Noise generation** — `PerlinNoise` is a read-only lookup table.
- **Picking** — likely disabled on server.

---

## 6. Migration Strategy (High-Level)

### Phase 1: MissionInstance Entity Pattern

1. Define a `MissionId(u32)` component and a `MissionInstance` marker component.
2. Create a `MissionBundle` that groups all per-mission data as components on a single `MissionInstance` entity:

   ```rust
   #[derive(Bundle)]
   struct MissionBundle {
       marker: MissionInstance,
       id: MissionId,
       phase: MissionPhase,
       topology: BoardTopology,
       collision: BoardCollisionField,
       entity_field: BoardEntityField,
       thermal: ThermalGrid,
       light: LightGrid,
       miasma: MiasmaGrid,
       room_db: RoomDB,
       haunt: HauntState,
       // ... etc
   }
   ```

3. Add `MissionId` component to every spawned entity (players, ghosts, gear, map tiles).

### Phase 2: System Refactoring

1. Replace `Res<BoardTopology>` with a query for the `MissionInstance` entity. Consider a `MissionContext` `SystemParam`
   that bundles the query and provides helper methods.
2. Add `With<MissionId>` (or a specific mission value filter) to all gameplay entity queries.
3. Migrate `Local<T>` state to components or `HashMap<MissionId, T>`.
4. Replace `in_state(AppState::InGame)` run conditions with a check for "any mission in InGame phase" or remove them for
   always-running systems.

### Phase 3: Network Multi-Tenancy

1. Add `MissionId` to `ClientConnection` and `PlayerRegistry`.
2. Add `mission_id` field to `SnapshotMsg` and relevant `NetworkMessage` variants.
3. Scope `host_build_snapshot` to build one `SnapshotMsg` per mission, sending each only to clients in that mission.
4. Route incoming `InputAction`/`GearAction` messages to the correct mission context.

### Phase 4: Lifecycle Management

1. Create a mission orchestrator that can start/stop individual missions without affecting others.
2. Implement per-mission cleanup that despawns only entities with the matching `MissionId` and removes the
   `MissionInstance` entity.
3. Handle edge cases: what happens when the last player leaves a mission? When does a mission timeout?

---

## 7. Effort Estimate

| Area                                                            | Estimated Scope                  |
| --------------------------------------------------------------- | -------------------------------- |
| `MissionInstance` entity + `MissionId` component infrastructure | Small                            |
| Per-mission resource → component migration (~30 resources)      | Medium                           |
| System query scoping (~50+ systems)                             | Large — mechanical but pervasive |
| `Local<T>` migration (~15 systems)                              | Small–Medium                     |
| Network protocol changes                                        | Medium                           |
| Mission lifecycle orchestrator                                  | Medium                           |
| Cleanup scoping                                                 | Small                            |
| Testing & integration                                           | Large                            |

**Overall:** This is a large refactoring project. No individual change is architecturally difficult, but the breadth of
changes across ~50+ system files and ~30 resources makes it a multi-week effort.

---

## 8. Risks and Open Questions

1. **Performance:** N missions mean N× the grid computation (thermal diffusion, light propagation, fog simulation).
   These are the most CPU-intensive systems. Profiling is needed to determine how many missions a single server can
   handle.

2. **Bevy Schedule Ordering:** With multiple missions, system ordering constraints (e.g., "lighting runs after map
   load") must hold per-mission, not globally. Bevy's scheduling may need careful use of `ambiguity_detection` to verify
   correctness.

3. **Entity ID Space:** Bevy's `Entity` IDs are globally unique within an `App`. This is fine for multi-mission —
   entities from different missions won't collide. However, `NetworkId` assignment in `PlayerRegistry` (currently a
   simple incrementing counter starting at 2) should be reviewed for uniqueness across missions.

4. **Map Loading Concurrency:** The current map loading pipeline assumes a single map loads at a time. Loading two maps
   simultaneously may conflict if any intermediate state is stored in resources rather than on the loading entity
   itself.

5. **`HostSnapshotParams` SystemParam:** This bundles ~15 queries and resources. It would need significant rework to
   accept a `MissionId` parameter for scoping, likely becoming a regular system with explicit query filters rather than
   a `SystemParam`.

6. **Backward Compatibility:** The network protocol changes (adding `mission_id` fields) will break compatibility with
   existing clients. A protocol version negotiation mechanism may be needed.
