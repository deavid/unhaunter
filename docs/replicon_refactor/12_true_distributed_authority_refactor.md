# 12 — True Distributed Authority Refactor: Execution Plan

- **Date:** March 1, 2026
- **Branch:** `dev-deavid`
- **Status:** Design finalized. Implementation not yet started.
- **Supersedes:** `11_universal_position_authority_refactor.md` (Phase B section is deprecated)
- **Depends on:** `11_universal_position_authority_refactor.md` (Phase A / gate fixes are still correct)

---

## 1. We Screwed Up — Twice

This document exists because the replicon refactor failed fundamentally, _even after we audited it_.

### The First Failure: Shipping a Broken Architecture

The original replicon refactor (`01` through `08`) introduced `bevy_replicon` and built an entire multiplayer
infrastructure on top of a single misunderstood primitive: `ServerState::Running`. Every authority-side gate became
`in_state(ServerState::Running)`. Every client-side gate became `not(in_state(ServerState::Running))`.

`ServerState::Running` means: _"the network socket is open."_ It has never meant: _"I am the simulation authority."_

The result: singleplayer (Offline mode) — the game's primary single-player experience — silently stopped working
entirely. The ghost stopped moving. Pressing "End Mission" froze the game indefinitely. The lobby never initialized.
None of this was caught before shipping phases 1–8 because the bugs only manifest in Offline mode, and apparently nobody
tested it.

### The Second Failure: An Audit That Still Got It Wrong

Document 11 correctly diagnosed the `ServerState` bug and produced what looked like a clean solution: a "Universal
Position Chain" where `NetworkPosition → Position → Transform` runs on every node for every entity, and the local
player's entity simply has no `NetworkPosition`.

That design was reasonable. But it wasn't the right architecture — it was a better version of the same anti-pattern.

The core issue: `NetworkPosition` is still a mirror component. It is still a separate thing that shadows `Position`. The
direction of the arrow (who writes `NetworkPosition`, who reads it) still has to be arranged differently per entity
type. The authority-side still has to manually copy game state into the mirror. The codebase is still split between
"game components" and "net components" that mean the same thing.

The plan described in document 11, Phase B, would have resulted in a slightly cleaner version of what already existed —
`GhostStateNet` would still be there, `PlayerStateNet` would still be there, and every gear component would still have a
`*Net` twin. We would have refactored the positioning problem and left ten other mirror components untouched.

### The Third Time Must Be Right

A second AI review (Gemini, March 2026) confirmed and extended the critique with two concrete traps that the
document-11-based execution plan walked straight into:

1. **The Entity Serialization Bomb** — Phase 3 of the old plan told us to replicate `PlayerGear` without mentioning that
   `PlayerGear` contains `Entity` fields. Serializing raw `Entity` values across the network is undefined behaviour:
   Entity 104v1 on the server and Entity 104v1 on the client are unrelated things. `bevy_replicon` provides
   `MapNetworkEntities` for this. The old plan missed it entirely.

2. **The Disguised Net Component** — Phase 2 of the old plan suggested that if `GhostSprite` was "too large" to
   replicate, we should extract a `GhostVisualState` containing only the fields clients need. That is `GhostStateNet`
   wearing a fake mustache. It violates Pillar 1 identically.

This document replaces document 11's Phase B entirely. Phase A from document 11 (the gate fixes) remains correct and is
prerequisite work.

---

## 2. What Document 11 Got Right (Keep It)

The Phase A analysis in document 11 is complete, correct, and must be implemented first. Summary:

- **22 `ServerState` violations** across 4 files (lobby.rs, ghost.rs, players.rs, animation.rs)
- **Group 2 fix:** Authority systems gated on `ServerState::Running` → `resource_exists::<AuthorityRole>()`
- **Group 3 fix:** Client systems gated on `not(ServerState::Running)` → `is_pure_client()`
- **Query scope fix:** `interpolate_remote_players` must add `Without<GhostTag>` to its filter
- **is_pure_client:** must be defined once as a named `Condition` in `unreplicon-plugin`

The Four-Node Matrix is the correct analytical frame. Every system that gates on topology must be evaluated against it:

| Node      | `AuthorityRole` | `LocalPlayerRole` | `ServerState::Running` |
| --------- | :-------------: | :---------------: | :--------------------: |
| Offline   |        ✓        |         ✓         |           ✗            |
| Host      |        ✓        |         ✓         |           ✓            |
| Dedicated |        ✓        |         ✗         |           ✓            |
| Join      |        ✗        |         ✓         |           ✗            |

---

## 3. The Finalized Architecture: Five Pillars

The following five pillars were defined by the project owner and are non-negotiable. They replace the "Universal
Position Chain" design from document 11.

### Pillar 1 — Component Purity

> **No `NetXYZ` mirror components. Replicate the actual gameplay component.**

Every component in `unreplicon-core/src/net_components.rs` is a violation of this pillar. `FlashlightNet` mirrors
`FlashlightStatus`. `GhostStateNet` mirrors a subset of `GhostSprite`. `PlayerStateNet` mirrors player vitals. These
components exist only because we were afraid to serialize the real components. That fear must be eliminated.

The architectural end state: `net_components.rs` is deleted. Every replicated piece of data is the real component,
derived only once, in the crate that owns it.

**Critical rule (from Gemini's review, Trap 2):** If `GhostSprite` looks "too large" to replicate, the solution is _not_
to extract a `GhostVisualState` that contains only the fields clients need. That is `GhostStateNet` with a fake
mustache. It violates this pillar identically. The ECS-correct answer is to split `GhostSprite` into a `GhostAI`
component (not replicated) and a `GhostSprite` component (replicated). **Do not create mirror components.**

**Project owner override on `GhostSprite`:** Replicate the entire `GhostSprite` component without splitting. The client
needs this data for sanity computation and health computation. There is no reason to split it now. Do not be clever.
Replicate the whole thing.

### Pillar 2 — LerpPosition Decoupling

> **`Position` = logic truth. `LerpPosition` = visual buffer. `Transform` = screen output.**

The current architecture has one direct `Position → Transform` step (`apply_perspective` in `unrender-plugin`). This
means any network-driven write to `Position` causes an immediate visual snap to the new location.

The fix: introduce `LerpPosition` as a separate component (in `unspatial-core`). It stores a smoothed copy of `Position`
that converges toward `Position` each frame at a configurable speed. `apply_perspective` is changed to read
`LerpPosition.current` when the component is present, falling back to `Position` when it is absent.

The result: network overwrites `Position` directly (no `NetworkPosition` at all), visual movement is smooth because
`LerpPosition` lags behind and catches up gradually, and the local player simply does not get a `LerpPosition` component
— WASD writes to `Position` hit `Transform` at zero latency. No special code path is needed for the local player; the
component is absent, so the fallback fires.

**Note on local player (from Gemini's review):** Confirm that `MainPlayer` entity does NOT receive `LerpPosition`. Input
response must be instantaneous. This is naturally guaranteed by the component's absence — do not accidentally add
`LerpPosition` to the local player during player setup.

### Pillar 3 — Remote Boundary

> **`Without<Remote>` = we own it. `With<Remote>` = network owns it.**

`bevy_replicon` automatically inserts the `Remote` component on every entity spawned by the server on the client. This
is the correct, stable authority boundary. It replaces all `ServerState`-based gates, all `Without<MainPlayer>` hacks,
and all `is_pure_client` workarounds.

Systems that simulate gameplay write to entities `Without<Remote>`. Systems that display received state read from
entities `With<Remote>`.

`is_pure_client` from Phase A is still needed as a run condition for some message-handling systems (those that only make
sense on a client that is not the authority). But entity-level authority — "should I simulate this entity?" — is
answered entirely by the presence or absence of `Remote`.

### Pillar 4 — Owner + Visibility

> **`Owner(ClientId)` marks which client simulates a player entity. `ClientVisibility` hides the entity from that
> client.**

For player entities: the server spawns the entity with `Owner(client_id)`. The server then hides that entity from the
owning client via `ClientVisibility`. The owning client simulates the player locally using its own authority over that
entity. It sends an `ExportStateMessage` (a `ClientMessage`) containing the player's full state. The server applies it,
then broadcasts to all non-owning clients via replicon.

`Owner(ClientId)` lives in `unreplicon-core`. `LocallyOwned` (a zero-size marker component) is inserted by the client on
entities it directly simulates (i.e., the local player). Together these two components let any query precisely select
"entities I am responsible for simulating."

### Pillar 5 — Stable Replicated Entities

> **All gear is real server-authoritative entities. No `FloorGearCache`, no broadcast-spawn hacks.**

Under the current architecture, gear is replicated by type-code (`GearKind`) because gear entities themselves are not
stable across clients. The server broadcasts a "loadout changed" message, the client destroys its local gear entities
and spawns new ones. This breaks late-join, breaks reconnection, and creates the entire `FloorGearCache` subsystem.

The correct architecture: gear entities are spawned by the server, get the `Replicated` component, and replicon handles
distribution. Ownership is transferred to a player when they pick up gear (Orphan & Re-Adopt pattern — see section 5).
Gear on the floor is an entity with no `Owner`. Gear in a player's hands is an entity with `Owner(client_id)`.
Reconnections and late-join work natively.

---

## 4. New Trap: Entity References in Replicated Components

**This is the most dangerous implementation trap and the plan in document 11 walked straight into it.**

`PlayerGear` stores `Entity` references:

```rust
pub struct PlayerGear {
    pub left_hand: Option<Entity>,
    pub right_hand: Option<Entity>,
    pub inventory: Vec<Entity>,
    pub held_item: Option<HeldObject>,
}
```

`HeldObject` also stores an `Entity`:

```rust
pub struct HeldObject {
    pub entity: Entity,
}
```

**You cannot serialize raw `Entity` values across the network.** `Entity` is a 64-bit index+generation value. Entity
104v1 on the server and Entity 104v1 on the client are completely unrelated ECS objects. There is no shared entity ID
space between server and client.

`bevy_replicon` provides a solution: the `MapNetworkEntities` trait. When replicating a component that contains `Entity`
fields, implement `MapNetworkEntities` so replicon can translate server-local entity IDs into the client's corresponding
mapped entity IDs. The translation table is maintained by replicon's `ServerEntityMap` (client-side) and the server's
`ClientEntityMap`.

**Required before `app.replicate::<PlayerGear>()` can work:**

1. Add `#[derive(MapNetworkEntities)]` **or** manually implement `MapNetworkEntities` for `PlayerGear`.
2. Do the same for `HeldObject`.
3. Register with `app.replicate_mapped::<PlayerGear>()` (not `app.replicate::<PlayerGear>()`).

If this is not done, client-side `PlayerGear` will contain raw server entity IDs. Those IDs, when used as queries, will
silently query the wrong entity or return `None`. Players will appear to hold nothing, or the wrong thing. The bug will
be subtle and intermittent (it depends on entity ID coincidences).

**Audit checklist — check every replicated component for `Entity` fields before registering it:**

- `PlayerGear` — Yes, Entity fields. Needs `MapNetworkEntities`.
- `HeldObject` — Yes, Entity field. Needs `MapNetworkEntities`.
- `GhostSprite` — Has `breach_id: Option<Entity>`. Needs `MapNetworkEntities`.
- `Position`, `GhostType`, `GhostGuess`, `SummaryData`, gear status components — No Entity fields. Safe.

---

## 5. Orphan & Re-Adopt: Ownership Transfer Without Despawn

When a player picks up gear from the floor, the authority over that gear entity must transfer from the server (floor
entity, no owner) to the picking-up client. When dropped, authority transfers back to the server.

**The Orphan step (picking up):**

1. Server sets `Owner(client_id)` on the gear entity.
2. Server hides the entity from the owning client via `ClientVisibility::hide_from(client_id)`. The client will no
   longer receive replicon updates for this entity.
3. Client receives an `OwnershipGranted { entity, client_id }` server message.
4. Client removes the gear entity from `ServerEntityMap` (so replicon's incoming updates cannot touch it).
5. Client removes the `Remote` marker from the entity (it is now locally owned).
6. Client inserts `LocallyOwned` on the entity.
7. Client simulates the gear locally; it sends `ExportStateMessage` back to the server.

**The Re-Adopt step (dropping):**

1. Client sends `OwnershipReleased { entity }` message to the server.
2. Client re-inserts the gear entity into `ServerEntityMap`.
3. Client removes `LocallyOwned`, re-inserts `Remote`.
4. Server removes `Owner` from the entity.
5. Server restores `ClientVisibility` so the entity replicates to the releasing client again.
6. Next replicon tick overwrites the client's local state with the server's authoritative state.

This pattern requires no entity despawn or respawn. The ECS entity persists through the entire transfer. Replicon's
entity map and visibility system are the only things adjusted.

---

## 6. The Complete Execution Plan

### Phase 0 — Gate Fixes (Prerequisite, Unblocks Singleplayer)

**Files:** `unreplicon-plugin/src/systems/ghost.rs`, `lobby.rs`, `players.rs`,
`unghost-plugin/src/systems/gis/animation.rs`

**Step 0.1** — Define `is_pure_client` in `unreplicon-plugin/src/systems/mod.rs` (or a new `conditions.rs`):

```rust
pub fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}
```

**Step 0.2** — Fix authority gates (Group 2 from doc 11 §3):

| System                         | File         | Change                                                        |
| ------------------------------ | ------------ | ------------------------------------------------------------- |
| `setup_lobby_entity`           | `lobby.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `set_server_state_ingame`      | `lobby.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `sync_ghost_state_to_net`      | `ghost.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `sync_evidence_to_net`         | `ghost.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `sync_mission_result_to_net`   | `ghost.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `server_teardown_grace_period` | `ghost.rs`   | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |
| `reset_floor_gear_cache`       | `players.rs` | `ServerState::Running` → `resource_exists::<AuthorityRole>()` |

**Step 0.3** — Fix client gates (Group 3 from doc 11 §3):

| System                        | File           | Change                                         |
| ----------------------------- | -------------- | ---------------------------------------------- |
| `interpolate_ghost_position`  | `ghost.rs`     | `not(ServerState::Running)` → `is_pure_client` |
| `apply_ghost_state_net`       | `ghost.rs`     | `not(ServerState::Running)` → `is_pure_client` |
| `apply_evidence_net`          | `ghost.rs`     | `not(ServerState::Running)` → `is_pure_client` |
| `apply_mission_result_net`    | `ghost.rs`     | `not(ServerState::Running)` → `is_pure_client` |
| All `apply_*_net` for players | `players.rs`   | `not(ServerState::Running)` → `is_pure_client` |
| `apply_remote_movable_motion` | `animation.rs` | `not(ServerState::Running)` → `is_pure_client` |

**Step 0.4** — Fix query scope: add `Without<GhostTag>` to `interpolate_remote_players` in `players.rs`.

**Checkpoint 0:** `cargo clippy` passes. Offline mode: singleplayer mission can be started, ghost moves, mission can be
ended. Bug 09 and Bug 10 are resolved. This phase is the safety net — it makes the existing architecture consistent.
Nothing else changes.

---

### Phase 1 — LerpPosition (Pillar 2)

**Files:** `unspatial-core` (new component), `unrender-plugin` (modified system)

**Step 1.1** — In `unspatial-core`, create `src/lerp_position.rs`:

```rust
use bevy::prelude::*;
use crate::position::Position;

/// Visual interpolation buffer. Present on remote entities (ghost, remote players).
/// Absent on the local player — direct Position→Transform at zero latency.
///
/// Each frame, `advance_lerp_position` converges `current` toward the entity's `Position`.
/// `apply_perspective` reads `current` instead of `Position` when this component is present.
#[derive(Component, Debug, Clone)]
pub struct LerpPosition {
    pub current: Position,
    pub speed: f32,
}

impl LerpPosition {
    pub fn new(initial: Position) -> Self {
        Self { current: initial, speed: 15.0 }
    }
    pub fn with_speed(initial: Position, speed: f32) -> Self {
        Self { current: initial, speed }
    }
}
```

Add `pub mod lerp_position;` to `unspatial-core/src/lib.rs`.

**Step 1.2** — In `unrender-plugin`, add a `PreUpdate` system `advance_lerp_position`:

```rust
fn advance_lerp_position(
    mut q: Query<(&Position, &mut LerpPosition)>,
    time: Res<Time>,
) {
    for (pos, mut lerp) in q.iter_mut() {
        let alpha = (lerp.speed * time.delta_secs()).min(1.0);
        lerp.current = lerp.current.lerp(pos, alpha);
    }
}
```

**Step 1.3** — Modify `apply_perspective` in `unrender-plugin/src/plugin.rs` to read `LerpPosition` when present:

```rust
fn apply_perspective(
    mut q: Query<
        (&Position, Option<&LerpPosition>, &mut Transform, Option<&SpriteLayer>),
        Or<(Changed<Position>, Changed<LerpPosition>, Changed<SpriteLayer>)>,
    >,
    ...,
) {
    for (pos, lerp, mut transform, sprite_layer) in q.iter_mut() {
        let effective_pos = if let Some(l) = lerp { &l.current } else { pos };
        // pass effective_pos to perspective::to_screen_coord(...)
    }
}
```

**Step 1.4** — `LerpPosition` is added to entities by the systems that spawn them, not universally. The ghost spawn
system (in `unreplicon-plugin` or `unghost-plugin`) inserts `LerpPosition::new(initial_pos)` on the ghost entity. Remote
player spawn inserts `LerpPosition::new(initial_pos)`. The local `MainPlayer` entity does NOT get `LerpPosition`. This
is the only enforcement needed — absence of the component is its own guard.

**Checkpoint 1:** `cargo clippy` passes. Remote entities move smoothly. Local player input has zero visual latency.

---

### Phase 2 — Ghost Net Component Purge (Pillar 1 for Ghost)

**Files:** `unghost-core`, `unreplicon-plugin/src/systems/ghost.rs`, `unreplicon-core/src/net_components.rs`

**Step 2.1** — Add `Serialize`, `Deserialize` derives to `GhostSprite` in `unghost-core`. Add `serde` as a dependency of
`unghost-core` in its `Cargo.toml`. (`Position` and `BoardPosition` already need to be serializable — confirm or add
their derives too.)

**Step 2.2** — Implement `MapNetworkEntities` for `GhostSprite` (it has `breach_id: Option<Entity>`):

```rust
impl bevy_replicon::prelude::MapNetworkEntities for GhostSprite {
    fn map_entities<M: bevy_replicon::prelude::EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(id) = self.breach_id.as_mut() {
            *id = mapper.map(*id);
        }
    }
}
```

**Step 2.3** — In `unreplicon-plugin/src/systems/ghost.rs`:

- Replace `app.replicate::<GhostStateNet>()` with `app.replicate_mapped::<GhostSprite>()`
- Replace `app.replicate::<NetworkPosition>()` for ghost with — nothing (ghost `Position` is replicated directly;
  `Position` receives `Serialize`/`Deserialize` derives — see Step 2.4 — and is registered with
  `app.replicate::<Position>()`)

**Step 2.4** — Add `Serialize`, `Deserialize` derives to `Position` in `unspatial-core`. Register
`app.replicate::<Position>()` in the setup block.

**Step 2.5** — The ghost AI (in `unghost-plugin`) writes `Position` directly. This was always the case for
authority-side code. No change needed to ghost AI movement systems. The universal replicon tick sends `Position` to
clients. Clients receive it and `LerpPosition` drives the visual update.

**Step 2.6** — Delete the now-redundant systems:

- `sync_ghost_state_to_net` — deleted (was: authority copies `GhostSprite` → `GhostStateNet`)
- `apply_ghost_state_net` — deleted (was: client copies `GhostStateNet` → `GhostSprite`)
- `interpolate_ghost_position` — deleted (was: client lerps `NetworkPosition` → `Position`)

**Step 2.7** — Remove `GhostStateNet` and the ghost's `NetworkPosition` from `net_components.rs`. (Do not delete the
whole file yet — player components remain.)

**Step 2.8** — Handle `EvidenceFoundNet` and `MissionResultNet` the same way:

- Replace with replication of the actual `GhostGuess` and `SummaryData` (or mission-result equivalent) components.
- Add `Serialize`/`Deserialize` to those components in their owning crates.
- Register `app.replicate::<GhostGuess>()` and `app.replicate::<SummaryData>()` (or whatever the correct type is).
- Delete `EvidenceFoundNet`, `MissionResultNet`, and their sync/apply systems.

**Checkpoint 2:** `cargo clippy` passes. Ghost position and state replicate correctly in Host mode. Ghost behavior on
Join client is visually smooth (LerpPosition). Evidence and mission result flow correctly to clients.

---

### Phase 3 — Player Net Component Purge (Pillars 1, 3, 4)

**Files:** `unreplicon-plugin/src/systems/players.rs`, `unplayer-core`, `unreplicon-core`

**Step 3.1** — Add `Owner(ClientId)` and `LocallyOwned` to `unreplicon-core/src/lib.rs` (new file
`unreplicon-core/src/ownership.rs`):

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::ClientId;

/// Marks which client is the simulation authority for this entity.
/// Inserted by the server. Replicated.
#[derive(Component, Debug, Clone, Serialize, Deserialize)]
pub struct Owner(pub ClientId);

/// Marker inserted by the client on entities it directly simulates.
/// Never replicated. Client-local only.
#[derive(Component, Debug, Clone)]
pub struct LocallyOwned;
```

**Step 3.2** — Implement `MapNetworkEntities` for `PlayerGear` and `HeldObject` in `ungear-core`. Register with
`app.replicate_mapped::<PlayerGear>()`.

```rust
impl bevy_replicon::prelude::MapNetworkEntities for HeldObject {
    fn map_entities<M: bevy_replicon::prelude::EntityMapper>(&mut self, mapper: &mut M) {
        self.entity = mapper.map(self.entity);
    }
}

impl bevy_replicon::prelude::MapNetworkEntities for PlayerGear {
    fn map_entities<M: bevy_replicon::prelude::EntityMapper>(&mut self, mapper: &mut M) {
        if let Some(ref mut h) = self.left_hand  { *h = mapper.map(*h); }
        if let Some(ref mut h) = self.right_hand { *h = mapper.map(*h); }
        for slot in self.inventory.iter_mut() { *slot = mapper.map(*slot); }
        if let Some(ref mut ho) = self.held_item { ho.map_entities(mapper); }
    }
}
```

**Step 3.3** — Add `Serialize`/`Deserialize` to player vitals and state components. Register with
`app.replicate::<PlayerState>()` (or whichever the actual canonical components are), replacing all the `PlayerStateNet`,
`PlayerNetInfo`, etc. registrations.

**Step 3.4** — Implement `ExportStateMessage`: a `ClientMessage` that the locally-owning client sends to the server each
frame with its player's full state (`Position`, `PlayerState`, `PlayerGear`, health, etc.). The server applies it and
the replicon tick distributes it to all non-owning clients.

**Step 3.5** — Delete all ten `*Net` player components and their sync/apply systems: `NetworkPosition` (player variant),
`PlayerNetInfo`, `PlayerStateNet`, `FlashlightNet`, `ThermometerNet`, `EMFMeterNet`, `SpiritBoxNet`, `SageBundleNet`,
`RepellentFlaskNet`, `PlayerGearKindNet`.

**Step 3.6** — Player spawn (`setup_mission_players`) inserts `Owner(client_id)` and hides the entity from the owning
client. The owning client receives `OwnershipGranted` and begins local simulation. `MainPlayer` entity on the owning
client gets `LocallyOwned` and has no `Remote`.

**Checkpoint 3:** `cargo clippy` passes. Two-player Host+Join test: both players see each other move correctly. Player
vitals are correct on Join client.

---

### Phase 4 — Gear Rewrite (Pillar 5, Orphan & Re-Adopt)

**Files:** `unreplicon-plugin/src/systems/players.rs`, `ungear-plugin`, `untruck-plugin`

**Step 4.1** — All gear entities are spawned by the server (authority-side), have `Replicated`, and are distributed to
clients via replicon. No client spawns gear entities independently.

**Step 4.2** — Implement the Orphan step in `unreplicon-plugin`: when a player picks up gear, the server sends
`OwnershipGranted { entity, client_id }`. The client removes the entity from `ServerEntityMap`, removes `Remote`,
inserts `LocallyOwned`.

**Step 4.3** — Implement the Re-Adopt step: when a player drops gear, the client sends `OwnershipReleased { entity }`.
The server removes `Owner`, restores `ClientVisibility`. The client re-inserts `Remote` and restores the entity to
`ServerEntityMap`. Next replicon tick overwrites client state with server state.

**Step 4.4** — Delete `FloorGearCache` and all broadcast-spawn gear logic. The `reset_floor_gear_cache` system (which
was broken in Offline anyway) is deleted.

**Checkpoint 4:** `cargo clippy` passes. Host+Join: picking up and dropping gear is stable. Late-join test: gear state
is correct on a freshly connected client.

---

### Phase 5 — God Crate Purge

**Files:** `unreplicon-core/src/net_components.rs`, `unreplicon-core/src/lib.rs`

**Step 5.1** — Verify that `net_components.rs` has zero remaining structs. It should be empty by this point (all
components deleted in Phases 2, 3, 4).

**Step 5.2** — Delete `net_components.rs`. Remove the `pub mod net_components;` line from `unreplicon-core/src/lib.rs`.

**Step 5.3** — Run `cargo clippy`. Resolve any remaining references. There should be none if Phases 2–4 were complete.

**Checkpoint 5:** `cargo clippy` passes cleanly. `net_components.rs` does not exist. All five pillars are in place.

---

## 7. Invariants That Must Hold After Each Phase

Every checkpoint must satisfy:

1. `cargo clippy` — zero new warnings.
2. The Four-Node Matrix — every gated system must behave correctly for all four nodes. Specifically, Offline must never
   be gated by `ServerState`.
3. No `NetXYZ` component is registered with `app.replicate::<XyzNet>()`. Every replicated component is a real gameplay
   component living in its owning crate.
4. No system writes in both the `Position → NetworkPosition` and the `NetworkPosition → Position` directions on the same
   entity.
5. `MainPlayer` entity does not have `LerpPosition`. `Remote` entities (ghost, remote players) do.
6. Every component that contains `Entity` fields and is registered with replicon uses `replicate_mapped` and implements
   `MapNetworkEntities`.

---

## 8. Items Explicitly Out of Scope

The following are acknowledged but deferred:

| Item                                            | Reason                                                                                                                                                                                                  |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unreplicon-core` inverse coupling              | `unghost-core` depending on `unreplicon-core` is now reversed (replicon-core knows about ghost, gear, etc.). Deferred — project owner accepted this for now.                                            |
| Visual fade-to-black on mission end             | `MissionConcludingCinematic` has a timer but no render system produces a visual fade. Deferred.                                                                                                         |
| `D-01/F-18` TMX visual/logic separation         | Deferred from `08_execution_plan.md`.                                                                                                                                                                   |
| `D-02/F-03` `LocalPlayer` resource redesign     | No agreed design. Deferred.                                                                                                                                                                             |
| `O-01/F-20` Dedicated server / Lobby coupling   | Needs own design session.                                                                                                                                                                               |
| `bevy_replicon` `ClientVisibility` API auditing | The Orphan & Re-Adopt pattern requires specific replicon API calls. Before implementing Phase 4, verify the exact API surface for `ServerEntityMap` mutation and `ClientVisibility` in replicon 0.38.2. |

---

## 9. Cross-Reference: How This Supersedes Document 11

| Doc 11 Section                            | Status in Doc 12                                                                                                       |
| ----------------------------------------- | ---------------------------------------------------------------------------------------------------------------------- |
| §2 Core Architectural Flaw (ServerState)  | **Unchanged — still accurate**                                                                                         |
| §3 ServerState Sweep                      | **Unchanged — still accurate. Phase 0 implements this.**                                                               |
| §4 Two Directions for One Arrow           | **Still accurate as diagnosis. Solution is different: delete the mirror component entirely, replicate the real one.**  |
| §5 Project Owner Design Intent            | **Still accurate. §5.2 (Universal Position Chain) is superseded by Pillar 2 of this doc.**                             |
| §6.1 Phase A Gate Corrections             | **Unchanged — Phase 0 of this doc implements it verbatim.**                                                            |
| §6.2 Phase B Universal Position Authority | **DEPRECATED. Replace with Phases 2–5 of this document. The `NetworkPosition` universal chain approach is abandoned.** |
| §7 Out of Scope                           | **Carried forward**                                                                                                    |
| §8 Open Questions                         | **Q about ghost having `NetworkPosition` in Offline is now moot — ghost replicates `Position` directly.**              |
