# 06. Multiplayer State Machine Redesign Plan

- **Date:** 2026-02-26
- **Scope:** `unreplicon-plugin`, `unreplicon-core`, `unengine-core`, `unlobby-plugin`
- **Status:** Implemented — revised during execution (see §4 notes)

---

## 1. Problem Statement

The multiplayer code is experiencing cascading bugs because two independent concerns were conflated into one data type:

> **`AppState` is the client UI navigation state. It is not a description of what the server is doing.**

These two concerns happen to share many of the same names (`Lobby`, `InGame`, `Summary`) but they are not the same
thing. The server does not have a "main menu". The client does not have a concept of "mission in progress" in the same
way the server does. Encoding both meanings in the same enum and then replicating the server's raw `AppState` to the
client and blindly applying it to the client's own `NextState<AppState>` is the root cause of every observed symptom.

---

## 2. Symptom → Root Cause Map

### 2.1 Freeze on dark blue background (most common)

**Symptom:** The client freezes on startup, server and client are both running.

**Cause:** `follow_server_app_state` in `bridge.rs` runs in `Update` with no guard on the client's current `AppState`.
The client starts in `AppState::Loading` while `bevy_asset_loader` loads assets. Before loading finishes, bevy_replicon
replicates `LobbyInfo` and `ServerAppState` from the server. `Changed<ServerAppState>` fires immediately.
`follow_server_app_state` calls `next_state.set(AppState::Lobby)` while the client is still in `AppState::Loading`. The
asset loader is simultaneously trying to do `Loading → MainMenu`. Two conflicting state transitions race. The result is
undefined — bevy_states behaviour under this race condition produces a frozen app.

### 2.2 Involuntary Lobby jump without user action

**Symptom:** On rare successful runs the client went to the Lobby without clicking anything.

**Cause:** Same as above — but this time the asset loader won the race and completed first. `AppState` moved to
`MainMenu`. Then replication arrived, `follow_server_app_state` saw `ServerAppState(Lobby)` and called
`next_state.set(AppState::Lobby)`. Worked — but only by accident, and the UX is wrong (Join clients should get to Lobby
automatically, but the mechanism doing it was `follow_server_app_state`, which has no knowledge of "has the user
explicitly asked to join this server?").

### 2.3 Ghost players after session close

**Symptom:** Re-joining shows a player from a previous session that was force-closed.

**Cause:** `on_client_connected` adds to `LobbyInfo.players` on `Insert<ConnectedClient>` but there is no
`On<Remove, ConnectedClient>` observer to remove them on disconnect. Players accumulate forever until the server
restarts.

### 2.4 Empty lobby after ESC → back

**Symptom:** Going to MainMenu and returning to Lobby shows an empty player list.

**Cause:** `bridge_lobby_info_system` only runs when `Changed<LobbyInfo>` is true. Entering `AppState::Lobby` does not
cause `LobbyInfo` to change on the server (nothing changed server-side). So the bridge never fires on re-entry and
`LobbyData` retains its previous (or default empty) state.

### 2.5 Summary screen never reached by clients (latent)

**Cause:** `sync_mission_result_to_net` in `ghost.rs` writes `ServerAppState(Summary)` to signal clients.
`follow_server_app_state` then fires — but it has `AppState::InGame | AppState::Loading` as exceptions that are silently
dropped. `AppState::Summary` is handled by the `_ =>` arm so it should work. However this path is entangled with the
same timing race as 2.1: if replication of `ServerAppState(Summary)` arrives while the client is in a transition, the
behaviour is again undefined. This is a latent bug, not yet reported, but it exists in the same code.

---

## 3. Wrong Design: `ServerAppState(pub AppState)`

### What it is

`ServerAppState` is defined in `unreplicon-core/src/components.rs` as:

```rust
pub struct ServerAppState(pub AppState);
```

It is placed on the lobby entity alongside `LobbyInfo`, marked `Replicated`, and updated in three places:

| Location                                | Triggered by                       | Value written       |
| --------------------------------------- | ---------------------------------- | ------------------- |
| `setup_lobby_entity` (lobby.rs)         | `OnEnter(AppState::Lobby)` server  | `AppState::Lobby`   |
| `set_server_state_ingame` (lobby.rs)    | `OnEnter(AppState::InGame)` server | `AppState::InGame`  |
| `sync_mission_result_to_net` (ghost.rs) | `SummaryData::is_changed()` server | `AppState::Summary` |

On the client side, `follow_server_app_state` in `bridge.rs` reads `Changed<ServerAppState>` and calls
`next_state.set(server_state.0)` for all states except `InGame` and `Loading`.

### Why it is wrong

1. `AppState` is a UI navigation enum. It contains states like `MainMenu`, `SettingsMenu`, `MissionSelect`, `Hub`,
   `MapHub`, `UserManual`, `PreplayManual` — states that have no meaning on a dedicated server. The server will never be
   in those states, but encoding the server's phase as an `AppState` implies it could be.

2. `follow_server_app_state` makes the client's `AppState` a direct slave of the server's `AppState` regardless of what
   the client's current state is or whether the transition makes sense. There is no guard, no precondition, no consent
   from the client.

3. The two machines run asynchronously. The server can write `ServerAppState(Lobby)` at any time. The client may be in
   `Loading`, `MainMenu`, `InGame`, or any other state when that write arrives. The current code has no answer to "what
   should the client do if `ServerAppState` changes while the client is in `AppState::Loading`?" — it just blindly
   applies the change.

4. The three write sites are in different files with different run conditions and are not exhaustive.
   `AppState::Loading` is never written to `ServerAppState`. `AppState::Summary` is written only from ghost.rs as a
   side-effect of the summary data write. This is fragile.

---

## 4. Correct Design

### 4.1 Principle: independent state machines, explicit signals

The server's game phase and the client's UI navigation state are two independent finite state machines. The server's
machine is the authoritative source of game events. The client's machine is driven by user intent plus a small set of
explicit, guarded reactions to specific server events.

The server should never "push" its raw state to the client. Instead it should emit discrete, semantically meaningful
signals that the client can react to in a context-aware way.

### 4.2 `ServerGamePhase` — the replacement for `ServerAppState`

Define a new component in `unreplicon-core/src/components.rs`:

```rust
/// The authoritative game phase as determined by the server.
///
/// Replicated to all clients. Clients react to transitions in a context-aware way
/// (see `bridge.rs` observers) rather than blindly copying this into their own
/// `NextState<AppState>`.
///
/// This is NOT an `AppState`. It describes what the server's game session is doing,
/// not how any client's UI is laid out.
#[derive(Component, Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServerGamePhase {
    /// Server is up and accepting players. No mission is running.
    Lobby,
    /// A mission is in progress.
    InProgress,
    /// The mission has ended and results are available.
    Ended,
}
```

This replaces `ServerAppState`. It only contains states the server can actually be in. It has no coupling to `AppState`.

### 4.3 Server-side writes (replacing current `ServerAppState` writes)

| Trigger                                                   | New value                     |
| --------------------------------------------------------- | ----------------------------- |
| `OnEnter(AppState::Lobby)` server                         | `ServerGamePhase::Lobby`      |
| `OnEnter(AppState::InGame)` server                        | `ServerGamePhase::InProgress` |
| `SummaryData::is_changed()` in `AppState::Summary` server | `ServerGamePhase::Ended`      |

These replace the three write sites identically. No logic changes to the server — just the type changes from
`ServerAppState(AppState::X)` to `ServerGamePhase::X`.

### 4.4 Client-side reactions — revised principle

> **Implemented differently from the original plan.** The plan proposed a `react_to_server_game_phase` system with
> guarded transitions. During implementation this was recognised as the same disease as `follow_server_app_state`, only
> with narrower guards. It was removed entirely.

**The rule adopted:**

> Every `AppState` transition on the client must be the direct result of a player action on their own local client. The
> only exceptions are pure in-game events that would also occur in single-player (e.g. all players dying → Summary). No
> networking event of any kind may cause a client state transition.

Consequences:

- `react_to_server_game_phase` — **not implemented**. `ServerGamePhase` is replicated and available as display data, but
  no client system reads it to change `NextState<AppState>`.
- `on_selected_mission_added` observer in `bridge.rs` — **deleted**. This observer previously called
  `next_state.set(AppState::Loading)` when a `SelectedMission` component was replicated from the server, which is a
  networking event driving a state transition. It was removed.
- Mission loading is now entirely owned by click handlers in `unlobby-plugin` (see §4.5).
- `ServerGamePhase` remains replicated. It is used as **display data only** — e.g. to decide whether to show "Start
  Mission" vs "Join Mission" in the lobby UI.

### 4.5 Navigation — user-driven only

> **Implemented differently from the original plan.** The plan proposed `auto_join_to_lobby` to automatically redirect
> Join clients from `MainMenu` to `Lobby` after asset loading. This was also removed — an automatic redirect is a
> networking event driving navigation, which violates the principle in §4.4.

**What actually navigates the client:**

| Transition           | Trigger                                                                       |
| -------------------- | ----------------------------------------------------------------------------- |
| `Loading → MainMenu` | Asset loader (`unengine-core`) — unchanged, correct                           |
| `MainMenu → Lobby`   | User clicks "Multiplayer Lobby" in the main menu — unchanged                  |
| `Lobby → Loading`    | User clicks "Start Mission" (owner) or "Join Mission" (non-owner) — see below |
| `InGame → Summary`   | Game-internal event (all players dead, repellent depleted, etc.) — unchanged  |
| `Summary → Lobby`    | User clicks a button in the summary screen — unchanged                        |

**Mission loading in `unlobby-plugin/src/systems/lobby_main.rs`:**

- `host_in_mission` is derived from `!q_selected_mission.is_empty()` — a local ECS query, not a networking callback.
- **Owner, no mission running:** clicking "Start Mission" sends `RequestStartMission` to the server AND immediately
  triggers local load (`CurrentMapSeed`, `CurrentDifficulty`, `LoadLevelEvent`, `AppState::Loading`). The owner does not
  wait for replication to initiate their own load.
- **Non-owner, mission running:** the "Start Mission" button is replaced by "Join Mission". Clicking it reads
  `SelectedMission` (already replicated) to obtain the seed, difficulty, and map path, then triggers the same local load
  pipeline. This is "always late-join" — joining seconds to minutes after the owner started.
- **Non-owner, no mission running:** button is hidden.

### 4.6 Disconnect cleanup

Add an observer for `On<Remove, ConnectedClient>` in `lobby.rs`:

```rust
fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    q_network_id: Query<Option<&RepliconNetworkId>>,
    mut q_lobby: Query<&mut LobbyInfo>,
) {
    let entity = trigger.entity;
    let client_id_u64 = q_network_id
        .get(entity)
        .ok()
        .flatten()
        .map(|n| n.get())
        .unwrap_or(0);

    for mut lobby in q_lobby.iter_mut() {
        let before = lobby.players.len();
        lobby.players.retain(|p| p.client_id != client_id_u64);
        let removed = before - lobby.players.len();
        if removed > 0 {
            info!("Client {} disconnected; removed from lobby", client_id_u64);
        }

        // If the room owner disconnected, assign ownership to the next player.
        if lobby.owner_client_id == client_id_u64 {
            lobby.owner_client_id = lobby.players.first().map(|p| p.client_id).unwrap_or(0);
            info!(
                "Owner disconnected; new owner is {}",
                lobby.owner_client_id
            );
        }
    }
}
```

Registered with `app.add_observer(on_client_disconnected)` alongside `on_client_connected`.

### 4.7 Lobby data re-hydration on re-enter

`bridge_lobby_info_system` must also run when the client re-enters `AppState::Lobby`, not only when `Changed<LobbyInfo>`
fires. The fix is a second system registered on `OnEnter(AppState::Lobby)` that reads `LobbyInfo` unconditionally and
re-populates `LobbyData` and `RoomOwner`:

```rust
fn rehydrate_lobby_data_on_enter(/* same params as bridge_lobby_info_system */) {
    // Unconditional read — no Changed<> filter.
}
```

Or, simpler: split the query in `bridge_lobby_info_system` to remove the `Changed<>` filter and add it back as a run
condition instead — but that fires every frame. The cleanest solution is a separate `OnEnter(AppState::Lobby)` system
that re-runs the bridge logic once without the `Changed<>` filter.

---

## 5. Impact Map — Files Touched

| File                                       | Change                                                                                                                                                                    |
| ------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unreplicon-core/src/components.rs`        | Added `ServerGamePhase` enum; removed `ServerAppState`                                                                                                                    |
| `unreplicon-core/src/resources.rs`         | Removed `host_app_state` field from `LobbyData`                                                                                                                           |
| `unreplicon-plugin/src/systems/lobby.rs`   | Replaced `ServerAppState` with `ServerGamePhase`; added `on_client_disconnected` observer                                                                                 |
| `unreplicon-plugin/src/systems/ghost.rs`   | Replaced `ServerAppState(Summary)` write with `ServerGamePhase::Ended`                                                                                                    |
| `unreplicon-plugin/src/systems/bridge.rs`  | Deleted `follow_server_app_state`, `on_selected_mission_added`, `react_to_server_game_phase`, `auto_join_to_lobby`; kept `rehydrate_lobby_data_on_enter` (data sync only) |
| `unlobby-plugin/src/systems/lobby_main.rs` | Added full mission load pipeline to Start/Join Mission click handler; `host_in_mission` derived from ECS query on `SelectedMission`                                       |
| `unengine-core/src/plugin.rs`              | No change — `Loading → MainMenu` unchanged                                                                                                                                |

---

## 6. Step-by-Step Execution Plan

Each step is self-contained and independently compilable. The project must build clean after every step.

### Step 1 — REFACTOR markers (no logic change)

Add `// FIXME REFACTOR:` comments to every site that embodies the wrong coupling:

1. `ServerAppState` struct definition in `unreplicon-core/src/components.rs`
2. `follow_server_app_state` function in `bridge.rs`
3. All three `ServerAppState` write sites (`lobby.rs` lines ~108, ~147; `ghost.rs` line ~338)
4. `LobbyData.host_app_state` field in `unreplicon-core/src/resources.rs` (dead field — never set to a non-None value
   since the migration)

**Goal:** Make the technical debt visible in the code. No behaviour change.

### Step 2 — Disconnect cleanup

In `lobby.rs`:

- Add `on_client_disconnected` as described in 4.6
- Register it with `app.add_observer(on_client_disconnected)`

**Goal:** Ghost players are removed when their renet connection drops. **Risk:** Low. Self-contained observer.

### Step 3 — Lobby re-hydration on re-enter

In `bridge.rs`:

- Add a new `rehydrate_lobby_data_on_enter` system registered on
  `OnEnter(AppState::Lobby).run_if(not(in_state(ServerState::Running)))`
- It queries `LobbyInfo` without `Changed<>` and runs the same bridge logic

**Goal:** Returning from MainMenu to Lobby correctly shows current player list. **Risk:** Low. Additive only.

### Step 4 — Define `ServerGamePhase` and wire it server-side

In `unreplicon-core/src/components.rs`:

- Add the `ServerGamePhase` enum as specified in 4.2

In `lobby.rs` (server-side):

- In `setup_lobby_entity`: spawn the lobby entity with `ServerGamePhase::Lobby` in addition to (not yet replacing)
  `ServerAppState`
- In `set_server_state_ingame`: also write `ServerGamePhase::InProgress`

In `ghost.rs` (server-side):

- In `sync_mission_result_to_net`: also write `ServerGamePhase::Ended`

Add `app.replicate::<ServerGamePhase>()` in `lobby.rs::app_setup`.

**Goal:** `ServerGamePhase` is now replicated alongside the old `ServerAppState`. The client receives both. Nothing on
the client reacts to `ServerGamePhase` yet — it is inert. **Risk:** Low. Additive only. Both old and new components
coexist.

### Step 5 — Remove all server-driven client navigation; implement Join Mission button

In `bridge.rs`:

- Delete `follow_server_app_state` and its registration
- Delete `on_selected_mission_added` observer (no longer drives `NextState`)
- Do **not** add `react_to_server_game_phase` or `auto_join_to_lobby` — these were designed and then rejected as
  violations of the independence principle (see §4.4)
- `app_setup` now registers only: `bridge_lobby_info_system` (data sync) and `rehydrate_lobby_data_on_enter` (data sync
  on Lobby entry). Neither touches `NextState<AppState>`.

In `unlobby-plugin/src/systems/lobby_main.rs`:

- Add `q_selected_mission: Query<Entity, With<SelectedMission>>` to `update_display` — drives `host_in_mission` flag and
  "Join Mission" button visibility
- Add full mission load pipeline to `handle_clicks` for the `StartMission` action:
  - Owner path: send `RequestStartMission` + set `CurrentMapSeed` + set `CurrentDifficulty` + write `LoadLevelEvent` +
    `next_state.set(AppState::Loading)`
  - Non-owner path (mission running): read `SelectedMission` directly, same load pipeline
  - Non-owner path (no mission): button hidden

**Goal:** No networking event drives client navigation. All state transitions are explicit user actions. Freeze,
involuntary redirect, and ESC-back bugs resolved. **Risk:** Medium — main behaviour change.

### Step 6 — Remove `ServerAppState` and `LobbyData.host_app_state` (completed alongside Step 5)

- Removed `ServerAppState` from `unreplicon-core/src/components.rs`
- Removed all import and usage sites
- Removed `host_app_state` from `LobbyData` in `unreplicon-core/src/resources.rs`
- `lobby_main.rs` now derives `host_in_mission` from `q_selected_mission.is_empty()` instead

**Goal:** Dead types gone. `ServerGamePhase` is the single server-side phase indicator. **Risk:** Low — pure removal.

---

## 7. What Is Not in This Plan

These are known gaps that this plan does not address and should not be addressed in the same changeset:

- **Late-join (joining a game in progress):** Redesigned as the standard join flow. "Start Mission" becomes "Join
  Mission" for non-owners when a `SelectedMission` entity exists in the ECS. All client joins are effectively late-joins
  — the client loads independently whenever the player clicks the button, regardless of when the owner started. No
  synchronisation point is enforced.
- **Hub-mode authentication changes:** `auth.rs` hub-mode bypass is separate work.
- **Player tint colour conflicts:** When a player reconnects they may get a duplicate tint index. Out of scope.
- **`LobbyInfo` persistence on server restart / round-trip:** Out of scope.
- **`FloorGearCache` and gear replication:** Completed in prior sessions, unchanged.

---

## 8. Acceptance Criteria

After all six steps are complete and tested:

1. `cargo clippy` produces zero errors and zero warnings on any of the changed crates.
2. Starting a dedicated server and joining with a client results in: the client loading assets, arriving at `MainMenu`,
   and staying there until the user manually navigates to the Lobby. The client is never automatically redirected by the
   server or by any replication event.
3. The lobby displays the correct player(s) with correct ownership.
4. Closing the client (force-kill) removes the player from the lobby on the server within one connection timeout cycle.
5. Pressing ESC from the Lobby, returning to MainMenu, and clicking "Multiplayer Lobby" again shows the correct current
   player list without requiring a server event.
6. Starting a mission, playing, ending, and returning to the lobby for a second mission works end-to-end without freezes
   or state machine confusion.
7. `ServerAppState` does not exist anywhere in the codebase.
8. `LobbyData.host_app_state` does not exist anywhere in the codebase.
