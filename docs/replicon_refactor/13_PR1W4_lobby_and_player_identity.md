# PR 1 Specification: Lobby State, Roles, and Player Identity (Workstream 4)

> **Document status:** Refined against the live codebase (March 2026). All file paths, struct names, field names, system
> names, and enum variants have been verified by direct inspection. Line-number references are indicative and may drift;
> the symbol names are authoritative.

UPDATE: This specification has been executed and the codebase already has these changes.

---

## 1. Objective

Refactor the lobby lifecycle, player colour assignment, and connection teardown to use a purely **Data-Driven and
Role-Based ECS architecture**. This PR isolates the Logical Lobby from the UI state machine, replaces a naive monotonic
colour counter with a "Lowest Available Slot" algorithm, implements context-aware (anti-DoS) disconnect logic, and
provides a clean "Disconnect" flow for Hub clients that requires no knowledge of the transport layer inside the UI
plugin.

### 1.1 Scope Boundaries

| In scope                                             | Out of scope                                      |
| ---------------------------------------------------- | ------------------------------------------------- |
| `crates/unreplicon-plugin/src/systems/lobby.rs`      | Map loading (`untmxmap-plugin`, `unmapload-core`) |
| `crates/unreplicon-plugin/src/systems/connection.rs` | Ghost AI, damage, combat systems                  |
| `crates/unreplicon-core/src/messages.rs`             | Any Bevy rendering or asset pipeline code         |
| `crates/untypes-core/src/roles.rs`                   | The `unengine-plugin` role-insertion logic        |
| `crates/unmainmenu-plugin/src/mainmenu.rs`           | The `unlobby-plugin` lobby UI screens             |
| `crates/unmainmenu-plugin/Cargo.toml`                | Any Cargo workspace-level changes                 |

---

## 2. Background: Key Types to Understand Before Coding

The following types are central to every step in this PR. Read these carefully before touching any file.

### 2.1 Role Resources (`crates/untypes-core/src/roles.rs`)

Three zero-sized marker resources describe what _this process_ is doing in a given session. They are inserted once at
startup by `crates/unengine-plugin/src/systems.rs` → `insert_roles_at_startup`, based on `CliOptions`, and are never
mutated during normal play (the Disconnect teardown in Step 4 is the first place they are changed at runtime).

```
AuthorityRole      — this process runs the authoritative server (Dedicated, PeerHost, Offline).
LocalPlayerRole    — this process has a human at a screen (Offline, PeerHost, Join/HubClient).
LobbyPresenceRole  — this process is part of a multiplayer lobby session (Dedicated,
                     PeerHost, Join/HubClient). Absent in pure Offline single-player.
```

The insertion matrix (from `insert_roles_at_startup`):

| CLI mode             | AuthorityRole | LocalPlayerRole | LobbyPresenceRole |
| -------------------- | ------------- | --------------- | ----------------- |
| `Offline`            | ✔             | ✔               | —                 |
| `PeerHost`           | ✔             | ✔               | ✔                 |
| `Join` (Hub Client)  | —             | ✔               | ✔                 |
| Dedicated (headless) | ✔             | —               | ✔                 |

A **pure client** is therefore: `LocalPlayerRole` present AND `AuthorityRole` absent. The helper
`untypes_core::roles::is_pure_client(local, authority) -> bool` encodes this.

### 2.2 Lobby Data Components (`crates/unreplicon-core/src/components.rs`)

`LobbyInfo` — a `Component` replicated to all clients. Spawned on a single "lobby state entity" by the server. Contains:

- `players: Vec<LobbyPlayerInfo>` — every tracked player in join order.
- `selected_map: Option<String>`
- `selected_difficulty: String`
- `leader_uuid: Option<Uuid>`

`LobbyPlayerInfo` — per-player record inside `LobbyInfo`. Contains:

- `player_uuid: Uuid` — stable persistent identity.
- `current_socket: Option<OwnerId>` — active transport socket, `None` when disconnected.
- `tint_color_index: u8` — index into the colour palette.
- `connected: bool`
- `nickname: Option<String>`

`ServerGamePhase` — a `Component` (also replicated). Lives on the same entity as `LobbyInfo`. Variants: `Lobby`,
`InProgress`, `Concluding`, `Ended`. This is **not** an `AppState`. It describes the logical server game phase
independently of any client's UI state.

### 2.3 The Colour Palette (`crates/unfoundation-core/src/colors.rs`)

```rust
pub fn player_color(index: usize) -> Color {
    let i = index as f32;
    let hue = (23.0 + (i * 120.0 + (i / 3.0).floor() * 40.0)) % 360.0;
    Color::Hsla(Hsla::new(hue, 0.8, 0.6, 1.0))
}
```

Slots 0–8 produce nine perceptually distinct hues. Slot 9 and above wrap modulo 360 and silently re-use earlier hues —
there is no fallback to `Color::WHITE` today. This PR adds that fallback and prevents the wrap-around from occurring in
the first place.

### 2.4 Key Systems and Observers (`crates/unreplicon-plugin/src/systems/lobby.rs`)

- **`setup_lobby_entity`** — spawns the `LobbyInfo` + `ServerGamePhase::Lobby` entity the first time, or resets
  `ServerGamePhase` and `selected_map` on re-entry after a mission. Currently runs `OnEnter(AppState::Lobby)`, gated by
  `resource_exists::<AuthorityRole>`.
- **`process_newly_connected_clients`** — runs every `Update` frame on the authority. For each `ConnectedClient` entity
  that has a known UUID in `ClientUuidMap`, either reconnects an existing player record or adds a new one. Contains the
  naive colour-assignment code.
- **`on_client_disconnected`** — Bevy **Observer** (not a regular system), registered via
  `app.add_observer(on_client_disconnected)`. Fires on `On<Remove, ConnectedClient>` using observer trigger syntax.
  Currently always performs a soft-remove (set `connected = false`, `current_socket = None`) regardless of
  `ServerGamePhase`.

### 2.5 The Main Menu (`crates/unmainmenu-plugin/src/mainmenu.rs`)

`MenuID` — component enum for menu entry identity:

```rust
pub(crate) enum MenuID {
    Campaign, CustomMission, MultiplayerLobby, Hub, Manual, Settings, Quit
}
```

`MenuID::Hub` displays as `"Play Online"` via its `Display` impl.

`setup_ui` — `OnEnter(AppState::MainMenu)` system. Already accepts
`lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>`. When `lobby_presence.is_some()` it already shows
only `MultiplayerLobby`; when absent it shows the full menu. This existing branch is the insertion point for the
Disconnect variant.

`menu_event` — `Update` system handling `MenuItemClicked` messages. Currently has no `commands`, no `authority`, and no
`lobby_presence` in its parameter list. All three must be added for the Disconnect action.

---

## 3. Implementation Steps

### Step 1 — Fix the Colour Pool (Lowest Available Slot)

**Files touched:**

- `crates/unfoundation-core/src/colors.rs`
- `crates/unreplicon-plugin/src/systems/lobby.rs`

#### 3.1.1 Add a `Color::WHITE` fallback to `player_color`

In `colors.rs`, update `player_color` so that any index ≥ 9 returns `Color::WHITE` instead of silently wrapping:

```rust
pub fn player_color(index: usize) -> Color {
    if index >= 9 {
        return Color::WHITE;
    }
    let i = index as f32;
    let hue = (23.0 + (i * 120.0 + (i / 3.0).floor() * 40.0)) % 360.0;
    Color::Hsla(Hsla::new(hue, 0.8, 0.6, 1.0))
}
```

The intent is that `Color::WHITE` is a visible signal that the lobby is at capacity (more than 9 players), rather than
two players sharing a colour silently.

#### 3.1.2 Replace the naive colour counter in `process_newly_connected_clients`

In `lobby.rs`, the "New player" branch currently reads:

```rust
let color_index = lobby.players.len() as u8;
```

This is wrong because it assigns slot N = current player count, which does not account for players who left and freed
their slot. Replace it with a Lowest Available Integer search over the 9 defined slots:

```rust
// Find the lowest colour slot (0..=8) not currently used by any player.
let mut used = [false; 9];
for p in lobby.players.iter() {
    let idx = p.tint_color_index as usize;
    if idx < 9 {
        used[idx] = true;
    }
}
let color_index = used.iter().position(|&u| !u).unwrap_or(9) as u8;
```

This guarantees that if Player 1 (slot 0 / Orange) disconnects during Lobby phase (see Step 3) and is hard-removed, the
very next player to join receives slot 0 again rather than slot N.

---

### Step 2 — Decouple Lobby Creation from UI State (Split `setup_lobby_entity`)

**Files touched:**

- `crates/unreplicon-plugin/src/systems/lobby.rs`

#### 2.1 Why the current design is fragile

`setup_lobby_entity` currently runs on `OnEnter(AppState::Lobby)`. This means:

1. On a PeerHost or Offline session, the host must navigate to `AppState::Lobby` before the `LobbyInfo` entity exists.
2. If the host never touches the UI (e.g., a headless dedicated server, or a PeerHost who just launched), `LobbyInfo`
   does not exist until the state transition fires.

The dedicated-server path already works around this with `auto_start_headless_lobby`, which forces an `AppState::Lobby`
transition. But a PeerHost currently sits on `AppState::MainMenu` after launch and clients connecting before the host
opens the lobby UI will be left waiting with no entity to register into.

#### 2.2 Split into two responsibilities

**Do NOT merge these two responsibilities.** The current function already cleanly handles both cases; we need to
separate them into two distinct systems:

**System A — `spawn_lobby_entity_if_missing`** (new name, new system):

- Runs in `Update`, every frame.
- Run condition: `resource_exists::<AuthorityRole>`.
- Checks `q_lobby: Query<(), With<LobbyInfo>>`. If the query is **not** empty (entity already exists), return
  immediately. Early return is the happy path.
- If the query is empty: spawn the entity exactly as `setup_lobby_entity` currently does in its "First-time spawn"
  branch (lines ~140–170 of the current file). This includes populating the local player into `players` if
  `LocalPlayerRole` is present, setting `leader_uuid`, inserting into `ClientUuidMap`, and spawning
  `(Replicated, LobbyInfo { … }, ServerGamePhase::Lobby)`.

**System B — `reset_lobby_entity_on_reenter`** (renamed from `setup_lobby_entity`):

- Runs `OnEnter(AppState::Lobby)`.
- Run condition: `resource_exists::<AuthorityRole>`.
- Calls `q_existing.single_mut()`. If the entity doesn't exist yet, log a warning and return (System A will spawn it
  shortly). If it does exist: reset `*game_phase = ServerGamePhase::Lobby` and `lobby.selected_map = None`. This is the
  "re-entering Lobby after a mission" path.

The net effect is:

- The moment a PeerHost or Dedicated process starts and `AuthorityRole` is present, System A spawns the `LobbyInfo`
  entity within the first Update tick, regardless of what `AppState` is showing.
- When the server _re-enters_ `AppState::Lobby` after finishing a mission, System B resets the phase and map selection
  without re-spawning.

#### 2.3 Registration in `app_setup`

In `lobby.rs → app_setup`, replace:

```rust
app.add_systems(
    OnEnter(AppState::Lobby),
    setup_lobby_entity.run_if(resource_exists::<AuthorityRole>),
);
```

With:

```rust
app.add_systems(
    Update,
    spawn_lobby_entity_if_missing.run_if(resource_exists::<AuthorityRole>),
);
app.add_systems(
    OnEnter(AppState::Lobby),
    reset_lobby_entity_on_reenter.run_if(resource_exists::<AuthorityRole>),
);
```

---

### Step 3 — Context-Aware Disconnects (Anti-DoS)

**Files touched:**

- `crates/unreplicon-plugin/src/systems/lobby.rs`

#### 3.1 The problem with the current soft-remove-always behaviour

Today, `on_client_disconnected` always does:

```rust
player.connected = false;
player.current_socket = None;
```

This is correct during a mission — you want to preserve the player slot. But in the lobby, a malicious or
repeatedly-cycling client can occupy all 9 colour slots permanently, because disconnecting during
`ServerGamePhase::Lobby` never frees the slot.

#### 3.2 Observer parameter changes

`on_client_disconnected` is a Bevy **Observer**, registered with `app.add_observer(…)`. It receives its triggering
information via the `On<Remove, ConnectedClient>` parameter. To implement phase-aware logic, add the following
parameters to its signature:

```rust
fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    mut q_lobby: Query<(&mut LobbyInfo, &ServerGamePhase)>,  // <-- add ServerGamePhase
    uuid_map: Res<ClientUuidMap>,
    mut commands: Commands,                                   // <-- new
    q_sprites: Query<(Entity, &PlayerSprite)>,               // <-- new
) {
```

The query must now fetch both `&mut LobbyInfo` **and** `&ServerGamePhase` together on the same entity (they live on the
same entity — the "lobby state entity" spawned in Step 2).

`PlayerSprite` is defined in `crates/unplayer-core/src/components.rs`. `unplayer-core` is already a direct dependency of
`unreplicon-plugin` (Cargo.toml line 29). No new dependency is needed.

#### 3.3 Hard Remove (Lobby phase)

When `*phase == ServerGamePhase::Lobby`:

```rust
// Hard remove: free the colour slot immediately.
lobby.players.retain(|p| p.player_uuid != uuid);

// If the removed player was the leader, assign the next connected player.
if lobby.leader_uuid == Some(uuid) {
    lobby.leader_uuid = lobby
        .players
        .iter()
        .find(|p| p.connected && p.current_socket.is_some())
        .map(|p| p.player_uuid);
    info!("Leader left during Lobby; new leader_uuid={:?}", lobby.leader_uuid);
}
info!("Player {} hard-removed from lobby (ServerGamePhase::Lobby)", uuid);
```

**Critical:** the leader reassignment must also be performed in the hard-remove path. The existing soft-remove code
already reassigns leadership; the new hard-remove path must do the same. Failing to do so leaves `lobby.leader_uuid`
pointing at a UUID that is no longer in `lobby.players`.

#### 3.4 Soft Remove (In-mission phases)

When `*phase != ServerGamePhase::Lobby` (i.e., `InProgress`, `Concluding`, or `Ended`):

Keep the current soft-remove behaviour:

```rust
if let Some(player) = lobby.players.iter_mut().find(|p| p.player_uuid == uuid) {
    player.connected = false;
    player.current_socket = None;
    info!("Player {} soft-disconnected (phase={:?})", uuid, phase);
}
```

Additionally, find the physical `PlayerSprite` entity (if one exists) matching this UUID and insert the
`PlayerDisconnected` marker component on it:

```rust
// Mark the physical avatar as disconnected so rendering/AI can dim it.
if let Some((entity, _)) = q_sprites.iter().find(|(_, s)| s.id == uuid) {
    commands.entity(entity).insert(PlayerDisconnected);
    info!("Inserted PlayerDisconnected on avatar entity for player {}", uuid);
}
```

`PlayerDisconnected` is confirmed at `crates/unplayer-core/src/components.rs` lines 21–23. `PlayerSprite.id: Uuid` is
the correct field to match against.

The leader-reassignment that already exists in the current code (reassigning `leader_uuid` when the departing player was
the leader) must be kept in the soft-remove branch as well.

#### 3.5 Complete revised skeleton

```rust
fn on_client_disconnected(
    trigger: On<Remove, ConnectedClient>,
    mut q_lobby: Query<(&mut LobbyInfo, &ServerGamePhase)>,
    uuid_map: Res<ClientUuidMap>,
    mut commands: Commands,
    q_sprites: Query<(Entity, &PlayerSprite)>,
) {
    let client_id = ClientId::Client(trigger.entity);
    let Some(uuid) = client_uuid(client_id, &uuid_map) else { return; };

    for (mut lobby, phase) in q_lobby.iter_mut() {
        if *phase == ServerGamePhase::Lobby {
            // Hard remove
            lobby.players.retain(|p| p.player_uuid != uuid);
            if lobby.leader_uuid == Some(uuid) {
                lobby.leader_uuid = lobby.players.iter()
                    .find(|p| p.connected && p.current_socket.is_some())
                    .map(|p| p.player_uuid);
            }
        } else {
            // Soft remove
            if let Some(player) = lobby.players.iter_mut().find(|p| p.player_uuid == uuid) {
                player.connected = false;
                player.current_socket = None;
            }
            if lobby.leader_uuid == Some(uuid) {
                lobby.leader_uuid = lobby.players.iter()
                    .find(|p| p.connected && p.current_socket.is_some())
                    .map(|p| p.player_uuid);
            }
            if let Some((entity, _)) = q_sprites.iter().find(|(_, s)| s.id == uuid) {
                commands.entity(entity).insert(PlayerDisconnected);
            }
        }
    }
}
```

---

### Step 4 — Role-Based Client Disconnect UI (The Architectural Constraint)

**Files touched:**

- `crates/untypes-core/src/roles.rs`
- `crates/unreplicon-plugin/src/systems/connection.rs`
- `crates/unreplicon-plugin/src/systems/lobby.rs` (just the `app_setup` registration)
- `crates/unmainmenu-plugin/src/mainmenu.rs`

**Files NOT touched:**

- `crates/unmainmenu-plugin/Cargo.toml` — no new dependencies are added to this crate.
- `crates/unreplicon-core/src/messages.rs` — this file contains network wire-protocol types (carrying
  `Serialize`/`Deserialize`, UUIDs, gear state, etc). A local UI event does not belong there.

#### 4.1 The Architectural Rule

`unmainmenu-plugin` must **not** hold a dependency on `bevy_renet` or `bevy_replicon`. The menu plugin describes what
the human sees; it has no business knowing the UDP transport layer. The correct pattern is:

> The UI emits a **local message** (`DisconnectRequest`). The plugin that owns the transport (`unreplicon-plugin`) is
> the sole listener. It performs the teardown.

This is identical in pattern to how `HostInteractionOccurred`, `HostMovableMotionEvent`, etc. are used throughout the
codebase as local cross-plugin signals.

#### 4.2 Define `DisconnectRequest` in `crates/untypes-core/src/roles.rs`

`untypes-core` is already a dependency of **both** `unmainmenu-plugin` and `unreplicon-plugin`, making it the
zero-new-dep home for this type. It is semantically correct too: `DisconnectRequest` is a role-transition signal
("please stop being a client"), which belongs alongside `AuthorityRole`, `LocalPlayerRole`, and `LobbyPresenceRole`.

Add to `crates/untypes-core/src/roles.rs`:

```rust
/// Sent by the UI when the local player wants to disconnect from the current
/// multiplayer session and return to single-player offline authority.
///
/// Consumed exclusively by `unreplicon-plugin/src/systems/connection.rs`.
/// The UI must not directly manipulate transport resources; it only writes this message.
#[derive(bevy::prelude::Message, Debug, Clone)]
pub struct DisconnectRequest;
```

Note: `#[derive(Message)]` is the project convention for buffered inter-system communication (see
`docs/bevy_0.16_migration_filtered.md`). This message is local-only — it requires no `Serialize`/`Deserialize`.

#### 4.3 Register `DisconnectRequest` — where and how

Registration with `app.add_message::<DisconnectRequest>()` must happen in a plugin, not in a core crate. The consumer is
`unreplicon-plugin`. Register it in `crates/unreplicon-plugin/src/systems/lobby.rs → app_setup` alongside the other
message registrations:

```rust
// In lobby.rs app_setup:
app.add_message::<untypes_core::roles::DisconnectRequest>();
```

Alternatively it can go in `connection.rs → app_setup` if that is cleaner; the important thing is that it is registered
exactly once in `unreplicon-plugin`.

#### 4.4 Handle `DisconnectRequest` in `connection.rs`

Add a new system to `crates/unreplicon-plugin/src/systems/connection.rs`:

```rust
fn handle_disconnect_request(
    mut ev: MessageReader<untypes_core::roles::DisconnectRequest>,
    mut commands: Commands,
    q_replicated: Query<Entity, With<Replicated>>,
) {
    if ev.is_empty() {
        return;
    }
    ev.clear();

    info!("DisconnectRequest received — tearing down client transport and resetting to offline authority");

    // 1. Remove the transport-layer resources. bevy_renet stops ticking
    //    and closes the UDP socket automatically when these are dropped.
    commands.remove_resource::<bevy_renet::RenetClient>();
    commands.remove_resource::<bevy_renet::netcode::NetcodeClientTransport>();

    // 2. Despawn all entities that were replicated from the remote server.
    for entity in q_replicated.iter() {
        commands.entity(entity).despawn();
    }

    // 3. Retract the network role resources and restore local authority.
    commands.remove_resource::<untypes_core::roles::LobbyPresenceRole>();
    commands.insert_resource(untypes_core::roles::AuthorityRole::default());

    // 4. (Callers are responsible for transitioning AppState back to MainMenu or similar.)
}
```

Register it in `connection.rs → app_setup`:

```rust
app.add_systems(
    Update,
    handle_disconnect_request.run_if(resource_exists::<untypes_core::roles::LobbyPresenceRole>),
);
```

The run condition ensures this system does nothing in offline sessions where no disconnect is ever possible.

**Important note on ordering:** `commands` operations are deferred; the role resources are not actually removed/inserted
until the next command flush. Systems reading `AuthorityRole` or `LobbyPresenceRole` in the same frame may still observe
the old state. This is consistent with how Bevy resources work throughout the codebase and is acceptable here.

#### 4.5 Update `setup_ui` in `mainmenu.rs`

The current signature is:

```rust
pub(crate) fn setup_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>,
)
```

Add an `authority` parameter:

```rust
pub(crate) fn setup_ui(
    mut commands: Commands,
    ui_assets: Res<UiAssets>,
    player_profile: Res<Persistent<PlayerProfileData>>,
    lobby_presence: Option<Res<untypes_core::roles::LobbyPresenceRole>>,
    authority: Option<Res<untypes_core::roles::AuthorityRole>>,     // <-- new
)
```

Then update the `menu_items` construction block. Currently:

```rust
let mut menu_items = if lobby_presence.is_none() {
    vec![
        (MenuID::Campaign,      MenuID::Campaign.to_string()),
        (MenuID::CustomMission, MenuID::CustomMission.to_string()),
        (MenuID::Hub,           MenuID::Hub.to_string()),
    ]
} else {
    vec![(MenuID::MultiplayerLobby, MenuID::MultiplayerLobby.to_string())]
};
```

Replace with three cases — offline, PeerHost/Dedicated (authority present, lobby present), and pure client (lobby
present, no authority):

```rust
let is_pure_client = untypes_core::roles::is_pure_client(
    local.as_ref().map(|r| r.as_ref()),   // if LocalPlayerRole is needed; see note below
    authority.as_ref().map(|r| r.as_ref()),
);
// Simpler equivalent that avoids needing LocalPlayerRole as a param:
let is_pure_client = lobby_presence.is_some() && authority.is_none();

let mut menu_items = if is_pure_client {
    // Hub Client / Join-only: can only go to the lobby, or disconnect.
    vec![
        (MenuID::MultiplayerLobby, MenuID::MultiplayerLobby.to_string()),
        (MenuID::Disconnect,       MenuID::Disconnect.to_string()),
    ]
} else if lobby_presence.is_some() {
    // PeerHost or Dedicated with local player: show lobby entry.
    vec![(MenuID::MultiplayerLobby, MenuID::MultiplayerLobby.to_string())]
} else {
    // Offline single-player: full menu.
    vec![
        (MenuID::Campaign,      MenuID::Campaign.to_string()),
        (MenuID::CustomMission, MenuID::CustomMission.to_string()),
        (MenuID::Hub,           MenuID::Hub.to_string()),
    ]
};
```

Note: `is_pure_client = lobby_presence.is_some() && authority.is_none()` is the inline equivalent of the
`untypes_core::roles::is_pure_client()` helper. Use whichever is cleaner; there is no need to add `LocalPlayerRole` as
an additional system parameter just to call the helper function, because the inline expression is equivalent for this
context.

#### 4.6 Add `MenuID::Disconnect` to the enum and its `Display` impl

In `mainmenu.rs`, add the variant:

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub(crate) enum MenuID {
    Campaign,
    CustomMission,
    MultiplayerLobby,
    Hub,
    Manual,
    Settings,
    Disconnect,            // <-- new
    #[cfg(not(target_arch = "wasm32"))]
    Quit,
}
```

In the `Display` impl, add the arm:

```rust
MenuID::Disconnect => "Disconnect from Server",
```

#### 4.7 Handle `MenuID::Disconnect` in `menu_event`

The current `menu_event` signature:

```rust
pub(crate) fn menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut current_mission_select_mode: ResMut<CurrentMissionSelectMode>,
    menu_items: Query<(&MenuID, &MenuItemInteractive)>,
)
```

Add two new parameters:

```rust
pub(crate) fn menu_event(
    mut click_events: MessageReader<MenuItemClicked>,
    #[cfg(not(target_arch = "wasm32"))] mut exit: MessageWriter<AppExit>,
    mut next_app_state: ResMut<NextState<AppState>>,
    mut next_map_hub_state: ResMut<NextState<MapHubState>>,
    mut current_mission_select_mode: ResMut<CurrentMissionSelectMode>,
    menu_items: Query<(&MenuID, &MenuItemInteractive)>,
    mut ev_disconnect: MessageWriter<untypes_core::roles::DisconnectRequest>,  // <-- new
)
```

Add the match arm in the `match menu_id` block:

```rust
MenuID::Disconnect => {
    ev_disconnect.write(untypes_core::roles::DisconnectRequest);
    // The actual teardown happens in unreplicon-plugin/connection.rs.
    // Transition back to MainMenu so setup_ui re-runs and shows the offline menu.
    next_app_state.set(AppState::MainMenu);
    info!("DisconnectRequest sent; transitioning to MainMenu");
}
```

The `next_app_state.set(AppState::MainMenu)` re-triggers `OnEnter(AppState::MainMenu)` → `setup_ui`, which will now see
`LobbyPresenceRole` absent (once the command flush has processed the `remove_resource` from Step 4.4) and render the
full offline menu on the next frame. Because `setup_ui` runs `OnEnter`, and `AppState` is already `MainMenu`, the set
may not re-trigger `OnEnter`. The agent should verify whether `setup_ui` needs to be triggered a second time (e.g., by
briefly transitioning to a transient state and back, or by despawning and re-spawning the menu UI entities directly).

A safe alternative is to explicitly call `commands.trigger(RebuildMenuUI)` or similar if the project has such a
mechanism. If not, the simplest approach is to transition to a brief intermediate state — but this depends on whether
the existing cleanup/setup system pair handles it. Investigate `OnExit(AppState::MainMenu)` cleanup in `unengine-plugin`
before choosing an approach.

---

## 4. New/Modified Cargo Dependencies

| Crate                          | Change                                                                       |
| ------------------------------ | ---------------------------------------------------------------------------- |
| `unmainmenu-plugin/Cargo.toml` | **None.** `untypes-core` is already listed.                                  |
| `unreplicon-plugin/Cargo.toml` | **None.** `untypes-core` and `unplayer-core` are already listed.             |
| `unreplicon-core/Cargo.toml`   | **None.** `DisconnectRequest` lives in `untypes-core`, not here.             |
| `untypes-core/Cargo.toml`      | **None.** The `Message` derive is from `bevy::prelude::*`, already imported. |

No `Cargo.toml` file needs to be changed as part of this PR.

---

## 5. Success Criteria

The agent should verify the following before considering the PR complete:

- [ ] **Lowest Available Slot:** Run with 3 players in the lobby. Player 1 (slot 0 / first hue) disconnects. The next
      player to join receives slot 0, not slot 3.
- [ ] **Lobby Always Exists:** A PeerHost launches and remains on `AppState::MainMenu`. A Join client connects. The join
      client's `process_newly_connected_clients` succeeds (finds a `LobbyInfo` entity to write into) without the
      PeerHost ever opening the lobby UI.
- [ ] **Hard Remove During Lobby:** A player disconnects while `ServerGamePhase::Lobby`. They are absent from the player
      list entirely on the next replication tick.
- [ ] **Soft Remove During Mission:** A player disconnects while `ServerGamePhase::InProgress` (or `Concluding`). Their
      row remains in the player list with `connected = false`. Their physical avatar entity is still present in the
      world and has the `PlayerDisconnected` component.
- [ ] **Disconnect UI Shown Only to Pure Clients:** A PeerHost's main menu does not show the "Disconnect from Server"
      option. A Join/Hub Client's main menu shows it (and hides Campaign, Custom Mission, Play Online).
- [ ] **Clean Disconnect Flow:** A Hub Client clicks "Disconnect from Server". The binary does not crash or restart. The
      next frame's main menu shows the full offline menu (Campaign, Custom Mission, Play Online). `AuthorityRole` is
      present and `LobbyPresenceRole` is absent.
- [ ] **`cargo clippy` clean**: No new warnings on any crate in this PR's scope.
- [ ] **Leader handoff on Hard Remove**: If the lobby leader disconnects during `ServerGamePhase::Lobby`, leadership
      transfers to the next connected player (same logic as the soft-remove path).

---

## 6. Verification Commands

```sh
# Lint — run globally (do not use -p <crate>)
cargo check
```

Do not run game tests. Do not run the game.
