# Plan 25: Session Layer — Player Roster, Heartbeat, Late Join & Disconnect Resilience

## Problem Statement

The multiplayer protocol conflates connection-level events with UI-level logic, creating several broken scenarios:

1. **Player roster is lobby-UI-gated:** `lobby_broadcast_state_system` is the only system that builds
   `LobbyData.players`, and it only runs in `AppState::Lobby`. If a client completes handshake while the host is in
   `InGame`, the `PlayerJoinedEvent` expires unconsumed. The host itself doesn't appear in the roster until it opens the
   lobby screen.

2. **Player entity spawning has a timing problem:** `spawn_joined_player` requires both `AppState::InGame` AND
   `on_message::<PlayerJoinedEvent>`. A player who joins during the lobby phase fires the event while InGame is inactive
   — their entity is never created on the host.

3. **Late join / reconnect is a dead end:** A client who connects while the host is mid-mission gets `LobbyWelcome` but
   then has no way to reach `AppState::InGame`. `StartMission` was a one-shot. Snapshots carry `app_state: InGame` but
   the client-side reader runs only in InGame. Chicken-and-egg.

4. **No heartbeat:** There is no way for the host to detect a silently-dead client vs. an active one. The only
   "liveness" signal is TCP — which can lag behind a crashed app by minutes.

5. **Disconnected players block mission end:** `evaluate_mission_end` already skips `PlayerDisconnected` entities, but
   there's no concept of "AFK" or "inactive" — a player who is connected but frozen (app hung) counts as active and
   blocks the truck-based end condition.

6. **Disconnected players are vulnerable:** A disconnected player's entity stands in the open, vulnerable to the ghost.
   There's no grace period or protective effect.

## Design Decisions

### Player Roster Ownership

The player roster moves from the lobby UI (`lobby_broadcast_state_system`) to a **state-independent session system** in
`unnet-plugin`. This system runs in `PreUpdate` with no `AppState` gate. It manages the canonical `LobbyData.players`
list. The lobby UI becomes a pure reader.

### Host Status Message

After `LobbyWelcome`, the host immediately sends a **`HostStatus`** message telling the client what the host is
currently doing. This replaces the one-shot nature of `StartMission`.

### Client State Bootstrap

A minimal **state-independent snapshot reader** is added that reads `SnapshotMsg.app_state` when the client is NOT in
`InGame`. This allows clients to transition to the correct state. The full entity sync remains `InGame`-gated.

### Heartbeat

A new `Heartbeat` message is sent by clients at `FixedUpdate` (~30 fps). It carries the client's current `AppState`
(what screen they're on). The host tracks `last_heartbeat_time` per client.

### Player Activity Tracking

A new `PlayerInactive` marker component (distinct from `PlayerDisconnected`) is added for players who are connected but
unresponsive (no heartbeat for >5 seconds). Both `PlayerDisconnected` and `PlayerInactive` entities are excluded from
the "all in truck" check and ghost AI targeting.

### Disconnect Protection

When a player gains `PlayerDisconnected`, they also receive the `Hiding` component (with no hiding spot). This makes
them invisible to the ghost. The `Hiding` is removed when they reconnect.

### Late Join from Lobby

When a client is in the lobby and the host is mid-mission, the lobby shows mission-in-progress information and a "Join
Mission" button. Clicking it sends a `RequestLateJoin` message. The host either re-activates their existing entity or
spawns a new one at a spawn point with starting gear.

### Reconnecting to a Dead Entity

If a player's entity is dead (has `PlayerSpectating`), they rejoin as a spectator. No respawn.

---

## Phase A: Player Roster at Connection Layer

### Goal

Move player list management from `unlobby-plugin/broadcast.rs` to `unnet-plugin`, running in `PreUpdate` with no state
gate. Make the host always appear in the roster.

### Files Changed

| File                                             | Change                                                                       |
| ------------------------------------------------ | ---------------------------------------------------------------------------- |
| `crates/unnet-plugin/src/systems/connection.rs`  | New system `session_roster_system`                                           |
| `crates/unnet-plugin/src/systems/setup.rs`       | Register `session_roster_system` in `PreUpdate` chain                        |
| `crates/unlobby-plugin/src/systems/broadcast.rs` | Remove player-list-building logic, keep only periodic `LobbyState` broadcast |

### Detail: `session_roster_system`

```rust
/// Maintains the canonical player roster. Runs every frame, all states.
/// On the host: reads PlayerJoinedEvent, adds to LobbyData.players.
/// Ensures host is always in the list.
pub(crate) fn session_roster_system(
    mut lobby_data: ResMut<LobbyData>,
    mut ev_player_joined: MessageReader<PlayerJoinedEvent>,
    cli: Res<CliOptions>,
    local_player: Res<LocalPlayer>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }

    let mut changed = false;

    // Consume PlayerJoinedEvent — this MUST run every frame so events aren't lost
    for ev in ev_player_joined.read() {
        if !lobby_data.players.iter().any(|p| p.id == ev.id) {
            let next_tint = (lobby_data.players.len() % 9) as u8;
            lobby_data.players.push(LobbyPlayer {
                id: ev.id,
                tint_color_index: next_tint,
            });
            changed = true;
        }
    }

    // Ensure host is always in the list
    if let Some(host_id) = local_player.0
        && !lobby_data.players.iter().any(|p| p.id == host_id)
    {
        lobby_data.players.insert(0, LobbyPlayer {
            id: host_id,
            tint_color_index: 0,
        });
        changed = true;
    }

    if !changed {
        lobby_data.bypass_change_detection();
    }
}
```

This is essentially the same logic that currently lives in `lobby_broadcast_state_system`, but running unconditionally.

### Detail: Updated `lobby_broadcast_state_system`

The lobby broadcast system becomes a pure broadcaster — no roster building:

```rust
pub(crate) fn lobby_broadcast_state_system(
    lobby_data: Res<LobbyData>,
    mut ev_send: MessageWriter<SendNetworkMessage>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    mut last_broadcast: Local<f32>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }
    // Broadcast every 500ms
    if time.elapsed_secs() - *last_broadcast > 0.5 {
        ev_send.write(SendNetworkMessage(NetworkMessage::LobbyState {
            players: lobby_data.players.clone(),
            selected_map: lobby_data.selected_map.clone(),
            selected_difficulty: lobby_data.selected_difficulty.clone(),
        }));
        *last_broadcast = time.elapsed_secs();
    }
}
```

### Registration

In `unnet-plugin/src/systems/setup.rs`, add `session_roster_system` to the `PreUpdate` chain, after
`handshake_handler_system` (so `PlayerJoinedEvent` is available):

```rust
PreUpdate,
(
    network_io_system,
    host_handle_disconnects_system,
    handshake_handler_system,
    session_roster_system,        // NEW — always runs
    client_lobby_state_handler,
    client_start_mission_handler,
    host_apply_input_system.run_if(in_state(AppState::InGame)),
)
    .chain(),
```

---

## Phase B: Host Status Message & Client State Bootstrap

### Goal

When a client connects, immediately tell them what the host is doing. If the host is mid-mission, send them everything
they need to catch up. Also, make the client able to read `app_state` from snapshots even when not in `InGame`.

### Files Changed

| File                                            | Change                                                                                                                                     |
| ----------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------ |
| `crates/unnet-core/src/messages.rs`             | Add `HostStatus` variant to `NetworkMessage`                                                                                               |
| `crates/unnet-core/src/resources.rs`            | Add `host_app_state`, `mission_elapsed_secs`, `evidences_found_count`, `repellent_used`, `alive_count`, `dead_count` fields to `LobbyData` |
| `crates/unnet-plugin/src/systems/connection.rs` | After sending `LobbyWelcome`, also send `HostStatus`. New system `host_status_updater_system`                                              |
| `crates/unnet-plugin/src/systems/connection.rs` | New system `client_state_bootstrap_system` — reads snapshots when NOT in InGame                                                            |
| `crates/unnet-plugin/src/systems/setup.rs`      | Register new systems                                                                                                                       |

### Detail: `HostStatus` Message

```rust
/// Sent by host to a newly connected client immediately after LobbyWelcome.
/// Also broadcast periodically so lobby clients know what the host is doing.
HostStatus {
    app_state: AppState,
    game_state: GameState,
    /// If host is in a mission, these fields are populated:
    map_filepath: Option<String>,
    difficulty_id: Option<String>,
    mission_elapsed_secs: f32,
    evidences_found_count: u8,
    repellent_used: u32,
    alive_count: u8,
    dead_count: u8,
    /// Per-player status for display in the lobby
    player_statuses: Vec<PlayerStatusInfo>,
},
```

Where `PlayerStatusInfo` is:

```rust
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct PlayerStatusInfo {
    pub id: NetworkId,
    pub tint_color_index: u8,
    pub is_alive: bool,
    pub is_in_truck: bool,
    pub is_disconnected: bool,
    pub is_inactive: bool,
    pub is_in_lobby: bool,
}
```

### Detail: `host_status_updater_system`

Runs on the host in all states. Gathers current host state and periodically broadcasts `HostStatus` (every 1 second).
This gives lobby-waiting clients a view of the ongoing mission.

The `LobbyData` resource gets new fields to hold this info client-side:

```rust
pub struct LobbyData {
    pub players: Vec<LobbyPlayer>,
    pub selected_map: Option<String>,
    pub selected_difficulty: Option<String>,
    // New: host session state
    pub host_app_state: Option<AppState>,
    pub host_game_state: Option<GameState>,
    pub mission_elapsed_secs: f32,
    pub evidences_found_count: u8,
    pub repellent_used: u32,
    pub alive_count: u8,
    pub dead_count: u8,
    pub player_statuses: Vec<PlayerStatusInfo>,
}
```

### Detail: `client_state_bootstrap_system`

This system runs on the client when NOT in `AppState::InGame`. It reads `NetworkDataEvent` and handles:

1. **`HostStatus`** — updates `LobbyData` with host state info. If in lobby, this drives the mission-in-progress
   display.
2. **`Snapshot`** — extracts ONLY `app_state` and transitions the client. Does NOT process entity data (that remains
   `InGame`-gated). This handles the chicken-and-egg: client reads `app_state` from the snapshot to know it should
   transition to `InGame`.

```rust
pub(crate) fn client_state_bootstrap_system(
    mut ev_reader: MessageReader<NetworkDataEvent>,
    mut lobby_data: ResMut<LobbyData>,
    cli: Res<CliOptions>,
    current_app_state: Res<State<AppState>>,
    mut next_app_state: ResMut<NextState<AppState>>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    // Only run when NOT in InGame (InGame has its own snapshot handler)
    if *current_app_state.get() == AppState::InGame {
        return;
    }
    for ev in ev_reader.read() {
        match &ev.message {
            NetworkMessage::HostStatus {
                app_state,
                game_state,
                map_filepath,
                difficulty_id,
                mission_elapsed_secs,
                evidences_found_count,
                repellent_used,
                alive_count,
                dead_count,
                player_statuses,
            } => {
                lobby_data.host_app_state = Some(*app_state);
                lobby_data.host_game_state = Some(*game_state);
                lobby_data.mission_elapsed_secs = *mission_elapsed_secs;
                lobby_data.evidences_found_count = *evidences_found_count;
                lobby_data.repellent_used = *repellent_used;
                lobby_data.alive_count = *alive_count;
                lobby_data.dead_count = *dead_count;
                lobby_data.player_statuses = player_statuses.clone();

                // Update map/difficulty from host status if we don't have them
                if lobby_data.selected_map.is_none() {
                    lobby_data.selected_map = map_filepath.clone();
                }
                if lobby_data.selected_difficulty.is_none() {
                    lobby_data.selected_difficulty = difficulty_id.clone();
                }
            }
            _ => {}
        }
    }
}
```

### Detail: Sending `HostStatus` on Handshake

In `handshake_handler_system`, after queueing `LobbyWelcome`, also queue `HostStatus` with current state. This means a
freshly-connected client immediately knows whether the host is in the lobby or mid-mission.

### Registration

`client_state_bootstrap_system` runs in `PreUpdate`, after `client_lobby_state_handler`. It must NOT conflict with
`client_apply_snapshots_system` — they are mutually exclusive by `AppState` gate.

`host_status_updater_system` runs in `Update`, all states, host only.

---

## Phase C: Heartbeat & Activity Detection

### Goal

Clients send periodic heartbeats. Host tracks liveness. Unresponsive players get `PlayerInactive`, which excludes them
from mission-end checks and ghost targeting.

### Files Changed

| File                                                     | Change                                                                                                           |
| -------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------- |
| `crates/unnet-core/src/messages.rs`                      | Add `Heartbeat { app_state: AppState }` to `NetworkMessage`                                                      |
| `crates/unnet-plugin/src/resources.rs`                   | Add `last_heartbeat: f32` to `ClientConnection`                                                                  |
| `crates/unnet-plugin/src/systems/connection.rs`          | New system `client_heartbeat_system` (client-side sender). New system `host_liveness_system` (host-side checker) |
| `crates/unplayer-core/src/components.rs`                 | Add `PlayerInactive` component                                                                                   |
| `crates/unmission-plugin/src/lib.rs`                     | Update `evaluate_mission_end` to also skip `PlayerInactive`                                                      |
| `crates/unghost-plugin/src/systems/ghost_ai/movement.rs` | Add `Without<PlayerInactive>` to player queries                                                                  |
| `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs`   | Add `Without<PlayerInactive>` to player queries                                                                  |
| `crates/unnet-plugin/src/systems/setup.rs`               | Register new systems                                                                                             |

### Detail: `Heartbeat` Message

```rust
Heartbeat {
    app_state: AppState,
},
```

Sent by the client at `FixedUpdate` rate (~30fps). Tiny payload.

### Detail: `client_heartbeat_system`

Runs on the client, all states, at `FixedUpdate`. Sends `Heartbeat` if handshake is completed.

```rust
pub(crate) fn client_heartbeat_system(
    mut conn: ResMut<NetworkConn>,
    cli: Res<CliOptions>,
    current_app_state: Res<State<AppState>>,
) {
    if !matches!(cli.net_mode, NetMode::Join { .. }) {
        return;
    }
    if !conn.is_active() {
        return;
    }
    conn.client_send(NetworkMessage::Heartbeat {
        app_state: *current_app_state.get(),
    });
}
```

### Detail: `last_heartbeat` Field

`ClientConnection` gets a new field:

```rust
pub(crate) struct ClientConnection {
    // ... existing fields ...
    pub last_heartbeat: f32,
    pub client_app_state: Option<AppState>,
}
```

Initialized to `0.0` on connection acceptance. Updated whenever a `Heartbeat` message is received (in `do_client_io` or
a dedicated handler).

### Detail: `host_liveness_system`

Runs on the host, all states. Checks each connected client's `last_heartbeat`. If more than 5 seconds old, marks the
corresponding player entity as `PlayerInactive`. If heartbeat resumes, removes `PlayerInactive`.

```rust
pub(crate) fn host_liveness_system(
    conn: Res<NetworkConn>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    mut commands: Commands,
    query_players: Query<(Entity, &NetworkId), (With<PlayerTag>, Without<PlayerDisconnected>)>,
    query_inactive: Query<Entity, With<PlayerInactive>>,
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }
    let NetworkConn::Host { clients, .. } = &*conn else { return };

    for client in clients.iter() {
        let Some(id) = client.associated_id else { continue };
        let elapsed = time.elapsed_secs() - client.last_heartbeat;

        for (entity, player_id) in query_players.iter() {
            if *player_id != id { continue; }
            if elapsed > 5.0 {
                commands.entity(entity).insert(PlayerInactive);
            } else {
                // Only remove if it was there
                if query_inactive.contains(entity) {
                    commands.entity(entity).remove::<PlayerInactive>();
                }
            }
        }
    }
}
```

### Detail: `PlayerInactive` Component

In `crates/unplayer-core/src/components.rs`:

```rust
/// Marks a player entity that is connected but unresponsive (no heartbeat for >5s).
/// Excluded from mission-end checks and ghost AI targeting.
#[derive(Component, Debug, Clone, Default)]
pub struct PlayerInactive;
```

### Detail: Ghost AI Filter Updates

Add `Without<PlayerInactive>` alongside existing `Without<PlayerDisconnected>` in:

- `ghost_ai/movement.rs` line 50: player query
- `ghost_ai/enrage.rs` line 45, 216, 255, 369: player queries

### Detail: `evaluate_mission_end` Update

Add `Has<PlayerInactive>` to the query. Skip players that are either disconnected OR inactive:

```rust
for (_, in_truck, spectating, disconnected, inactive) in query_players.iter() {
    if disconnected || inactive {
        continue;
    }
    // ... rest as before
}
```

### Detail: Heartbeat Processing

In the host's message processing (`do_client_io` or `network_io_system`), when a `Heartbeat` message is received, update
`client.last_heartbeat = time.elapsed_secs()` and `client.client_app_state`. Since `do_client_io` doesn't have access to
`Time`, we handle this in a new spot: in `handshake_handler_system` (which reads all `NetworkDataEvent`), or in a
dedicated `host_heartbeat_receiver_system` that runs in `PreUpdate`.

Preferred approach: Add a new system `host_process_heartbeats_system` in `PreUpdate` that:

1. Reads `NetworkDataEvent` for `Heartbeat` messages
2. Updates the `ClientConnection.last_heartbeat` and `client_app_state` fields on the matching client

---

## Phase D: Disconnect Protection (Hidden Effect)

### Goal

When a player disconnects, grant them the `Hiding` component so the ghost ignores them. Remove it when they reconnect.

### Files Changed

| File                                            | Change                                                                                         |
| ----------------------------------------------- | ---------------------------------------------------------------------------------------------- |
| `crates/unnet-plugin/src/systems/connection.rs` | In `host_handle_disconnects_system`, add `Hiding` when marking `PlayerDisconnected`            |
| `crates/unnet-plugin/src/systems/connection.rs` | In `handshake_handler_system`, remove `Hiding` when removing `PlayerDisconnected` on reconnect |

### Detail

In `host_handle_disconnects_system`, when inserting `PlayerDisconnected`:

```rust
commands.entity(entity)
    .insert(PlayerDisconnected)
    .insert(Hiding { hiding_spot: None });
```

In `handshake_handler_system`, when removing `PlayerDisconnected` for a reconnecting player:

```rust
commands.entity(entity)
    .remove::<PlayerDisconnected>()
    .remove::<Hiding>();
```

The ghost AI already uses `Without<PlayerDisconnected>` AND checks `Option<&Hiding>`, so both layers work. The `Hiding`
protects against edge cases where `PlayerDisconnected` filter might not be applied (e.g., a system that wasn't updated).
Belt and suspenders.

**Note:** The `Hiding` status is permanent until they rejoin. They CAN be killed if the ghost clips through their
location during a hunt. This is intentional — it's a bonus, not invincibility.

---

## Phase E: Late Join from Lobby

### Goal

A client in the lobby can see the host is mid-mission and click "Join Mission" to enter the game. The host either
reactivates their existing entity or spawns a new one.

### Files Changed

| File                                                       | Change                                                                                                               |
| ---------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------- |
| `crates/unnet-core/src/messages.rs`                        | Add `RequestLateJoin { player_id: NetworkId }` to `NetworkMessage`                                                   |
| `crates/unlobby-plugin/src/systems/lobby_main.rs`          | Show mission-in-progress info when `lobby_data.host_app_state == Some(AppState::InGame)`. Add "Join Mission" button. |
| `crates/unnet-plugin/src/systems/host_input.rs`            | Handle `RequestLateJoin` — send `StartMission` to that client, set `needs_full_sync`                                 |
| `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` | Update `spawn_joined_player` to also handle late joins (runs on `PlayerJoinedEvent` during InGame)                   |
| `crates/unnet-plugin/src/systems/connection.rs`            | In `client_start_mission_handler`, handle receiving `StartMission` while in Lobby                                    |

### Detail: Lobby UI — Mission In Progress View

When `lobby_data.host_app_state == Some(AppState::InGame)`, the lobby main screen transforms:

**Left strip changes:**

- "Select Map" and "Select Difficulty" are disabled/hidden (host is already in a mission)
- "Start Mission" is replaced by **"Join Mission"** (sends `RequestLateJoin`)
- "Exit Lobby" remains

**Right panel shows mission-in-progress info instead of the usual preview:**

```
═══ Mission In Progress ═══

Map: [map display name]
Difficulty: [difficulty name]
Elapsed: 4m 32s

Players in Mission:
  ■ Player 1 (Host) - Alive
  ■ Player 2 - Alive, In Truck
  ■ Player 3 - Dead

Evidence: 2 found
Repellent: 1 used

── You ──
► Player 4 - Waiting in Lobby
```

The data for this comes from `LobbyData.player_statuses` and the mission stats, all populated by `HostStatus` messages.

### Detail: `RequestLateJoin` Flow

1. Client clicks "Join Mission" → sends `RequestLateJoin { player_id }`
2. Host receives it in `host_apply_input_system` (or a dedicated handler)
3. Host sends `StartMission { map_filepath, difficulty_id, map_seed }` to that specific client (via `host_send_to`)
4. Host marks `client.needs_full_sync = true`
5. Client receives `StartMission` → triggers `client_start_mission_handler` → transitions to `AppState::Loading`
6. Client loads map → enters `InGame` → receives full snapshot → entities spawn via `client_apply_snapshots_system`
7. On the host, `spawn_joined_player` already handles the entity: if the player already has an entity (reconnect), it
   skips spawn and removes `PlayerDisconnected`/`Hiding`. If not (fresh late join), it spawns a new entity at a spawn
   point with starter gear.

### Detail: Handling the Dead Reconnect Case

If the player's existing entity has `PlayerSpectating`, they rejoin as a spectator. The `spawn_joined_player` system
sees the existing entity, removes `PlayerDisconnected` and `Hiding`, but leaves `PlayerSpectating` in place. The client
sees themselves as a spectator.

### Detail: `spawn_joined_player` Trigger Fix

Currently `spawn_joined_player` runs on `on_message::<PlayerJoinedEvent>` gated by `in_state(AppState::InGame)`. The
timing problem is:

- Join during lobby: event fires, InGame not active, event lost.
- Late join: `RequestLateJoin` handled while InGame IS active, but `PlayerJoinedEvent` may not fire (the handshake
  happened earlier).

**Fix:** Change `spawn_joined_player` to NOT rely on `PlayerJoinedEvent`. Instead, it should run every frame in `InGame`
and check for player `NetworkId`s in the roster (`LobbyData.players`) that don't have corresponding entities. This is a
poll-based approach:

```rust
pub(crate) fn spawn_joined_player(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    lobby_data: Res<LobbyData>,
    existing_players: Query<&NetworkId, With<PlayerTag>>,
    q_player_spawns: Query<&Position, With<PlayerSpawnPoint>>,
    q_disconnected: Query<(Entity, &NetworkId), With<PlayerDisconnected>>,
) {
    let existing_ids: HashSet<NetworkId> = existing_players.iter().copied().collect();

    for lobby_player in &lobby_data.players {
        let id = lobby_player.id;

        // Check if entity exists
        if existing_ids.contains(&id) {
            // Entity exists — if it's disconnected, reconnect detection is handled
            // by handshake_handler_system (removes PlayerDisconnected).
            // Nothing to do here.
            continue;
        }

        // No entity for this player — but only spawn if the host has an active
        // connection to them (they're not just in the roster from a past session)
        // This is checked via the connection being active.
        // For the host itself (id == 1), the entity is created by the orchestrator.
        if id == NetworkId(1) {
            continue; // Host entity is spawned by classic_mode_orchestrator
        }

        // Spawn new player entity with starting gear
        // ... (existing spawn logic)
    }
}
```

The `run_if` in `plugin.rs` changes from:

```rust
spawn_joined_player
    .run_if(in_state(AppState::InGame))
    .run_if(on_message::<PlayerJoinedEvent>),
```

To:

```rust
spawn_joined_player
    .run_if(in_state(AppState::InGame)),
```

And the system itself does the "does this player need spawning?" check internally. It should also add a guard so it
doesn't try to spawn for players who are in the roster but have no active TCP connection (e.g. they disconnected and
their roster entry persists). This can be done by adding a parameter that reads `NetworkConn` and checking if the client
is actually connected.

However, `NetworkConn` is in `unnet-plugin` and `spawn_joined_player` is in `unclassic-mode-plugin`. To avoid a
cross-crate dependency, we add an `is_connected: bool` field to `LobbyPlayer` (or a separate resource). Simpler: we add
the field to `LobbyPlayer`.

### Detail: Updated `LobbyPlayer`

```rust
pub struct LobbyPlayer {
    pub id: NetworkId,
    pub tint_color_index: u8,
    pub connected: bool,  // NEW: true if TCP connection is active
}
```

`session_roster_system` sets `connected = true` on join, and a new piece of logic sets it to `false` on disconnect (when
`NetworkDisconnectEvent` fires). The spawn system only spawns for `connected == true` players.

---

## Phase F: LobbyState Broadcast Scope Change

### Goal

`LobbyState` should be broadcast by the host regardless of the host's `AppState`. A client in the lobby needs to receive
roster updates even when the host is mid-mission.

### Files Changed

| File                                         | Change                                                                                                                     |
| -------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------- |
| `crates/unlobby-plugin/src/systems/setup.rs` | Change `lobby_broadcast_state_system` run condition from `in_state(AppState::Lobby)` to always-run (host only, self-gated) |

### Detail

`lobby_broadcast_state_system` already has an internal `if !matches!(cli.net_mode, NetMode::Host { .. })` guard. Remove
the `.run_if(in_state(AppState::Lobby))` condition. The system runs every frame on the host and broadcasts `LobbyState`
every 500ms.

Alternatively, move this system from `unlobby-plugin` into `unnet-plugin` (since it's now connection-layer logic, not UI
logic). This is cleaner but means moving the system between crates.

**Decision:** Move it to `unnet-plugin/src/systems/connection.rs` and register it in
`unnet-plugin/src/systems/setup.rs`. The `unlobby-plugin` no longer has any broadcast logic.

---

## Phase G: `HostStatus` Updater Implementation

### Goal

The host periodically gathers mission-in-progress stats and broadcasts `HostStatus`.

### Files Changed

| File                                            | Change                                  |
| ----------------------------------------------- | --------------------------------------- |
| `crates/unnet-plugin/src/systems/connection.rs` | New system `host_status_updater_system` |
| `crates/unnet-plugin/src/systems/setup.rs`      | Register it                             |

### Detail

```rust
pub(crate) fn host_status_updater_system(
    mut ev_send: MessageWriter<SendNetworkMessage>,
    cli: Res<CliOptions>,
    time: Res<Time>,
    mut last_update: Local<f32>,
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    lobby_data: Res<LobbyData>,
    query_players: Query<
        (&NetworkId, Has<PlayerSpectating>, Has<InTruck>,
         Has<PlayerDisconnected>, Has<PlayerInactive>),
        With<PlayerTag>,
    >,
    ghost_guess: Res<GhostGuess>,            // for evidence count
    repellent_tracker: Res<RepellentTracker>, // for repellent count
    // mission_timer or similar for elapsed time
) {
    if !matches!(cli.net_mode, NetMode::Host { .. }) {
        return;
    }
    if time.elapsed_secs() - *last_update < 1.0 {
        return;
    }
    *last_update = time.elapsed_secs();

    let player_statuses: Vec<PlayerStatusInfo> = lobby_data.players.iter().map(|lp| {
        let (alive, in_truck, disconnected, inactive) = query_players.iter()
            .find(|(id, ..)| **id == lp.id)
            .map(|(_, spec, truck, disc, inact)| (!spec, truck, disc, inact))
            .unwrap_or((true, false, false, false));

        // Check if player is in lobby (from heartbeat client_app_state)
        let is_in_lobby = /* derived from ClientConnection.client_app_state */ false;

        PlayerStatusInfo {
            id: lp.id,
            tint_color_index: lp.tint_color_index,
            is_alive: alive,
            is_in_truck: in_truck,
            is_disconnected: disconnected,
            is_inactive: inactive,
            is_in_lobby,
        }
    }).collect();

    let (alive_count, dead_count) = /* count from player_statuses */;

    ev_send.write(SendNetworkMessage(NetworkMessage::HostStatus {
        app_state: *app_state.get(),
        game_state: *game_state.get(),
        map_filepath: lobby_data.selected_map.clone(),
        difficulty_id: lobby_data.selected_difficulty.clone(),
        mission_elapsed_secs: /* from mission timer */,
        evidences_found_count: ghost_guess.evidences_found.len() as u8,
        repellent_used: repellent_tracker.crafted_count,
        alive_count,
        dead_count,
        player_statuses,
    }));
}
```

**Note:** Access to resources like `GhostGuess` and repellent tracker may require checking which exact resources hold
this data. The `SnapshotMsg` builder in `host_sync.rs` already queries all of this, so we can look at the same params.
Some of these may only exist during `InGame`, so we use `Option<Res<...>>` and default to 0 when unavailable.

---

## Phase H: Client-Side `client_start_mission_handler` Fix

### Goal

Allow `client_start_mission_handler` to work when the client is in the Lobby state (not just MainMenu/Loading).

### Files Changed

| File                                       | Change                                                        |
| ------------------------------------------ | ------------------------------------------------------------- |
| `crates/unnet-plugin/src/systems/setup.rs` | Ensure `client_start_mission_handler` runs in Lobby state too |

### Detail

Currently `client_start_mission_handler` runs in the `PreUpdate` chain with no state gate — it already works in all
states. ✓

However, `client_process_pending_map` (which processes the `PendingMapLoad` and fires `LoadLevelEvent`) has a run
condition:

```rust
client_process_pending_map.run_if(
    in_state(AppState::MainMenu)
        .or(in_state(AppState::Lobby))
        .or(in_state(AppState::Loading)),
),
```

This already includes `AppState::Lobby`. ✓

So the late-join flow (host sends `StartMission` → client receives in Lobby → transitions to Loading) should work
without additional changes here.

---

## Implementation Order

1. **Phase A** — Move roster to connection layer. Safe refactor, no new messages.
2. **Phase F** — Broadcast `LobbyState` in all states. Requires Phase A.
3. **Phase C** — Heartbeat. New message type. Client sender + host receiver.
4. **Phase D** — Disconnect protection. Small change in 2 existing systems.
5. **Phase B** — `HostStatus` message + client state bootstrap. New message, new systems.
6. **Phase G** — `HostStatus` updater. Depends on Phase B and C.
7. **Phase E** — Late join. Depends on all above. UI changes + spawn logic fix.
8. **Phase H** — Verify client mission handler works in Lobby (likely no changes needed).

---

## Testing Checklist

### Phase A Verification

- [ ] Host starts, lobby shows "Player 1 (Host)" immediately without opening lobby UI first
- [ ] Client joins, host is in lobby → client appears in player list
- [ ] Client joins, host is in InGame → client appears in roster (visible when host returns to lobby)

### Phase C Verification

- [ ] Client sends heartbeat continuously
- [ ] Host marks player as `PlayerInactive` after 5s silence
- [ ] Inactive player removed from "all in truck" calculation
- [ ] Ghost ignores inactive players

### Phase D Verification

- [ ] Disconnected player gets `Hiding` → ghost ignores them
- [ ] Reconnected player loses `Hiding` → ghost can see them again

### Phase E Verification

- [ ] Client in lobby, host mid-mission → lobby shows mission info + "Join Mission" button
- [ ] Client clicks "Join Mission" → transitions to Loading → enters InGame → sees their player
- [ ] Late join - fresh player → spawned at spawn point with starting gear
- [ ] Late join - reconnect, alive → resumes from where they left off
- [ ] Late join - reconnect, dead → enters as spectator
- [ ] Host finishes mission → clients in lobby see state change back to Lobby

### Full Flow

- [ ] 3 players join lobby → start mission → play
- [ ] Player 2 crashes → gets Hiding, marked Disconnected
- [ ] Player 3 goes AFK (app frozen) → marked Inactive after 5s
- [ ] Player 1 (host) + remaining active players can end mission without Player 2 and 3
- [ ] Player 2 reboots, reconnects → enters lobby → sees mission in progress → clicks "Join Mission" → resumes alive
      with Hiding removed
- [ ] Mission ends → all return to lobby → roster shows all 3 players

---

## Risk Assessment

| Risk                                                                                                                        | Severity | Mitigation                                                                                                                                                                                     |
| --------------------------------------------------------------------------------------------------------------------------- | -------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `PlayerJoinedEvent` now consumed by `session_roster_system` instead of `lobby_broadcast_state_system` — double consumption? | Medium   | Events in Bevy can be read by multiple `MessageReader`s. Both systems can read the same event. `session_roster_system` replaces the broadcast system's consumption.                            |
| `spawn_joined_player` poll-based approach spawns entities for players still loading                                         | Low      | Guard: only spawn if `LobbyPlayer.connected == true` AND player does NOT have an existing entity. The spawn uses starting gear, which is appropriate for a player who just joined mid-mission. |
| `host_status_updater_system` accessing InGame-only resources while in Lobby                                                 | Medium   | Use `Option<Res<...>>` pattern. Default to empty/zero when resources don't exist.                                                                                                              |
| `HostStatus` message adds bandwidth                                                                                         | Low      | Sent once per second, small payload. Negligible compared to 60Hz snapshots.                                                                                                                    |
| `Hiding` on disconnect could be exploited (intentional disconnect to hide from ghost)                                       | Low      | This is a cooperative game, not competitive. Players who disconnect are helping nobody. The hidden status is a QoL feature, not a balance concern.                                             |
| Heartbeat at FixedUpdate (30fps) adds message volume                                                                        | Low      | Heartbeat is ~50 bytes. At 30Hz that's ~1.5 KB/s per client. Negligible vs. the existing PlayerInput stream. Could be reduced to 10Hz if needed.                                               |
