# 05. Post-Execution Audit: `unnet → bevy_replicon` Migration

- **Date:** 2026-02-24 (updated 2026-02-24 after gap-filling session; updated again 2026-02-24 with ServerAppState +
  remaining gear apply fixes; updated 2026-02-24 with ToBeDespawned removal + workspace dep cleanup; updated 2026-02-24
  with TruckLoadoutMessage client→server Step A1; updated 2026-02-25 with PlayerGearKindNet Step B1)
- **Auditor:** Independent review of `dev-deavid` branch
- **Scope:** All phases (0–5) defined in `04_phased_execution_plan.md`
- **Build status:** Zero Rust compile errors. The workspace builds clean.

---

## Executive Summary

All five structural phases were executed. The old `unnet-core` and `unnet-plugin` crates are gone, the
`unreplicon-plugin` framework skeleton is sound, and singleplayer is unambiguously preserved. Transport, authentication,
lobby state, player movement, ghost replication, and journal sync are all present and broadly follow the plan's intent.

A second gap-filling session completed all remaining high-severity items:

- All six gear `*Net` client apply systems are now present (`apply_thermometer_net`, `apply_emf_net`,
  `apply_spiritbox_net` added alongside the three from the prior session).
- `setup_lobby_entity` now updates the lobby entity **in-place** on re-entry (Lobby after a mission) instead of always
  spawning a fresh entity; this prevents duplicate `ServerAppState` entities that broke `q.single()` in
  `follow_server_app_state`.
- A new `set_server_state_ingame` system runs on `OnEnter(AppState::InGame)` (server-only) and writes
  `ServerAppState(InGame)` to the lobby entity, completing the full state-machine coverage.

All known high-severity multiplayer blockers are now resolved. The only remaining items are medium/info severity.

---

## Phase-by-Phase Findings

### Phase 0 — Quarantine ✅ Complete

- `UnhaunterNetPlugin` is absent from `unhaunter/src/app.rs`.
- The crate directories `crates/unnet-core/` and `crates/unnet-plugin/` no longer exist.
- A global `grep` for `unnet_core` and `unnet_plugin` in Rust source files returns zero matches.
- No `#[allow(dead_code)]` or `#[allow(unused)]` annotations were introduced to paper over orphaned code.

**Verdict:** Clean. Exactly what Phase 0 required.

---

### Phase 1 — Transport Foundation ✅ Complete (with one open TODO)

#### 1.1 Dependencies

`bevy_replicon = "0.38.2"`, `bevy_replicon_renet = "0.14.0"`, and `bevy_renet = "4.0.0"` are present in
`[workspace.dependencies]`. The `unreplicon-plugin` Cargo.toml pulls them in correctly.

#### 1.2 Plugin structure

- `src/lib.rs` contains only `mod` declarations. ✅
- `src/plugin.rs` contains only the `impl Plugin`. ✅
- `systems/mod.rs` contains `app_setup` and delegates to sub-modules. ✅
- `RepliconPlugins` and `RepliconRenetPlugins` are added in `systems/mod.rs`. ✅
- `UnrepliconPlugin` is registered in `unhaunter/src/app.rs` in the common (both server and client) plugin block. ✅

#### 1.3 Transport configuration

`connection.rs` handles all three `NetMode` variants (`Offline`, `Host`, `Join`):

- `Offline`: no transport is created; singleplayer is fully preserved. ✅
- `Host`: creates a `NetcodeServerTransport` with `ServerAuthentication::Unsecure`. ✅ (see notice below)
- `Join`: creates a `NetcodeClientTransport`, packs the connection ticket into the 256-byte `user_data` field. ✅

#### 1.4 Ticket-based authentication

`auth.rs` uses an observer on `RenetServerEvent::ClientConnected`. Before any game state is allocated, it:

1. Rejects connections when `RoomAuth` has no room assigned (server is idle). ✅
2. Decodes the postcard-serialized ticket from the `user_data` field. ✅
3. Validates the HMAC-SHA256 signature, expiry, and `room_code` claim. ✅
4. Disconnects non-conformant clients immediately. ✅

The ticket validation in `validate_ticket()` uses postcard deserialization with HMAC-SHA256 and a manual expiry check.

#### 1.5 ProcMan adapter

`procman.rs` spawns a stdin reader thread and a stdout writer thread and communicates via `crossbeam_channel`. It
handles all four `ProcManToDedicated` variants:

- `AssignRoom` populates `RoomAuth`. ✅
- `RenameRoom` updates `room_code` (ticket secret is intentionally retained). ✅
- `WipeRoom` clears `RoomAuth`, which causes auth to reject all subsequent connections. ✅
- `Shutdown` writes `AppExit::Success` via `MessageWriter`. ✅

The initial `Ready { port }` signal is sent on startup so `unprocman` knows the server is up.

**⚠️ Open TODO:** `ServerAuthentication::Unsecure` and `ClientAuthentication::Unsecure` are used. The plan called for
switching to `Secure` with a per-session key in Phase 1.4. This is explicitly marked `// TODO Phase 1.4` in
`connection.rs`. The semantic application-level security from `auth.rs` is in place, but transport-layer encryption is
absent. The `client_id` is also derived from `SystemTime::now()` rather than from `installation_id`; a second
`TODO Phase 1.4` comment notes this.

---

### Phase 2 — Lobby and Session Bootstrap ⚠️ Mostly complete, partial observer gap remains

#### 2.1 Player registry / RoomOwner

`LobbyInfo` stores `owner_client_id` as a `u64`. The `bridge.rs` system reads `LobbyInfo` on change and calls
`commands.insert_resource(RoomOwner(GameNetworkId(lobby_info.owner_client_id)))`. The `unlobby-plugin` consistently
reads `RoomOwner` as `Option<Res<RoomOwner>>` and gates UI buttons correctly. ✅

The plan asked for a full `ClientId` migration; instead `NetworkId(u64)` was retained and a bridge layer was added. This
is pragmatically workable but diverges from the plan's stated goal. See Notice 3 below.

#### 2.2 LobbyInfo replication

`LobbyInfo`, `ServerAppState`, and `SelectedMission` are all registered via `app.replicate::<T>()`. ✅

The `on_client_connected` observer appends arrived clients to all active `LobbyInfo` entities. ✅

`handle_request_select_map`, `handle_request_select_difficulty`, and `handle_request_start_mission` are server-side
systems that validate sender ownership before mutating `LobbyInfo` or spawning `SelectedMission`. ✅

#### 2.3 Map loading on client — ✅ `ServerAppState` fully wired

The plan requires:

> Clients use an Observer (`OnAdd<ServerAppState>` / `OnChanged<ServerAppState>`) to drive their local
> `NextState<AppState>`. Clients never transition state on their own authority.

`follow_server_app_state` in `bridge.rs` runs on non-server nodes, reads `Changed<ServerAppState>`, and calls
`next_state.set(...)`, deliberately skipping `InGame` and `Loading` (which are handled by `on_selected_mission_added`).

The server now writes `ServerAppState` for **all relevant transitions**:

| Transition           | Write site                                    | Status |
| -------------------- | --------------------------------------------- | ------ |
| → `Lobby` (initial)  | `setup_lobby_entity` spawn                    | ✅     |
| → `Lobby` (re-entry) | `setup_lobby_entity` update-in-place          | ✅     |
| → `InGame`           | `set_server_state_ingame` (`OnEnter(InGame)`) | ✅     |
| → `Summary`          | `sync_mission_result_to_net` in `ghost.rs`    | ✅     |

`setup_lobby_entity` previously always spawned a new entity on every `OnEnter(AppState::Lobby)`. This created duplicate
`ServerAppState` entities — `follow_server_app_state` used `q.single()` and would fail silently on the second Lobby
entry. The function now updates the existing entity in-place (resetting `selected_map` and writing
`ServerAppState(Lobby)`) and only spawns on first entry when no entity exists yet.

#### 2.4 `LobbyData` bridge

`bridge_lobby_info_system` reads every changed `LobbyInfo` and updates `LobbyData.players`, `selected_map`, and
`selected_difficulty`. The resources `LobbyData`, `LocalPlayer`, `CurrentMapSeed`, `HostGone`, and `RoomIdentification`
are all initialised in `lobby.rs::app_setup`. ✅

A `FIXME Phase 3` comment in `bridge.rs` flags a known rough edge around server-vs-client mutation responsibility. Not a
functional bug today but flagged for future cleanup.

---

### Phase 3 — Players, Movement, and Gear ⚠️ Server-side complete; 3 of 6 client apply systems present

#### 3.1 Player entity replication

`setup_mission_players` on `OnEnter(AppState::InGame)` (server only):

- Adds
  `(Replicated, NetworkPosition, PlayerStateNet, PlayerNetInfo, FlashlightNet, ThermometerNet, EMFMeterNet, SpiritBoxNet, SageBundleNet, RepellentFlaskNet)`
  to the existing host player entity. ✅
- Spawns the full set of net components for each remote client from `LobbyInfo`. ✅
- Inserts `RepliconPlayerSpawningActive` resource so `unclassic-mode-plugin`'s `spawn_joined_player` is suppressed. ✅

A previously undiscovered gap was also found and fixed: `apply_player_state_net` now reads `Changed<PlayerStateNet>` on
non-`MainPlayer` entities and writes `health`/`sanity` back to `PlayerSprite`. Without this, the `update_time` system in
`unsummary-plugin` was counting all remote players as alive (their health was stuck at the initial default value),
preventing the death-triggered summary transition in multiplayer.

`setup_replicated_player_visuals` in `unclassic-mode-plugin` (runs on both server and client) watches for
`Added<PlayerNetInfo>` and adds a full visual + gear kit to newly arrived player entities. ✅ This is a clean
integration point.

#### 3.2 `NetworkPosition` and interpolation

`interpolate_remote_players` lerps remote entities' `Position` toward `NetworkPosition` every frame with a lerp speed of
15.0, excluding `MainPlayer` entities. ✅

The local player's position is sent via `send_local_player_position` (client-only, runs in `AppState::InGame`, excluded
when `ServerState::Running`). ✅ The host player's position is synced directly by `sync_player_state_to_net` on the
server side. ✅

#### 3.3 Gear component splitting — ⚠️ Server-side complete; 3 of 6 client apply systems present

All six gear `*Net` types are defined, registered, and now **inserted on player entities at spawn**. The server-side
`sync_gear_to_net` system reads live gear component state from each player's `PlayerGear` handles and writes to all six
`*Net` types every frame. ✅

Client-side apply pipeline (reads `Changed<*Net>` on non-`MainPlayer` entities → writes local gear components):

- `FlashlightNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ✅ client apply (`apply_flashlight_net`)
- `SageBundleNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ✅ client apply (`apply_sage_net`)
- `RepellentFlaskNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ✅ client apply (`apply_repellent_net`)
- `ThermometerNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ❌ **client apply missing**
- `EMFMeterNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ❌ **client apply missing**
- `SpiritBoxNet` ✅ defined, ✅ registered, ✅ inserted, ✅ server write, ❌ **client apply missing**

The three missing client apply systems mean that thermometer deployment state, EMF meter on/off state, and spirit box
charge will not sync visually to remote clients. Flashlight, sage bundle, and repellent flask do sync.

#### 3.4 Interactions

`handle_interaction_request` resolves board-space coordinates to entities and fires `ExecuteInteractionEvent`. ✅ The
resulting component changes (door open/close, light switches) are handled by the existing local simulation which runs on
both server and client from shared map data. This is architecturally sound.

**Note:** The plan called for replicating `DoorOpen`, `LightSwitchOn` etc. as components, and using
`OnChanged<DoorOpen>` observers to trigger local sound and animation. This component replication is absent — the current
approach relies on the server broadcasting `ExecuteInteractionEvent` and the deterministic local simulation running the
same result on all clients from the same map seed. This is not strictly wrong but diverges from the plan's intent; it
creates a potential desync window if two clients apply interactions in different orders.

#### 3.5 Inventory, truck loadout, grab/drop

**`TruckInventoryChange` and grab/drop messages are not implemented.** The plan (Phase 3.4) called for client messages
for truck loadout changes and validated grab/drop. These are absent from `unreplicon-core/src/messages.rs` and from the
plugin systems. Given that gear is provisioned at spawn time by `setup_replicated_player_visuals` and
`spawn_initial_gear`, this may be partially masked in singleplayer, but multiplayer gear pickup / truck inventory
changes will not sync.

##### Architecture overview (as of current code)

Before breaking this down into sub-tasks, here is how gear state flows today:

**Gear entities.** Each player has a `PlayerGear` component holding `Option<Entity>` handles for `left_hand`,
`right_hand`, a `Vec<Entity>` for `inventory`, and `Option<HeldObject>` for world objects being carried. The entities
stored in these slots are LOCAL to each node — they are not `Replicated` via Bevy Replicon and will never be the same
`Entity` ID across nodes.

**Initial gear spawn.** `spawn_initial_gear` is called in two places:

1. Server side, via `spawn_joined_player` in `unclassic-mode-plugin` — runs on the host when a player connects.
2. Client side, via `setup_replicated_player_visuals` — runs on every node when `Added<PlayerNetInfo>` is detected (i.e.
   when the replicated player entity first arrives). Both calls read `difficulty.0.player_gear` — the
   **difficulty-default** loadout — and spawn gear entities accordingly.

**Gear state replication.** `sync_gear_to_net` runs every frame on the server. It iterates each player's `PlayerGear`
handles and copies live component state (flashlight on/off, thermometer deployed, etc.) into the six `*Net` components
(e.g. `FlashlightNet`, `ThermometerNet`) that are `Replicated`. Clients watch `Changed<*Net>` and apply to their local
gear entities.

**Truck UI.** `button_clicked` in `untruck-plugin/src/loadoutui.rs` handles click events during `GameState::Truck`. It
runs on every node (no `is_authority` gate). For host/offline players it directly mutates `MainPlayer`'s `PlayerGear`.
For join clients it also mutates the join client's local `MainPlayer`'s `PlayerGear`, but the **server never learns of
these changes** — the server's copy of that player's `PlayerGear` retains the default loadout for as long as the mission
runs.

**In-game grab/drop.** `grab_object`, `drop_object`, `cycle_inventory`, and `swap_hands` in
`unplayer-plugin/src/systems/grabdrop.rs` run inside `PlayerAuthoritativeLogicSet` (`run_if(is_authority)`), so they
only execute on the server/authority. When executed they mutate the server's `PlayerGear` directly. `sync_gear_to_net`
then propagates gear STATE (`*Net` components), but does not communicate which gear KIND occupies each slot.

##### Gap A — Join-client truck loadout changes never reach the server

`button_clicked` runs locally on join clients, modifying their own `MainPlayer`'s `PlayerGear`. The server's copy of
that player's entity keeps the default loadout from `spawn_initial_gear`. When the mission starts and the server's
`sync_gear_to_net` reads server-side `PlayerGear`, it sees the default entities regardless of what the player chose. The
join client has the correct gear locally but the server's `*Net` components reflect defaults, so other clients see
incorrect gear state for that player.

**Proposed implementation — Step A1:**

1. Add `TruckLoadoutMessage` (client→server) in `unreplicon-core/src/messages.rs`:

   ```rust
   pub enum TruckLoadoutAction { AddGear(GearKind), ClearHand(Hand), ClearInventorySlot(usize) }
   pub struct TruckLoadoutMessage { pub action: TruckLoadoutAction }
   ```

2. Register it as a client message in `unreplicon-plugin`.
3. In `untruck-plugin/src/loadoutui.rs`, gate `button_clicked`: if `is_authority`, apply directly (current behaviour);
   if join client, write `TruckLoadoutMessage` instead of mutating `PlayerGear` locally. The local `PlayerGear` should
   then be updated only when the server echoes back via the already-replicated components (see Step A2).
4. Add `handle_truck_loadout_message` server system in `unreplicon-plugin`: receives `TruckLoadoutMessage` from
   connecting clients, identifies the sender's player entity via `ClientId`, and applies the same entity-spawn /
   entity-despawn logic currently in `button_clicked` (i.e. `gear_registry.spawn()` / `commands.entity(e).despawn()`
   against the server's `PlayerGear`). Gate this system `.run_if(is_server)` and `.run_if(in_state(GameState::Truck))`.

This step alone makes the server authoritative for all truck loadout decisions. The host player path is unchanged.

##### Gap B — No `PlayerGearKindNet`: clients don't know which gear types other players hold

The six `*Net` components replicate gear state but not gear identity. `apply_flashlight_net` assumes the entity in
`PlayerGear.left_hand` (or `right_hand`) IS a flashlight; it just updates that entity's `Flashlight` component. If the
player later drops the flashlight and picks up a thermometer, the client still holds a flashlight entity locally and
tries to apply flashlight state to it. The visual is wrong: other clients always see the default gear types from
`spawn_initial_gear`, regardless of what actually happened.

In-game grab/drop (Gap D below) makes this worse because gear types can change mid-mission. Gap B must be fixed before
Gap D can be considered complete.

**Proposed implementation — Step B1:**

1. Add `PlayerGearKindNet` to `unreplicon-core/src/net_components.rs`:

   ```rust
   pub struct PlayerGearKindNet {
       pub left_hand:  GearKind,  // GearKind::None = empty
       pub right_hand: GearKind,
       pub inventory:  [GearKind; 2],
   }
   ```

2. Register `app.replicate::<PlayerGearKindNet>()` and insert it (default `GearKind::None`) alongside the other `*Net`
   components when player entities are spawned in `unreplicon-plugin`.
3. Extend `sync_gear_to_net` on the server to also write `PlayerGearKindNet` by querying `&GearKind` on each slot's
   entity.
4. Add `apply_gear_kind_net` client system (runs `Changed<PlayerGearKindNet>`, `Without<MainPlayer>`): for each slot
   that changed, despawn the old local gear entity (if any) and call `gear_registry.spawn()` to spawn a fresh one of the
   new kind, then update `PlayerGear` to point at the new entity.

This replaces the one-time population from `spawn_initial_gear` with a continuously authoritative server-driven gear
roster. Once this is in place, `spawn_initial_gear` for remote players (in `setup_replicated_player_visuals`) can stop
spawning gear entirely — it can insert an empty `PlayerGear` and let the first `Changed<PlayerGearKindNet>` populate it.

##### Gap C — Floor gear entities are not replicated

`drop_object` inserts `FloorItemCollidable` on a gear entity and positions it in the world, making it pickable by any
nearby player. Because gear entities are local, this entity does not exist on other clients. Players on remote nodes
cannot see, walk around, or pick up gear that a teammate placed on the floor.

This gap is separate from the gear-in-hands gap (B) and can be addressed independently.

**Proposed implementation — Step C1:**

1. Add `FloorGearBroadcast` server→all-clients message in `unreplicon-core/src/messages.rs`:

   ```rust
   pub struct FloorGearSpawnBroadcast  { pub kind: GearKind, pub pos: [f32; 3] }
   pub struct FloorGearDespawnBroadcast { pub pos: [f32; 3] }   // within ~0.1 unit radius
   ```

2. On the server, emit `FloorGearSpawnBroadcast` whenever `drop_object` places gear. Emit `FloorGearDespawnBroadcast`
   whenever `grab_object` picks it up.
3. Client system: on `FloorGearSpawnBroadcast`, call `gear_registry.spawn()` for the given kind and insert
   `FloorItemCollidable` + `Position` on the new entity. On `FloorGearDespawnBroadcast`, find the nearest floor gear
   entity within radius and despawn it.

> **Note:** A `Replicated` + `NetworkPosition` approach for floor gear is cleaner long-term but requires gear entities
> to carry `Replicated` at spawn time (or add/remove it dynamically), which is more invasive. The broadcast approach is
> simpler to implement now and preserves the existing architecture.

##### Gap C2 — Late-joining clients miss existing floor gear

Step C1 uses fire-and-forget broadcast messages. A client that connects (or reconnects) mid-game after gear has already
been dropped will not see any of those items. The server has no record of current floor state to replay.

**Proposed implementation — Step C2:**

1. Add a `FloorGearCache` resource to `unreplicon-core` (a
   `Vec<FloorGearEntry { kind: GearKind, pos: [f32; 3], direction: [f32; 3] }>`).
2. Maintain it in `broadcast_floor_gear_drop` (push entry) and `broadcast_floor_gear_pickup` (remove nearest entry by
   position, within 0.6 units).
3. In the `on_client_connected` observer (or a dedicated `ClientConnected` system on the server), iterate the cache and
   send each entry to the newly connected client via `SendMode::Direct(client_id)` as individual
   `FloorGearSpawnBroadcast` messages. The client's existing `apply_floor_gear_spawn` handles them identically to live
   broadcasts.

This step is **blocking** for the feature to be considered complete — players who join a mission in progress must see
the correct floor state or they will walk through invisible gear and be unable to pick it up.

##### Gap D — In-game grab/drop of world objects (held_item) not visible to remote players

`grab_object` also handles non-gear world objects (`Behavior.p.object.pickable` items such as furniture). It sets
`PlayerGear.held_item` and moves the object's `Position` each frame via `update_held_object_position`. Because the
object entity IS a map entity (present on all nodes via Tiled map load), its position is authoritative on the server but
join clients don't apply the position update — only the server's `update_held_object_position` fires (it runs on all
nodes but only modifies local `PlayerGear.held_item`, which is empty on clients).

Clients therefore see the carried object frozen at its original map position.

**Implemented — Step D1:** ✅ See the eleventh session entry in the session log below.

Extended `PlayerStateNet.held_object_bpos: Option<[i32; 3]>` (server-authoritative, replicated every frame). Server
`sync_held_object_to_net` reads `PlayerGear.held_item` and looks up `NetworkOriginalMapPosition`. Client
`apply_held_object_position` sets the map entity's `Position` to `NetworkPosition + z+0.25` each frame while carried.

✅ **Step D2 fixed** in the thirteenth session: on drop the client now snaps the object to `net_pos` (z without offset)
so the floating-0.25 artefact no longer appears. See the thirteenth session entry below for implementation details.

##### Implementation order and dependencies

| Step | Scope                           | Depends on | Status  | Priority |
| ---- | ------------------------------- | ---------- | ------- | -------- |
| A1   | Truck loadout client→server     | —          | ✅ Done | High     |
| B1   | `PlayerGearKindNet` replication | A1         | ✅ Done | High     |
| C1   | Floor gear broadcast            | B1         | ✅ Done | Medium   |
| C2   | Late-join floor gear sync       | C1         | ✅ Done | High     |
| D1   | World-object carry replication  | —          | ✅ Done | Medium   |
| D2   | World-object drop position sync | D1         | ✅ Done | Low      |

Steps A1 and B1 are tightly coupled: A1 fixes who the server trusts for loadout; B1 ensures every node always renders
the correct gear type. C1 and D1 are independent of each other but C1 benefits from B1 being done first (so that the
spawned floor entity type is consistent with what `apply_gear_kind_net` would spawn when the player picks it back up).

---

### Phase 4 — Ghost, World State, and Mission Flow ⚠️ Mostly complete, minor gaps remain

#### 4.1 Ghost replication

`setup_ghost_entities` adds `(Replicated, NetworkPosition, GhostStateNet)` to the server's ghost entity. ✅
`setup_replicated_ghost_visuals` in `unclassic-mode-plugin` watches for `Added<GhostStateNet>` and builds visuals for
the ghost on client nodes. ✅ `interpolate_ghost_position` lerps ghost position with a lerp speed of 15.0. ✅
`apply_ghost_state_net` propagates `GhostStateNet` changes to local `GhostSprite` + `GhostBehaviorDynamics`. ✅
`RepliconGhostSpawningActive` is inserted on `OnEnter(AppState::InGame)` and removed on exit. ✅

**However:** `RepliconGhostSpawningActive` is only checked in `unclassic-mode-plugin/src/plugin.rs` for
`spawn_joined_player` (the player spawning suppression), not for ghost spawning. The `classic_mode_orchestrator`
docstring at line 273 mentions that `RepliconGhostSpawningActive` should suppress the ghost-spawn block, but the
`run_if` guard on `setup_replicated_ghost_visuals` fires on both server and client regardless.

**Analysis (confirmed no duplicate):** `setup_replicated_ghost_visuals` uses the query filter `Without<GhostSprite>` on
top of `Added<GhostStateNet>`. On the listen-server host, the ghost entity is spawned with `GhostSprite` present (by the
classic mode orchestrator) _before_ `setup_ghost_entities` adds `GhostStateNet` to it. When `Added<GhostStateNet>`
fires, the entity already carries `GhostSprite`, so `Without<GhostSprite>` excludes it and no duplicate is created. The
concern is resolved — no code change needed.

#### 4.2a Room interaction sync — ✅ Fixed (all paths covered)

Door / switch state changes were not propagated between clients. Analysis:

- All nodes (host, join client, offline) fire `ExecuteInteractionEvent` locally when the local player interacts.
  `interaction_event_handler` processes it on the node that fired it.
- There was no mechanism to propagate join-client interactions to the server or to other join clients.
- `InteractionRequestMessage` was registered but never sent by any system.

**Fix applied (join-client interactions, fourth gap-filling session):**

1. `RemoteInteractionBroadcast { position, ietype, force_tuid }` added to `unreplicon-core/messages.rs` as a
   server-to-client message type.
2. `player_interaction_system` (`unplayer-plugin`) now also writes `InteractionRequestMessage` to the server when
   `!cli.is_authority()` (join clients only), using the interactive entity's board position.
3. `handle_interaction_request` (`unreplicon-plugin`) now broadcasts `RemoteInteractionBroadcast` with
   `SendMode::BroadcastExcept(sender_client_id)` after processing the request, so other join clients receive the
   interaction.
4. New `apply_remote_interaction` system reads `RemoteInteractionBroadcast` on join clients and fires
   `ExecuteInteractionEvent` locally — identical to the local-player interaction path.

**Fix applied (host-player interactions, fifth gap-filling session):**

1. `HostInteractionOccurred { position, ietype, force_tuid }` added to `unreplicon-core/messages.rs` as a local-only
   (non-network) bridge event.
2. `player_interaction_system` now also writes `HostInteractionOccurred` when `cli.is_authority()`, using the same board
   position.
3. New `broadcast_host_interactions` system (server-only) reads `HostInteractionOccurred` and broadcasts
   `RemoteInteractionBroadcast` via `SendMode::Broadcast` to all connected join clients. The host already applied the
   interaction locally, and `SendMode::Broadcast` targets only connected clients (not the server itself), so no
   double-toggle occurs.

**Fix applied (ghost AI interactions, sixth gap-filling session):**

1. `ghost_interaction_execution_system` (`unghost-plugin`) now also accepts
   `mut ev_host_interact: MessageWriter<HostInteractionOccurred>` and passes it to the four sub-functions that call
   `ExecuteInteractionEvent` (Toggle, DoorSlam, DoorCreak, TripBreaker).
2. Each sub-function emits `HostInteractionOccurred` on success, immediately after the `ExecuteInteractionEvent` write.
   The existing `broadcast_host_interactions` server system broadcasts the change to all join clients with no further
   changes needed.

**Remaining gap:** None — all interaction paths (join-client, host-player, ghost AI) are now propagated.

`EvidenceFoundNet` and `GhostGuess` sync is implemented cleanly:

- Server: `sync_evidence_to_net` copies `GhostGuess` → `EvidenceFoundNet` every frame. ✅
- Client: `apply_evidence_net` reads `Changed<EvidenceFoundNet>` and updates local `GhostGuess`. ✅
- Client messages `RequestJournalEvidenceToggle` and `RequestJournalGhostToggle` handled on server. ✅

#### 4.4 Mission events and summary — ✅ Fixed

Both halves of the mission result pipeline are now implemented in `ghost.rs`:

- **Server:** `sync_mission_result_to_net` runs in `AppState::Summary` with `ServerState::Running`. When `SummaryData`
  is marked changed (i.e., after `calculate_rewards_and_grades` populates it), all fields are written to
  `MissionResultNet`, `ready` is set to `true`, and `ServerAppState` is updated to `AppState::Summary` so the
  `follow_server_app_state` system in `bridge.rs` drives clients to the summary screen.
- **Client:** `apply_mission_result_net` detects `Changed<MissionResultNet>` where `ready == true`, populates the local
  `SummaryData` resource from all net fields (using `Grade::from` for grade string conversion), and calls
  `next_state.set(AppState::Summary)`.

#### 4.4a Player death replication — ✅ Fixed

Two targeted fixes were applied:

1. **`detect_and_apply_death` authority gate removed** (`unplayer-plugin/src/systems/sanityhealth.rs`): the system no
   longer requires `.run_if(is_authority)`. It is safe without the gate because the query includes
   `Without<PlayerSpectating>` (prevents double-fire per entity) and remote-player entities on the server have no
   `PlayerSprite` component (so the query skips them). Join clients now correctly enter spectator mode and accumulate
   death stats when their own `PlayerSprite` health reaches zero.

2. **`apply_player_state_net` mirrors `is_spectating`** (`unreplicon-plugin/src/systems/players.rs`): the system now
   queries `Has<PlayerSpectating>` and inserts the `PlayerSpectating` marker component on remote-player entities whose
   `PlayerStateNet.is_spectating` becomes `true`. This propagates the authoritative death state to all connected clients
   so remote players are visually shown as dead/spectating.

`PlayerDiedEvent` is defined in `messages.rs` and is consumed locally; the authoritative death event is now mirrored
through the `PlayerStateNet.is_spectating` replicated field.

#### 4.5 `TransientEvent` → `SpawnParticleNetEvent`

`SpawnParticleNetEvent` is defined, registered as a server→client message, and handled in `ghost.rs`. The
`handle_spawn_particle` system spawns the local smoke particle entity from the received position. ✅

**Sage smoke particles on the authority node — ✅ Fixed** (`ungearitems-plugin/src/components/sage.rs`): `update_sage`
(authority) previously only broadcast `SpawnParticleNetEvent` — it never spawned `SageSmokeParticle` entities locally.
Because `handle_spawn_particle` in `ghost.rs` runs only under `not(in_state(ServerState::Running))`, the
host/listen-server and offline-mode player never had smoke particles, which meant `sage_smoke_system`'s ghost-calming
logic (`ghost.rage -=`, `calm_time_secs +=`) never fired on the authority node. The fix adds a direct
`commands.spawn(SageSmokeParticle)` call in `update_sage` immediately after the broadcast, using the same component set
as `handle_spawn_particle`. Join clients continue to receive the broadcast and spawn locally as before; the authority
node now spawns independently.

**`exclude_player` field** — The original plan required this field to prevent double-spawn for the triggering player.
Architectural analysis shows this is not a live bug in the current design: the server never receives its own broadcast,
and Join clients do not spawn particles locally (only from the event). The concern is therefore moot. The field has been
omitted intentionally.

---

### Phase 5 — Purge ✅ Complete

- `crates/unnet-core/` and `crates/unnet-plugin/` do not exist.
- Neither crate appears in workspace `members` or `[workspace.dependencies]`.
- Zero Rust-source references to `unnet_core::` or `unnet_plugin::`.

---

## Notices (Non-blocking but worth tracking)

### Notice 1 — ✅ `ToBeDespawned` dead code removed

`ToBeDespawned { in_frames: usize }` has been deleted from `unreplicon-core/src/network_id.rs`. The type was the old
`unnet-core` delayed-despawn mechanism and was referenced nowhere. In the Replicon model, `commands.entity().despawn()`
on the server propagates to clients automatically.

### Notice 2 — Transport is `Unsecure` end-to-end

Both server and client use `*Authentication::Unsecure`. The plan's intent was to eventually use
`ServerAuthentication::Secure` with a per-session key. Application-level auth via `auth.rs` provides the meaningful
protection, but the UDP stream is unencrypted in transit. This is an acceptable temporary state but should be documented
as a known gap in the security model and tracked as an open item.

### Notice 3 — `NetworkId(u64)` was not replaced by Replicon's `ClientId`

Phase 2 states: _"Replace `NetworkId` as the player identity with Replicon's `ClientId`."_ Instead, `NetworkId(u64)` was
carried forward into `unreplicon-core/src/network_id.rs`, and a mapping layer converts from Replicon's
`RepliconNetworkId` to this game-level `NetworkId` in `bridge.rs` and `lobby.rs`. While functionally correct today, the
`NetworkId` abstraction now lives in two places (the game world and the transport wrapper), and any future systems that
need Replicon's actual `ClientId` for targeted sends, visibility masks, or client-authority features must bridge through
this layer. The migration is half-complete.

### Notice 4 — ✅ All `unreplicon-core` path deps migrated to workspace

All 11 crates that referenced `unreplicon-core = { path = "../unreplicon-core" }` have been updated to
`unreplicon-core = { workspace = true }`. The workspace root already declared `unreplicon-core` correctly in
`[workspace.dependencies]`; the dependents simply were not consuming it that way. Crates updated: `unreplicon-plugin`,
`unhub-plugin`, `unlobby-plugin`, `untruck-plugin`, `unrender-std`, `unplayer-plugin`, `unplayer-core`,
`unmission-plugin`, `ungearitems-plugin`, `unghost-plugin`, `unengine-plugin`, `unclassic-mode-plugin`.

### Notice 5 — FIXME and open TODO comments

| File                                          | Location     | Text                                                                                                   |
| --------------------------------------------- | ------------ | ------------------------------------------------------------------------------------------------------ |
| `unreplicon-plugin/src/systems/bridge.rs`     | Line 58      | `FIXME Phase 3: consider a cleaner split between server-authoritative mutation and client-side bridge` |
| `unreplicon-plugin/src/systems/connection.rs` | Lines 54, 87 | `TODO Phase 1.4: switch to Secure with a per-session private key`                                      |
| `unreplicon-plugin/src/systems/connection.rs` | Line 84      | `TODO Phase 1.4: use the installation_id from CliOptions as the stable client_id`                      |

These are honest markers left by the executing agent and are not violations per se. They should be tracked in a
follow-up phase or issue.

---

## Summary Table

| Area                                              | Phase | Status                                   | Severity |
| ------------------------------------------------- | ----- | ---------------------------------------- | -------- |
| Quarantine (`unnet-*` removal)                    | 0     | ✅ Complete                              | —        |
| Plugin / crate structure                          | 1     | ✅ Correct                               | —        |
| UDP transport init (`Offline`/`Host`/`Join`)      | 1     | ✅ Complete                              | —        |
| Ticket auth (server-side, postcard+HMAC)          | 1     | ✅ Complete                              | —        |
| Ticket packing (client-side, postcard+HMAC)       | 1     | ✅ Complete                              | —        |
| ProcMan stdin/stdout adapter                      | 1     | ✅ Complete                              | —        |
| Transport uses `Unsecure` auth                    | 1     | ⚠️ Open TODO                             | Info     |
| Lobby entity + Replicon replication               | 2     | ✅ Complete                              | —        |
| `LobbyData` bridge                                | 2     | ✅ Complete                              | —        |
| `RoomOwner` gate for lobby UI                     | 2     | ✅ Complete                              | —        |
| `ServerAppState` → client `NextState` system      | 2     | ✅ Complete (all transitions wired)      | —        |
| Player entity replication at InGame enter         | 3     | ✅ Complete (all 6 `*Net` inserted)      | —        |
| `NetworkPosition` interpolation                   | 3     | ✅ Complete                              | —        |
| Local player → server position send               | 3     | ✅ Complete                              | —        |
| Host player → `NetworkPosition` sync              | 3     | ✅ Complete                              | —        |
| `setup_replicated_player_visuals`                 | 3     | ✅ Complete                              | —        |
| `RepliconPlayerSpawningActive` suppression        | 3     | ✅ Complete                              | —        |
| `PlayerStateNet` → `PlayerSprite` apply           | 3     | ✅ Fixed (`apply_player_state_net`)      | —        |
| Gear `*Net` server write (`sync_gear_to_net`)     | 3     | ✅ Complete                              | —        |
| Gear `*Net` client apply — Flashlight, Sage, RF   | 3     | ✅ Fixed                                 | —        |
| Gear `*Net` client apply — Thermometer, EMF, SB   | 3     | ✅ Fixed (thermometer/emf/sb)            | —        |
| Interaction request messages                      | 3     | ✅ Complete                              | —        |
| Room interaction replication (`DoorOpen` etc.)    | 3     | ✅ Fixed (all paths covered)             | —        |
| Truck loadout / grab-drop messages                | 3     | ❌ Not implemented                       | High     |
| Ghost replication + interpolation                 | 4     | ✅ Complete                              | —        |
| `apply_ghost_state_net` observer                  | 4     | ✅ Complete                              | —        |
| `setup_replicated_ghost_visuals`                  | 4     | ✅ Complete                              | —        |
| Possible duplicate ghost visuals on listen server | 4     | ✅ Not a bug (see §4.1)                  | —        |
| `MovableStateNet` / movable objects               | 4     | ✅ Fixed (tween broadcast)               | —        |
| Evidence / journal sync                           | 4     | ✅ Complete                              | —        |
| `MissionResultNet` → `SummaryData` bridge         | 4     | ✅ Fixed (both server write + client)    | —        |
| Player death server→client replication            | 4     | ✅ Fixed (spectating state + death gate) | —        |
| `SpawnParticleNetEvent` (smoke)                   | 4     | ✅ Complete                              | —        |
| Sage smoke on authority node (offline/host)       | 4     | ✅ Fixed (direct spawn in update_sage)   | —        |
| `exclude_player` field for particle events        | 4     | ✅ Moot — architecture prevents double   | —        |
| `unnet-*` final purge                             | 5     | ✅ Complete                              | —        |
| `ToBeDespawned` dead code                         | —     | ✅ Removed                               | Info     |
| `NetworkId` not migrated to `ClientId`            | 2     | ⚠️ Pragmatic divergence                  | Info     |
| `unlobby-plugin` path vs workspace dep            | —     | ✅ Fixed (all 12 crates updated)         | Info     |

### High-severity gaps (functional blockers for multiplayer)

**Fixed since initial audit:**

- ✅ `MissionResultNet` → `SummaryData` bridge: both server write and client apply are implemented.
- ✅ `PlayerStateNet` → `PlayerSprite` apply: remote player health/sanity now written on clients (was an undiscovered
  gap; also fixes the `update_time` alive-detection dead-zone in multiplayer).
- ✅ Gear `*Net` entity insertion and server-side sync: all 6 types are inserted at spawn and written by
  `sync_gear_to_net` every frame.
- ✅ Client apply for Flashlight, SageBundle, RepellentFlask: `apply_flashlight_net`, `apply_sage_net`,
  `apply_repellent_net` are present and wired.

**Fixed in second gap-filling session:**

- ✅ `ServerAppState` is now written for all transitions: `set_server_state_ingame` handles `→ InGame`, and
  `setup_lobby_entity` update-in-place handles `Summary → Lobby`. The duplicate entity bug is also resolved.
- ✅ `apply_thermometer_net`, `apply_emf_net`, `apply_spiritbox_net` are all implemented and wired in `players.rs`.

**Fixed in third gap-filling session:**

- ✅ Player death replication: `detect_and_apply_death` no longer requires `is_authority`; `apply_player_state_net` now
  mirrors `PlayerStateNet.is_spectating` by inserting `PlayerSpectating` on remote entities.
- ✅ Sage smoke particles on authority node: `update_sage` now spawns `SageSmokeParticle` entities directly (with the
  full component set) so ghost-calming works in offline and listen-server modes.
- ✅ `exclude_player` concern closed: architectural analysis confirms no double-spawn occurs; no field needed.

**Fixed in fourth gap-filling session:**

- ✅ Ghost duplicate visuals on listen-server: confirmed not a real bug. `setup_replicated_ghost_visuals` uses
  `Without<GhostSprite>` which already excludes the server's ghost entity (it has `GhostSprite` before `GhostStateNet`
  is added).
- ✅ Room interaction replication (join-client path): `player_interaction_system` now sends `InteractionRequestMessage`
  from join clients. `handle_interaction_request` broadcasts `RemoteInteractionBroadcast` (via `BroadcastExcept`) to
  other clients. New `apply_remote_interaction` system fires `ExecuteInteractionEvent` locally on receiving clients.

**Fixed in fifth gap-filling session:**

- ✅ Room interaction replication (host-player path): `HostInteractionOccurred` local bridge event added.
  `player_interaction_system` writes it when `cli.is_authority()`. New `broadcast_host_interactions` server system reads
  it and broadcasts `RemoteInteractionBroadcast` via `SendMode::Broadcast` to all join clients. The host applies locally
  with no double-toggle.

**Fixed in sixth gap-filling session:**

- ✅ Room interaction replication (ghost AI path): `ghost_interaction_execution_system` in `unghost-plugin` now also
  writes `HostInteractionOccurred` alongside every `ExecuteInteractionEvent` call in the Toggle, DoorSlam, DoorCreak,
  and TripBreaker interaction sub-functions. The existing `broadcast_host_interactions` server system in
  `unreplicon-plugin` handles the broadcast to join clients with no additional changes needed.

**Fixed in seventh gap-filling session:**

- ✅ Movable object position replication: ghost-induced Throw/Nudge/HauntedMove animations now propagate to join
  clients. `HostMovableMotionEvent` and `MovableMotionBroadcast` message types added. `watch_tween_insertions` server
  system (using `Added<Tween>`) emits `HostMovableMotionEvent` with animation parameters. New `broadcast_movable_motion`
  system in `unreplicon-plugin` forwards this to all clients. New `apply_remote_movable_motion` system in
  `unghost-plugin` reconstructs and inserts the `Tween` locally so clients play back the same animation.

**Fixed in eighth gap-filling session:**

- ✅ Truck loadout Step A1 — join clients now send `TruckLoadoutMessage` to the server when the user clicks a loadout
  button. New `TruckLoadoutAction` enum (AddGear / ClearHand / ClearInventorySlot) and `TruckLoadoutMessage` type added
  in `unreplicon-core`. `handle_truck_loadout_message` server system in `unreplicon-plugin/systems/players.rs` receives
  the message, finds the player's `PlayerGear`, and applies the change authoritatively using `GearSpawnerRegistry`. Join
  clients apply the change optimistically (local-first) for immediate UI feedback before the server echo arrives.

**Fixed in ninth gap-filling session:**

- ✅ Truck loadout Step B1 — added `PlayerGearKindNet` replicated component to `unreplicon-core/src/net_components.rs`.
  Stores `left_hand: GearKind`, `right_hand: GearKind`, `inventory: [GearKind; 2]` (two fixed slots matching the in-game
  limit). Inserted alongside other `*Net` components at player spawn (host and remote). Extended `sync_gear_to_net` to
  read `GearKind` from each slot entity and write it to `PlayerGearKindNet` every frame. Added `apply_gear_kind_net`
  client system (`Changed<PlayerGearKindNet>`, `Without<MainPlayer>`) that despawns stale local gear entities and spawns
  correct-kind replacements via `GearSpawnerRegistry` when the server roster changes.

**Fixed in tenth gap-filling session:**

- ✅ Floor gear broadcast Step C1 — gear items dropped onto the floor by the server are now visible to all join clients.
  Four new types added in `unreplicon-core/src/messages.rs`: `HostFloorGearDroppedEvent` and
  `HostFloorGearPickedUpEvent` (local-only bridge events) plus `FloorGearSpawnBroadcast` and `FloorGearDespawnBroadcast`
  (server→client network messages, `Channel::Ordered`).
  - `drop_object` in `unplayer-plugin` now emits `HostFloorGearDroppedEvent` (kind + position + direction) after
    inserting `FloorItemCollidable` on the dropped gear entity. A `q_gear_kind: Query<&GearKind>` parameter was added to
    retrieve the gear type from the just-dropped entity.
  - `grab_object` now stores the pickable's `&Position` in the `closest` tuple and emits `HostFloorGearPickedUpEvent`
    (position) once `FloorItemCollidable` is removed from the grabbed entity.
  - `broadcast_floor_gear_drop` (server system) relays `HostFloorGearDroppedEvent` → `FloorGearSpawnBroadcast` via
    `SendMode::Broadcast`.
  - `broadcast_floor_gear_pickup` (server system) relays `HostFloorGearPickedUpEvent` → `FloorGearDespawnBroadcast` via
    `SendMode::Broadcast`.
  - `apply_floor_gear_spawn` (client system) calls `gear_registry.spawn()`, then inserts
    `(Position, FloorItemCollidable, EquipmentPosition::Deployed, DeployedGear { direction })` so that
    `update_deployed_gear_sprites` in `ungear-plugin` automatically adds the visual components.
  - `apply_floor_gear_despawn` (client system) finds the nearest `FloorItemCollidable` entity within 0.6 units and
    despawns it.
  - `unbehavior` added as a path dependency to `unreplicon-plugin/Cargo.toml` (needed for `FloorItemCollidable`).
  - ✅ **Gap C2 fixed** in the twelfth session: `FloorGearCache` resource + on-connect replay via
    `send_floor_gear_to_new_client` observer.

**Fixed in eleventh gap-filling session:**

- ✅ World-object carry replication Step D1 — remote clients now see non-gear pickable objects (furniture, etc.) move
  with the player who is carrying them.
  - `PlayerStateNet` in `unreplicon-core/src/net_components.rs` gained a new `held_object_bpos: Option<[i32; 3]>` field
    storing the board-space original spawn position of the carried world object (`None` when nothing is held).
  - New `sync_held_object_to_net` server system (runs alongside `sync_player_state_to_net`) reads each player's
    `PlayerGear.held_item` and looks up its `NetworkOriginalMapPosition`; writes the result into `PlayerStateNet` every
    frame. Gear entities do not have `NetworkOriginalMapPosition` so they correctly produce `None`.
  - New `apply_held_object_position` client system reads `PlayerStateNet.held_object_bpos` for each non-`MainPlayer`
    remote player. When `Some(bpos)`, it finds the map entity with that original board position and sets its `Position`
    to the player's replicated `NetworkPosition` with a +0.25 z-offset (matching `update_held_object_position` on the
    server). When `None`, the entity stays at its last position (approximately the drop location).
  - ⚠️ **Known cosmetic gap:** on drop, the map object's client-side position is left at `net_pos + 0.25z` rather than
    the exact floor position. The +0.25 z-offset lingers until the ghost or another player interacts with the object or
    until a tween resets its position. This is a cosmetic artefact only; gameplay (collidability, interaction) is
    unaffected because those checks use the server's authoritative `Position`.

**Fixed in twelfth gap-filling session:**

- ✅ Late-join floor gear sync Step C2 — clients that connect mid-mission now receive all gear items currently on the
  floor, matching the state of players already in the game.
  - `FloorGearEntry` struct and `FloorGearCache` resource added to `unreplicon-core/src/resources.rs`. `FloorGearEntry`
    stores `kind: GearKind`, `pos: [f32; 3]`, `direction: [f32; 3]`. `FloorGearCache` is a `Vec<FloorGearEntry>`
    resource that is server-only and defaults to empty.
  - `broadcast_floor_gear_drop` (server) now pushes a `FloorGearEntry` into the cache before broadcasting.
  - `broadcast_floor_gear_pickup` (server) now removes the nearest cache entry within 0.6 units after broadcasting the
    despawn. Uses a simple iteration rather than a comparator closure for clarity.
  - New `floor_gear_pos_dist` private helper computes Euclidean distance between two `[f32; 3]` positions.
  - New `reset_floor_gear_cache` server system registered on both `OnEnter(AppState::InGame)` and
    `OnExit(AppState::InGame)` — clears the cache so no stale items leak between missions.
  - New `send_floor_gear_to_new_client` observer triggered by `On<Insert, ConnectedClient>` (server-only; fires when
    bevy_replicon inserts the `ConnectedClient` component on a newly connected client). If the cache is non-empty,
    iterates it and sends each entry as a `FloorGearSpawnBroadcast` via `SendMode::Direct(ClientId::Client(entity))`.
    Clients handle these messages with the existing `apply_floor_gear_spawn` system — no client-side changes required.
  - `app_setup` in `players.rs` registers `init_resource::<FloorGearCache>()`, both reset systems, and the new observer.

**Fixed in thirteenth gap-filling session:**

- ✅ World-object drop position sync Step D2 — the carry-offset artefact on remote clients is resolved. When a remote
  player drops a world object, the client now snaps the object to floor level instead of leaving it floating at
  `net_pos.z + 0.25`.
  - `apply_held_object_position` in `players.rs` gained a `Local<HashMap<Entity, Entity>>` parameter (`held_cache`)
    mapping each remote player `Entity` to the map entity it is currently holding.
  - `q_remote` query now includes `Entity` so the player entity is available as a cache key.
  - In the `Some(bpos_arr)` branch, the found map entity is stored in `held_cache` before setting the carry position.
  - In the `else` (None) branch, if `held_cache` contains an entry for this player, the held entity's position is
    snapped to `net_pos.x`, `net_pos.y`, `net_pos.z` (no +0.25 offset) and the cache entry is removed.
  - `use std::collections::HashMap;` added to the file's imports.

**All steps complete.** All gear-replication gaps (A1, B1, C1, C2, D1, D2) are resolved. No items remain open.
