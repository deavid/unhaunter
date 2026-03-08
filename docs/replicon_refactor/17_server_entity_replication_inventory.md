# Server Entity Replication Inventory

**Scope**: This document covers only **server-entity-load** replication — entities spawned by the authority and
replicated to join clients at runtime. Map-load hydration (tile entities, movables loaded from TMX files) is explicitly
out of scope.

**Last updated**: reflects changes from the `replicon: add replication + hydration for ghost/breach/gear entities`
commit.

---

## 1. Background: How bevy_replicon Entity Replication Works

An entity is replicated when the authority inserts `bevy_replicon::prelude::Replicated` on it. At that point every
component type registered via `app.replicate::<T>()` that is present on the entity will be serialized and sent to
connected clients. The client receives a new entity (with a different ECS `Entity` ID) and only the registered component
data is deserialized onto it. Components that are **not** registered via `app.replicate` are silently absent on the
client.

Some components contain `Entity` references that need to be translated from server space to client space. These must
implement `MapEntities` and be annotated with `#[component(map_entities)]`.

Hydration is the act of attaching the remaining visual/audio/input components that were not transmitted (and must not
be, since they reference heavy assets). Hydration systems typically use a "marker-absent" query pattern such as
`Without<XxxHydrated>`.

---

## 2. Registered Replicated Components

All `app.replicate::<T>()` calls in the codebase (non-map-load):

| Component               | Registered in                              |
| ----------------------- | ------------------------------------------ |
| `LobbyInfo`             | `unreplicon-plugin/src/systems/lobby.rs`   |
| `ServerGamePhase`       | `unreplicon-plugin/src/systems/lobby.rs`   |
| `SelectedMission`       | `unreplicon-plugin/src/systems/lobby.rs`   |
| `TmxEntityId`           | `unreplicon-plugin/src/systems/players.rs` |
| `Owner`                 | `unreplicon-plugin/src/systems/players.rs` |
| `Position`              | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerSprite`          | `unreplicon-plugin/src/systems/players.rs` |
| `Stamina`               | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerGear`            | `unreplicon-plugin/src/systems/players.rs` |
| `HeldObject`            | `unreplicon-plugin/src/systems/players.rs` |
| `Hiding`                | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerSpectating`      | `unreplicon-plugin/src/systems/players.rs` |
| `GearMarker`            | `unreplicon-plugin/src/systems/players.rs` |
| `GearKind`              | `unreplicon-plugin/src/systems/players.rs` |
| `GhostTag`              | `unreplicon-plugin/src/systems/ghost.rs`   |
| `GhostBreach`           | `unreplicon-plugin/src/systems/ghost.rs`   |
| `GhostSprite`           | `unreplicon-plugin/src/systems/ghost.rs`   |
| `GhostBehaviorDynamics` | `unreplicon-plugin/src/systems/ghost.rs`   |
| `SpectralClarity`       | `unreplicon-plugin/src/systems/ghost.rs`   |
| `GhostGuess`            | `unreplicon-plugin/src/systems/ghost.rs`   |
| `SummaryData`           | `unreplicon-plugin/src/systems/ghost.rs`   |

> **Note**: Gear-specific type components (e.g. `Flashlight`, `Battery`, `Thermometer`, etc.) are **not** in this list.
> They are reconstructed on the client by `hydrate_gear_system` via the `GearSpawnerRegistry` builder, using the
> replicated `GearKind` to select the right builder. State within those components (battery level, on/off) is not yet
> replicated — see Known Issues.

---

## 3. Replicated Entity Types

### 3.1 Lobby Entity

**Marker component**: `LobbyInfo` (structural singleton; no dedicated marker type) **Spawn location**:
`unreplicon-plugin/src/systems/lobby.rs` **Spawn function**: `spawn_lobby_entity_if_missing()` **Trigger**: Runs every
frame on authority; spawns once when the entity is absent.

**Authority spawn bundle**:

```rust
(Replicated, LobbyInfo { … }, ServerGamePhase::Lobby)
```

**Replicated → client**: `LobbyInfo`, `ServerGamePhase`

**Client hydration**: None required. The UI reads these components directly.

---

### 3.2 SelectedMission Entity

**Marker component**: `SelectedMission` **Spawn location**: `unreplicon-plugin/src/systems/lobby.rs` **Spawn function**:
`handle_request_start_mission()` (server message handler) **Trigger**: Leader client sends `RequestStartMission`; server
spawns this entity as a signal for all join clients to start loading the map.

**Authority spawn bundle**:

```rust
(Replicated, SelectedMission { map_path, map_seed, difficulty_id })
```

**Replicated → client**: `SelectedMission`

**Client reaction**: Observer `on_selected_mission_added` fires on `On<Add, SelectedMission>` and calls
`LoadLevelEvent`.

---

### 3.3 Player Entity

**Marker component**: `PlayerTag` (tag), `PlayerSprite` (data, hydration trigger) **Spawn location**:
`unreplicon-plugin/src/systems/players.rs`

**Spawn functions** (two code paths, both on authority):

1. `setup_mission_players()` — fires on `OnEnter(AppState::InGame)`.
2. `spawn_late_joining_players()` — runs every frame during `AppState::InGame` for late joiners.

#### Authority spawn bundle

| Component                      | Replicated? | Notes                                     |
| ------------------------------ | ----------- | ----------------------------------------- |
| `Position`                     | yes         |                                           |
| `LerpPosition::new(spawn_pos)` | no          | Visual interpolation; client-local        |
| `PlayerSprite`                 | yes         | Hydration trigger                         |
| `NetworkId`                    | no          | Only used for gear ID generation          |
| `Stamina`                      | yes         |                                           |
| `PlayerGear`                   | yes         | Entity refs mapped via MapEntities        |
| `Direction::new_right()`       | no          | Re-added by hydration                     |
| `Movable`                      | no          | Re-added by hydration                     |
| `WaypointQueue`                | no          | Re-added by hydration                     |
| `MapEntityFieldBPos`           | no          | Re-added by hydration                     |
| `PlayerTag`                    | no          | Re-added by hydration                     |
| `VisibilityData`               | no          | Added by hydration for local player only  |
| `PlayerInput`                  | no          | Re-added by hydration                     |
| `Owner`                        | yes         | Inserted after spawn; Client(e) or Server |
| `HeldObject`                   | yes         |                                           |
| `Hiding`                       | yes         |                                           |
| `PlayerSpectating`             | yes         |                                           |
| `Replicated`                   | n/a         | bevy_replicon marker                      |

**Components added by hydration** (`hydrate_players_system`):

- Visual/render: `GameSprite`, `MapColor`, `ShadowCaster`, `LightSensitive`, `Mesh2d`, `MeshMaterial2d`, `Transform`,
  `ResolutionFactor`, `MapTileSprite`, `SpriteLayer`
- Gameplay: `Direction`, `Movable`, `WaypointQueue`, `MapEntityFieldBPos`, `PlayerTag`, `LerpPosition`, `PlayerInput`,
  `AnimationTimer`
- Local player only: `PlayerInputMapping` (real key bindings), `MainPlayer`, `Viewer`, `SpatialListener`,
  `VisibilityData`
- Remote player: `PlayerInputMapping` (null key bindings)
- Child entity: `FocusRing` sprite

**Hydration fidelity**: ✅ All non-replicated components are re-added by hydration. The join-client player entity is
functionally equivalent to the authority-spawned one, with two intentional exceptions:

- `NetworkId` is not present on join-client copies (all current usages go through `PlayerSprite.network_id` instead).
- `VisibilityData` is only added for the local player (remote player copies do not need it).

---

### 3.4 Gear Entities (Player Equipment)

**Marker component**: `GearMarker`, `GearKind` **Spawn location**: `unreplicon-plugin/src/systems/players.rs` **Spawn
functions**: Same two as for Player. Gear is spawned via `gear_registry.spawn()`, then `Replicated` and `NetworkId` are
inserted.

#### Authority spawn bundle (via `GearSpawnerRegistry::spawn()`)

| Component                          | Replicated? | Notes                                       |
| ---------------------------------- | ----------- | ------------------------------------------- |
| `GearMarker`                       | yes         | Hydration trigger (with GearKind)           |
| `GearKind`                         | yes         | Hydration trigger; selects builder          |
| `Position::new_i64(0, 0, 0)`       | yes         | Gear starts at (0,0,0) until equipped       |
| `ItemName`, `ItemDescription`      | no          | Re-added by builder via hydration           |
| `GearSprite`, `StatusText`         | no          | Re-added by builder via hydration           |
| `LightEmitter` (if applicable)     | no          | Re-added by builder via hydration           |
| `Toggleable`, `Electronic`         | no          | Re-added by builder via hydration           |
| `Battery`                          | no          | Re-added at defaults; see Known Issues      |
| `Handheld`, `EquipmentPosition`    | no          | Re-added by builder via hydration           |
| `Flashlight` / `Thermometer` / etc | no          | Re-added by builder via hydration           |
| `InteractableByGhost`, `Collision` | no          | Re-added by builder via hydration           |
| `NetworkId`                        | no          | Only used for gear ID counting on authority |
| `Replicated`                       | n/a         | bevy_replicon marker                        |

**Components added by hydration** (`hydrate_gear_system` in `unreplicon-plugin/src/systems/players.rs`):

- Calls `GearSpawnerRegistry::hydrate()` which runs the same builder closure used at spawn time.
- Inserts `GearHydrated` marker when done.

**Hydration fidelity**: ⚠️ Structural fidelity is good — the join-client entity has all the right component types after
hydration. However all mutable state (battery level, on/off status, etc.) is initialized to default values, not the
current server state. A late-joining client will see all remote gear at its initial state. See Known Issues.

`PlayerGear` holds `Option<Entity>` refs mapped via `PlayerGear::map_entities()`, so the structural link player→gear is
correct on join clients.

---

### 3.5 Ghost Entity

**Marker component**: `GhostTag`, `GhostSprite` (hydration trigger) **Spawn location (two-phase)**:

**Phase 1 — Entity creation**: `classic_mode_orchestrator()` in `unclassic-mode-plugin/src/systems/orchestrator.rs`.
Trigger: authority receives `MapEntitiesReadyEvent`.

**Phase 2 — Replication activation**: `setup_ghost_entities()` in `unreplicon-plugin/src/systems/ghost.rs`. Trigger:
`OnEnter(SimulationState::Spawning)`, authority only.

#### Authority spawn bundle

| Component                                            | Replicated? | Notes                                                      |
| ---------------------------------------------------- | ----------- | ---------------------------------------------------------- |
| `Position`                                           | yes         |                                                            |
| `GhostSprite` (with `breach_id` set)                 | yes         | Hydration trigger; `breach_id` mapped via MapEntities      |
| `GhostBehaviorDynamics` (from `haunt_state`)         | yes         | Per-ghost randomized evidence/behavior values              |
| `GhostTag`                                           | yes         | Needed for `sync_ghost_visuals` query on client            |
| `NetworkId(0)`                                       | no          | Not used by any client-side system                         |
| `GameSprite`                                         | no          | NOT added by hydration — see Known Issues §3               |
| `MapEntityFieldBPos`                                 | no          | Authority-only; spatial board field not needed client-side |
| `Movable`                                            | no          | Authority-only; ghost AI runs server-side                  |
| `LightSensitive { exposure_factor: 0.5, bias: 0.01}` | no          | NOT added by hydration — see Known Issues §5               |
| `UltravioletSensitive`                               | no          | NOT added by hydration — see Known Issues §5               |
| `InfraredSensitive`                                  | no          | Authority-only sensor; not needed client-side              |
| `ThermalEmitter`                                     | no          | Authority-only physics emitter                             |
| `FluidEmitter`                                       | no          | Authority-only physics emitter                             |
| `SoundEmitter`                                       | no          | Authority-only physics emitter                             |
| `LerpPosition` (added by `setup_ghost_entities`)     | no          | NOT added by hydration — see Known Issues §4               |
| `Replicated`                                         | n/a         | bevy_replicon marker                                       |

`SpectralClarity` is written by `sync_ghost_visuals` (runs on all nodes with `AppState::InGame`) which derives it from
`GhostBehaviorDynamics`. Both are replicated, so the client also runs the sync redundantly — the result is identical.

**Components added by hydration** (`hydrate_ghosts_system`):

- Visual/render: `Mesh2d`, `MeshMaterial2d`, `Transform`, `MapTileSprite`, `ResolutionFactor`, `SpriteLayer`
- Visual effects: `Ethereal`, `Emissive`, `SpectralClarity`, `AlphaModulator`, `EctoplasmVisuals`
- Child entity: `FocusRing` sprite

**Hydration fidelity**: ⚠️ Three gaps exist on join clients — see Known Issues §3, §4, §5.

---

### 3.6 GhostBreach Entity

**Marker component**: `GhostBreach` **Spawn location**: `classic_mode_orchestrator()` in
`unclassic-mode-plugin/src/systems/orchestrator.rs`. **Replication activation**: `setup_ghost_entities()` inserts
`Replicated` (no `LerpPosition` — breach does not move).

#### Authority spawn bundle

| Component              | Replicated? | Notes                                        |
| ---------------------- | ----------- | -------------------------------------------- |
| `Position`             | yes         | Breach is stationary; never changes          |
| `GhostBreach`          | yes         | Hydration trigger                            |
| `GameSprite`           | no          | NOT added by hydration — see Known Issues §3 |
| `MapEntityFieldBPos`   | no          | Authority-only; not needed client-side       |
| `LightSensitive`       | no          | `{1.1, 0.02}`; NOT in hydration — see §6     |
| `UltravioletSensitive` | no          | `{1.0, 1.0}`; NOT in hydration — see §6      |
| `ThermalEmitter`       | no          | Authority-only physics emitter               |
| `FluidEmitter`         | no          | Authority-only physics emitter               |
| `SoundEmitter`         | no          | Authority-only physics emitter               |
| `Replicated`           | n/a         | bevy_replicon marker                         |

**Components added by hydration** (`hydrate_breach_system`):

- Visual/render: `Mesh2d`, `MeshMaterial2d`, `Transform`, `MapTileSprite`, `SpriteLayer`
- Visual effects: `AlphaModulator`, `EctoplasmVisuals`
- Child entity: `FocusRing` sprite

**Hydration fidelity**: ⚠️ Three gaps exist on join clients — see Known Issues §3, §4, §6.

Note: `GhostSprite.breach_id` on the ghost entity references the breach entity. Since breach is now replicated, this
entity reference is correctly mapped by bevy_replicon via `GhostSprite::map_entities()`.

---

### 3.7 MissionGoalEntity (Journal / Summary Singleton)

**Marker component**: `MissionGoalEntity` **Spawn location**: `unreplicon-plugin/src/systems/ghost.rs` **Spawn
function**: `setup_goal_entity()` **Trigger**: `OnEnter(AppState::InGame)`, authority only.

**Authority spawn bundle**:

```rust
(Replicated, MissionGoalEntity, GhostGuess::default(), SummaryData::default())
```

**Replicated → client**: `GhostGuess`, `SummaryData`

The server writes the current `GhostGuess` and `SummaryData` resources into this entity every frame. Clients read back
into local resources. **Client hydration**: None required. **Client cleanup**: Entity is despawned on
`OnExit(AppState::InGame)`.

---

## 4. Entities That Are NOT Replicated

| Entity type   | Marker       | Spawn location                                      | Reason                                                     |
| ------------- | ------------ | --------------------------------------------------- | ---------------------------------------------------------- |
| Truck gear    | `GearMarker` | `untruck-plugin/src/truckgear.rs`                   | Truck inventory is server-local state                      |
| Particles     | various      | various                                             | Ephemeral visual-only effects                              |
| Ambient audio | `GameSound`  | `unclassic-mode-plugin/src/systems/orchestrator.rs` | Client-local audio, spawned only when local player present |

---

## 5. Summary: Hydration Status per Entity Type

| Entity type       | Hydration system                           | Trigger                                            | Fidelity                        |
| ----------------- | ------------------------------------------ | -------------------------------------------------- | ------------------------------- |
| Lobby entity      | None (N/A)                                 | N/A                                                | ✅ Full (UI reads directly)     |
| SelectedMission   | None (observer)                            | N/A                                                | ✅ Full                         |
| Player            | `hydrate_players_system` (orchestrator.rs) | `PlayerSprite` present + `Without<PlayerHydrated>` | ✅ Full (see §3.3)              |
| Gear              | `hydrate_gear_system` (players.rs)         | `GearKind + GearMarker` + `Without<GearHydrated>`  | ⚠️ Structure OK; state defaults |
| Ghost             | `hydrate_ghosts_system` (orchestrator.rs)  | `GhostSprite` present + `Without<GhostHydrated>`   | ⚠️ 3 gaps (§6)                  |
| GhostBreach       | `hydrate_breach_system` (orchestrator.rs)  | `GhostBreach` + `Without<BreachHydrated>`          | ⚠️ 3 gaps (§6)                  |
| MissionGoalEntity | None (N/A)                                 | N/A                                                | ✅ Full (resource bridge)       |

---

## 6. Known Issues / Gaps Identified

### §1 — Gear state replication gap (deferred)

The gear builder runs at default values when `hydrate_gear_system` fires on a join client. All mutable state
(`Battery { level: 1.0 }`, `Toggleable { is_on: false }`, etc.) is reset to initial defaults. The server-side gear may
already be in a different state. Late-joining clients will see all remote players' gear at its initial state. Fixing
this requires replicating gear state components separately.

### §2 — FocusRing child entity orphan risk (deferred)

`FocusRing` is spawned as a client-local child entity during hydration of player, ghost, and breach. It is NOT
replicated. If bevy_replicon despawns the parent entity without using `despawn_recursive`, the `FocusRing` child becomes
an orphaned entity. This code path has not been tested end-to-end in multiplayer.

### §3 — `GameSprite` missing from ghost and breach on join clients

Both `GhostTag` (ghost entity) and `GhostBreach` (breach entity) are spawned on authority with `GameSprite`. The cleanup
systems `cleanup_game` (unengine-plugin) and `load_level_handler` (unmapload-plugin) query `With<GameSprite>` to despawn
all game entities on mission exit or map reload. Without `GameSprite`, the ghost and breach entities on join clients are
**invisible to these cleanup queries**.

In practice, entity despawn is handled by bevy_replicon when the authority despawns the entity (authority's
`cleanup_game` triggers replicon's entity removal notification). But if replicon cleanup fails or misses an edge case,
ghost/breach entities on join clients will leak and persist across map loads.

**Recommendation**: Add `GameSprite` to `hydrate_ghosts_system` and `hydrate_breach_system`.

### §4 — `LerpPosition` missing from ghost on join clients

On the authority, `setup_ghost_entities` inserts `LerpPosition::new(pos)` onto the ghost when enabling replication.
`LerpPosition` is not registered for replication, and `hydrate_ghosts_system` does not add it. Without it, the
`apply_perspective` render system falls back to raw `Position` (no `Option<&LerpPosition>` result), causing the ghost to
**snap/teleport** to new positions on join clients instead of smoothly interpolating.

**Recommendation**: Add `LerpPosition::new(*pos)` to `hydrate_ghosts_system`.

### §5 — `LightSensitive` / `UltravioletSensitive` missing from ghost on join clients

The ghost is spawned with `LightSensitive { exposure_factor: 0.5, bias: 0.01 }` and `UltravioletSensitive`. These are
used by the lighting render system (`apply_lighting` in unlight-plugin) as `Option<&LightSensitive>` /
`Option<&UltravioletSensitive>`. Without them on join clients the ghost will not react to flashlights or UV torches.

**Recommendation**: Add `LightSensitive` and `UltravioletSensitive` to `hydrate_ghosts_system`.

### §6 — `LightSensitive` / `UltravioletSensitive` missing from breach on join clients

The breach is spawned with `LightSensitive { exposure_factor: 1.1, bias: 0.02 }` and
`UltravioletSensitive { intensity: 1.0, color_shift: 1.0 }`. Same issue as §5 but for the breach effect. Without them
the breach portal will appear unlit / unaffected by UV on join clients.

**Recommendation**: Add `LightSensitive` and `UltravioletSensitive` to `hydrate_breach_system`.
