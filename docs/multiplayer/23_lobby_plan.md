# Plan 23: Multiplayer Lobby

## Goal

Replace the current CLI-driven "connect and auto-start" flow with a lobby where players gather before a mission. The
host picks map + difficulty; clients see the selection in real-time; the host starts the mission when ready. After a
mission, players return to the lobby (not MainMenu), keeping the connection alive for multi-mission sessions.

## Current State

- **Connection:** `--host <port>` / `--join <ip:port>` CLI flags. Connection established at startup in
  `startup_network_system` (Startup schedule).
- **Auto-start:** `autostart_net_game` runs on `OnEnter(AppState::MainMenu)`. If host has `--map`, it fires
  `LoadLevelEvent` immediately. Client receives `Welcome` with map+difficulty, stores it in `PendingMapLoad`, and
  `client_process_pending_map` (running in `MainMenu` state) triggers the load.
- **Handshake:** Client sends `Hello` → Host sends `Welcome { id, map_seed, map_filepath, difficulty_id }`. The
  `Welcome` currently serves double duty: identity assignment AND mission launch.
- **Summary exit:** `keyboard()` in `unsummary-plugin` transitions to `AppState::MissionSelect` on Enter/Escape.
- **AppState variants:** Loading, MainMenu, SettingsMenu, InGame, Summary, MapHub, UserManual, PreplayManual,
  MissionSelect.
- **No lobby state exists.**

## Architecture

### New AppState

Add `AppState::Lobby` to `untypes-core/src/states.rs`.

### Protocol Changes (unnet-core/src/messages.rs)

Split the current `Welcome` into three messages:

```text
// Sent on successful handshake (replaces current Welcome for identity)
LobbyWelcome {
    id: NetworkId,
}

// Broadcast periodically from host to all clients while in lobby
LobbyState {
    players: Vec<LobbyPlayer>,         // id + tint color
    selected_map: Option<String>,       // map filepath
    selected_difficulty: Option<String>, // difficulty id
}

// Host → all clients: start the mission now
StartMission {
    map_seed: u64,
    map_filepath: String,
    difficulty_id: String,
}
```

The existing `Welcome` message stays temporarily for backward compatibility but becomes unused once the lobby is wired.

### New Resource (unnet-core or unnet-plugin)

```rust
#[derive(Resource, Default, Debug, Clone)]
pub struct LobbyData {
    pub players: Vec<LobbyPlayer>,
    pub selected_map: Option<String>,
    pub selected_difficulty: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LobbyPlayer {
    pub id: NetworkId,
    pub tint_color_index: u8,
}
```

## Phases

### Phase A: Foundation (Protocol + State)

**Files touched:** | File | Change | |------|--------| | `crates/untypes-core/src/states.rs` | Add `Lobby` variant to
`AppState` | | `crates/unnet-core/src/messages.rs` | Add `LobbyWelcome`, `LobbyState`, `StartMission` to
`NetworkMessage` enum | | `crates/unnet-core/src/resources.rs` | Add `LobbyData`, `LobbyPlayer` structs |

**Details:**

1. Add `Lobby` to `AppState` enum (after `MainMenu`).
2. Add three new variants to `NetworkMessage`.
3. Add `LobbyData` resource with `Default` impl.
4. Keep existing `Welcome` variant — don't remove yet.

**Verification:** `cargo clippy` — no logic changes, just new types.

### Phase B: Connection & Handshake Rework

**Files touched:** | File | Change | |------|--------| | `crates/unnet-plugin/src/systems/connection.rs` | Handshake
sends `LobbyWelcome` instead of `Welcome`. `autostart_net_game` → goes to Lobby instead of auto-starting. | |
`crates/unnet-plugin/src/systems/setup.rs` | Wire new systems, adjust state scheduling. |

**Details:**

1. **`handshake_handler_system`** (host side): On receiving `Hello`, send `LobbyWelcome { id }` instead of `Welcome`.
   Don't send map/difficulty — that comes later via `StartMission`.

2. **`handshake_handler_system`** (client side): On receiving `LobbyWelcome`, store `local_id`, mark handshake
   completed. Do NOT trigger map load. (Currently `Welcome` triggers `PendingMapLoad`.)

3. **`autostart_net_game`**: Remove the auto-start logic entirely. This system currently runs on
   `OnEnter(AppState::MainMenu)` and fires `LoadLevelEvent` if `--map` is provided. With the lobby, the host should
   arrive at MainMenu and wait for the user to navigate to the lobby. The `--map` and `--difficulty` CLI flags can
   pre-populate `LobbyData.selected_map` / `LobbyData.selected_difficulty` instead.

4. **Client `LobbyState` handler**: New system that listens for incoming `LobbyState` messages and updates the local
   `LobbyData` resource.

5. **Client `StartMission` handler**: New system that listens for `StartMission`, stores map in `PendingMapLoad`,
   applies difficulty, and transitions to the game. This replaces the current `Welcome`-triggered map load path.

**Verification:** `cargo clippy`. Manual test: host and client connect, arrive at MainMenu, no auto-start. Handshake
completes (check logs for `LobbyWelcome`).

### Phase C: MainMenu Modification

**Files touched:** | File | Change | |------|--------| | `crates/unmainmenu-plugin/src/mainmenu.rs` | Conditionally show
"Multiplayer Lobby" when in networked mode. Hide Campaign and Custom Mission when networked. |

**Details:**

1. `setup_ui`: Read `CliOptions` resource. If `net_mode != Offline`, replace menu items:
   - Remove `Campaign` and `CustomMission`
   - Add `MultiplayerLobby` variant to `MenuID`
   - Keep `Manual`, `Settings`, `Quit`

2. `menu_event`: Handle `MenuID::MultiplayerLobby` → `next_app_state.set(AppState::Lobby)`.

**Verification:** `cargo clippy`. Manual test: launch with `--host`, MainMenu shows "Multiplayer Lobby" instead of
Campaign/Custom Mission.

### Phase D: Lobby UI & Systems

This is the largest phase. It needs a new crate or new module.

**Option A — New crate:** `unlobby-plugin` + optionally `unlobby-core`. **Option B — Add to `unnet-plugin`.**

Recommendation: **Option A (new crate)**. The lobby is a distinct screen with its own UI, like `unmainmenu-plugin` or
`unmaphub-plugin`. It doesn't belong in the networking crate.

**New crate: `crates/unlobby-plugin`**

Structure:

```text
crates/unlobby-plugin/
  Cargo.toml
  src/
    lib.rs          # mod statements only
    plugin.rs       # Plugin impl
    systems.rs      # app_setup, lobby systems
    lobby_ui.rs     # UI setup and update
```

**Plugin responsibilities:**

- `OnEnter(AppState::Lobby)`: Setup lobby UI (camera, layout).
- `OnExit(AppState::Lobby)`: Cleanup lobby UI.
- `Update` systems (gated by `in_state(AppState::Lobby)`):
  - **Host: `lobby_broadcast_state`** — periodically send `LobbyState` to all clients (every ~500ms or on change).
  - **Host: map/difficulty selection UI** — reuse the map list from `Maps` resource and difficulty list from
    `undifficulty-core`. Host can browse maps and pick difficulty. Selection updates `LobbyData`.
  - **Client: display lobby state** — render `LobbyData` (players, selected map, selected difficulty). Read-only for
    clients.
  - **Host: "Start Mission" button** — triggers `StartMission` broadcast and local state transition.
  - **Handle incoming `LobbyState`** (client) and **`StartMission`** (client) messages.

**UI layout (simple first pass):**

```text
┌─────────────────────────────────────┐
│           MULTIPLAYER LOBBY         │
├─────────────────────────────────────┤
│  Players:                           │
│    ● Player 1 (Host)   [red tint]   │
│    ● Player 2          [blue tint]  │
│    ● Player 3          [green tint] │
├─────────────────────────────────────┤
│  Map: [Riverside Manor ▾]  (host)   │
│  Difficulty: [Medium ▾]    (host)   │
│                                     │
│  Map: Riverside Manor     (client)  │
│  Difficulty: Medium       (client)  │
├─────────────────────────────────────┤
│  [Start Mission]          (host)    │
│  [Back to Menu]                     │
└─────────────────────────────────────┘
```

For the map selection, the simplest approach is to list map names as text menu items (not the full thumbnail browser).
The existing `Maps` resource already has the list. The full thumbnail browser can come later.

**Key interaction patterns:**

- Host uses Up/Down to browse maps, Left/Right to cycle difficulty.
- Enter on "Start Mission" triggers the mission.
- Escape goes back to MainMenu.
- Client sees the same layout but map/difficulty are display-only.

**Files touched (outside the new crate):** | File | Change | |------|--------| | `unhaunter/src/app.rs` | Add
`UnhaunterLobbyPlugin` | | `Cargo.toml` (workspace) | Add `unlobby-plugin` to workspace members | |
`unhaunter/Cargo.toml` | Add `unlobby-plugin` dependency | | `PROJECT_FILE_DESCRIPTIONS.md` | Document new crate |

**Verification:** `cargo clippy`. Manual test: navigate to lobby from MainMenu, see player list, host can browse maps
and difficulties, start a mission.

### Phase E: Mission Start Flow

**Files touched:** | File | Change | |------|--------| | `crates/unlobby-plugin/src/systems.rs` | Host: on "Start
Mission", broadcast `StartMission`, fire `LoadLevelEvent`, transition to InGame | |
`crates/unnet-plugin/src/systems/connection.rs` or `client_input.rs` | Client: on receiving `StartMission`, store in
`PendingMapLoad`, transition states | | `crates/unnet-plugin/src/systems/setup.rs` | Ensure `client_process_pending_map`
runs in `Lobby` state too (or in a broader context) |

**Details:**

1. **Host side (in lobby):**
   - User clicks "Start Mission".
   - System reads `LobbyData.selected_map` and `LobbyData.selected_difficulty`.
   - Broadcasts `StartMission { map_seed, map_filepath, difficulty_id }` to all clients.
   - Fires `LoadLevelEvent` locally.
   - Also updates `cli.map_path` and `cli.difficulty_id` so existing host systems (orchestrator, etc.) work unchanged.

2. **Client side:**
   - New handler for `StartMission` message: stores map in `PendingMapLoad`, applies difficulty to `CurrentDifficulty`.
   - `client_process_pending_map` is currently gated to `AppState::MainMenu`. It needs to also run in `AppState::Lobby`.
     Change: `.run_if(in_state(AppState::MainMenu).or(in_state(AppState::Lobby)))` or move to a more general schedule.

**Verification:** `cargo clippy`. Manual test: host starts mission from lobby, both host and client enter the game.

### Phase F: Return to Lobby After Mission

**Files touched:** | File | Change | |------|--------| | `crates/unsummary-plugin/src/summary.rs` | If networked, go to
`AppState::Lobby` instead of `AppState::MissionSelect` |

**Details:**

1. In `keyboard()`: check `CliOptions` resource. If `net_mode != Offline`, set `AppState::Lobby` instead of
   `AppState::MissionSelect`.

2. The connection (`NetworkConn`) persists across states — it's a resource, not tied to a specific `AppState`. So
   returning to lobby should just work.

3. The host's `LobbyData` should be preserved (map/difficulty selection remembered).

4. Game cleanup (despawning entities, resetting resources) already happens on `OnExit(AppState::InGame)` and
   `OnEnter(AppState::Summary)` via existing systems.

**Verification:** `cargo clippy`. Manual test: complete a mission, press Enter at Summary, arrive back in lobby with
connection intact, players still listed, can start another mission.

## Risks & Open Questions

1. **Map list availability on client:** The `Maps` resource is populated from asset loading. Both host and client have
   the full map list locally. The lobby only needs to display the host's selection (a string), not browse locally on the
   client. So this should be fine.

2. **Game cleanup between missions:** Existing cleanup systems should handle despawning game entities. The concern about
   ndarray size mismatches exists in solo play too and isn't a lobby blocker.

3. **Lobby UI complexity:** The first pass should be intentionally minimal — text-based menu, no thumbnails, no
   animations. Polish later.

4. **`client_process_pending_map` state gating:** Currently only runs in `MainMenu`. Must also run in `Lobby`. This is a
   small change but easy to miss.

5. **Host player entity in lobby:** The host's player entity is spawned by the orchestrator when the mission starts, not
   in the lobby. So the lobby is purely UI — no game entities exist yet.

6. **New client joining mid-lobby:** The handshake sends `LobbyWelcome`, and the next `LobbyState` broadcast updates
   them. This should work naturally.

7. **Client joining during a mission (late join):** Out of scope for this plan. Currently if someone joins after
   `StartMission`, they'll get `LobbyWelcome` but no `StartMission`. Handling this needs a "game in progress" state
   check in the handshake. Defer to a future plan.

## Implementation Order

1. Phase A (types) — foundation, zero risk
2. Phase B (connection rework) — changes handshake, moderate risk
3. Phase C (main menu) — small UI change
4. Phase D (lobby crate + UI) — largest phase, most new code
5. Phase E (mission start) — wires lobby to game launch
6. Phase F (return to lobby) — small change in summary

Phases A–C can be done together as they're small. Phase D is the bulk. Phase E follows naturally. Phase F is trivial
once the rest works.

## Estimated Scope

- **Phase A:** ~20 lines across 3 files
- **Phase B:** ~80 lines changed in connection.rs, ~10 in setup.rs
- **Phase C:** ~30 lines in mainmenu.rs
- **Phase D:** ~400-500 lines in new crate (plugin, systems, UI)
- **Phase E:** ~50 lines across lobby + net systems
- **Phase F:** ~10 lines in summary.rs

Total: ~600-700 lines of new/changed code. Most of it is the lobby UI (Phase D).
