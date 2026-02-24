# 05. Post-Execution Audit: `unnet → bevy_replicon` Migration

- **Date:** 2026-02-24
- **Auditor:** Independent review of `dev-deavid` branch
- **Scope:** All phases (0–5) defined in `04_phased_execution_plan.md`
- **Build status:** Zero Rust compile errors. The workspace builds clean.

---

## Executive Summary

All five structural phases were executed. The old `unnet-core` and `unnet-plugin` crates are gone, the
`unreplicon-plugin` framework skeleton is sound, and singleplayer is unambiguously preserved. Transport, authentication,
lobby state, player movement, ghost replication, and journal sync are all present and broadly follow the plan's intent.

However, **three functional observers are missing** that were explicitly required by the plan. These gaps mean that
joined clients cannot follow the server's state machine, gear state silently fails to propagate to local components, and
the mission summary screen cannot populate for non-host players. The implementation is Phase 3-complete in structure but
Phase 2-and-a-half in actual runtime correctness.

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
- `Join`: creates a `NetcodeClientTransport`, packs the JWT ticket into the 256-byte `user_data` field. ✅

#### 1.4 Ticket-based authentication

`auth.rs` uses an observer on `RenetServerEvent::ClientConnected`. Before any game state is allocated, it:

1. Rejects connections when `RoomAuth` has no room assigned (server is idle). ✅
2. Extracts the null-terminated JWT string from the `user_data` field. ✅
3. Validates the JWT signature (HMAC-SHA256), expiry, and `room_code` claim. ✅
4. Disconnects non-conformant clients immediately. ✅

The JWT validation in `validate_ticket()` correctly uses `jsonwebtoken` with `validate_exp = true`.

#### 1.5 ProcMan adapter

`procman.rs` spawns a stdin reader thread and a stdout writer thread and communicates via `crossbeam_channel`. It
handles all four `ProcManToDedicated` variants:

- `AssignRoom` populates `RoomAuth`. ✅
- `RenameRoom` updates `room_code` (ticket secret is intentionally retained). ✅
- `WipeRoom` clears `RoomAuth`, which causes auth to reject all subsequent connections. ✅
- `Shutdown` writes `AppExit::Success` via `MessageWriter`. ✅

The initial `Ready { port }` signal is sent on startup so `unprocman` knows the server is up.

**⚠️ Open TODO:** `ServerAuthentication::Unsecure` and `ClientAuthentication::Unsecure` are used. The plan called for
switching to `Secure` with a per-session key distributed by the Hub JWT in Phase 1.4. This is explicitly marked
`// TODO Phase 1.4` in `connection.rs`. The semantic application-level security from `auth.rs` is in place, but
transport-layer encryption is absent. The `client_id` is also derived from `SystemTime::now()` rather than from
`installation_id`; a second `TODO Phase 1.4` comment notes this.

---

### Phase 2 — Lobby and Session Bootstrap ⚠️ Mostly complete, one observer missing

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

#### 2.3 Map loading on client — ❌ `ServerAppState` observer missing

The plan requires:

> Clients use an Observer (`OnAdd<ServerAppState>` / `OnChanged<ServerAppState>`) to drive their local
> `NextState<AppState>`. Clients never transition state on their own authority.

`ServerAppState` is replicated, but **no observer exists that reads a changed `ServerAppState` and calls
`next_state.set(AppState::...)`**. The only state transition for clients happens inside `on_selected_mission_added` in
`bridge.rs`, which specifically sets `AppState::Loading` when `SelectedMission` is inserted. This partially covers the
Lobby→Loading→InGame transition but does not cover any other state transitions (e.g., InGame→Summary, InGame→Lobby for a
mission restart, or a disconnect/return to MainMenu triggered by the server).

Without this observer, clients that lose state sync with the server (e.g., after a mission ends and the server returns
to Lobby) will be stuck in stale states. This is a **functional blocker** for any scenario beyond the happy path of a
single mission.

#### 2.4 `LobbyData` bridge

`bridge_lobby_info_system` reads every changed `LobbyInfo` and updates `LobbyData.players`, `selected_map`, and
`selected_difficulty`. The resources `LobbyData`, `LocalPlayer`, `CurrentMapSeed`, `HostGone`, and `RoomIdentification`
are all initialised in `lobby.rs::app_setup`. ✅

A `FIXME Phase 3` comment in `bridge.rs` flags a known rough edge around server-vs-client mutation responsibility. Not a
functional bug today but flagged for future cleanup.

---

### Phase 3 — Players, Movement, and Gear ⚠️ Structurally present, gear bridge missing

#### 3.1 Player entity replication

`setup_mission_players` on `OnEnter(AppState::InGame)` (server only):

- Adds `(Replicated, NetworkPosition, PlayerStateNet, PlayerNetInfo)` to the existing host player entity. ✅
- Spawns bare `(Replicated, NetworkPosition, PlayerStateNet, PlayerNetInfo)` entities for each remote client from
  `LobbyInfo`. ✅
- Inserts `RepliconPlayerSpawningActive` resource so `unclassic-mode-plugin`'s `spawn_joined_player` is suppressed. ✅

`setup_replicated_player_visuals` in `unclassic-mode-plugin` (runs on both server and client) watches for
`Added<PlayerNetInfo>` and adds a full visual + gear kit to newly arrived player entities. ✅ This is a clean
integration point.

#### 3.2 `NetworkPosition` and interpolation

`interpolate_remote_players` lerps remote entities' `Position` toward `NetworkPosition` every frame with a lerp speed of
15.0, excluding `MainPlayer` entities. ✅

The local player's position is sent via `send_local_player_position` (client-only, runs in `AppState::InGame`, excluded
when `ServerState::Running`). ✅ The host player's position is synced directly by `sync_player_state_to_net` on the
server side. ✅

#### 3.3 Gear component splitting — ❌ Client-side `*Net` → local component observers missing

All six gear `*Net` types are defined in `unreplicon-core/src/net_components.rs` and registered via
`app.replicate::<T>()` in `players.rs::app_setup`:

- `FlashlightNet` ✅ defined, ✅ registered
- `ThermometerNet` ✅ defined, ✅ registered
- `EMFMeterNet` ✅ defined, ✅ registered
- `SpiritBoxNet` ✅ defined, ✅ registered
- `SageBundleNet` ✅ defined, ✅ registered
- `RepellentFlaskNet` ✅ defined, ✅ registered

**However**, the plan requires (Phase 3.3):

> Use `OnAdd<*Net>` and `OnChanged<*Net>` Observers on the client to drive the local component.

The doc comment on `FlashlightNet` in `net_components.rs` even promises them: _"via `On<Insert, FlashlightNet>` and
`On<Changed, FlashlightNet>` observers (see `unreplicon-plugin`)"_.

**None of these observers exist.** Neither in `unreplicon-plugin` nor in any gear plugin. The `*Net` components are
replicated and land on entities, but that data is never applied back to `Flashlight`, `Thermometer`, etc.

This has concrete consequences:

- Remote players' flashlights will never turn on or off visually for other clients.
- Sage bundle burn state will not sync.
- Spirit box charge will not sync.
- The repellent flask quantity will not sync.

This also violates the explicit "No deferred cleanup" principle from the plan's core execution rules:

> If a phase introduces a component split (e.g., `FlashlightNet`), the local `Flashlight` component must be updated in
> the same phase. No stubs, no `#[allow(dead_code)]` left behind.

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
plugin systems. `MovableStateNet` (Phase 4.2) is also absent. Given that gear is provisioned at spawn time by
`setup_replicated_player_visuals` and `spawn_initial_gear`, this may be partially masked in singleplayer, but
multiplayer gear pickup / truck inventory changes will not sync.

---

### Phase 4 — Ghost, World State, and Mission Flow ⚠️ Mostly complete, mission result bridge missing

#### 4.1 Ghost replication

`setup_ghost_entities` adds `(Replicated, NetworkPosition, GhostStateNet)` to the server's ghost entity. ✅
`setup_replicated_ghost_visuals` in `unclassic-mode-plugin` watches for `Added<GhostStateNet>` and builds visuals for
the ghost on client nodes. ✅ `interpolate_ghost_position` lerps ghost position with a lerp speed of 15.0. ✅
`apply_ghost_state_net` propagates `GhostStateNet` changes to local `GhostSprite` + `GhostBehaviorDynamics`. ✅
`RepliconGhostSpawningActive` is inserted on `OnEnter(AppState::InGame)` and removed on exit. ✅

**However:** `RepliconGhostSpawningActive` is only checked in `unclassic-mode-plugin/src/plugin.rs` for
`spawn_joined_player` (the player spawning suppression), not for ghost spawning. The `classic_mode_orchestrator`
docstring at line 273 mentions that `RepliconGhostSpawningActive` should suppress the ghost-spawn block, but the
`run_if` guard on `setup_replicated_ghost_visuals` fires on both server and client regardless. Functionally this likely
causes duplicate ghost visuals on the server-side listen-server host. This should be verified.

#### 4.2 Movable objects

`MovableStateNet` is absent. The plan (Phase 4.2) required replicating this for any `Movable`-tagged entity. Not
implemented.

#### 4.3 Evidence and journal

`EvidenceFoundNet` and `GhostGuess` sync is implemented cleanly:

- Server: `sync_evidence_to_net` copies `GhostGuess` → `EvidenceFoundNet` every frame. ✅
- Client: `apply_evidence_net` reads `Changed<EvidenceFoundNet>` and updates local `GhostGuess`. ✅
- Client messages `RequestJournalEvidenceToggle` and `RequestJournalGhostToggle` handled on server. ✅

#### 4.4 Mission events and summary — ❌ `MissionResultNet` → `SummaryData` bridge missing

`MissionResultNet` is defined in `net_components.rs` and placed on the `MissionGoalEntity`. It has a `ready: bool` field
that signals when the server has populated the result. However:

**No observer or system reads `MissionResultNet` and populates the local `SummaryData` resource on client nodes.**

The doc comment in `net_components.rs` says: _"Clients observe `On<Add, MissionResultNet>` to populate their local
`SummaryData`"_, but this observer does not exist anywhere in the codebase. For a joined client, the mission summary
screen will be empty (it will read from an uninitialized `SummaryData` rather than from the server's result).

`PlayerDiedEvent` is defined in `messages.rs` and is consumed in `unplayer-plugin/src/systems/sanityhealth.rs` via a
local `add_message`. However, this is a local message only; no server-to-client replication of player death events is
wired through `unreplicon-plugin`.

#### 4.5 `TransientEvent` → `SpawnParticleNetEvent`

`SpawnParticleNetEvent` is defined, registered as a server→client message, and handled in `ghost.rs`. The
`handle_spawn_particle` system spawns the local smoke particle entity from the received position. ✅

**However**, the plan (Phase 4.5) and Technical Appendix (§4) specifically require an `exclude_player: Option<ClientId>`
field to prevent the originating player from hearing a sound or seeing a particle effect twice. This field is absent
from `SpawnParticleNetEvent`. The server will broadcast the smoke particle to all clients including the one who
triggered the sage bundle, causing a visible double-spawn for that player.

---

### Phase 5 — Purge ✅ Complete

- `crates/unnet-core/` and `crates/unnet-plugin/` do not exist.
- Neither crate appears in workspace `members` or `[workspace.dependencies]`.
- Zero Rust-source references to `unnet_core::` or `unnet_plugin::`.

---

## Notices (Non-blocking but worth tracking)

### Notice 1 — `ToBeDespawned` is dead code

`unreplicon-core/src/network_id.rs` defines `ToBeDespawned { in_frames: usize }`. It is referenced nowhere in the
codebase. It was the old `unnet-core` delayed-despawn mechanism. In the Replicon model, `commands.entity().despawn()` on
the server propagates to clients automatically; this type is obsolete. It should be deleted to avoid an eventual
dead-code warning.

### Notice 2 — Transport is `Unsecure` end-to-end

Both server and client use `*Authentication::Unsecure`. The plan's intent was to eventually use
`ServerAuthentication::Secure` with a per-session key derived from the Hub JWT. Application-level auth via `auth.rs`
provides the meaningful protection, but the UDP stream is unencrypted in transit. This is an acceptable temporary state
but should be documented as a known gap in the security model and tracked as an open item.

### Notice 3 — `NetworkId(u64)` was not replaced by Replicon's `ClientId`

Phase 2 states: _"Replace `NetworkId` as the player identity with Replicon's `ClientId`."_ Instead, `NetworkId(u64)` was
carried forward into `unreplicon-core/src/network_id.rs`, and a mapping layer converts from Replicon's
`RepliconNetworkId` to this game-level `NetworkId` in `bridge.rs` and `lobby.rs`. While functionally correct today, the
`NetworkId` abstraction now lives in two places (the game world and the transport wrapper), and any future systems that
need Replicon's actual `ClientId` for targeted sends, visibility masks, or client-authority features must bridge through
this layer. The migration is half-complete.

### Notice 4 — `unlobby-plugin` uses path-based `unreplicon-core` dependency

`crates/unlobby-plugin/Cargo.toml` uses `unreplicon-core = { path = "../unreplicon-core" }` rather than
`unreplicon-core = { workspace = true }`. This diverges from workspace convention. The project pattern is to declare
intra-workspace crates in the root `[workspace.dependencies]` table and then reference them via `workspace = true`.

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

| Area                                              | Phase | Status                                  | Severity |
| ------------------------------------------------- | ----- | --------------------------------------- | -------- |
| Quarantine (`unnet-*` removal)                    | 0     | ✅ Complete                             | —        |
| Plugin / crate structure                          | 1     | ✅ Correct                              | —        |
| UDP transport init (`Offline`/`Host`/`Join`)      | 1     | ✅ Complete                             | —        |
| JWT ticket auth (server-side)                     | 1     | ✅ Complete                             | —        |
| JWT ticket packing (client-side)                  | 1     | ✅ Complete                             | —        |
| ProcMan stdin/stdout adapter                      | 1     | ✅ Complete                             | —        |
| Transport uses `Unsecure` auth                    | 1     | ⚠️ Open TODO                            | Info     |
| Lobby entity + Replicon replication               | 2     | ✅ Complete                             | —        |
| `LobbyData` bridge                                | 2     | ✅ Complete                             | —        |
| `RoomOwner` gate for lobby UI                     | 2     | ✅ Complete                             | —        |
| `ServerAppState` → client `NextState` observer    | 2     | ❌ **Missing**                          | **High** |
| Player entity replication at InGame enter         | 3     | ✅ Complete                             | —        |
| `NetworkPosition` interpolation                   | 3     | ✅ Complete                             | —        |
| Local player → server position send               | 3     | ✅ Complete                             | —        |
| Host player → `NetworkPosition` sync              | 3     | ✅ Complete                             | —        |
| `setup_replicated_player_visuals`                 | 3     | ✅ Complete                             | —        |
| `RepliconPlayerSpawningActive` suppression        | 3     | ✅ Complete                             | —        |
| Gear `*Net` → local component observers           | 3     | ❌ **Missing**                          | **High** |
| Interaction request messages                      | 3     | ✅ Complete                             | —        |
| Room interaction replication (`DoorOpen` etc.)    | 3     | ⚠️ Not replicated (deterministic local) | Medium   |
| Truck loadout / grab-drop messages                | 3     | ❌ Not implemented                      | High     |
| Ghost replication + interpolation                 | 4     | ✅ Complete                             | —        |
| `apply_ghost_state_net` observer                  | 4     | ✅ Complete                             | —        |
| `setup_replicated_ghost_visuals`                  | 4     | ✅ Complete                             | —        |
| Possible duplicate ghost visuals on listen server | 4     | ⚠️ Unverified                           | Low      |
| `MovableStateNet` / movable objects               | 4     | ❌ Not implemented                      | Medium   |
| Evidence / journal sync                           | 4     | ✅ Complete                             | —        |
| `MissionResultNet` → `SummaryData` observer       | 4     | ❌ **Missing**                          | **High** |
| Player death server→client replication            | 4     | ❌ Not wired via replicon               | Medium   |
| `SpawnParticleNetEvent` (smoke)                   | 4     | ✅ Complete                             | —        |
| `exclude_player` field for particle events        | 4     | ❌ Missing                              | Medium   |
| `unnet-*` final purge                             | 5     | ✅ Complete                             | —        |
| `ToBeDespawned` dead code                         | —     | ⚠️ Dead code                            | Info     |
| `NetworkId` not migrated to `ClientId`            | 2     | ⚠️ Pragmatic divergence                 | Info     |
| `unlobby-plugin` path vs workspace dep            | —     | ⚠️ Convention                           | Info     |

### High-severity gaps (functional blockers for multiplayer)

1. **`ServerAppState` observer absent** — clients never follow the server's state machine beyond the initial mission
   load. Any multi-round session or error recovery will desync permanently.
2. **Gear `*Net` → local component observers absent** — flashlights, spirit boxes, sage bundles, thermometers, EMF
   meters, and repellent flasks never visually sync to remote clients.
3. **`MissionResultNet` → `SummaryData` bridge absent** — joined clients see a blank mission summary screen.

These three items are the primary remaining work before multiplayer can be considered functionally playable.
