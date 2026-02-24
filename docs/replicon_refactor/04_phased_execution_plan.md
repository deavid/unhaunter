# Phased Execution Plan: unnet → bevy_replicon

- **Date:** 2026-02-23
- **Status:** Completed execution but pending review and testing
- **Depends on:** `01_onboarding_decision.md`, `02_migration_roadmap.md`, `03_technical_appendix_and_nuances.md`

---

## Core Principles for This Migration

1. **No parallel systems.** Old and new networking are never both active at the same time.
2. **Each phase ends with a clean build.** The workspace must compile with zero errors and zero new warnings at the end
   of every phase. This is the only definition of "done" for a phase.
3. **Multiplayer will be completely broken for most phases.** This is expected, documented, and acceptable. Singleplayer
   must remain playable at every checkpoint.
4. **Old code is not deleted until it has no remaining reference value.** Files are not renamed. Systems are
   de-registered (removed from the plugin). The code sits inert as a reference.
5. **No deferred cleanup.** If a phase introduces a component split (e.g., `FlashlightNet`), the local `Flashlight`
   component must be updated in the same phase. No stubs, no `#[allow(dead_code)]` left behind.

---

## Instructions for AI Agents Executing This Plan

- **Do not run terminal commands.** Use the `get_errors` workspace tool to verify the build is clean after every
  significant change.
- **Check errors frequently.** After each logical unit of work (not just at phase end), run `get_errors`. Do not
  accumulate a large pile of errors to fix at the end.
- **Fix warnings too.** An unused import or dead-code warning from removed bindings must be resolved in the same phase
  that created it.
- **Use the old `unnet-core` and `unnet-plugin` code as a reference.** It documents exactly what each message variant
  does and what data was considered necessary. Do not delete it until Phase 6.
- **Respect the project's Bevy 0.18 patterns.** Use `MessageReader<T>` / `MessageWriter<T>` for buffered communication
  and `Trigger<T>` / `app.observe()` for Observers. Do not use `EventReader` / `EventWriter`.
- **Reflect attributes use only parentheses:** `#[reflect(Component, Default)]`.

---

## Phase 0: Quarantine

**Goal:** Disconnect all old networking code from the app without touching the files themselves. **Multiplayer after
this phase:** Completely broken. Players cannot connect at all. **Singleplayer after this phase:** Fully functional.

### What to do

1. In `unhaunter/src/app.rs`, remove `UnhaunterNetPlugin` from the app builder.
2. Audit every system in `unnet-plugin` that gets called from outside the plugin (there should be none by convention,
   but verify). If any exist, remove those call sites.
3. Remove `unnet-plugin` and `unnet-core` from the `Cargo.toml` dependency list of the root `unhaunter` crate. The
   crates remain in the workspace; they just are not linked.

### Checkpoint

- `get_errors` returns zero errors.
- The game launches in singleplayer and a mission can be completed.
- No new `#[allow(dead_code)]` or `#[allow(unused)]` annotations are introduced. Warnings caused by the de-registration
  must be resolved.

---

## Phase 1: Transport Foundation

**Goal:** The new transport layer exists, dedicated servers can accept UDP connections, and the Hub issues signed
tickets. No game data is exchanged yet. **Multiplayer after this phase:** Players can connect to a dedicated server and
receive a rejection or acceptance at the transport layer. No lobby, no game state. Effectively still broken for
gameplay. **Singleplayer after this phase:** Fully functional.

### What to do

#### 1.1. Add Dependencies

- Add `bevy_replicon`, `bevy_replicon_renet`, and `bevy_renet` to `[workspace.dependencies]` in the root `Cargo.toml`.
- Add these dependencies to the new `unreplicon-plugin` crate's `Cargo.toml`.

#### 1.2. Create `unreplicon-plugin`

- The crate already has an empty `src/systems/` directory. Wire it up:
  - `src/lib.rs` — only `mod` declarations.
  - `src/plugin.rs` — `impl Plugin` that calls `systems::app_setup(app)`.
  - `src/systems/mod.rs` — `app_setup` function.
- Add `unreplicon-plugin` to the workspace members and to the root `unhaunter` crate dependencies.
- Register `UnrepliconPlugin` in `unhaunter/src/app.rs`.

#### 1.3. Configure `bevy_renet` Transport

- Inside `unreplicon-plugin`, configure `RenetServer` / `RenetClient` to listen on / connect to the UDP port provided
  via `CliOptions`.
- No game channels yet. Connection setup only.

#### 1.4. Ticket-Based Authentication

- **Hub (`crates/tools/unhub`):** After a successful PoW challenge, the Hub issues a signed JWT containing: `room_id`,
  `installation_id`, expiry timestamp. Signed with an HMAC key shared with the dedicated server.
- **Dedicated Server:** On connection attempt, validate the JWT before allowing the Renet connection to proceed.
  Connections without a valid ticket are dropped before any ECS state is allocated.
- **`unprocman`:** Remove the TCP proxy logic. It becomes a pure process manager. Implement the Fast Expiry rule: if no
  valid ticketed connection arrives within 5 seconds of server spawn, kill the process and reclaim the port.

### Checkpoint

- `get_errors` returns zero errors.
- A dedicated server can be started. An unauthenticated connection attempt is dropped silently.
- The Hub issues a ticket after PoW; a client using that ticket can complete the transport handshake.
- Singleplayer still fully functional.

---

## Phase 2: Lobby and Session Bootstrap

**Goal:** Players can see each other in the lobby, select a map and difficulty, and initiate a mission load. The game
transitions through `AppState` and `GameState` under server authority. **Multiplayer after this phase:** Lobby is
functional. Mission loads on all clients. Clients appear in the world at initial spawn positions. No movement, no
interaction, no ghost. Effectively a frozen in-game state. **Singleplayer after this phase:** Fully functional.

### What to do

#### 2.1. Player Registry and Ownership

- On connection, the server assigns a `ClientId` (from Replicon) to each player and stores a mapping to the game's
  internal `PlayerSprite` / `PlayerName` / `PlayerColor` data.
- Replace `NetworkId` as the player identity with Replicon's `ClientId` where it is used in lobby logic. Audit all
  usages of `NetworkId` in the lobby crates (`unlobby-plugin`, `unnet-core` resources referenced from `unlobby-plugin`)
  and migrate them.

#### 2.2. Replicate `AppState` and `GameState`

- The server replicates a `ServerAppState` and `ServerGameState` resource (thin wrappers around the enums).
- Clients use an Observer (`OnAdd<ServerAppState>` / `OnChanged<ServerAppState>`) to drive their local
  `NextState<AppState>`. Clients never transition state on their own authority.

#### 2.3. Lobby State Replication

- Replicate a `LobbyInfo` component (selected map path, selected difficulty, room owner's `ClientId`, connected players
  list).
- The lobby UI in `unlobby-plugin` reads from `LobbyInfo` rather than from the old `LobbyData` resource populated by
  `unnet`. Remove `LobbyData` or repurpose it as a purely local UI cache derived from `LobbyInfo`.

#### 2.4. Map Loading

- The server replicates a `SelectedMission { map_path: String, map_seed: u64, difficulty_id: String }` entity.
- Clients use `OnAdd<SelectedMission>` to trigger `LoadLevelEvent` with the received data.
- The server does not replicate map tiles. It only replicates the identifier.

### Checkpoint

- `get_errors` returns zero errors.
- Multiple clients can connect and see a lobby.
- The room owner can select a map and difficulty; all clients see the update.
- The room owner can start the mission; all clients transition to `AppState::InGame` and load the map locally.
- Players are spawned at initial positions (no movement yet).
- Singleplayer still fully functional.

---

## Phase 3: Players, Movement, and Gear

**Goal:** Players can move, see each other move smoothly, use gear, and interact with the environment. **Multiplayer
after this phase:** Core gameplay loop is functional. Ghost is still absent. **Singleplayer after this phase:** Fully
functional.

### What to do

#### 3.1. Player Entity Replication

- The server spawns player entities and marks them with `Replicated`. Replicate: `NetworkPosition`, `PlayerName`,
  `PlayerColor`, `PlayerStateNet` (is_hiding, is_in_truck, is_spectating, stamina, health, sanity, is_running, animation
  frame).
- Each player entity is owned by its corresponding `ClientId`.

#### 3.2. `NetworkPosition` and Interpolation

- Do **not** replicate the core `Position` component directly. Introduce `NetworkPosition { x: f32, y: f32, z: f32 }`.
- **Local player:** `Position` is updated immediately by the local movement system (zero latency). The server excludes
  the owner from `NetworkPosition` replication.
- **Remote players:** An interpolation system in `unplayer-plugin` reads `NetworkPosition` on each frame and smoothly
  moves `Position` towards it. Without this, remote players will stutter at the network tick rate on a 60Hz+ display.
- The server validates movement and updates its authoritative `NetworkPosition` from the client's `PlayerMoveMessage`.

#### 3.3. Gear Component Splitting

For each piece of gear that has networked operative state, split into:

- `*Net { is_on: bool, ... }` — replicated, lives on both server and client.
- The existing full component (e.g., `Flashlight`, `EMFMeter`) — local only, never on the server.

Use `OnAdd<*Net>` and `OnChanged<*Net>` Observers on the client to drive the local component. The server spawns only the
`*Net` version.

The following gear requires splitting (reference `GearSyncState` and `GearDetails` in old `unnet-core/src/messages.rs`):

- `Flashlight` → `FlashlightNet { is_on: bool, battery: f32 }`
- `Thermometer` (deployed state only, reading computed locally)
- `EMFMeter` (deployed state only)
- `SpiritBox` (is_on, charge)
- `SageBundleData` (consumed, is_active, remaining_secs)
- `RepellentFlask` (qty, active, liquid_content)

#### 3.4. Inventory and Hands

- Replicate
  `PlayerGearNet { left_hand: Option<Entity>, right_hand: Option<Entity>, inventory: Vec<Entity>, held_item: Option<Entity> }`.
- Truck loadout changes (`TruckInventoryChange`) are sent as client messages to the server and validated before
  applying.
- Grab/drop of movable objects uses the same request/response pattern (send message, server validates, server applies,
  replication propagates the result).

#### 3.5. Interactions

- Client sends `InteractionRequestMessage { entity: Entity, ietype: InteractionExecutionType }` to the server.
- Server validates and fires `ExecuteInteractionEvent` locally.
- Changed room state (`DoorOpen`, `LightSwitchOn`, etc.) is replicated as components.
- Client uses `OnChanged<DoorOpen>` Observers to trigger local sound and animation.

### Checkpoint

- `get_errors` returns zero errors.
- Players can see each other moving smoothly.
- Players can toggle gear and see each other's gear state.
- Doors and switches interact correctly for all clients.
- Truck loadout, grab/drop, inventory changes work.
- Singleplayer still fully functional.

---

## Phase 4: Ghost, World State, and Mission Flow

**Goal:** The ghost operates, evidence and journal are synced, and the full mission flow (start → investigation →
summary) completes correctly for all players. **Multiplayer after this phase:** Fully playable end-to-end.
**Singleplayer after this phase:** Fully functional.

### What to do

#### 4.1. Ghost Replication

- The server runs the Ghost AI (`unbehavior`, navigation, hunting logic). This does not change.
- Replicate: `NetworkPosition` (same pattern as players),
  `GhostStateNet { is_hunting: bool, hunt_warning_active: bool, hunt_warning_intensity: f32, hunt_target: bool, visual_alpha_multiplier: f32 }`.
- Replicate ghost breach position (`GhostBreach` entity with `NetworkPosition`).
- Ghost influence objects (`GhostInfluence`, haunted objects) are replicated by entity presence and their replicated
  components.

#### 4.2. Movable Objects

- Replicate `MovableStateNet { current_position: NetworkPosition, held_by: Option<ClientId> }` for entities with the
  `Movable` component.

#### 4.3. Evidence and Journal

- The server maintains authoritative evidence state.
- Replicate `EvidenceFoundNet(Vec<Evidence>)` and `GhostTypeGuessNet(Option<GhostType>)`.
- Journal toggle requests (`RequestJournalEvidenceToggle`, `RequestJournalGhostToggle`) are sent as client messages and
  validated by the server.

#### 4.4. Mission Events and Summary

- Player death, sanity-driven effects, and the mission end condition are server-authoritative.
- `PlayerDiedMessage`, `MissionSummary`, and `MissionResult` are sent as server-to-client messages or replicated
  components.
- The mission summary screen reads from the replicated result.

#### 4.5. `TransientEvent` Equivalents

The old `TransientEvent::PlaySound` and `TransientEvent::SpawnParticle` must be replaced. Use dedicated reliable
Replicon client messages with an `exclude_player: Option<ClientId>` field so the originating client does not hear a
sound it already played locally.

### Checkpoint

- `get_errors` returns zero errors.
- A full multiplayer mission can be played from lobby to summary screen.
- The ghost hunts, evidence emits correctly, all clients see the same world state.
- Mission summary shows correct results for all players.
- Singleplayer still fully functional.

---

## Phase 5: Delete the Old Code

**Goal:** `unnet-core` and `unnet-plugin` are completely removed from the codebase. **Multiplayer after this phase:**
Fully functional (unchanged from Phase 4). **Singleplayer after this phase:** Fully functional.

### What to do

1. Verify that no crate in the workspace has `unnet-core` or `unnet-plugin` in its `Cargo.toml` dependencies. If any
   remain, that is a missed migration from a prior phase — fix it before deleting.
2. Delete `crates/unnet-core/` and `crates/unnet-plugin/` directories entirely.
3. Remove both from the workspace `members` list in the root `Cargo.toml`.
4. Remove both from `[workspace.dependencies]`.
5. Run `get_errors`. Resolve every error until zero remain. Any lingering import of `unnet_core::` or `unnet_plugin::`
   is a bug in a prior phase that must be fixed here.

### Checkpoint

- `get_errors` returns zero errors and zero warnings.
- `unnet-core` and `unnet-plugin` do not exist anywhere in the repository.

---

## Summary Table

| Phase | Name                        | Multiplayer State        | Singleplayer | Deletes old code?  |
| ----- | --------------------------- | ------------------------ | ------------ | ------------------ |
| 0     | Quarantine                  | Completely broken        | ✅ Working   | No (de-registers)  |
| 1     | Transport Foundation        | Transport only, no game  | ✅ Working   | No                 |
| 2     | Lobby & Session Bootstrap   | Lobby works, game frozen | ✅ Working   | Partial (lobby)    |
| 3     | Players, Movement & Gear    | Playable without ghost   | ✅ Working   | Partial (players)  |
| 4     | Ghost, World & Mission Flow | Fully playable           | ✅ Working   | Partial (ghost)    |
| 5     | Purge                       | Fully functional         | ✅ Working   | **Yes, all of it** |
