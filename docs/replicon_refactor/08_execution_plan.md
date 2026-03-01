# 08_execution_plan.md — Implementation Execution Plan

- **Status:** Planning
- **Depends on:** `07_target_architecture.md`
- **Date:** 2026-02-28

---

## Overview

This document translates `07_target_architecture.md` into a concrete, ordered execution plan. Every finding addressed in
that document is accounted for below. The goal is to specify _what to build_, _in what order_, _why that order_, and
_what can be parallelized_.

The plan is organized into **ten sub-phases (SP-1 through SP-10)**. Each is a coherent unit of work that leaves the
build in a clean, deployable state.

---

## Dependency Map (Why This Order)

Before detailing each sub-phase, here is the rationale for the ordering:

```text
SP-1: Infrastructure Primitives
  (BootState + Role resources inserted at boot)
         │
         ├── SP-2: AppState Rename  (mechanical rename; BootState must exist first)
         │         │
         │         └── SP-5: Mission Start/End Flow  (uses EngineBoot name + roles)
         │
         ├── SP-3: SimulationState Alignment  (uses AuthorityRole for F-04)
         │         │
         │         └── SP-5: Mission Start/End Flow  (observes SimulationState::Ready)
         │
         └── SP-4: Lobby Identity Refactor  (uses roles from SP-1 to fix setup_lobby_entity)
                   │
                   └── SP-5: Mission Start/End Flow  (consumes clean LobbyInfo)

SP-6: Role-Based Navigation & Movement  (uses roles from SP-1; otherwise independent)

SP-7: Documentation  (written after states are fully implemented)

SP-8: Role Cleanup — Remaining Crates  (uses roles from SP-1; depends on SP-1 only)

SP-9: Role Cleanup — Final Crates      (uses roles from SP-1; depends on SP-1 only; parallel with SP-8)

SP-10: Role Cleanup — is_authority Sweep  (uses roles from SP-1; depends on SP-1 only; parallel with SP-8/SP-9)
```

**SP-3 and SP-4 are independent of each other and can be done in parallel by two developers.** **SP-5 and SP-6 are
independent of each other and can be parallelized after SP-1–SP-4 are done.**

---

## Scope Boundary

The following items from `07_target_architecture.md` are **explicitly excluded** from this plan. They require a
dedicated design session before implementation begins:

| Item      | Description                                       | Reason Excluded                                                  |
| --------- | ------------------------------------------------- | ---------------------------------------------------------------- |
| D-01/F-18 | Full visual/logic separation in `untmxmap-plugin` | Structural complexity risks breaking the single-player flow.     |
| D-02/F-03 | `LocalPlayer` resource redesign                   | The correct design is not yet agreed upon; depends on F-14 data. |
| O-01/F-20 | Dedicated server coupled to `AppState::Lobby`     | Needs its own design session before coding begins.               |
| O-02/F-21 | State breadcrumbs / navigation history            | No agreed design yet.                                            |
| O-03/F-22 | No lobby disconnect UI                            | No agreed design yet.                                            |

---

## SP-1 — Infrastructure Primitives

**Addresses:** F-09, F-16 (partial); enables all other sub-phases. **Affected crates:** `untypes-core`,
`unengine-plugin`, `unhaunter` (app.rs). **Singleplayer after this phase:** Fully functional. Roles exist but are not
yet used for branching. **Multiplayer after this phase:** No behaviour change. Roles are silently inserted.

### 1.1 — Add `BootState` to `untypes-core`

**File:** `crates/untypes-core/src/states.rs`

Add a new `BootState` enum alongside the existing states:

```rust
#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum BootState {
    #[default]
    Loading,
    Ready,
}
```

This state is **one-directional**: once `Ready` is entered it must never be re-entered as `Loading`. It is never
replicated. There is no `#[derive(Serialize, Deserialize)]` on it.

### 1.2 — Register `BootState` in `unengine-plugin`

**File:** `crates/unengine-plugin/src/plugin.rs`

Register the new state with `app.init_state::<BootState>()`.

Add a system that monitors `Res<Maps>` and sets `BootState::Ready` as soon as the resource is non-empty:

```rust
fn set_boot_ready_when_maps_loaded(
    maps: Option<Res<Maps>>,
    boot_state: Res<State<BootState>>,
    mut next_boot: ResMut<NextState<BootState>>,
) {
    if *boot_state == BootState::Ready { return; } // one-way gate
    if let Some(m) = maps {
        if !m.maps.is_empty() {
            info!("BootState -> Ready ({} maps loaded)", m.maps.len());
            next_boot.set(BootState::Ready);
        }
    }
}
```

Register this system in `Update` with **no state guard** so it runs on both the client and the dedicated server
regardless of which `AppState` they are in. The dedicated server never enters `AppState::MainMenu`, so
`OnEnter(AppState::MainMenu)` would never fire for it — `Update` is the correct schedule here.

The key invariant: `BootState::Ready` is set iff `Res<Maps>` is fully populated. The `bevy_asset_loader` continues to
gate `AppState::MainMenu` (that gate is unchanged for now), but any system that operates on maps must additionally
`.run_if(in_state(BootState::Ready))`.

**Specifically:** The existing `auto_start_headless_lobby` in `crates/unreplicon-plugin/src/systems/lobby.rs` fires in
`OnEnter(ServerState::Running)`. Change it to also require `BootState::Ready` by adding
`.run_if(in_state(BootState::Ready))` to its registration, or by simply checking `!maps.maps.is_empty()` at the top of
the function body before transitioning. This fixes F-09.

### 1.3 — Create Role Resources

**File:** `crates/untypes-core/src/roles.rs` (new file; add `mod roles` to `lib.rs`)

Create three minimal marker resources:

```rust
/// Inserted if this process runs authoritative server logic
/// (Dedicated Server or the server-half of PeerHost).
#[derive(Resource, Debug, Default)]
pub struct AuthorityRole;

/// Inserted if this process has a human watching a screen
/// (pure Client or the client-half of PeerHost).
#[derive(Resource, Debug, Default)]
pub struct LocalPlayerRole;

/// Inserted if this session is part of a multiplayer lobby flow
/// (Client, Dedicated Server, or PeerHost — not Offline single-player).
#[derive(Resource, Debug, Default)]
pub struct LobbyPresenceRole;
```

These are Bevy `Resource`s. Their mere _existence_ is the signal; they carry no data. Code checks them via
`Option<Res<AuthorityRole>>` or `.run_if(resource_exists::<AuthorityRole>())`.

### 1.4 — Insert Roles at Startup

**File:** `crates/unengine-plugin/src/plugin.rs` (or a new `crates/unengine-plugin/src/systems/boot.rs`)

During `Startup`, read `Res<CliOptions>` and insert the appropriate role resources:

| CLI configuration                       | `AuthorityRole` | `LocalPlayerRole` | `LobbyPresenceRole` |
| --------------------------------------- | :-------------: | :---------------: | :-----------------: |
| `NetMode::Offline` (singleplayer)       |        ✓        |         ✓         |                     |
| `NetMode::Host` (PeerHost/ListenServer) |        ✓        |         ✓         |          ✓          |
| `NetMode::Join` (pure Client)           |                 |         ✓         |          ✓          |
| `--dedicated` flag                      |        ✓        |                   |          ✓          |

### 1.5 — Checkpoint

- `cargo clippy` passes with zero new warnings.
- Singleplayer mission can be completed.
- The three role resources exist and are accessible in a running trace; add a `debug!()` log listing which roles were
  inserted at startup so this can be verified without attaching a debugger.

---

## SP-2 — AppState Rename

**Addresses:** F-12 (groundwork only; logic comes in SP-5). **Affected crates:** `untypes-core`, `unengine-plugin`,
`ungear-plugin`, `unui-plugin`, `untruck-plugin`, `unplayer-plugin`, `unmapload-plugin`, `unmanual-plugin`,
`unghost-plugin`, `uncampaign-plugin`, `unreplicon-core`. **Risk:** Wide mechanical rename — no logic changes. Safe to
merge quickly.

### 2.1 — Rename `AppState::Loading` → `AppState::EngineBoot`

**File:** `crates/untypes-core/src/states.rs`

```rust
pub enum AppState {
    #[default]
    EngineBoot,   // was: Loading
    // ... rest unchanged
```

Remove `#[derive(Serialize, Deserialize)]` from the `AppState` enum if it exists, or keep it but add a note that
`EngineBoot` must never appear in any serialized save state.

### 2.2 — Update All `add_loading_state` Call Sites (8 crates)

**Files (mechanical, one-line change each):**

- `crates/ungear-plugin/src/plugin.rs`
- `crates/unui-plugin/src/plugin.rs`
- `crates/untruck-plugin/src/plugin.rs`
- `crates/unplayer-plugin/src/plugin.rs`
- `crates/unmapload-plugin/src/plugin.rs`
- `crates/unmanual-plugin/src/plugin.rs`
- `crates/unghost-plugin/src/plugin.rs`
- `crates/unengine-plugin/src/plugin.rs`

Change every `LoadingState::new(AppState::Loading)` → `LoadingState::new(AppState::EngineBoot)` and
`.continue_to_state(AppState::MainMenu)` stays unchanged.

### 2.3 — Add `AppState::MissionLoading` Variant (stub)

**File:** `crates/untypes-core/src/states.rs`

Add the new variant with a doc comment:

```rust
/// UX loading screen shown while the Authority Node prepares the mission.
/// The client enters this state when a mission is requested and exits to
/// `AppState::InGame` once `SimulationState::Ready` is confirmed.
MissionLoading,
```

No systems are wired to this state yet. That happens in SP-5.

### 2.4 — Update Mission Trigger Call Sites That Abuse `AppState::EngineBoot`

**Files:**

- `crates/unmanual-plugin/src/preplay_manual_ui.rs` (lines 88 and 97)
- `crates/uncampaign-plugin/src/unified_mission_selection.rs` (line 191)

Change each `next_state.set(AppState::Loading)` → `next_state.set(AppState::MissionLoading)`.

At this stage, `MissionLoading` has no exit condition, so missions will appear to freeze on the loading screen. This is
intentional and expected. The exit logic is added in SP-5. Add a comment at each changed call site:

```rust
// SP-5: exit to InGame is handled by SimulationState observer in unreplicon-plugin.
```

### 2.5 — Update Comment in `unreplicon-core/src/components.rs`

**File:** `crates/unreplicon-core/src/components.rs` (~line 55)

The `SelectedMission` doc comment says clients observe `On<Add, SelectedMission>` to "transition their own state to
`AppState::Loading`". Update it to reference `AppState::MissionLoading`.

### 2.6 — Checkpoint

- `cargo clippy` passes.
- Singleplayer boot works: game loads assets, enters `AppState::MainMenu`.
- Attempting to start a mission from the pre-play manual or campaign screen enters a black / spinner screen and stays
  there (expected — SP-5 not done yet). This must not panic.

---

## SP-3 — SimulationState Alignment

**Addresses:** F-04, F-10, F-11a; enables SP-5. **Affected crates:** `untypes-core`, `unengine-plugin`,
`unmapload-plugin`, `unthermal-plugin`, `unlight-plugin`, `unghost-plugin`, `uninteraction-plugin`, `unplayer-plugin`,
`unreplicon-plugin`. **Can run in parallel with SP-4.**

### 3.1 — Rename `SimulationState` Variants

**File:** `crates/untypes-core/src/states.rs`

The current four variants do not match the 07 doc vocabulary. Rename:

| Old name       | New name   | Meaning                                             |
| -------------- | ---------- | --------------------------------------------------- |
| `Inactive`     | `Unloaded` | No map loaded, arrays minimal or empty.             |
| `Initializing` | `Loading`  | Map geometry parsed, arrays being allocated.        |
| `Ready`        | `Spawning` | Arrays fully allocated; ghost/entity spawning runs. |
| `Running`      | `Ready`    | All invariants met; simulation ticks are safe.      |

The new name `Ready` replaces `Running` because this is the state 07 calls the "ironclad contract". The old `Ready` is
renamed `Spawning` to make its transitional role explicit.

This is a wide mechanical rename. Check all usages of `SimulationState::Running` (≥20 sites) and update them.

### 3.2 — Add `TearingDown` Variant

**File:** `crates/untypes-core/src/states.rs`

```rust
/// Mission has ended. Simulation ticks have stopped. Board entities are
/// being despawned and arrays are being zeroed before returning to Unloaded.
TearingDown,
```

No systems are wired to enter or exit `TearingDown` yet. The mission end flow that triggers it is implemented in SP-5.

### 3.3 — Verify Ghost Spawning Precondition (F-10)

**File:** `crates/unreplicon-plugin/src/systems/ghost.rs` (and related `unmapload-plugin` systems)

Confirm that `setup_ghost_entities` fires during `SimulationState::Spawning` (the new `Ready`), before the transition to
`SimulationState::Ready` (the new `Running`). The guarantee we need: when `SimulationState::Ready` is entered, all ghost
entities with `Replicated` already exist.

If ghost spawning currently runs in `OnEnter(AppState::InGame)` rather than being gated by `SimulationState`, move it to
`OnEnter(SimulationState::Spawning)` or `OnEnter(SimulationState::Ready)` (target) on the authority node only (guarded
by `run_if(resource_exists::<AuthorityRole>())`).

Document the invariant in the code with an explicit comment:

```rust
// Invariant: all ghost entities must exist with Replicated by the time
// SimulationState::Ready is entered. This system must run before that transition.
```

### 3.4 — Add F-04 Crash Prevention Guard

**File:** `crates/untmxmap-plugin/src/load_level.rs`, in `load_level_handler`

The real failure mode on the dedicated server is that `maps.get(&map_filepath)` returns `None` because the `.tmx` asset
has not been loaded (headless servers skip `bevy_asset_loader` for visual assets). The function signature is:

```rust
fn load_level_handler(
    mut ev: MessageReader<LoadLevelEvent>,
    // ...
    maps: Res<Maps>,
    tmx_assets: Res<Assets<TmxMap>>,
    // ...
)
```

The guard must be placed **after** reading the event (so the event is consumed) but **before** calling
`UnhaunterMapLoader::load`. Check that the map entry exists in `Maps` before dereferencing:

```rust
let Some(load_event) = ev_iter.next() else { return; };
let map_filepath = load_event.map_filepath.clone();

// F-04 patch: guard against missing map entries on headless server.
// Full visual/logic separation is deferred to D-01 (F-18).
if !maps.maps.contains_key(&map_filepath) {
    warn!(
        "load_level_handler: map '{}' not found in Maps resource; \
         skipping load (headless server with missing asset?)",
        map_filepath
    );
    return;
}
```

The `maps.maps` field is the `HashMap<String, ...>` inside `Res<Maps>`. Verify the exact field name from
`crates/unassets-core/src/resources/maps.rs` before writing the guard.

### 3.5 — Checkpoint

- `cargo clippy` passes.
- Singleplayer mission runs and completes normally.
- On a dedicated server, loading a map with a missing/invalid asset handle emits a `warn!()` rather than panicking.

---

## SP-4 — Lobby Identity Refactor

**Addresses:** F-02, F-14, F-15. **Affected crates:** `unreplicon-core`, `unreplicon-plugin`, `unlobby-plugin`. **Can
run in parallel with SP-3.**

This sub-phase has the highest risk of merge conflicts because it touches the networked data model. Plan to merge SP-3
and SP-4 within the same sprint to minimize divergence time.

### 4.1 — F-14: Replace `u64` Socket ID With UUID Identity

**File:** `crates/unreplicon-core/src/components.rs`

Change the `LobbyPlayerInfo` struct:

```rust
pub struct LobbyPlayerInfo {
    // Old: pub client_id: u64,
    /// Stable persistent identity (from the player's profile installation_id).
    pub player_uuid: Uuid,
    /// The current active transport socket, or None if disconnected.
    pub current_socket: Option<ClientId>,
    // ... rest unchanged
}
```

Change `LobbyInfo`:

```rust
pub struct LobbyInfo {
    // Old: pub owner_client_id: u64,
    /// UUID of the player currently holding lobby leader permissions.
    /// None = no leader (server boot, dedicated server before first player joins).
    pub leader_uuid: Option<Uuid>,
    // ... rest unchanged
}
```

These are replicated structs; Serde derives must be preserved. `Uuid` must use the `serde` feature of the `uuid` crate.
`ClientId` (from bevy_replicon) must not be serialized — if included in the networked component it needs
`#[serde(skip)]` or wrapping in `Option`.

### 4.2 — Update Lobby Handlers in `unreplicon-plugin`

**File:** `crates/unreplicon-plugin/src/systems/lobby.rs`

All code that previously compares `client_id: u64` or `owner_client_id == 0` must be updated:

- `auto_start_headless_lobby` now calls the role check instead of `cli.is_headless()`. The dedicated server boot with no
  leader is simply `leader_uuid: None`.
- `setup_lobby_entity`: The headless branch (empty player list, `owner_client_id: 0`) becomes: spawn entity with
  `leader_uuid: None` (always, regardless of topology). The guest-player is no longer hardcoded at slot 0 with
  `client_id: 0`. Instead, if `LocalPlayerRole` exists, a separate companion system reads the profile's
  `RuntimeInstallationId` and inserts the local player into the lobby. No in-function topology branching.
- `on_client_connected`: the join/reconnect logic uses `player_uuid` as the primary key. If the UUID is already in the
  `players` list, it is a reconnect: update `current_socket`. If new and `leader_uuid.is_none()`, assign.
- `on_client_disconnected`: set `current_socket = None`. If it was the leader's UUID, assign `leader_uuid` to the next
  connected player (`current_socket.is_some()`), or set `None` if nobody remains.
- Permission guards (`set_server_state_ingame`, map selection, difficulty selection): compare `sender_uuid` against
  `lobby.leader_uuid`.

The challenge: where does the sender's UUID come from at the message handler? The JWT ticket carried in the `user_data`
field already has the answer. In `crates/unreplicon-plugin/src/systems/auth.rs`, the `TicketClaims` struct already
contains `player_uuid: String`. The validated UUID must be captured and stored server-side.

**Step A — Create a `ClientUuidMap` resource** in `crates/unreplicon-core/src/resources.rs`:

```rust
/// Server-side mapping from an active bevy_replicon ClientId to the
/// player's stable installation UUID (extracted from the JWT ticket at
/// connection time).
#[derive(Resource, Debug, Default)]
pub struct ClientUuidMap(pub HashMap<ClientId, Uuid>);
```

Initialise it in `lobby.rs::app_setup` with `app.init_resource::<ClientUuidMap>()`.

**Step B — Populate the map in `auth.rs`** after a successful JWT validation. Change `validate_new_connection_observer`
to extract the `player_uuid` claim and insert it into `ClientUuidMap`:

```rust
// After the ticket validates successfully:
if let Ok(uuid) = Uuid::parse_str(&claims.player_uuid) {
    uuid_map.entry(client_id).or_insert(uuid);
}
```

The function will need `mut uuid_map: ResMut<ClientUuidMap>` added to its parameters.

**For hub-less direct-connect** (no JWT, `procman.is_none()` path): there is no ticket to extract a UUID from. For this
mode, generate a deterministic UUID from the `ClientId` integer as a temporary fallback:
`Uuid::from_u128(client_id.get() as u128)`. This is sufficient for the hub-less development path and avoids a
special-case in the permission guards.

**Step C — Look up UUID in message handlers.** Replace the `client_network_id()` helper function with a new
`client_uuid()` helper that reads from `ClientUuidMap`:

```rust
fn client_uuid(client_id: ClientId, uuid_map: &Res<ClientUuidMap>) -> Option<Uuid> {
    uuid_map.0.get(&client_id).copied()
}
```

Every permission guard in `handle_request_select_map`, `handle_request_select_difficulty`, and
`handle_request_start_mission` then compares:

```rust
let sender_uuid = client_uuid(msg.client_id, &uuid_map);
if sender_uuid != lobby.leader_uuid {
    warn!("Request from non-leader {:?}; ignored", sender_uuid);
    continue;
}
```

**Step D — Clean up the map on disconnect** in `on_client_disconnected`:

```rust
uuid_map.0.remove(&trigger.event().client_id);
```

Also remove the TODO comment in `connection.rs` line 84 once Phase SP-4 is complete, since the stable identity is now
tracked via UUID rather than `ClientId`.

### 4.3 — F-15: Delete `LobbyData` and the Bridge

**Files affected:**

- Delete: `crates/unreplicon-plugin/src/systems/bridge.rs` (the file)
- Remove `bridge` from `mod` in `crates/unreplicon-plugin/src/systems/mod.rs`
- Delete: `LobbyData` struct from `crates/unreplicon-core/src/resources.rs`
- Delete: `LobbyPlayer` from the same file (only used by `LobbyData`)
- Delete: `RoomOwner` resource from `crates/unreplicon-core/src/resources.rs`
- Remove: `app.init_resource::<LobbyData>()` from `unreplicon-plugin/src/systems/lobby.rs`

**Update UI in `unlobby-plugin`:**

All systems in `crates/unlobby-plugin/src/systems/` that read `Res<LobbyData>` or `Option<Res<RoomOwner>>` must be
updated to query `LobbyInfo` directly:

```rust
// Before:
fn my_system(lobby_data: Res<LobbyData>, room_owner: Option<Res<RoomOwner>>) { ... }

// After:
fn my_system(q_lobby: Query<&LobbyInfo>) {
    let Ok(lobby) = q_lobby.single() else { return; };
    // use lobby.players, lobby.leader_uuid, etc.
}
```

The `RoomOwner` check (is the local player the leader?) becomes:

```rust
let i_am_leader = local_player_uuid
    .map(|uuid| lobby.leader_uuid == Some(uuid))
    .unwrap_or(false);
```

Where `local_player_uuid` is read from `Res<RuntimeInstallationId>`.

Affected files in `unlobby-plugin`:

- `systems/lobby_main.rs` — reads `LobbyData`, `RoomOwner`
- `systems/difficulty_select.rs` — reads `LobbyData`, `RoomOwner`
- `systems/map_select.rs` — reads `LobbyData`, `RoomOwner`

### 4.4 — Offline Mode: Ensure Universal `LobbyInfo` Spawning

**File:** `crates/unreplicon-plugin/src/systems/lobby.rs`

In offline (single-player) mode, a `LobbyInfo` entity must now always be spawned when entering the lobby phase. This
entity is not replicated (no `Replicated` component) but must exist so that `Query<&LobbyInfo>` in UI systems gracefully
finds data instead of panicking.

The offline spawn path must:

1. Spawn an entity with `LobbyInfo` (no `Replicated`).
2. Set `leader_uuid = Some(local_installation_id)`.
3. Add the local player to `players` with `current_socket: None` (no network in offline mode).

### 4.5 — Checkpoint

- `cargo clippy` passes.
- Singleplayer: lobby screen renders, map selection works.
- `LobbyData` and `bridge.rs` are fully deleted with no remaining references.
- Multiplayer (PeerHost or Join): lobby player list updates correctly from `LobbyInfo` without a bridge.

---

## SP-5 — Mission Start / End Flow Overhaul

**Addresses:** F-07, F-08, F-12, F-13. **Depends on:** SP-1 (roles), SP-2 (MissionLoading state), SP-3
(SimulationState::Ready), SP-4 (clean LobbyInfo). **Affected crates:** `unreplicon-core`, `unreplicon-plugin`,
`unlobby-plugin`, `unmission-plugin` (or equivalent), `unsummary-plugin`, `unmanual-plugin`, `uncampaign-plugin`.

### 5.1 — F-12 Activation: Wire `AppState::MissionLoading` Exit

**File:** `crates/unreplicon-plugin/src/systems/` (new system or extension of an existing one)

The stubs placed in SP-2 must now be activated. Add a system that:

1. Runs in `Update` while in `AppState::MissionLoading`.
2. Observes `SimulationState::Ready` (the new name from SP-3).
3. When both are true, transitions `AppState::MissionLoading → AppState::InGame`.

```rust
fn observe_simulation_ready_to_enter_game(
    sim_state: Res<State<SimulationState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if *sim_state == SimulationState::Ready {
        next_app_state.set(AppState::InGame);
    }
}
// Registered with: .run_if(in_state(AppState::MissionLoading))
```

Remove the old `On<Add, SelectedMission>` observer that previously set `AppState::Loading` (this was only needed because
the old flow triggered the boot-loader). The new flow is observation-based, not event-based.

**Replacing `LoadLevelEvent` in the client-side lobby paths:**

There are two client-side `LoadLevelEvent` write sites in `crates/unlobby-plugin/src/systems/lobby_main.rs` that must be
replaced:

1. **Owner path** (~line 345): After writing `ev_start.write(RequestStartMission {...})`, the current code also writes
   `ev_load.write(LoadLevelEvent {...})`. In the new flow, the client should _not_ load the level directly. Instead, it
   enters `AppState::MissionLoading` and waits for `SimulationState::Ready`:

   ```rust
   // Old: ev_load.write(LoadLevelEvent { map_filepath });
   // New:
   next_app_state.set(AppState::MissionLoading);
   // SP-5: exit to InGame is handled by SimulationState observer in unreplicon-plugin.
   ```

2. **Non-owner join path** (~line 322): When a non-owner client clicks "Start Mission" while a `SelectedMission` entity
   already exists (meaning the server started a mission), the current code writes
   `ev_load.write(LoadLevelEvent { map_filepath: mission.map_path.clone() })`. In the new flow, this same observation of
   `SelectedMission` arriving via replication is the trigger. Remove this manual `ev_load.write` entirely and instead
   ensure the `LocalPlayerRole` client already observes `On<Add, SelectedMission>` and enters `AppState::MissionLoading`
   automatically.

**The server-side `LoadLevelEvent` write in `handle_request_start_mission`
(`crates/unreplicon-plugin/src/systems/lobby.rs` ~line 325)** must remain. The server still uses `LoadLevelEvent` to
drive its own map geometry loading and `SimulationState` transitions. This write site is _not_ deleted in SP-5.

Delete `LoadLevelEvent` from `unmapload-core` only after confirming the two client-side sites above have been removed
and the singleplayer paths (in `unmanual-plugin` and `uncampaign-plugin`, already updated in SP-2) now only set
`AppState::MissionLoading` without writing the event.

### 5.2 — F-07 + F-13: Soft Landing Mission End Flow

**File:** `crates/unreplicon-core/src/components.rs`

Add `Concluding` variant to `ServerGamePhase`:

```rust
pub enum ServerGamePhase {
    Lobby,
    InProgress,
    /// Mission over; server is computing/publishing results. All ticking has stopped.
    Concluding,
    Ended,  // Results published and available on SummaryData
}
```

**Server side** (`crates/unreplicon-plugin/src/systems/ghost.rs` or `mission.rs`):

- When the "End Mission" action fires or TPK occurs, the server transitions `SimulationState → TearingDown` (added in
  SP-3) and sets `ServerGamePhase::Concluding`.
- In `OnEnter(SimulationState::TearingDown)`: compute scores, populate `SummaryData`/`MissionResultNet`, set
  `ServerGamePhase::Ended`. After a grace period (≥5 seconds), despawn board entities and return to
  `SimulationState::Unloaded` + `ServerGamePhase::Lobby`.

**Client side:**

- Add a guard to `apply_mission_result_net`: it must **only** execute when `in_state(AppState::InGame)`. This is the
  F-13 fix — a stale `Changed<MissionResultNet>` from a previous session cannot fire while in the lobby.
- Replace the hard-cut `next_state.set(AppState::Summary)` in `apply_mission_result_net` with a two-phase process.

**Step A — Create the cinematic state resource** in `crates/unreplicon-core/src/resources.rs`:

```rust
/// Inserted by the client when ServerGamePhase::Concluding is observed.
/// Tracks the local fade-to-black timer before transitioning to Summary.
#[derive(Resource)]
pub struct MissionConcludingCinematic {
    /// Countdown timer. When finished, the client transitions to AppState::Summary
    /// provided SummaryData also exists.
    pub timer: Timer,
    /// Whether player inputs have been disabled for the duration of this cinematic.
    pub inputs_blocked: bool,
}
```

**Step B — Observe `ServerGamePhase::Concluding`** with a new client-only system (gated by
`.run_if(resource_exists::<LocalPlayerRole>())`). Register it in `unreplicon-plugin`:

```rust
fn on_mission_concluding(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    mut commands: Commands,
    // audio writer for the "we're leaving" cue:
    // mut walkie: MessageWriter<WalkieEvent>,  ← add if a clip exists
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Concluding {
            info!("ServerGamePhase::Concluding observed — starting cinematic");
            commands.insert_resource(MissionConcludingCinematic {
                timer: Timer::from_seconds(2.5, TimerMode::Once),
                inputs_blocked: true,
            });
        }
    }
}
```

**Step C — Drive the transition** with a separate `Update` system that runs while `in_state(AppState::InGame)` and
`resource_exists::<MissionConcludingCinematic>()`:

```rust
fn tick_mission_concluding(
    mut cinematic: ResMut<MissionConcludingCinematic>,
    summary_data: Option<Res<SummaryData>>,
    mut next_app_state: ResMut<NextState<AppState>>,
    time: Res<Time>,
) {
    cinematic.timer.tick(time.delta());
    if cinematic.timer.finished() && summary_data.is_some() {
        info!("Cinematic complete + SummaryData present — transitioning to Summary");
        next_app_state.set(AppState::Summary);
    }
}
```

The existing `apply_mission_result_net` function in `unreplicon-plugin/src/systems/ghost.rs` must be refactored: remove
the `next_state.set(AppState::Summary)` call from it entirely (its only job now is to copy net data into `SummaryData`),
and add `.run_if(in_state(AppState::InGame))` to its registration.

### 5.3 — F-08: Summary → Lobby Navigation

**File:** `crates/unsummary-plugin/src/summary.rs`

The `keyboard` function at ~line 74 already handles Enter/Escape to leave the summary screen. It currently reads:

```rust
if matches!(cli.net_mode, NetMode::Offline) {
    app_next_state.set(AppState::MissionSelect);
} else {
    app_next_state.set(AppState::Lobby);
}
```

Replace the `cli: Res<CliOptions>` parameter with `lobby_presence: Option<Res<LobbyPresenceRole>>` and update the
branch:

```rust
if lobby_presence.is_some() {
    app_next_state.set(AppState::Lobby);
} else {
    app_next_state.set(AppState::MissionSelect);
}
```

Remove the `use untypes_core::cli::{CliOptions, NetMode};` import once `cli` is no longer needed in this file.

**Add the AFK fallback timer** by creating a local `Resource` and a companion system in the same file:

```rust
#[derive(Resource)]
struct SummaryAfkTimer(Timer);
```

Insert it in `OnEnter(AppState::Summary)` alongside `setup_ui`:

```rust
fn insert_afk_timer(mut commands: Commands) {
    commands.insert_resource(SummaryAfkTimer(
        Timer::from_seconds(30.0, TimerMode::Once)
    ));
}
```

Add a system that ticks the timer, resets it on any key press, and fires the same navigation as the keyboard handler
when it expires:

```rust
fn afk_timeout(
    mut timer: ResMut<SummaryAfkTimer>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    lobby_presence: Option<Res<LobbyPresenceRole>>,
    mut app_next_state: ResMut<NextState<AppState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    time: Res<Time>,
) {
    if keyboard_input.get_just_pressed().next().is_some() {
        timer.0.reset();
        return;
    }
    timer.0.tick(time.delta());
    if timer.0.finished() {
        if lobby_presence.is_some() {
            app_next_state.set(AppState::Lobby);
        } else {
            app_next_state.set(AppState::MissionSelect);
        }
        game_next_state.set(GameState::None);
    }
}
```

Register both in `summary.rs::app_setup`:

```rust
app.add_systems(OnEnter(AppState::Summary), (setup_ui, insert_afk_timer));
app.add_systems(
    Update,
    (keyboard, afk_timeout).run_if(in_state(AppState::Summary)),
);
app.add_systems(OnExit(AppState::Summary), cleanup_ui);
// Also remove the AFK timer resource on exit:
app.add_systems(OnExit(AppState::Summary), |mut commands: Commands| {
    commands.remove_resource::<SummaryAfkTimer>();
});
```

### 5.4 — Checkpoint

- `cargo clippy` passes.
- Singleplayer: starting a mission from any entry point goes `MissionLoading → InGame`. Ending the mission shows a short
  cinematic, then the summary screen, then correctly returns to mission select.
- Multiplayer (PeerHost): same flow, but returns to `AppState::Lobby` after summary.
- The summary screen no longer fires if a player is in `AppState::Lobby` due to a stale `MissionResultNet`.

---

## SP-6 — Role-Based Navigation and Movement

**Addresses:** F-05, F-06, F-11b, F-19. **Depends on:** SP-1 (roles). **Independent of SP-4 and SP-5; can be done in
parallel with SP-4 once SP-1 is complete.** **Affected crates:** `unreplicon-plugin`, `unplayer-plugin`, `ungame-plugin`
(pause menu), `unsummary-plugin`.

### 6.1 — F-05: Summary Screen Navigation by Role

**File:** `crates/unsummary-plugin/src/summary.rs`, `keyboard` function (~line 74)

This function already contains the exact `NetMode` branch described above. The change is a direct swap of the condition
(detailed in SP-5.3 above, which must be completed as part of the same edit pass).

### 6.2 — F-06: Pause Menu Navigation by Role

**File:** `crates/unengine-plugin/src/pause_ui.rs`, `keyboard` function (~line 17)

The current code:

```rust
if matches!(cli.net_mode, untypes_core::cli::NetMode::Offline) {
    next_state.set(AppState::MissionSelect);
} else {
    next_state.set(AppState::Lobby);
}
```

Replace `cli: Res<untypes_core::cli::CliOptions>` with
`lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>` and update the branch:

```rust
if lobby_presence.is_some() {
    next_state.set(AppState::Lobby);
} else {
    next_state.set(AppState::MissionSelect);
}
```

Remove the `use untypes_core::cli::{CliOptions, NetMode}` import from `pause_ui.rs` once `cli` is no longer used.

### 6.3 — F-11b: Host Player Init Behind `LocalPlayerRole`

**File:** `crates/unreplicon-plugin/src/systems/players.rs`

The system that initializes the host player's `PlayerSprite` / initial entity state must be registered behind:

```rust
.run_if(resource_exists::<LocalPlayerRole>())
```

On a dedicated server, `LocalPlayerRole` does not exist, so this system never runs. The dedicated server no longer
silently iterates zero `MainPlayer` entities — the system simply does not exist for it.

### 6.4 — F-19: Unify Player Movement to Single Path

**File:** `crates/unreplicon-plugin/src/systems/players.rs`

1. **Delete** `sync_player_state_to_net` entirely (lines 295–313 approx). Remove it from the system registration in
   `app_setup`.
2. **Remove** the topology guard from `send_local_player_position`:

   ```rust
   // Old:
   send_local_player_position
       .run_if(in_state(AppState::InGame))
       .run_if(not(in_state(ServerState::Running))),
   // New:
   send_local_player_position
       .run_if(in_state(AppState::InGame))
       .run_if(resource_exists::<LocalPlayerRole>()),
   ```

3. **Verify scheduling** (R-01 risk): confirm that `handle_player_move` (the server-side receiver) is scheduled **in the
   same frame** as `send_local_player_position` and **before rendering**. Add a comment documenting the intended
   schedule label (e.g., `FixedUpdate` vs `Update`) so it cannot be accidentally broken.

### 6.5 — Checkpoint

- `cargo clippy` passes.
- Singleplayer: player movement feels identical to before. No frame-latency regression.
- Dedicated server: no panic or warning about missing player entities at startup.
- Pause menu and summary screen navigate correctly without reading `NetMode`.

---

## SP-7 — Documentation

**Addresses:** F-17. **Depends on:** SP-1 through SP-5 being implemented (so the document accurately reflects reality).
**No code changes.**

### 7.1 — Write `docs/multiplayer/server_lifecycle.md`

Create the directory and file. The document must describe the dedicated server's full lifecycle as a state machine:

```text
Process Start
  → BootState::Loading  (asset scanning via bevy_asset_loader)
  → BootState::Ready  (Maps resource populated)
  → Awaiting connections  (SimulationState::Unloaded, ServerGamePhase::Lobby)

Mission requested (RequestStartMission message received):
  → SimulationState::Loading  (map arrays allocated)
  → SimulationState::Spawning  (ghost entities spawned with Replicated)
  → SimulationState::Ready  (arrays valid; ticking begins)
  → AppState::InGame (server perspective; not a UX concept)

Mission end trigger:
  → SimulationState::TearingDown  (ticking stops; SummaryData published)
  → ServerGamePhase::Concluding → Ended
  → SimulationState::Unloaded  (arrays cleared, board entities despawned)
  → ServerGamePhase::Lobby  (back to waiting for next mission)
```

The document must also list which `AppState` variants the dedicated server **never** enters (e.g., `MainMenu`,
`Summary`, `UserManual`, `PreplayManual`, `MissionSelect`, `Hub`). This makes the separation between authority-node
states and UX states explicit and permanent.

---

## SP-8 — Role Cleanup: Remaining Crates

**Addresses:** Completion criteria #6 and #7 for crates not covered by SP-6. **Depends on:** SP-1 (role resources must
exist). **Independent of SP-3 through SP-7 except that SP-1 must be committed first.** **Affected crates:**
`unplayer-plugin`, `unghost-plugin`, `unlobby-plugin`, `untruck-plugin`.

SP-6 cleaned `unreplicon-plugin` of `NetMode`/`is_headless()` usage. The following crates still branch on
`cli.is_headless()` or `cli.net_mode` inside game-logic systems.

### Permitted Sites (do NOT change these)

Before touching anything, confirm the following sites are explicitly permitted and must be left alone:

| File                                   | Line(s)  | Reason permitted                                                                                                                                                        |
| -------------------------------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unmapload-plugin/src/plugin.rs`       | 19, 25   | Plugin `build()` time — same pattern as the SP-1 boot insertion in `unengine-plugin`.                                                                                   |
| `unmapload-plugin/src/assets_debug.rs` | 43, 49   | Debug-only asset monitoring registered in `build()`.                                                                                                                    |
| `untmxmap-plugin/src/load_level.rs`    | 51       | Passes `is_headless` flag into `bevy_load_map` to control visual loading. Covered by D-01 (visual/logic separation in `untmxmap-plugin`), which is explicitly deferred. |
| `unhub-plugin/src/ui.rs`               | 257, 267 | _Writes_ `cli.net_mode = NetMode::Join { .. }` — this is boot-time configuration mutation when the user picks a server from the hub browser, not topology branching.    |

### 8.1 — `unplayer-plugin`: Sanity/Health System Registration

**File:** `crates/unplayer-plugin/src/systems/sanityhealth.rs`, `app_setup` function

The system registrations currently import and use function-item references `is_headless` and `is_authority` from
`untypes_core::cli`:

```rust
use untypes_core::cli::{is_authority, is_headless};
// ...
lose_sanity.run_if(not(is_headless)),
visual_health.run_if(not(is_headless)),
update_profile_death_stats.run_if(not(is_headless)),
health_regen.run_if(is_authority),
server_apply_client_sanity.run_if(is_authority),
```

Replace with role resource checks:

```rust
lose_sanity.run_if(resource_exists::<LocalPlayerRole>()),
visual_health.run_if(resource_exists::<LocalPlayerRole>()),
update_profile_death_stats.run_if(resource_exists::<LocalPlayerRole>()),
health_regen.run_if(resource_exists::<AuthorityRole>()),
server_apply_client_sanity.run_if(resource_exists::<AuthorityRole>()),
```

Remove the `use untypes_core::cli::{is_authority, is_headless};` import once unused.

### 8.2 — `unghost-plugin`: Visual Effect Systems

**Files:**

- `crates/unghost-plugin/src/systems/gis/visual_effects.rs` (lines 38, 206)
- `crates/unghost-plugin/src/systems/gis/execution.rs` (line 735)

**`visual_effects.rs`** — Two systems (`spawn_interaction_particles_system`, `door_lock_indicator_system`) each contain
an `if cli.is_headless() { return; }` early-exit guard. These systems should not run on the dedicated server at all.
Replace the in-body guard with a registration-time gate:

1. Remove the `cli: Res<untypes_core::cli::CliOptions>` parameter from each system.
2. Remove the `if cli.is_headless() { return; }` body.
3. In the `app_setup` function that registers these systems, add `.run_if(resource_exists::<LocalPlayerRole>())` to each
   registration.

**`execution.rs`** — Line 735 contains a conditional visual-effects spawn inside a larger mixed-logic system:

```rust
if !cli.is_headless() {
    visual_effects::spawn_electrical_sparks(commands, asset_server, *position);
}
```

This cannot be hoisted to registration level because the surrounding system does real authority-side work. Replace the
condition in place:

```rust
if local_player_role.is_some() {
    visual_effects::spawn_electrical_sparks(commands, asset_server, *position);
}
```

Add `local_player_role: Option<Res<untypes_core::roles::LocalPlayerRole>>` to the system's parameter list and remove the
`cli: Res<CliOptions>` parameter if it is no longer used elsewhere in the same function.

### 8.3 — `unlobby-plugin`: Lobby UI Authority Check

**Files:**

- `crates/unlobby-plugin/src/systems/lobby_main.rs` (lines 71, 269, 383)
- `crates/unlobby-plugin/src/systems/difficulty_select.rs` (line 132)
- `crates/unlobby-plugin/src/systems/map_select.rs` (line 170)

All five sites share the same pattern inside `match` arms that determine which UI buttons are enabled for the local
player:

```rust
(Some(_), None) => cli.is_authority() && !cli.is_headless(),
```

The semantic is: "I am the server-half of a PeerHost session (authority + local screen)." That is exactly
`AuthorityRole` AND `LocalPlayerRole` both present. Replace with:

```rust
(Some(_), None) => authority.is_some() && local_player.is_some(),
```

Where `authority: Option<Res<AuthorityRole>>` and `local_player: Option<Res<LocalPlayerRole>>` are added to each
system's parameter list. Remove the `cli: Res<CliOptions>` parameter (and any `NetMode` import) from each system once
`cli` is no longer referenced.

### 8.4 — `untruck-plugin`: Journal, Evidence, and Loadout Topology Branches

**Files:**

- `crates/untruck-plugin/src/journal.rs` (lines 125, 158, 199)
- `crates/untruck-plugin/src/evidence.rs` (lines 89, 98)
- `crates/untruck-plugin/src/loadoutui.rs` (line 526)
- `crates/untruck-plugin/src/systems/truck_ui_systems.rs` (line 340)

**Journal and Evidence (`journal.rs`, `evidence.rs`)** — Both files contain a
`match cli.net_mode { Offline | Host { .. } => modify state directly, Join { .. } => send message }` branch. The
semantic is: "if I am the authority node, apply the change locally; otherwise send a message to the server." Replace
with an `AuthorityRole` check:

```rust
// Before:
match cli.net_mode {
    NetMode::Offline | NetMode::Host { .. } => { /* mutate directly */ },
    NetMode::Join { .. } => { ev.write(...); },
}

// After:
if authority.is_some() {
    /* mutate directly */
} else {
    ev.write(...);
}
```

The auto-select/deselect logic in `journal.rs` (line 199: `if !matches!(p.cli.net_mode, NetMode::Join { .. })`) uses the
same semantic — a join client does not auto-select because the server will push the authoritative state. Replace with
`if authority.is_some()`.

Add `authority: Option<Res<AuthorityRole>>` to the system parameter list (or the `SystemParam` struct if the system uses
one). Remove `CliOptions` / `NetMode` imports once unused.

**`loadoutui.rs` and `truck_ui_systems.rs`** — Both files contain:

```rust
if !matches!(cli.net_mode, NetMode::Offline) {
    // Ensure spawned gear entity has a NetworkId so it can be synced to clients.
    commands.entity(entity).insert(NetworkId(...));
}
```

The semantic is: "in any networked session, gear must have a stable `NetworkId`." That is exactly `LobbyPresenceRole`.
Replace with:

```rust
if lobby_presence.is_some() {
    commands.entity(entity).insert(NetworkId(...));
}
```

Add `lobby_presence: Option<Res<LobbyPresenceRole>>` to each system's parameter list. Remove the `CliOptions` /
`NetMode` import from each file once unused.

### 8.5 — Checkpoint

- `cargo clippy` passes.
- Singleplayer: full mission loop unchanged.
- `git grep 'is_headless'` in `crates/` returns only the permitted sites listed above (plus the definition in
  `untypes_core::cli`).
- `git grep 'NetMode'` in `crates/` returns zero gameplay-system matches outside the permitted list (only boot-transport
  setup in `unreplicon-plugin/connection.rs` + `procman.rs`, boot config writes in `unhub-plugin/ui.rs`, and the enum
  definition itself in `untypes_core/cli.rs`).

---

## SP-9 — Role Cleanup: Final Crates

**Addresses:** Completion criteria #6 and #7 for crates not covered by SP-6 or SP-8. **Depends on:** SP-1 (role
resources must exist). **Affected crates:** `unclassic-mode-plugin`, `unplayer-plugin`, `unmission-plugin`,
`unghost-plugin`, `ungearitems-plugin`, `unmainmenu-plugin`.

After SP-8, the global `git grep 'is_headless'` and `git grep 'NetMode'` sweeps found these remaining violation sites.
All are in game-logic systems; none are in the boot-time permitted list.

### 9.1 — `unclassic-mode-plugin`: Orchestrator Visual Guards

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

This file is the mission entity spawner. It is a large `SystemParam`-based system. All 18 `is_headless()` call sites
guard rendering/visual code — mesh handle creation, material insertion, sprite child-spawning, and ambient sound
spawning. No site guards authority-side logic.

There are also two `net_mode` match arms near the start of the function body:

1. **`player_ids_to_spawn` match (lines ~140–151):** The whole match collapses to a single semantic — "spawn one local
   player iff `LocalPlayerRole` is present":

   ```rust
   // Before:
   let player_ids_to_spawn: Vec<Uuid> = match p.cli.net_mode {
       NetMode::Offline => vec![Uuid::from_u128(1)],
       NetMode::Host { .. } => {
           if p.cli.is_headless() { vec![] } else { vec![Uuid::from_u128(1)] }
       }
       NetMode::Join { .. } => vec![],
   };

   // After:
   let player_ids_to_spawn: Vec<Uuid> = if p.local_player_role.is_some() {
       vec![Uuid::from_u128(1)]
   } else {
       vec![]
   };
   ```

2. **Gear-spawn guard (line ~162):** `!matches!(p.cli.net_mode, NetMode::Join { .. })` — semantic is "I am the authority
   node, so I own this player's gear." Replace with `p.authority_role.is_some()`:

   ```rust
   // Before:
   if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Join { .. }) {
       player_gear = spawn_initial_gear(...);
   }

   // After:
   if p.authority_role.is_some() {
       player_gear = spawn_initial_gear(...);
   }
   ```

3. **All 18 `is_headless()` guards:** Every occurrence in this file is `!p.cli.is_headless()` (guard around visual
   insertion) or `p.cli.is_headless()` (used as a branch inside the `player_ids_to_spawn` match, which is collapsed
   above). After replacing the `player_ids_to_spawn` match, the only remaining pattern is:

   ```rust
   // Before:
   if !p.cli.is_headless() { /* visual insertion */ }
   // or:
   if !p.cli.is_headless() && let Some(asset) = ... { /* visual insertion */ }

   // After:
   if p.local_player_role.is_some() { /* visual insertion */ }
   // or:
   if p.local_player_role.is_some() && let Some(asset) = ... { /* visual insertion */ }
   ```

**How to wire the roles into the `SystemParam`:**

The orchestrator system uses a `SystemParam` struct (call it `OrchestratorParams` or similar). Add two new optional
fields:

```rust
local_player_role: Option<Res<'w, untypes_core::roles::LocalPlayerRole>>,
authority_role:    Option<Res<'w, untypes_core::roles::AuthorityRole>>,
```

These replace all accesses to `p.cli.is_headless()` and `p.cli.net_mode` inside the spawner body. Remove the
`CliOptions` import from this file once `p.cli` is no longer referenced anywhere in it.

### 9.2 — `unplayer-plugin`: Movement and Keyboard Authority Check

**Files:**

- `crates/unplayer-plugin/src/systems/movement.rs` (line ~170)
- `crates/unplayer-plugin/src/systems/keyboard.rs` (line ~17)

Both files derive a local boolean from `NetMode`:

```rust
let is_authority = !matches!(cli.net_mode, untypes_core::cli::NetMode::Join { .. });
```

The semantic is identical: "I own non-local players' positions" (authority node moves all players; join client only
moves its own). Replace by adding `authority: Option<Res<AuthorityRole>>` to each system's parameter list and changing
the binding:

```rust
let is_authority = authority.is_some();
```

Remove the `cli: Res<CliOptions>` parameter from each system once no other code in the same function body references
`cli`. Remove the `CliOptions` / `NetMode` imports once unused.

### 9.3 — `unmission-plugin`: Mission-End Evaluation Authority Gate

**File:** `crates/unmission-plugin/src/systems/evaluate_mission_end.rs` (line ~29)

The system body begins with:

```rust
if !matches!(cli.net_mode, NetMode::Host { .. } | NetMode::Offline) {
    return;
}
```

The semantic is: "only the authority node evaluates mission-end conditions." This is a system-body early-return that
should instead be a registration-time guard. Two options; prefer Option A for clarity:

**Option A — registration-time gate (preferred):**

1. Remove the `if !matches!(...)` early-return entirely.
2. In the `app_setup` that registers this system, add `.run_if(resource_exists::<AuthorityRole>())`.
3. Remove `cli: Res<CliOptions>` and the `NetMode` import from the file.

**Option B — inline replacement (only if the system is registered in a way that makes a gate awkward):**

```rust
// Before:
if !matches!(cli.net_mode, NetMode::Host { .. } | NetMode::Offline) { return; }

// After:
if authority.is_none() { return; }
```

Where `authority: Option<Res<AuthorityRole>>` is added to the system's parameter list.

Check which option applies by reading the `app_setup` registration site first.

### 9.4 — `unghost-plugin`: Dynamic Behavior Update Authority Gate

**File:** `crates/unghost-plugin/src/systems/dynamic_behavior_update.rs`, `app_setup` function (line ~164)

The current registration uses a locally-defined closure:

```rust
use untypes_core::cli::{CliOptions, NetMode};
let is_authority =
    |cli: Res<CliOptions>| -> bool { !matches!(cli.net_mode, NetMode::Join { .. }) };

app.add_systems(
    Update,
    (
        update_ghost_behavior_dynamics_system.run_if(is_authority),
        sync_ghost_emitters,
    ).chain(),
);
```

Replace the closure with the standard `resource_exists::<AuthorityRole>()` condition:

```rust
app.add_systems(
    Update,
    (
        update_ghost_behavior_dynamics_system
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
        sync_ghost_emitters,
    ).chain(),
);
```

Remove the `use untypes_core::cli::{CliOptions, NetMode};` import from `app_setup` once unused. Remove the
`is_authority` closure binding entirely.

### 9.5 — `ungearitems-plugin`: Sage and Repellent Flask Authority Check

**Files:**

- `crates/ungearitems-plugin/src/components/sage.rs` (line ~41)
- `crates/ungearitems-plugin/src/components/repellentflask.rs` (line ~49)

Both files derive an inline boolean:

```rust
let is_authority = !matches!(cli.net_mode, untypes_core::cli::NetMode::Join { .. });
```

The semantic is: "only the authority node triggers item activation and game-state mutation." Replace identically to 9.2:
add `authority: Option<Res<AuthorityRole>>` to each system's parameter list and change the binding:

```rust
let is_authority = authority.is_some();
```

Remove `cli: Res<CliOptions>` and `CliOptions` / `NetMode` imports from each file once unused.

### 9.6 — `unmainmenu-plugin`: Menu Item Visibility by Role

**File:** `crates/unmainmenu-plugin/src/mainmenu.rs`, `setup_ui` function (line ~78)

The current code selects which menu items to show based on `NetMode`:

```rust
let mut menu_items = if matches!(cli.net_mode, NetMode::Offline) {
    vec![
        (MenuID::Campaign, ...),
        (MenuID::CustomMission, ...),
        (MenuID::Hub, ...),
    ]
} else {
    vec![(MenuID::MultiplayerLobby, ...)]
};
```

The semantic is: "if the user launched without a lobby (offline/singleplayer), show the single-player menu; if they
launched into a multiplayer session, show the lobby entry point." That is exactly `LobbyPresenceRole` absent vs.
present. Replace:

```rust
// Replace:
cli: Res<CliOptions>,
// With:
lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>,

// Replace the condition:
if lobby_presence.is_none() {
    vec![
        (MenuID::Campaign, ...),
        (MenuID::CustomMission, ...),
        (MenuID::Hub, ...),
    ]
} else {
    vec![(MenuID::MultiplayerLobby, ...)]
}
```

Remove the `use untypes_core::cli::{CliOptions, NetMode};` import from `mainmenu.rs` once unused.

### 9.7 — Checkpoint

- `cargo clippy` passes.
- Singleplayer: full mission loop, entity spawning, item activation, and main menu all work unchanged.
- `git grep 'is_headless'` in `crates/` returns **only** the following permitted sites and the definition site:
  - `unmapload-plugin/src/plugin.rs` lines 19, 25
  - `unmapload-plugin/src/assets_debug.rs` lines 43, 49
  - `untmxmap-plugin/src/load_level.rs` line 51 (D-01 deferred)
  - `untypes_core/src/cli.rs` (the function definition itself)
- `git grep 'NetMode'` in `crates/` returns **only**:
  - `untypes_core/src/cli.rs` (the enum definition)
  - `unreplicon-plugin/src/systems/connection.rs` and `procman.rs` (boot-transport setup)
  - `unhub-plugin/src/ui.rs` lines 257, 267 (boot-time config write)

---

## SP-10 — Role Cleanup: `is_authority` Global Sweep

**Addresses:** Completion criteria #6, #7, and #9 — eliminating all remaining `is_authority(cli)` / `cli.is_authority()`
/ `run_if(is_authority)` sites from game-logic systems. **Depends on:** SP-1 (role resources must exist). **Independent
of SP-3 through SP-9 except that SP-1 must be committed first.** **Affected crates:** `unghost-plugin`,
`ungearitems-plugin`, `unplayer-plugin`, `untruck-plugin`, `uninteraction-plugin`, `unsummary-plugin`.

### Background: Why SP-10 Exists

The checkpoint greps in SP-8.X and SP-9.7 searched for `is_headless` and `NetMode` but not for `is_authority`. The
`is_authority` function in `untypes_core::cli` wraps the same topology-branching logic (`NetMode::Join` check) without
embedding either keyword, so it survived all previous audits. This sub-phase removes every surviving game-logic call
site.

### SP-10 Permitted Sites (do NOT change these)

| File                                  | Lines | Reason permitted                                            |
| ------------------------------------- | ----- | ----------------------------------------------------------- |
| `untypes_core/src/cli.rs`             | 40–46 | The `is_authority` **definition** itself.                   |
| `unreplicon-plugin/src/procman.rs`    | —     | Boot-transport setup; not game-logic topology branching.    |
| `unreplicon-plugin/src/connection.rs` | —     | Boot-transport setup; not game-logic topology branching.    |
| `unhub-plugin/src/ui.rs`              | —     | Config write at hub browser selection; not topology branch. |

---

### 10.1 — `unghost-plugin`: Ghost AI and GIS Registration-Time Gates

**Files:**

- `crates/unghost-plugin/src/systems/ghost_ai/mod.rs`, `app_setup` function (lines ~195–205)
- `crates/unghost-plugin/src/systems/gis/execution.rs`, `app_setup` function (lines ~168–171)
- `crates/unghost-plugin/src/systems/gis/selection.rs`, `app_setup` function (lines ~24–27)

All three files import `untypes_core::cli::is_authority` inside a local `app_setup` scope and pass the function item as
a `.run_if(is_authority)` argument. Replace each with `resource_exists::<AuthorityRole>()`.

**`ghost_ai/mod.rs`** — four systems registered with `.run_if(is_authority)`:

```rust
// Before:
use untypes_core::cli::is_authority;
// ...
ghost_movement.run_if(is_authority),
ghost_enrage.run_if(is_authority),
ghost_fade_out_system.run_if(is_authority),
ghost_scale_glitch_system.run_if(is_authority),

// After (each system, same pattern):
ghost_movement.run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
ghost_enrage.run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
ghost_fade_out_system.run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
ghost_scale_glitch_system.run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
```

Remove the `use untypes_core::cli::is_authority;` import from the `app_setup` scope once unused.

**`gis/execution.rs`** — one tuple registered with `.run_if(is_authority)`:

```rust
// Before:
use untypes_core::cli::is_authority;
app.add_systems(
    bevy::prelude::Update,
    (ghost_interaction_execution_system, watch_tween_insertions).run_if(is_authority),
);

// After:
app.add_systems(
    bevy::prelude::Update,
    (ghost_interaction_execution_system, watch_tween_insertions)
        .run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
);
```

Remove the `use untypes_core::cli::is_authority;` import from the `app_setup` scope once unused.

**`gis/selection.rs`** — one system registered with `.run_if(is_authority)`:

```rust
// Before:
use untypes_core::cli::is_authority;
app.add_systems(
    bevy::prelude::Update,
    ghost_interaction_selection_system.run_if(is_authority),
);

// After:
app.add_systems(
    bevy::prelude::Update,
    ghost_interaction_selection_system
        .run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
);
```

Remove the `use untypes_core::cli::is_authority;` import from the `app_setup` scope once unused.

---

### 10.2 — `ungearitems-plugin`: EMF Meter and Thermometer Inline Checks

**Files:**

- `crates/ungearitems-plugin/src/components/emfmeter.rs` (lines ~51, ~69)
- `crates/ungearitems-plugin/src/components/thermometer.rs` (lines ~42, ~59)

Both files call `untypes_core::cli::is_authority(cli)` inside a system body to derive a local `is_authority: bool`, then
use it to gate battery drain state writes. Apply the same replacement used for `sage.rs` / `repellentflask.rs` in
SP-9.5.

Add `authority: Option<Res<untypes_core::roles::AuthorityRole>>` to each system's parameter list and change the binding:

```rust
// Before (both files):
cli: Res<untypes_core::cli::CliOptions>,
// ...
let is_authority = untypes_core::cli::is_authority(cli);

// After:
authority: Option<Res<untypes_core::roles::AuthorityRole>>,
// ...
let is_authority = authority.is_some();
```

Remove the `cli: Res<untypes_core::cli::CliOptions>` parameter from each system once unused.

---

### 10.3 — `unplayer-plugin`: Setup Registration Gate

**File:** `crates/unplayer-plugin/src/systems/setup.rs`, `app_setup_core` function (line ~25)

The `PlayerAuthoritativeLogicSet` configure-set call uses a function-item reference:

```rust
app.configure_sets(
    Update,
    unplayer_core::authoritative::PlayerAuthoritativeLogicSet
        .run_if(untypes_core::cli::is_authority)
        .after(unplayer_core::PlayerInputSet),
);
```

Replace:

```rust
app.configure_sets(
    Update,
    unplayer_core::authoritative::PlayerAuthoritativeLogicSet
        .run_if(resource_exists::<untypes_core::roles::AuthorityRole>())
        .after(unplayer_core::PlayerInputSet),
);
```

No import change needed unless `untypes_core::cli` becomes unused after this edit — check all other import uses in the
file.

---

### 10.4 — `unplayer-plugin`: Interaction System Inline Branch

**File:** `crates/unplayer-plugin/src/systems/movement.rs`, `player_interaction_system` function (line ~102)

The function uses `cli: Res<CliOptions>` to branch between two network message writers:

```rust
if cli.is_authority() {
    ev_host_interact.write(HostInteractionOccurred { ... });
} else {
    ev_interaction_req.write(InteractionRequestMessage { ... });
}
```

This is a two-path dispatch (host vs. client), not a simple gate. Replace with `authority: Option<Res<AuthorityRole>>`:

```rust
// Before:
cli: Res<CliOptions>,

// After:
authority: Option<Res<untypes_core::roles::AuthorityRole>>,

// Condition:
if authority.is_some() {
    ev_host_interact.write(HostInteractionOccurred { ... });
} else {
    ev_interaction_req.write(InteractionRequestMessage { ... });
}
```

Remove `use untypes_core::cli::CliOptions;` from the file once unused (verify against `player_movement_system` — that
function was already cleaned in SP-9.2 and no longer needs `CliOptions`).

---

### 10.5 — `unplayer-plugin`: Waypoint System Inline Check

**File:** `crates/unplayer-plugin/src/systems/waypoint.rs`, `waypoint_following_system` function (line ~238)

The function uses `cli: Res<untypes_core::cli::CliOptions>` and `is_authority(cli)` to decide whether to speculatively
fire `ExecuteInteractionEvent` on the client:

```rust
cli: Res<untypes_core::cli::CliOptions>,
// ...
let is_authority = is_authority(cli);
// ...
if !is_authority {
    ev_interaction.write(ExecuteInteractionEvent { ... });
}
```

Replace with:

```rust
authority: Option<Res<untypes_core::roles::AuthorityRole>>,
// ...
let is_authority = authority.is_some();
```

Remove the `use untypes_core::cli::is_authority;` and the `Res<untypes_core::cli::CliOptions>` parameter once unused.

---

### 10.6 — `unplayer-plugin`: Mouse Interaction Inline Check

**File:** `crates/unplayer-plugin/src/systems/input/mouse_interaction.rs`, `player_gear_usage_system` function (lines
~25, ~50–55, ~67, ~83–88)

The function calls `is_authority(cli)` once to derive a local `is_authority: bool`, used in multiple
`if is_main || is_authority` guards that control whether the sound emitter fires:

```rust
cli: Res<CliOptions>,
// ...
let is_authority = is_authority(cli);
```

Replace:

```rust
authority: Option<Res<untypes_core::roles::AuthorityRole>>,
// ...
let is_authority = authority.is_some();
```

Remove `use untypes_core::cli::{CliOptions, is_authority};` from the file imports once unused.

---

### 10.7 — `untruck-plugin`: Loadout UI and Truck Gear Registration

**Files:**

- `crates/untruck-plugin/src/loadoutui.rs`, `button_clicked` function (lines ~494, ~507, ~546)
- `crates/untruck-plugin/src/truckgear.rs`, `app_setup` function (line ~10)

**`loadoutui.rs`** — `button_clicked` already has `lobby_presence: Option<Res<LobbyPresenceRole>>` in its signature. The
three `cli.is_authority()` calls gate whether to emit a `TruckLoadoutMessage` to the server. Add
`authority: Option<Res<untypes_core::roles::AuthorityRole>>` and replace:

```rust
// Before (three call sites):
if !cli.is_authority() {
    ev_loadout.write(TruckLoadoutMessage { ... });
}

// After:
if authority.is_none() {
    ev_loadout.write(TruckLoadoutMessage { ... });
}
```

Remove `cli: Res<CliOptions>` from `button_clicked`'s parameter list once unused. Verify that `CliOptions` is not used
elsewhere in `loadoutui.rs`; if not, remove the `use untypes_core::cli::CliOptions;` import.

**`truckgear.rs`** — `initialize_truck_gear` is registered with a function-item gate:

```rust
app.add_systems(
    Update,
    initialize_truck_gear.run_if(untypes_core::cli::is_authority),
);
```

Replace:

```rust
app.add_systems(
    Update,
    initialize_truck_gear
        .run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),
);
```

---

### 10.8 — `uninteraction-plugin`: Interaction Event Handler Inline Check

**File:** `crates/uninteraction-plugin/src/systems/mod.rs`, `interaction_event_handler` function (line ~62)

The function converts `is_authority(cli)` into the `Authority` enum:

```rust
use untypes_core::cli::{CliOptions, is_authority};
// ...
cli: Res<CliOptions>,
// ...
let authority = if is_authority(cli) {
    Authority::Host
} else {
    Authority::Client
};
```

Replace:

```rust
authority_role: Option<Res<untypes_core::roles::AuthorityRole>>,

// Rename to avoid shadowing:
let authority = if authority_role.is_some() {
    Authority::Host
} else {
    Authority::Client
};
```

Remove `use untypes_core::cli::{CliOptions, is_authority};` from the file imports and `cli: Res<CliOptions>` from the
parameter list once unused.

---

### 10.9 — `unsummary-plugin`: Reward Calculation Gates

**File:** `crates/unsummary-plugin/src/plugin.rs`, both `Plugin::build` implementations (lines ~19 and ~33)

`UnhaunterSummaryCorePlugin` runs `calculate_rewards_and_grades` only on the authority:

```rust
calculate_rewards_and_grades.run_if(untypes_core::cli::is_authority),
```

`UnhaunterSummaryPlugin` runs it on **non**-authority (clients reach `AppState::Summary`; the dedicated server does not
show the summary screen):

```rust
calculate_rewards_and_grades.run_if(not(untypes_core::cli::is_authority)),
```

Replace both:

```rust
// First plugin:
calculate_rewards_and_grades
    .run_if(resource_exists::<untypes_core::roles::AuthorityRole>()),

// Second plugin:
calculate_rewards_and_grades
    .run_if(not(resource_exists::<untypes_core::roles::AuthorityRole>())),
```

Verify that `untypes_core::cli` is not imported elsewhere in `plugin.rs` and remove the import once unused.

---

### 10.10 — Checkpoint

- `cargo clippy` passes with zero new warnings.
- Singleplayer: full mission loop works — item toggling, interactions, waypoint following, summary screen, and main menu
  all function correctly.
- `git grep 'is_headless'` in `crates/` returns **only** the permitted sites listed in SP-9.7.
- `git grep 'NetMode'` in `crates/` returns **only** the permitted sites listed in SP-9.7.
- `git grep 'is_authority'` in `crates/` returns **only**:
  - `untypes_core/src/cli.rs` (the function definition lines 40–46)
  - Any permitted boot-transport sites explicitly listed in 10.0 above (`unreplicon-plugin`, `unhub-plugin`)
- `git grep 'CliOptions'` in `crates/` returns **only** the definition site and the permitted boot and config sites
  (`unreplicon-plugin`, `unhub-plugin`, `unmapload-plugin`, `untmxmap-plugin`).

---

## Deferred Work Tracker

Items not in scope for this plan. Revisit after SP-1–SP-10 are merged:

| Tag  | Source | Title                                        | Blocking?                          |
| ---- | ------ | -------------------------------------------- | ---------------------------------- |
| D-01 | F-18   | Visual/logic separation in `untmxmap-plugin` | No; F-04 guard is sufficient patch |
| D-02 | F-03   | `LocalPlayer` resource redesign              | No; deferred pending F-14 data     |
| O-01 | F-20   | Server coupled to `AppState::Lobby`          | No; needs design session           |
| O-02 | F-21   | State breadcrumbs                            | No; needs design session           |
| O-03 | F-22   | Lobby disconnect UI                          | No; needs design session           |

---

## Risk Register

**R-01 — F-19: Frame latency on PeerHost movement through message queue** Routing the PeerHost player's position through
`PlayerMoveMessage` adds an event-queue hop. Mitigation: verify `handle_player_move` and `send_local_player_position`
share a schedule label and run before rendering. Document the intended schedule order in a code comment.

**R-02 — F-04: Silent early return in `load_level_handler` masking broken loads** A guard that returns early without
logging is indistinguishable from a successful load. Mitigation: the guard must emit `warn!()` with the map path; no
silent failure is acceptable.

**R-03 — SP-4 + SP-3 merge conflict risk when developed in parallel** Both sub-phases touch `unreplicon-core` and
`unreplicon-plugin`. Mitigation: merge SP-3 first, then rebase SP-4 before merging. Keep branches short-lived.

**R-04 — SP-4 UUID identity: `ClientId → Uuid` mapping must exist before auth handlers fire** The server must know a
sender's UUID when processing lobby messages, but `ClientId` is the only identifier available at handler time.
Mitigation: trace the handshake path in `auth.rs` / `connection.rs` before writing the new handler. The `TODO Phase 1.4`
comment in `connection.rs` line 84 is the starting point.

**R-05 — SP-5: `LoadLevelEvent` deletion may break the lobby-triggered load path** `LoadLevelEvent` is used in at least
five call sites across different crates. Mitigation: trace all usages before deleting. Confirm every call site is fully
migrated to the `SimulationState` observation pattern before the delete is committed.

---

## Completion Criteria

The entire plan is **done** when:

1. `cargo clippy` passes globally with zero warnings.
2. Singleplayer: a full mission can be started (from campaign, pre-play manual, and classic select), played, ended, and
   the player returns to the correct screen (not `AppState::EngineBoot`, not a blank screen).
3. Multiplayer (PeerHost): same as singleplayer, plus the lobby player list and map selection are functional without any
   `LobbyData` resource.
4. Dedicated server: starts, accepts connections, runs a mission, and returns to lobby without crashing on map load even
   with incomplete asset handles.
5. `docs/multiplayer/server_lifecycle.md` exists and accurately describes the state progression.
6. No `cli.is_headless()` call sites remain in game-logic systems (only in SP-1's role insertion at boot).
7. No `NetMode` branching inside gameplay system logic (only in SP-1's role insertion at boot).
8. `LobbyData`, `bridge_lobby_info_system`, `sync_player_state_to_net`, and `RoomOwner` are fully deleted with no dead
   references.
9. No `cli.is_authority()`, `is_authority(cli)`, or `run_if(is_authority)` call sites remain in game-logic systems (only
   the definition in `untypes_core/src/cli.rs` and the boot-time insertion in SP-1 are permitted).
