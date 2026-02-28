# 07_target_architecture.md

## High-Level Vision

This document outlines the planned state machine refactor intended to solve architectural violations from
`06_design_audit.md`. The core premise is entirely separating "Engine Booting", "Map Simulation Lifecycle", and "User
Experience (UX)".

## 1. BootState (Solving the F-09 "Empty Maps" Problem)

The initial setup phase must become a strict, universal, 1-way gate.

### Problem Addressed (F-09)

Currently, `Res<Maps>` is hot-loaded dynamically over several frames. Because `bevy_asset_loader` uses the cyclical
`AppState::Loading`, fast asset loading allows the game/server to transition to `AppState::MainMenu` or
`AppState::Lobby` before `Res<Maps>` is actually complete. This creates missing maps, empty lists on slow computers, and
crashes on the server.

### 1 - The Solution

- **`enum BootState { Loading, Ready }`**: A strict, un-cyclical state machine layer.
- **Behavior**: It blocks the _entire_ program from running its UX systems or joining/hosting multiplayer matches until
  `Res<Maps>` is definitively populated.
- **Interaction with Asset Loader**: The UI lens will still use `bevy_asset_loader` during a loading phase, but the
  actual transition to "Start Menu" or "Listen Server" is firmly gated by `BootState::Ready`.

## 2. MapSimulationState (Solving the F-10/F-11 Temporal Coupling Problems)

The dedicated server and locally-run games must manage fast-path arrays explicitly, completely ignorant of UX states.

### Problem Addressed (F-10 / F-11)

Highly-coupled engine ndarrays (like `CollisionField` and `TemperatureField`) are resized and populated across multiple
frames during a map load. Blindly relying on `AppState::InGame` implies these arrays are ready, but there is no explicit
system to guarantee it. Attempting an uncontrolled "wait until the resource exists" pattern creates race conditions that
crash the fast-path (bounds-check-omitted) array queries.

### 2 - The Solution

- **`enum MapSimulationState { Unloaded, Loading, Ready, TearingDown }`**: A cyclical state strictly for engine
  invariants.
- **Unloaded**: The resting state. Arrays are minimal or clear.
- **Loading**: A mission starts. Here, `.tmx` maps are iterated, and heavy `Array3` resources (`bf.map_size`, `p.bcf`)
  are synchronously sized and allocated. No simulation ticks run.
- **Ready**: The ironclad contract. By the time `Ready` is entered, the engine guarantees that all arrays match and
  entities are properly spawned. Engine systems `.run_if(in_state(MapSimulationState::Ready))` safely using
  `unsafe_get`.
- **TearingDown**: Stop ticks. Despawn board entities. Zero arrays.

### F-10 vs F-11: Ghost Spawning and the Late-Join Exception

F-10 (ghost entities) and F-11 (player entities) are addressed differently and must not be conflated:

- **Ghosts (F-10):** Ghost entities have no concept of "late arrival." They are a fixed property of the map. Ghost
  spawning belongs strictly to the `MapSimulationState::Loading → Ready` transition. By the time `Ready` is entered, all
  ghost entities are guaranteed to exist and carry `Replicated`.

- **Player Avatars (F-11, partial):** Player entity spawning is **explicitly exempt** from the `MapSimulationState`
  boundary. Players can join a mission already in progress (late join). A player avatar must be spawned when the
  player's network connection reaches the game — not when the map finishes loading. The temporal precondition fix for
  `PlayerSpawnPoint` (F-11a) is solved by `MapSimulationState` (spawn points exist by `Ready`), but the player avatar
  itself is spawned on-demand at connection time.

  The second half of F-11 (F-11b) — the topology assumption that a host `PlayerSprite` exists on a dedicated server — is
  addressed by the Role System in Section 5: the host-player initialization path must be a distinct system guarded by
  `LocalPlayerRole`, so it is completely absent on a dedicated server rather than silently iterating zero entities.

## 3. AppState (The UX Lens)

`AppState` remains, but strictly controls user experience.

### Addressed

- _Law 2 (State Machine Independence)_
- _Law 6 (The Illiterate Engine)_

### 3 - The Solution

Servers fundamentally do not possess `AppState` (e.g., `MainMenu`, `Summary`). Those states are strictly local client
choices on how to visualize facts in the engine.

#### Fixing F-07 & F-13: The Mission End Flow (Soft Landing & Data Races)

**The Original Problem:** When a mission ends, the server flips a `ready=true` boolean on a networked component. The
client instantly reacts to this by hard-cutting `AppState` to `Summary`. This manifests in two critical ways:

- **F-07:** Violates Law 2 (Server forcibly driving Client UX). It creates a jarring instant cut while players are
  playing, and opens network data races.
- **F-13:** The checking system (`apply_mission_result_net`) lacks `AppState` guards and runs continually in `Update`.
  If a stale component is caught, it wildly navigates the game to the summary screen from _any_ menu or state.

> **The Solution: The Decoupled "Soft Landing" Flow**

The mission ending sequence must separate "Calculation Time" (when the simulation ends) from "Presentation Time" (when
the user sees the summary).

1. **The Trigger & Server Freeze:**
   - A player holds "End Mission" in the truck, or a Total Party Kill (TPK) occurs.
   - The Server immediately stops the simulation by transitioning `MapSimulationState` from `Playing` to `Concluding`
     (or `TearingDown`). All ticking AI and temperature systems pause.
   - Using this frozen snapshot of the simulation, the Server calculates final scores, survived players, and money
     earned.
   - The Server publishes this data into a replicated `SummaryData` component/resource.
   - It changes a replicated orchestrator resource (e.g. `ServerGamePhase`) to `Concluding`.

2. **The Client Cinematic (UX Autonomy):**
   - The Client observes `ServerGamePhase::Concluding` and initiates a localized UX cutscene.
   - It plays an aesthetic sound locally (e.g. a walkie-talkie clip saying "We're leaving!").
   - It initiates a 2-3 second visual fade-to-black.
   - It disables player inputs (like exiting the truck).

3. **The Data-Guarded Transition:**
   - The Client's transition to the Summary screen is gated by two explicit conditions:
     1. `.run_if(cinematic_timer_finished)`
     2. `.run_if(resource_exists::<SummaryData>)`
   - If the player has a 2000ms ping and the `SummaryData` component hasn't arrived yet, the screen just safely holds on
     black until the data arrives. No empty stats can ever be rendered.

4. **Server Teardown:**
   - The Server waits a generous grace period (e.g. 5 seconds) to ensure the `SummaryData` packet correctly replicates
     to all clients, then despawns the board entities and sets its phase to `Lobby`.

#### Fixing F-08: The Return to Lobby

- The path from `AppState::Summary` back to `AppState::Lobby` must be driven by **local intentional user action**.
- Pressing "Continue" or Enter on the Summary screen fires a local system that executes
  `next_state.set(AppState::Lobby)`.
- The client seamlessly lands back in the lobby, reading the replicated `LobbyInfo` that the server has already
  transitioned and maintains. There is no forced transition driven by the server's state.

**AFK Fallback Timer:** `AppState::Summary` must also maintain a countdown timer (approximately 30 seconds) that
automatically executes `next_state.set(AppState::Lobby)` if no input is received. This prevents players from being
stranded on the summary screen indefinitely due to inattention or a broken input device. The timer resets if any
relevant input is detected. The automatic transition is identical to the manual one — no special case.

#### Fixing F-12: Eradicating `AppState::Loading` Abuse (CRITICAL)

**The Original Problem:** The audit originally marked F-12 as MINOR, but this is a **CRITICAL** architectural flaw.
Currently, UI flows (like the manual screen) trigger a mission start by writing `LoadLevelEvent` and simultaneously
setting `AppState::Loading`. `AppState::Loading` is the engine's _initial boot state_ wired into `bevy_asset_loader` to
transition to `AppState::MainMenu`. Forcing the entire app back into its boot state mid-game is extremely dangerous; it
only bypasses total failure right now because the assets are already cached, tricking the loader into immediately
finishing and violently thrashing the state machine.

> **The Solution: Distinct Mission Loading UX**

1. **Rename the Boot State:** We will heavily enforce the 1-way nature of the boot process defined in section 1
   (`BootState`) by renaming the existing UX state from `AppState::Loading` to `AppState::EngineBoot`. It will be
   entirely illegal to transition to this state after the opening seconds of the program.
2. **Dedicated Mission Loading State:** We introduce `AppState::MissionLoading`. When a player initiates a match, the
   Local Player's UX safely enters `MissionLoading` (drawing a black screen or spinner).
3. **State Observation over `LoadLevelEvent`:** The UX waits. The Authority Node orchestrates the actual heavy lifting
   behind the scenes using `MapSimulationState::Loading`. Once the Authority Node finalizes the map arrays and
   transitions to `MapSimulationState::Ready`, the Client-side systems observe this readiness and transition the UI from
   `AppState::MissionLoading` into `AppState::InGame`. The temporary event `LoadLevelEvent` is deleted completely in
   favor of reacting to this persistent state change.

## 4. Identity & Vocabulary (Law 5)

The concept of a player's identity and permissions must be fundamentally decoupled from network topology indicators
(like the "Host" label) and temporary network sockets.

### Fixing F-14: The Overloaded `0` ID & Lobby Ownership

**The Original Problem:** `LobbyInfo` stores `owner_client_id: u64`. The value `0` is dangerously mathematically
overloaded to mean:

1. The server process itself (on a dedicated server).
2. The local human player (on a listen-server/PeerHost).
3. "No owner assigned yet." Because of this, `if owner == 0` behaves unpredictably across different network topologies.
   Furthermore, mapping a socket's `u64` to identity means a dropped socket permanently breaks the player's connection
   to their logic.

**The Solution: Persistent Identity (`installation_id`)**

We abandon ephemeral socket IDs (and the `0` hack) for tracking game logic. We instead use the player's persistent
identity (`installation_id`, currently a `Uuid`).

1. **`LobbyInfo` Data Model Changes:**
   - `LobbyPlayerInfo` changes: it no longer tracks the `u64` socket. Instead, it tracks `player_uuid: Uuid` as the
     primary key, and `current_socket: Option<ClientId>` to track their current physical connection status.
   - `owner_client_id` is renamed and re-typed to `leader_uuid: Option<Uuid>`.

2. **The Logic Flow:**
   - **No Magic Numbers:** `None` definitively means "No leader exists".
   - **Dedicated Server Boot:** The lobby naturally initializes with `leader_uuid = None`.
   - **Player Joins:** The server checks the joining player's `installation_id`.
     - _First join:_ If `leader_uuid` is `None`, the server assigns `leader_uuid = Some(joining_uuid)`.
     - _Reconnects:_ If the UUID is already in the `players` list, the server recognizes it as a reconnect. It simply
       updates `current_socket = Some(new_client_id)`. The player instantly regains their privileges and entity
       possession without losing progress.
   - **PeerHost Server Boot:** The host is treated identical to any other player. They register their own
     `installation_id` into the lobby, and the logic cleanly assigns them the leader role based on the exact same rules
     as remote clients.
   - **Player Disconnects:** The server sets `current_socket = None` for that player. If the disconnecting player’s UUID
     matches `leader_uuid`, the server sets `leader_uuid = None`, and assigns it to the next connected player in the
     list.
   - **Message Validation:** Security guards simply check:
     `if request_sender_uuid != lobby.leader_uuid { ignore_packet() }`.

This completes the `TODO Phase 1.4` (using `installation_id` as the stable identity), fixes reconnects, and permanently
eliminates the `0` topology conflation.

### Fixing F-15: Eradication of `LobbyData` (The Split-Brain Bridge)

**The Original Problem:** Currently, there are two distinct memories of the lobby state:

1. `LobbyInfo`: The authoritative networked `Component` owned by the Server.
2. `LobbyData`: A `Resource` owned by the Client UI.

A system (`bridge_lobby_info_system`) constantly copies data from the `Component` into the `Resource`. This violates Law
5 (Separation of Meanings) because there is no single source of truth—the UI reads stale cloned data, causing
initialization races, frame-delay flickering, and deep ambiguity about where data actually lives. When the game was
played in Offline (Single-Player) mode, the network `LobbyInfo` component was never even spawned, and the game silently
relied on the `Default::default()` implementation of the `LobbyData` Resource.

#### The Solution: Unified Authority Data

We will completely eradicate the `LobbyData` Resource and its accompanying `bridge` synchronization system. The UI will
directly observe the universal "Game Session Ledger".

1. **Mandatory Universal Spawning (`Offline` Mode Included):**
   - The idea that `LobbyInfo` is "just a network struct" is abandoned. It is the authoritative ledger of the game.
   - Even in `Offline` single-player mode, the orchestrator MUST spawn an entity with the `LobbyInfo` component upon
     entering the Lobby phase.
   - For offline mode, this immediately inserts the local player into the list and sets them as the `leader_uuid`.
   - This ensures that Single-Player and Multiplayer routes use the exact same data structures.

2. **Direct UI Querying & Explicit Boilerplate:**
   - Instead of reading `Res<LobbyData>`, all UI systems (like drawing the player list or map preview) must explicitly
     query the ECS for the component: `q_lobby: Query<&LobbyInfo>`.
   - Do NOT create a "clever" abstraction, trait, or `SystemParam` to hide this.
   - Embrace the boilerplate: UI systems must explicitly write the guard
     `let Ok(lobby) = q_lobby.get_single() else { return; };` so they intentionally gracefully sleep or render a blank
     state if the orchestrator hasn't spawned the Lobby entity yet.

3. **Strict Network Message Write-Path:**
   - Removing `LobbyData` does NOT mean the UI systems gain the right to forcefully mutate `&mut LobbyInfo`.
   - To make a change (e.g., clicking "Select Map"), the UI must keep firing events like `RequestSelectMap`.
   - For remote clients, this travels over the wire.
   - For PeerHost or Offline players, `bevy_replicon` natively routes these to the local `handle_request_select_map`
     system.
   - The authoritative handler receives the event, validates the `leader_uuid`, mutates the `LobbyInfo` component, and
     then the updated state is picked up on the very next frame by the `Query<&LobbyInfo>` UI read systems. The
     write-path remains completely uniform regardless of topology.

### Fixing F-16: Clarified Vocabulary

We strictly enforce Law 5 (Separation of Meanings) by replacing the overloaded word "Host" with exact terminology. We
will rename the CLI flag to `--peer-host`, but inside the system logic, we will strictly use the following terms (and
accompanying capability roles):

- **The Authority Node (`AuthorityRole`)**: The machine processing the simulation algorithms and authoritative state.
  This applies to both Dedicated Servers AND the server-half of a Peer-Hosted process.
- **The Client (`LocalPlayerRole`)**: The local process rendering the game and taking inputs. This applies to remote
  clients AND the client-half of a Peer-Hosted process.
- **Peer-Hosted Mode**: The network topology where an Authority Node and a Client run simultaneously inside the same OS
  process.
- **The Lobby Leader (`LobbyLeaderRole`)**: The specific human player (identified by `leader_uuid`) who holds the UI
  permissions to start missions or select maps.

Developers must not write systems that check "if I am the host". They must ask: "Am I the Authority Node?", "Am I
drawing a screen for a Local Player?", or "Does this Local Player currently hold the Lobby Leader permission?".

## 5. Roles & Topology (Law 1)

The codebase currently relies heavily on `cli.is_headless()` and `cli.net_mode` to branch logic. This makes the code
brittle and requires developers to hold the entire network matrix in their heads.

### Fixing F-01, F-02, F-03, F-05, F-06: The Role System (And Open Questions)

We will replace all `cli.is_headless()` and `NetMode` branching with **Role Resources**. A Role is a minimal struct
inserted into the `App` during boot based on the deployment parameters. Systems then query for the _Role_ they require,
utilizing Bevy's `run_if` (or `If<Res<Role>>` system arguments).

**Defined Roles:**

- `AuthorityRole`: Inserted if this instance is running the server logic (Dedicated Server OR PeerHost). Systems that
  spawn authoritative entities, govern AI, or validate network messages require this role. Allows abandoning
  `cli.is_headless()`.
- `LocalPlayerRole`: Inserted if this instance has a human looking at a screen (Client OR PeerHost). UI systems, audio
  systems, and local input capture require this. Allows abandoning `cli.net_mode == Offline | Host`.
- `LobbyPresenceRole`: Inserted if the current session operates within a multiplayer lobby flow (Client, Dedicated
  Server, PeerHost).

**The Breakdown of Fixes:**

- **F-02 (`setup_lobby_entity` branching):** Fixed inherently by the architecture from F-14 & F-15. The server always
  spawns an empty lobby (as if headless). Then, if the process holds the `LocalPlayerRole`, a separate system uses the
  profile's `installation_id` to add themselves to the participant list. No topology branching necessary.

**New Unsolved Findings to Track:** During analysis of F-01, F-03, F-05, and F-06, we identified deeper structural
issues that are recorded here for a future refactor phase, without immediate solutions dictated:

- **F-03 (LocalPlayer resource redesign — Deferred):** `bridge.rs::set_local_player_system` reads `NetMode` to decide
  which `ClientId` to bind to the `LocalPlayer` resource. The naive fix of "only insert `LocalPlayer` if
  `LocalPlayerRole` exists" does not address the underlying issue: the `LocalPlayer` resource design needs a deeper
  rethink to handle offline, PeerHost, and Join modes uniformly. Deferred to a later refactor phase; do not treat as
  solved in Phase 1.
- **F-20 (Server-Side UX Coupling / F-01 Evolution):** `auto_start_headless_lobby` reveals that the dedicated server is
  deeply coupled to UX concepts (`AppState::Lobby`). The fact that a headless server manually advances a UI state
  machine just to allow networking to start is a smell that needs untangling (likely via the previously discussed
  existence-driven or Simulation State pattern).
- **F-21 (Missing State Breadcrumbs / F-05 & F-06 Evolution):** The pause menu and summary screen using `NetMode` to
  decide "where to go next" exposes that the game has no "Breadcrumb" or "History" concept. The game doesn't know if we
  came from a Campaign, single-player mission, or a Multiplayer Lobby, so it tries to guess based on the network socket
  state. We need a proper context/breadcrumb mechanic to remember where to return.
- **F-22 (Missing Lobby Disconnect UI):** Exploring F-05 revealed that if a continuous lobby _is_ tracked, we currently
  lack the UI/mechanics for a player to gracefully "Disconnect from Lobby" and return to the root main menu.

## 6. Structural Documentation (Law 5 / General)

### F-17: Server State Machine Documentation

**The Problem:** The headless server's lifecycle currently lacks an explicit, documented state machine. Unlike the
client, which flows cleanly through `EngineBoot -> MainMenu -> Lobby -> InGame`, the server's lifecycle is scattered
across system run conditions.

**The Target Goal:** As part of the upcoming refactor implementing `BootState` and `MapSimulationState`, we must create
a centralized piece of documentation. This should exist as a formal design document (e.g.,
`docs/multiplayer/server_lifecycle.md`) or a comprehensive top-level docstring in the root server orchestration plugin.
It must define the exact state progression a dedicated server expects to make from Process Start -> Ready for
Connections -> Loading Mission -> Running Mission -> Discarding Map -> Returning to Lobby.

### F-18: The `untmxmap-plugin` Visual/Logic Spaghettification (Deferred)

**The Problem:** Closely related to the `F-04` headless map loading bug, the `load_level_handler` system currently
executes two completely different logical tracks inside `bevy_load_map` based on `cli.is_headless()`. The headless
server requires dozens of graphical system parameters (like `Assets<TextureAtlasLayout>`) just to execute an empty `if`
block, masking the contract.

**The Target Goal (Deferred):** Ultimately, the map loading pipeline should be cleanly separated into strictly logical
`TTM` parsing (which both Node types perform) and visual `TTM` texturing (which dynamically only runs using the
`LocalPlayerRole`). However, because resolving this requires tearing apart the core `.tmx` loading engine which is
structurally complex, this architectural fix is explicitly deferred to a later phase (only the immediate
crash-prevention aspects of F-04 will be addressed in phase 1).

### F-19: Unified Player Movement — One Path, One Contract

**The Problem:** There are currently two completely separate functions that both solve "How does a local player's
position reach the Authority Node's `NetworkPosition`?":

1. `sync_player_state_to_net` — runs under `.run_if(in_state(ServerState::Running))`. When the process holds the
   `AuthorityRole` (PeerHost), it directly reads `&mut NetworkPosition` and overwrites it from `&Position` in the same
   memory space. No message is sent. This is a "God bypass" that directly mutates the authoritative component.
2. `send_local_player_position` — runs under `.run_if(not(in_state(ServerState::Running)))`. When the process is a pure
   Client, it serializes `Position` into a `PlayerMoveMessage` and sends it over the socket. The server then validates
   and writes `NetworkPosition`.

This is a direct Law 1 violation: the game logic forks based on network topology (`ServerState::Running`), and the two
paths are entirely siloed. No single location declares the contract "a player's `NetworkPosition` is always updated by
exactly one system." If a tagging bug causes `MainPlayer` to exist on a Join client while `ServerState::Running` is also
somehow active, both systems race to write the same component.

> **The Solution: Route All Movement Through the Message Path**

`bevy_replicon` handles local message routing natively: when a client message is sent during PeerHost mode, the library
routes it to the server-side handler in the same frame without touching any socket. This means the message path is
already zero-cost in offline and PeerHost topologies.

The fix is therefore:

1. **Delete `sync_player_state_to_net` entirely.**
2. **Remove the topology guard** `.run_if(not(in_state(ServerState::Running)))` from `send_local_player_position`. Every
   process with a `LocalPlayerRole` unconditionally sends `PlayerMoveMessage` every frame.
3. The server-side `handle_player_move` handler receives the message and writes `NetworkPosition`. It is the single,
   canonical write-path.

The result: one function, one contract, zero topology checks.

---

### Audit Scorecard

| Finding                                                   | Status                                                                          | Notes                                                                                                               |
| --------------------------------------------------------- | ------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **F-01** `auto_start_headless_lobby` reads `CliOptions`   | Deferred (F-20)                                                                 | Will be solved once `AuthorityRole` replaces `cli.is_headless()` in Phase 1 LAW 1 pass.                             |
| **F-02** `setup_lobby_entity` branches on `is_headless`   | Addressed (Section 5)                                                           | Unified lobby spawning via F-14/F-15; `LocalPlayerRole` adds the local player entry.                                |
| **F-03** `set_local_player_system` matches `NetMode`      | Deferred (Section 5)                                                            | `LocalPlayer` resource needs deeper redesign; do not patch in Phase 1.                                              |
| **F-04** `load_level_handler` hides headless switch       | **Phase 1: crash-prevention guard only.** Architectural split deferred to F-18. | Adding a guard so empty-handle execution is skipped; full visual/logic separation is F-18.                          |
| **F-05** Summary screen navigates by `NetMode`            | Addressed (Section 5)                                                           | `LobbyPresenceRole` resource answers "is there a lobby to return to?" without topology check.                       |
| **F-06** Pause menu navigates by `NetMode`                | Addressed (Section 5)                                                           | Same fix as F-05.                                                                                                   |
| **F-07** `apply_mission_result_net` drives client state   | Addressed (Section 3)                                                           | Soft Landing flow: server sets `ServerGamePhase::Concluding`; client cinematic + data guard owns the transition.    |
| **F-08** No path back to lobby after mission              | Addressed (Section 3)                                                           | Client-side input-driven navigation; 30-second AFK fallback timer added.                                            |
| **F-09** `Res<Maps>` empty on server                      | Addressed (Section 1)                                                           | `BootState::Ready` guarantees `Maps` is populated before any session logic runs.                                    |
| **F-10** `setup_ghost_entities` temporal precondition     | Addressed (Section 2)                                                           | Ghosts spawn at `MapSimulationState::Loading → Ready` boundary; always present before `Ready`.                      |
| **F-11a** `PlayerSpawnPoint` not guaranteed present       | Addressed (Section 2)                                                           | Spawn points exist by `MapSimulationState::Ready`, so player placement query is safe.                               |
| **F-11b** Host `PlayerSprite` assumed on dedicated server | Addressed (Section 2 + 5)                                                       | Host-player init moved to a distinct system guarded by `LocalPlayerRole`.                                           |
| **F-12** `AppState::Loading` abused as mission trigger    | Addressed (Section 3)                                                           | Renamed to `AppState::EngineBoot` (one-way); dedicated `AppState::MissionLoading` introduced.                       |
| **F-13** `apply_mission_result_net` missing state guard   | Addressed (Section 3)                                                           | Soft Landing replaces the system; `AppState::InGame` guard prevents stale triggers.                                 |
| **F-14** `owner_client_id: u64` overloaded `0` sentinel   | Addressed (Section 4)                                                           | Replaced by `leader_uuid: Option<Uuid>`. `None` = no leader; no magic numbers.                                      |
| **F-15** `LobbyData` vs `LobbyInfo` split-brain           | Addressed (Section 4)                                                           | `LobbyData` deleted; all UI queries `Query<&LobbyInfo>` directly.                                                   |
| **F-16** "Host" means three different roles               | Addressed (Section 4)                                                           | Strict vocabulary enforced: `AuthorityRole`, `LocalPlayerRole`, `LobbyLeaderRole`.                                  |
| **F-17** No server state machine documentation            | Deferred (Section 6)                                                            | Target: `docs/multiplayer/server_lifecycle.md` written alongside `BootState` / `MapSimulationState` implementation. |
| **F-18** `load_level_handler` visual/logic spaghetti      | Deferred (Section 6)                                                            | Deep separation of TTM parsing vs texturing deferred to a later phase.                                              |
| **F-19** Dual player movement paths                       | Addressed (Section 6)                                                           | `sync_player_state_to_net` deleted; all `LocalPlayerRole` movement routes through `PlayerMoveMessage`.              |
| **F-20** Server coupled to UX `AppState::Lobby`           | Open — future phase                                                             | Needs existence-driven or simulation-state pattern to remove the UX state dependency.                               |
| **F-21** Missing state breadcrumbs                        | Open — future phase                                                             | Proper context/history mechanism needed so screens know where to return without reading topology.                   |
| **F-22** No lobby disconnect UI                           | Open — future phase                                                             | Explicit "Leave Lobby" path and UX flow needed.                                                                     |

---

## 7. Deferred Work & Open Questions (Phase 2+ Kickstarter)

This section consolidates everything that was **explicitly deferred** from Phase 1 or **identified but left without a
design**. Its purpose is to serve as the starting point for the next design conversation once Phase 1 work is complete —
so that context doesn't need to be reconstructed from scratch.

### 7.1 Explicitly Deferred from Phase 1

These items were discussed, their correct architectural solution is known in principle, but the cost/risk of
implementing them during Phase 1 was judged too high. They should be addressed in the next phase.

---

#### D-01 · F-18 — Full Visual/Logic Separation in `untmxmap-plugin`

**What was deferred:** `load_level_handler` inside `bevy_load_map` executes two entirely different code paths depending
on `cli.is_headless()`. The correct fix is two separate registered systems: one for `LocalPlayerRole` (loads textures
and geometry) and one for `AuthorityRole`-only (loads geometry only). This would also eliminate the
`AtlasData::Headless` sentinel variant.

**Why deferred:** The `.tmx` loading engine is structurally complex. Tearing it apart risks destabilizing the existing
single-player flow during the Phase 1 window.

**Phase 1 stand-in (F-04):** A crash-prevention guard will be added so the handler detects empty asset handles and
returns early rather than silently producing broken map state. This is a patch, not a fix.

**Starting point for Phase 2:** The split should follow the plugin registration pattern already used in
`unmapload-plugin/src/plugin.rs` — the `if is_headless { }` block there is the natural place to register the two
separate systems. Once `AuthorityRole` / `LocalPlayerRole` resources exist (Phase 1), the condition becomes
`.run_if(resource_exists::<LocalPlayerRole>())`.

---

#### D-02 · F-03 — `LocalPlayer` Resource Redesign

**What was deferred:** `bridge.rs::set_local_player_system` reads `NetMode` variants to decide which `ClientId` to bind
to the `LocalPlayer` resource. The Phase 1 Role System does not solve this — the `LocalPlayer` concept itself needs a
deeper rethink.

**The unresolved tension:** In offline and PeerHost modes, `LocalPlayer` must hold a meaningful value immediately at
startup, before any network transport is initialized. In Join mode, the value is only known once the handshake
completes. A resource that might be `None` post-boot is architecturally different from one that is always valid. The
right design is not obvious: candidates include a `LocalPlayer(Option<GameNetworkId>)` initialized lazily, a separate
`PendingLocalPlayer` marker replaced on handshake, or tying the concept entirely to the `installation_id` UUID rather
than a network-issued `ClientId`.

**Starting point for Phase 2:** Re-examine after F-14 (UUID-based identity) is live. The `installation_id` may make the
`ClientId`-based `LocalPlayer` unnecessary for most purposes. The remaining use-cases (e.g., filtering replicated
entities belonging to the local player) should be catalogued first.

---

#### D-03 · F-17 — Server Lifecycle Documentation

**What was deferred:** No formal document describes the dedicated server's state machine. Phase 1 will introduce
`BootState` and `MapSimulationState`, which together define most of the server's lifecycle, but the prose document
capturing the intended sequence has not been written yet.

**Starting point for Phase 2:** Once `BootState` and `MapSimulationState` are implemented, write
`docs/multiplayer/server_lifecycle.md`. The document should describe the full progression:

```text
Process Start
  → BootState::Loading (asset scanning)
  → BootState::Ready
  → Awaiting client connections (MapSimulationState::Unloaded)
  → RequestStartMission received
  → MapSimulationState::Loading (map arrays allocated, ghost entities spawned)
  → MapSimulationState::Ready (simulation ticks begin)
  → Mission end trigger
  → MapSimulationState::TearingDown (ticks stop, SummaryData published)
  → ServerGamePhase::Lobby (map entities despawned, back to waiting)
```

---

### 7.2 Open Design Questions (No Solution Yet)

These problems were identified during the audit or during the design conversation for this document. They have no
agreed-upon solution. They need a dedicated design session before implementation can begin.

---

#### O-01 · F-20 — Dedicated Server Coupled to `AppState::Lobby`

**The problem:** The dedicated server advances `AppState` through states like `AppState::Lobby` and `AppState::InGame`
as if it were a UX client. `AppState` was intended to be strictly a UX concept. The server should not be operating UI
states at all — it needs its own lifecycle primitive (likely `MapSimulationState` or a dedicated
`ServerOrchestrationState`) to trigger networking setup, lobby advertisement, and mission start.

**What needs designing:** A clear rule for which states the server is allowed to enter, and what replaces the current
`OnEnter(AppState::Lobby)` hooks that the server currently depends on to initialize.

---

#### O-02 · F-21 — Missing State History / Breadcrumbs

**The problem:** When a mission ends (or is paused), the game needs to know where to send the player next. Currently
this is answered by reading `NetMode`, which is wrong. The answer depends on the session context:

- Offline single-player → back to `MissionSelect`
- Campaign → back to `Campaign` flow
- Multiplayer lobby → back to `AppState::Lobby`

`LobbyPresenceRole` partially addresses the multiplayer case (Section 5), but the broader problem — the game has no
history of how it got to its current state — remains unsolved for Campaign vs standalone mission distinctions.

**What needs designing:** A small "navigation context" resource pushed onto a stack (or a simple enum) when entering a
mission, so that the exit path is unambiguous. This is intentionally minimal — it is not a full state history system,
just enough context to answer "where did I come from?"

---

#### O-03 · F-22 — No Lobby Disconnect / Leave UI

**The problem:** Once a player is in a multiplayer lobby, there is no implemented flow to voluntarily leave that lobby
and return to the main menu. The only existing exit paths are: start a mission (forward) or quit the application.

**What needs designing:**

- A "Leave Lobby" button or key binding in the lobby UI.
- Client-side behavior: disconnect from the server, despawn all replicated entities, return to `AppState::MainMenu`.
- Server-side behavior: handle the disconnect event, potentially re-assign `leader_uuid` if the leaving player was the
  leader.
- Corner case: what happens if the last player leaves? The dedicated server should return to waiting-for-connections
  state, not crash or stall.

---

### 7.3 Implementation Risks to Verify

These are not design questions but implementation details that could introduce subtle bugs. They should be explicitly
checked — with a code comment or a test — when the relevant Phase 1 work is merged.

---

#### R-01 · F-19 — Frame Latency on Unified Movement Path

**Risk:** Routing the PeerHost player's position through `PlayerMoveMessage` (instead of the previous direct memory
write) adds at least one additional hop through Bevy's event queue. Depending on system ordering, the position write to
`NetworkPosition` may occur one frame later than it did before. For 60 fps this is ~16ms — perceptible to sensitive
players as slightly "heavier" movement feel.

**What to check:** After deleting `sync_player_state_to_net` and running `send_local_player_position` unconditionally,
verify that `handle_player_move` is scheduled in the same frame and before rendering. Add a comment documenting the
intended schedule order so it is not accidentally broken later.

---

#### R-02 · F-04 Phase 1 Guard — Silent Early Return Risk

**Risk:** The crash-prevention guard added to `load_level_handler` (checking for empty asset handles before proceeding)
must log a warning or error when it fires. A silent early return would be indistinguishable from a successful map load
during testing, masking the very class of failure it is meant to prevent.

**What to check:** Ensure the guard emits at minimum a `warn!()` log with the map path, so any test run on a headless
server makes the incomplete load visible in the output.
