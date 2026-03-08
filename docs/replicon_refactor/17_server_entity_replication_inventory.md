# Server Entity Replication Inventory

**Scope**: This document covers only **server-entity-load** replication — entities spawned by the authority and
replicated to join clients at runtime. Map-load hydration (tile entities, movables loaded from TMX files) is explicitly
out of scope.

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

| Component          | Registered in                              |
| ------------------ | ------------------------------------------ |
| `LobbyInfo`        | `unreplicon-plugin/src/systems/lobby.rs`   |
| `ServerGamePhase`  | `unreplicon-plugin/src/systems/lobby.rs`   |
| `SelectedMission`  | `unreplicon-plugin/src/systems/lobby.rs`   |
| `TmxEntityId`      | `unreplicon-plugin/src/systems/players.rs` |
| `Owner`            | `unreplicon-plugin/src/systems/players.rs` |
| `Position`         | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerSprite`     | `unreplicon-plugin/src/systems/players.rs` |
| `Stamina`          | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerGear`       | `unreplicon-plugin/src/systems/players.rs` |
| `HeldObject`       | `unreplicon-plugin/src/systems/players.rs` |
| `Hiding`           | `unreplicon-plugin/src/systems/players.rs` |
| `PlayerSpectating` | `unreplicon-plugin/src/systems/players.rs` |
| `GhostSprite`      | `unreplicon-plugin/src/systems/ghost.rs`   |
| `GhostGuess`       | `unreplicon-plugin/src/systems/ghost.rs`   |
| `SummaryData`      | `unreplicon-plugin/src/systems/ghost.rs`   |

> **Note**: `GearMarker`, `GearKind`, and all gear-specific components (e.g. `Flashlight`, `Thermometer`, etc.) are
> **not** in this list. Gear entities are replicated as structural stubs only — their type identity and state reach the
> client only via `PlayerGear` entity references and any future hydration.

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

**Replicated components received by client**: `LobbyInfo`, `ServerGamePhase`

**Client hydration**: None required. The UI reads these components directly. **Client hydration system**: N/A

---

### 3.2 SelectedMission Entity

**Marker component**: `SelectedMission` **Spawn location**: `unreplicon-plugin/src/systems/lobby.rs` **Spawn function**:
`handle_request_start_mission()` (server message handler) **Trigger**: Leader client sends `RequestStartMission`; server
spawns this entity as a signal for all join clients to start loading the map.

**Authority spawn bundle**:

```rust
(Replicated, SelectedMission { map_path, map_seed, difficulty_id })
```

**Replicated components received by client**: `SelectedMission`

**Client reaction**: Observer `on_selected_mission_added` fires on `On<Add, SelectedMission>` and calls
`LoadLevelEvent`. **Client hydration system**: N/A (observer, not a hydration system)

---

### 3.3 Player Entity

**Marker component**: `PlayerTag` (tag), `PlayerSprite` (data, also the hydration trigger) **Spawn location**:
`unreplicon-plugin/src/systems/players.rs`

**Spawn functions** (two code paths, both on authority):

1. `setup_mission_players()` — fires on `OnEnter(AppState::InGame)`; spawns all players currently in `LobbyInfo`.
2. `spawn_late_joining_players()` — runs every frame during `AppState::InGame`; spawns players that joined after
   `setup_mission_players` already ran.

**Authority spawn bundle** (skeleton; identical between the two paths):

```rust
(
    Position (spawn_pos),
    LerpPosition::new(spawn_pos),
    PlayerSprite::new(uuid, net_id, spawn_pos),
    NetworkId,
    Stamina::default(),
    PlayerGear { left_hand, right_hand, inventory, … },
    Direction::new_right(),
    Movable,
    WaypointQueue::default(),
    MapEntityFieldBPos(spawn_pos.to_board_position()),
    PlayerTag,
    VisibilityData::default(),
    PlayerInput::default(),
    Owner(owner_id),        // OwnerId::Server for host, OwnerId::Client(e) for remotes
    Replicated,
)
```

Host player additionally gets `LocallyOwned` inserted immediately. Remote-player clients are notified via
`OwnershipGranted` message so they can insert `LocallyOwned` on their own replica.

**Replicated components received by client**: `Position`, `PlayerSprite`, `Stamina`, `PlayerGear` (with mapped entity
refs), `Owner`, `Hiding`, `HeldObject`, `PlayerSpectating`

**Client hydration** (visual + input): `hydrate_players_system()` in `unclassic-mode-plugin/src/systems/orchestrator.rs`
**Hydration trigger**: Query `(Entity, &PlayerSprite, &Position), Without<PlayerHydrated>` **Hydration marker
inserted**: `PlayerHydrated` **Guard**: Returns early when `LocalPlayerRole` resource is absent (dedicated servers are
headless and skip all visual hydration).

**Components added by hydration**:

- `GameSprite`, `MapColor`, `ShadowCaster`, `LightSensitive`
- `Direction`, `Movable`, `WaypointQueue`, `MapEntityFieldBPos`
- `LerpPosition`, `PlayerInput`, `AnimationTimer`
- `Mesh2d`, `MeshMaterial2d`, `Transform`, `ResolutionFactor`, `MapTileSprite`, `SpriteLayer`
- Local player only: `PlayerInputMapping`, `MainPlayer`, `Viewer`, `SpatialListener`
- Remote player only: `PlayerInputMapping` (with null key bindings)
- Child entity: `FocusRing` sprite

---

### 3.4 Gear Entities (Player Equipment)

**Marker component**: `GearMarker` (always present), `GearKind` (enum, identifies type) **Spawn location**:
`unreplicon-plugin/src/systems/players.rs` **Spawn functions**: Same two as for Player (`setup_mission_players` and
`spawn_late_joining_players`). Gear is spawned before the player skeleton using `gear_registry.spawn()`, then
`Replicated` and `NetworkId` are inserted.

**Authority spawn** (via `GearSpawnerRegistry::spawn()`):

```rust
commands.spawn((GearMarker, GearKind::Xxx, Position::new_i64(0, 0, 0)))
```

Then the registered builder function for that `GearKind` inserts kind-specific components (e.g. `Flashlight`,
`Thermometer`, uv-torch components, etc.). After spawn:

```rust
commands.entity(gear_entity).insert((NetworkId(gear_id_counter), Replicated));
```

The entity reference is then stored in `PlayerGear.left_hand`, `.right_hand`, or `.inventory`.

**Replicated components received by client**: `Position`, `Owner` (if present), and any other components that happen to
be in the global `app.replicate` list.

> **Critical gap**: `GearKind` and `GearMarker` are **not** registered for replication. The client receives a bare
> entity with only the globally-registered components. The gear type identity is never transmitted. There is currently
> **no gear hydration system** — the client has no way to know what kind of gear each replicated entity represents, and
> cannot attach the correct visual or state components to it.

**Gear entity references**: `PlayerGear` holds `Option<Entity>` fields that are mapped client-side via
`PlayerGear::map_entities()`. This correctly translates server entity IDs to client entity IDs, so the structural
connection is maintained — but the entities themselves are effectively empty on the client.

---

### 3.5 Ghost Entity

**Marker component**: `GhostTag` (tag), `GhostSprite` (data, also the hydration trigger) **Spawn location** (two-phase):

**Phase 1 — Entity creation**: `classic_mode_orchestrator()` in `unclassic-mode-plugin/src/systems/orchestrator.rs`.
Trigger: authority receives `MapEntitiesReadyEvent`.

Authority spawn bundle at orchestrator time:

```rust
(
    Position (ghost_spawn),
    GhostSprite (with breach_id set to breach entity),
    GhostBehaviorDynamics,
    GhostTag,
    NetworkId(0),
    GameSprite,
    MapEntityFieldBPos(ghost_spawn.to_board_position()),
    Movable,
    LightSensitive, UltravioletSensitive, InfraredSensitive,
    ThermalEmitter, FluidEmitter, SoundEmitter,
)
```

At this point the ghost entity does NOT yet have `Replicated`.

**Phase 2 — Replication activation**: `setup_ghost_entities()` in `unreplicon-plugin/src/systems/ghost.rs`. Trigger:
`OnEnter(SimulationState::Spawning)`, authority only.

```rust
commands.entity(ghost_entity).insert((Replicated, LerpPosition::new(pos)));
```

**Replicated components received by client**: `GhostSprite` (with `breach_id` entity ref, mapped via
`GhostSprite::map_entities()`), `Position`

**Client hydration** (visual only): `hydrate_ghosts_system()` in `unclassic-mode-plugin/src/systems/orchestrator.rs`
**Hydration trigger**: Query `(Entity, &Position, &GhostSprite), Without<GhostHydrated>` **Hydration marker inserted**:
`GhostHydrated` **Guard**: Returns early when `LocalPlayerRole` resource is absent (dedicated servers skip).

**Components added by hydration**:

- `Mesh2d`, `MeshMaterial2d`, `Transform`, `MapTileSprite`, `ResolutionFactor`, `SpriteLayer`
- `Ethereal`, `Emissive`, `SpectralClarity`, `AlphaModulator`, `EctoplasmVisuals`
- Child entity: `FocusRing` sprite

> **Notable gap**: The ghost entity does NOT receive `GhostTag`, `Movable`, `GhostBehaviorDynamics`, `ThermalEmitter`,
> `FluidEmitter`, `SoundEmitter`, etc. on the client via replication — these are registered nowhere and are therefore
> absent on join clients. Ghost behavior (hunting, influence, sound, etc.) runs authority-side only, which is
> intentional. However any client-side system that queries for `GhostTag` will find the replicated entity missing that
> component.

---

### 3.6 MissionGoalEntity (Journal / Summary Singleton)

**Marker component**: `MissionGoalEntity` **Spawn location**: `unreplicon-plugin/src/systems/ghost.rs` **Spawn
function**: `setup_goal_entity()` **Trigger**: `OnEnter(AppState::InGame)`, authority only.

**Authority spawn bundle**:

```rust
(Replicated, MissionGoalEntity, GhostGuess::default(), SummaryData::default())
```

**Replicated components received by client**: `GhostGuess`, `SummaryData`

The server writes the current `GhostGuess` and `SummaryData` resources into this entity every frame (via
`sync_ghost_guess_to_mission_goal` / `sync_summary_data_to_mission_goal`). Clients read from this entity back into local
resources (via `sync_mission_goal_to_ghost_guess` / `sync_mission_goal_to_summary_data`).

**Client hydration**: None required. **Client cleanup**: Entity is despawned on `OnExit(AppState::InGame)`.

---

## 4. Entities That Are NOT Replicated

The following entities are spawned by the authority but **intentionally lack `Replicated`**:

| Entity type   | Marker                      | Spawn location                                      | Reason                                                                                             |
| ------------- | --------------------------- | --------------------------------------------------- | -------------------------------------------------------------------------------------------------- |
| `GhostBreach` | `GhostBreach`               | `unclassic-mode-plugin/src/systems/orchestrator.rs` | Visual-only on the authority side; position is transmitted indirectly via `GhostSprite.breach_pos` |
| Truck gear    | `GearMarker` (+ `GearKind`) | `untruck-plugin/src/truckgear.rs`                   | Truck inventory is server-local state; clients manage their own truck UI                           |
| Particles     | various                     | various                                             | Ephemeral visual-only effects, never replicated                                                    |
| Ambient audio | `GameSound`                 | `unclassic-mode-plugin/src/systems/orchestrator.rs` | Client-local audio                                                                                 |

> **Problem with `GhostBreach`**: `GhostSprite.breach_id` holds an entity reference to the breach entity and implements
> `MapEntities`. However, the breach entity is never in the client's `ServerEntityMap`. When bevy_replicon unmaps
> `GhostSprite.breach_id` on the client, `mapper.get_mapped()` will not find a valid mapping, resulting in either a
> placeholder entity or a stale server-side ID being stored on the client. Any client-side code that dereferences
> `GhostSprite.breach_id` will get a dangling entity reference.

---

## 5. Summary: Hydration Status per Entity Type

| Entity type             | Has hydration system?              | Hydration trigger                                    | Missing on client                                      |
| ----------------------- | ---------------------------------- | ---------------------------------------------------- | ------------------------------------------------------ |
| Lobby entity            | No                                 | N/A                                                  | Nothing (UI reads directly)                            |
| SelectedMission         | No                                 | Observer reacts directly                             | Nothing                                                |
| Player                  | **Yes** — `hydrate_players_system` | `Without<PlayerHydrated>` on `PlayerSprite` presence | Visual mesh, animation, controls, camera               |
| Gear (player equipment) | **No**                             | N/A                                                  | `GearKind`, `GearMarker`, all type-specific components |
| Ghost                   | **Yes** — `hydrate_ghosts_system`  | `Without<GhostHydrated>` on `GhostSprite` presence   | Visual mesh; behavior components intentionally absent  |
| MissionGoalEntity       | No                                 | N/A                                                  | Nothing (resource bridge reads directly)               |

---

## 6. Known Issues / Gaps Identified

1. **Gear hydration is entirely missing.** Gear entities arrive on the client as empty stubs — only `Position` (and any
   other globally-registered components) are present. `GearKind` is not replicated, so the client cannot know what kind
   of item each entity represents. A gear hydration system needs to be written, and `GearKind` needs to be added to
   `app.replicate`.

2. **`GhostBreach` entity is not replicated, but `GhostSprite.breach_id` references it.** On join clients, `breach_id`
   will be an unmapped / invalid entity reference after deserialization. Any system using `GhostSprite.breach_id` on the
   client (e.g. for breach visibility, focus ring on breach, walkie events) will malfunction in multiplayer. Options:
   replicate the breach entity, or strip `breach_id` from replicated `GhostSprite` and rebuild the reference on the
   client via a query.

3. **Ghost entity skeleton is incomplete on client.** `GhostTag`, `Movable`, `GhostBehaviorDynamics`, physics emitters,
   etc. are absent. Authority-side ghost movement and behavior systems depend on these. This is intentional for a
   server-authoritative design, but client-side systems that query `With<GhostTag>` (e.g. sound, walkie triggers,
   environment queries) need to be audited to confirm they work correctly with only `GhostSprite` present.

4. **`GhostTag` not registered for replication.** The client ghost entity only carries `GhostSprite` + `Position` (+
   replicon internals). Every system using `With<GhostTag>` is effectively a server-only/host-only query.
