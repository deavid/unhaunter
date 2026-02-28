# Design Audit — Multiplayer Networking Subsystem

**Date:** 2026-02-27 **Scope:** All network-related code identified as relevant to the multiplayer initialization chain
and state management. **Method:** Manual review against the laws in `DESIGN_RULES.md`. **Audited crates:**
`unreplicon-plugin`, `unreplicon-core`, `unlobby-plugin`, `unmapload-plugin`, `untmxmap-plugin`,
`unclassic-mode-plugin`, `uncampaign-plugin`, `unsummary-plugin`, `unengine-plugin`, `untypes-core`.

---

## How to read this document

Each finding is tagged with the law it violates and a severity:

- **CRITICAL** — Will cause functional failures (crashes, broken flows, silent data corruption). Must be fixed before
  the feature can work.
- **MAJOR** — Violates a law clearly, degrades maintainability, and will cause bugs as the codebase grows.
- **MINOR** — Soft violation, smells wrong, not immediately broken but will become a problem.
- **NOTE** — No clear violation but an observation worth tracking.

---

## Findings: LAW 1 — Identity Over Topology

Systems must not read `CliOptions` or `NetMode` to change behavior. A `Stackable Roles` resource should be the only
source of topology identity.

---

### F-01 · `lobby.rs::auto_start_headless_lobby` · MAJOR

```rust
fn auto_start_headless_lobby(
    cli: Res<CliOptions>,
    procman: Option<Res<ProcManChannel>>,
    ...
) {
    if cli.is_headless() && procman.is_none() { ... }
}
```

Reads `cli.is_headless()` and the presence of `procman` to decide whether to start the lobby. This bundles topology
identity (`dedicated = true`) with infrastructure presence (`procman` channel) into a single conditional. Adding a third
deployment mode (e.g., a local-network server without a procman channel and without being "headless" in the CLI sense)
silently breaks this. The contract for "when should a server auto-start its lobby" is not expressed anywhere — it lives
only inside this `if` condition.

---

### F-02 · `lobby.rs::setup_lobby_entity` · MAJOR

```rust
fn setup_lobby_entity(cli: Res<CliOptions>, ...) {
    let players = if cli.is_headless() {
        Vec::new()
    } else {
        // Host (listen server): add local player with id = 0
        vec![LobbyPlayerInfo { client_id: 0, ... }]
    };
}
```

`cli.is_headless()` is used to decide whether to pre-populate the player list with a host entry. This means the function
has two distinct behaviors with one name. The question "does this node have a local human player?" is a role, not a
topology flag. The system has no way to express this without reading CLI options.

---

### F-03 · `bridge.rs::set_local_player_system` · MAJOR

```rust
match &cli.net_mode {
    NetMode::Offline => { /* nothing */ }
    NetMode::Host { .. } => {
        commands.insert_resource(LocalPlayer(Some(GameNetworkId(0))));
    }
    NetMode::Join { .. } => {
        // reads NetcodeClientTransport to find client_id
    }
}
```

Explicit match on `NetMode` variants. `LocalPlayer` is a role-level concept (which client_id belongs to the local
human), but here it is derived from topology. If a new mode is added (e.g., spectator, or a Replay mode), every arm must
be revisited. No single place declares the contract "LocalPlayer is set once at startup and never changes."

---

### F-04 · `untmxmap-plugin::load_level_handler` · CRITICAL

```rust
fn load_level_handler(
    cli: Res<untypes_core::cli::CliOptions>,
    ...
) {
    let (layers, floor_mapping) = bevy_load_map(
        tiled_map,
        ...,
        cli.is_headless(), // ← switches internal behavior
    );
}
```

This function uses `cli.is_headless()` to choose between two fundamentally different behaviors inside `bevy_load_map`:

1. **Non-headless:** loads texture atlases, builds renderable tile entities with visual component bundles.
2. **Headless:** skips atlas loading, builds geometry-only data for physics/navigation.

These are two distinct functions sharing one name. The caller cannot know which behavior it gets without reading the
CLI. The signature lies. The production invariant "a function is correct by reading only itself" is broken here.

Additionally, `Res<Maps>` is populated by `bevy_asset_loader` during `AppState::Loading`. On the headless server,
`AppState::Loading` is bypassed (`if !is_headless { ... }`), so `Maps` contains empty handles. When the server fires
`LoadLevelEvent`, this function accepts the call and silently operates on empty data. The event has no precondition
declaration. The function has no guard. The failure mode is invisible.

---

### F-05 · `unsummary-plugin::summary.rs::keyboard` · MAJOR

```rust
fn keyboard(cli: Res<CliOptions>, ...) {
    if matches!(cli.net_mode, NetMode::Offline) {
        app_next_state.set(AppState::MissionSelect);
    } else {
        app_next_state.set(AppState::Lobby);
    }
}
```

Post-summary destination is decided by `NetMode`. The question to ask is not "what mode am I running in" but "is there a
lobby to return to?" Those are different questions that happen to have the same answer today. A role-based resource
(`has_lobby: bool`) would express the intent correctly and survive topology changes.

---

### F-06 · `unengine-plugin::pause_ui::keyboard` · MAJOR

```rust
fn keyboard(cli: Res<untypes_core::cli::CliOptions>, ...) {
    if matches!(cli.net_mode, NetMode::Offline) {
        next_state.set(AppState::MissionSelect);
    } else {
        next_state.set(AppState::Lobby);
    }
}
```

Same violation as F-05. Q-quit-during-pause navigates based on topology rather than role. Same correction applies.

---

## Findings: LAW 2 — State Machine Independence

The client's `AppState` must be driven only by local intentional user actions. Replicated data may enable or display
information; it must never directly trigger `next_state.set`.

---

### F-07 · `ghost.rs::apply_mission_result_net` · CRITICAL

```rust
fn apply_mission_result_net(
    q_net: Query<&MissionResultNet, Changed<MissionResultNet>>,
    mut next_state: ResMut<NextState<AppState>>,
) {
    ...
    if !net.ready { return; }
    // copies summary data from net ...
    next_state.set(AppState::Summary); // ← direct server-driven state change
}
```

`MissionResultNet.ready` is set by the server (`sync_mission_result_to_net`). When the client's
`Changed<MissionResultNet>` fires and `ready == true`, the client immediately transitions to `AppState::Summary` with no
local decision point. The server controls the client's state machine through a replicated flag.

**Correct design:** The replicated component signals "server has computed the result." The client should read this data
and surface it through a UI event (e.g., a "Mission Complete" overlay with a Continue button). The user's press of that
button fires the navigation. The data arrives whenever, the navigation happens when the player acts.

Note: this case is acknowledged in the conversation as a possible exception ("in-game events that happen in
single-player too"). However the current implementation bypasses the player entirely — there is no "continue"
confirmation, and the transition happens mid-frame whenever the component changes, regardless of what the player is
currently doing (e.g., being hunted, or mid-interaction).

---

### F-08 · `lobby.rs::setup_lobby_entity` docstring references deleted system · MAJOR

```rust
/// Clients react to this via `react_to_server_game_phase` — but the InProgress
/// transition itself is driven by the `on_selected_mission_added` observer...
fn setup_lobby_entity(...) { ... }

/// ...updating the existing entity so bevy_replicon propagates Changed<ServerGamePhase>
/// to connected clients, triggering `react_to_server_game_phase` on each client
/// and returning them to the Lobby screen.
```

`react_to_server_game_phase` was deleted. The post-mission return-to-lobby path for clients is **functionally absent**.
The docstring describes a mechanism that no longer exists, while the code creates `Changed<ServerGamePhase>` events that
no client system reacts to. After a mission ends, the server re-enters `AppState::Lobby`. Clients are stranded in
`AppState::Summary` or `AppState::InGame` with no path back.

This is not a LAW 2 violation in the strict sense — we deleted the server-driven navigation correctly. But we didn't
replace it with a correctly designed client-side reaction. The feature is simply missing, and the docstring is a false
contract.

---

## Findings: LAW 3 — Strict Entity Authority

Every entity must have a declared owner. Event handlers must be able to fulfill their contract using only what is
provided to them.

---

### F-09 · `handle_request_start_mission` fires `LoadLevelEvent` toward a client-only handler · CRITICAL

```rust
// unreplicon-plugin/src/systems/lobby.rs (server)
ev_load.write(LoadLevelEvent { map_filepath: map_path.clone() });
```

```rust
// untmxmap-plugin/src/load_level.rs (shared)
fn load_level_handler(
    maps: Res<Maps>,            // ← populated by bevy_asset_loader (client only)
    tmx_assets: Res<Assets<TmxMap>>,
    tsx_assets: Res<Assets<TsxSheet>>,
    ...
)
```

`Maps`, `TmxMap`, and `TsxSheet` assets are loaded by `bevy_asset_loader` during `AppState::Loading`. On the headless
server, that loading block is skipped. `Res<Maps>` contains no data. The server fires the event, the handler runs with
empty resources, and map loading silently produces nothing. The server never enters `AppState::InGame`, no entities are
spawned, nothing is replicated. The event `LoadLevelEvent` has no declared preconditions, the handler has no guard, and
the failure is invisible.

---

### F-10 · `setup_ghost_entities` has an undeclared temporal precondition · CRITICAL

```rust
app.add_systems(
    OnEnter(AppState::InGame),
    setup_ghost_entities.run_if(in_state(ServerState::Running)),
);

fn setup_ghost_entities(
    q_ghost: Query<(Entity, &Position), With<GhostTag>>,
    ...
) {
    for (entity, pos) in q_ghost.iter() { // might be empty
        commands.entity(entity).insert((Replicated, NetworkPosition {...}, ...));
    }
    // always spawns MissionGoalEntity regardless
}
```

`GhostTag` entities are spawned by the level loading pipeline (`unclassic-mode-plugin::orchestrator`).
`OnEnter(AppState::InGame)` fires the moment `level_finalization::after_level_ready` transitions the state. On a
dedicated server that loaded the map headlessly, ghost entities may not exist yet (or may never exist if the map load
failed silently per F-09). The query returns empty, no entities gain `Replicated`, and no ghost is ever sent to clients.
The system succeeds silently while doing nothing useful. There is no `.run_if(any_with_component::<GhostTag>)` guard and
no logging on the empty case.

---

### F-11 · `setup_mission_players` has undeclared temporal preconditions · CRITICAL

```rust
app.add_systems(
    OnEnter(AppState::InGame),
    setup_mission_players.run_if(in_state(ServerState::Running)),
);

fn setup_mission_players(
    q_host_player: Query<(Entity, &Position, &PlayerSprite), Without<NetworkPosition>>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    ...
) {
    // queries PlayerSpawnPoint — spawned by level loading
    // queries PlayerSprite on host player — not present on dedicated server
}
```

Two undeclared preconditions:

1. `PlayerSpawnPoint` entities exist. These are spawned by the tile-spawning pipeline. If map loading fails or hasn't
   completed, `spawn_points` is empty and all remote players are placed at `(0, 0, 0)`.

2. A `PlayerSprite` entity without `NetworkPosition` belongs to "the host player." On a dedicated server, there is no
   host player entity. The host player loop silently iterates zero entities. This is handled gracefully only because the
   function falls through to the remote player loop. But the design has a silent assumption that was never declared.

---

## Findings: LAW 4 — No Temporal Coupling or Implicit Contracts

Systems must carry their own preconditions. World state that "should" exist is not a valid contract.

---

### F-12 · `uncampaign-plugin::unified_mission_selection` fires event then sets Loading · MINOR

```rust
ev_load_level.write(LoadLevelEvent {
    map_filepath: mission_data.map_filepath.clone(),
});
next_app_state.set(AppState::Loading);
```

`LoadLevelEvent` is fired and then `AppState::Loading` is set in the same system. `AppState::Loading` is the
`bevy_asset_loader` boot state — it loads `MapAssets` and `MissionAssets`, then continues to `AppState::MainMenu`. The
intent here is to ensure assets are loaded before the map handler runs. But `LoadLevelEvent` is an unbuffered event that
can be consumed in the same or the next frame, before `AppState::Loading` has actually loaded anything.

In practice this may not fail because the assets were already loaded on boot. But the contract is never stated.
`AppState::Loading` is being (ab)used both as "initial asset loading" and as "re-trigger asset check" without
documentation. Any reader of this code must trace the asset loader configuration to understand what happens.

---

### F-13 · `apply_mission_result_net` runs without a state guard · MAJOR

```rust
app.add_systems(
    Update,
    apply_mission_result_net.run_if(not(in_state(ServerState::Running))),
);
```

The only guard is "not server." This system can fire while the client is in `AppState::MainMenu`, `AppState::Loading`,
`AppState::Lobby`, or any other state. If a stale `MissionResultNet` (from a previous session) is still present in the
world and `Changed` triggers for any reason, this system will fire `next_state.set(AppState::Summary)` from an
unexpected state. There is no `run_if(in_state(AppState::InGame))` guard.

---

## Findings: LAW 5 — Separation of Meanings (Strict Vocabulary)

---

### F-14 · `owner_client_id: u64` uses `0` to mean three different things · MAJOR

`LobbyInfo.owner_client_id` initialized to `0` means:

1. **"The listen-server host"** — on a listen-server. `0` is the sentinel for the server process's own player.
2. **"No owner assigned yet"** — on a dedicated server before the first client connects. `on_client_connected` checks
   `if lobby.players.is_empty() && lobby.owner_client_id == 0` to assign the first client as owner.
3. **"The server itself is the owner"** — in message validation `handle_request_select_map`,
   `handle_request_start_mission`, etc., where `sender_id != lobby.owner_client_id` uses `0` to mean "this message must
   come from the server/host process."

These three meanings are conflated. A dedicated server starts with `owner = 0` (meaning 2), a client connects with id =
42, and the check in `on_client_connected` promotes them to owner. But if a second client connects before the first, the
same check fires again reading an `owner_client_id` of 42, sees `!players.is_empty()`, and correctly skips. So it works
today. But any code that checks `owner_client_id == 0` to mean "no owner" will fail on a listen-server where `0` means
"the host is the owner."

---

### F-15 · `LobbyData` vs `LobbyInfo` — two sources of truth · MAJOR

`LobbyInfo` is the replicated authoritative component (owned by the server entity, synced via bevy_replicon).
`LobbyData` is a local `Resource` on the client that mirrors `LobbyInfo` through `bridge_lobby_info_system`. Both hold
`selected_map`, `selected_difficulty`, and the player list.

There is no declaration of which is the source of truth for any given field. UI systems read `LobbyData`. Message
handlers write to `LobbyInfo`. The bridge syncs in one direction (LobbyInfo → LobbyData) on change. If anything reads
`LobbyData` before the bridge runs, it sees stale data. If anything writes to `LobbyData` directly, that write is
silently overwritten by the next bridge sync. Two systems that each "work" can interact incorrectly without any visible
contract violation.

---

### F-16 · "Host" means two different roles · MINOR

Following from the vocabulary in `DESIGN_RULES.md` LAW 5:

- "host" in `NetMode::Host` means the Authority Node (server + listen-server).
- "host player" in `setup_mission_players` means the local human playing on the listen-server.
- "host" in `lobby.owner_client_id = 0` means "the server-process itself."

These roles are legitimate and distinct. The problem is they share a name throughout comments, logs, and variable names
(`q_host_player`, `host_in_mission`, `is_headless`). When reading any one of these, the reader must track context to
know which "host" is meant. The Design Rules prescribe distinct terms: Authority Node, Local Player, Lobby Leader.

---

## Structural observations (no specific law violated)

---

### F-17 · No server initialization sequence is declared · NOTE

The client has a documented state machine: `Loading → MainMenu → Lobby → InGame`. The headless server has no equivalent
documentation. Its startup sequence is:

1. `ServerState::Running` (bevy_replicon transitions this automatically)
2. `OnEnter(ServerState::Running)` → `auto_start_headless_lobby` → `AppState::Lobby`
3. Waits for clients
4. `RequestStartMission` → fires `LoadLevelEvent` → (broken, see F-04, F-09)
5. Supposed to reach `AppState::InGame`, never does

There is no document or comment block that describes the intended server state machine. Without it, any developer adding
a system must infer the server's lifecycle by reading every system's run conditions.

---

### F-18 · `untmxmap-plugin::load_level_handler` is two functions · NOTE

Directly from F-04: the single function `load_level_handler` in `untmxmap-plugin` has two distinct execution paths
depending on `cli.is_headless()`. These should be two registered systems: one for nodes with a local player (loads
textures + geometry), one for the authority-only node (loads geometry only). Registered separately in the plugin's
`if is_headless { } else { }` block that already exists in `unmapload-plugin/src/plugin.rs`.

---

### F-19 · `sync_player_state_to_net` and `send_local_player_position` are parallel paths with no contract · NOTE

- Listen-server host: position is synced server-side via `sync_player_state_to_net` (reads `MainPlayer`, writes
  `NetworkPosition`/`PlayerStateNet` directly).
- Remote join client: position is sent via `send_local_player_position` → network message → `handle_player_move` on
  server.

Both paths produce the same result (updated `NetworkPosition` on the authority). But there is no declaration of "a
player entity always has its NetworkPosition updated by exactly one path." If `MainPlayer` is somehow present on a join
client (e.g., due to a component tagging bug), both paths could run simultaneously, racing to write the same components.
No run condition prevents this.

---

## Finding count by law

| Law                                | Critical | Major | Minor | Note  |
| ---------------------------------- | -------- | ----- | ----- | ----- |
| LAW 1 — Identity Over Topology     | 1        | 5     | 0     | 0     |
| LAW 2 — State Machine Independence | 1        | 1     | 0     | 0     |
| LAW 3 — Strict Entity Authority    | 3        | 0     | 0     | 0     |
| LAW 4 — No Temporal Coupling       | 0        | 1     | 1     | 0     |
| LAW 5 — Separation of Meanings     | 0        | 2     | 1     | 0     |
| Structural                         | 0        | 0     | 0     | 3     |
| **Total**                          | **5**    | **9** | **2** | **3** |

---

## Priority order for remediation

These findings depend on each other. The recommended fix order:

1. **F-04 + F-09** — The server's map loading pipeline is broken. Nothing above this works until the server can actually
   load a map. Split `load_level_handler` into headless and non-headless variants.
2. **F-10 + F-11** — These depend on the server being in `AppState::InGame` with map entities present. Fix after
   F-04/F-09.
3. **F-08** — The post-mission return-to-lobby path is missing. Implement a correct client-side reaction to
   `ServerGamePhase::Lobby` that does not violate LAW 2.
4. **F-07** — Design the mission-complete transition correctly per LAW 2.
5. **F-14** — The `0` sentinel problem. Introduce an explicit `OwnerId` newtype or `Option<NetworkId>` to distinguish
   "no owner" from "server is owner."
6. **F-15** — Consolidate `LobbyData` and `LobbyInfo` or declare the contract clearly.
7. **F-01 through F-06** — LAW 1 violations. Introduce a `ProcessRoles` resource and migrate all `CliOptions`/`NetMode`
   reads in gameplay/UI systems to use it.
8. **F-12, F-13, F-19** — Clean up temporal coupling and parallel update paths.
