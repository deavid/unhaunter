# Plan 21: Multi-Client Support (>2 Players)

## Goal

Transform the network layer from supporting exactly **1 host + 1 client** to supporting **1 host + N clients** (up to 8
remote players, 9 total including the host).

After this work is done, a host can accept multiple simultaneous TCP connections. Each connected client receives
snapshots of the full game state, and each client's inputs are applied independently on the host. No lobby, no UI
changes, no dedicated server — just the raw networking plumbing to handle N connections.

## Motivation

The game's multiplayer vision targets up to 9 players per mission. Currently the host's `NetworkConn` resource is an
enum that holds a single `TcpStream` in its `Active` variant. It literally rejects every connection after the first one.
Every system that sends data calls `conn.send()`, which writes to that single stream. This is the #1 blocker for any
meaningful multiplayer experience beyond 2 players.

## Context

### Relevant crates

| Crate          | Role                                                                                                           |
| -------------- | -------------------------------------------------------------------------------------------------------------- |
| `unnet-core`   | Data types: `NetworkId`, `NetworkMessage`, `SnapshotMsg`, `NetworkDataEvent`, etc. **No plugins, no systems.** |
| `unnet-plugin` | All network systems and the `UnhaunterNetPlugin`. This is the primary target.                                  |

### Key files (after the recent systems.rs split)

| File                                              | Contents                                                                                                                                                              |
| ------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/unnet-plugin/src/resources.rs`            | `NetworkConn` (enum), `HandshakeState`, `PendingMapLoad`, `PlayerRegistry`                                                                                            |
| `crates/unnet-plugin/src/systems/connection.rs`   | `startup_network_system`, `network_io_system`, `handshake_handler_system`, `host_handle_disconnects_system`, `client_connection_monitor_system`, `autostart_net_game` |
| `crates/unnet-plugin/src/systems/host_sync.rs`    | `HostSnapshotParams`, `host_send_snapshots_system`, `host_send_summary_system`                                                                                        |
| `crates/unnet-plugin/src/systems/host_input.rs`   | `HostApplyInputParams`, `host_apply_input_system`                                                                                                                     |
| `crates/unnet-plugin/src/systems/client_input.rs` | `client_sync_intended_gear_state`, `client_send_input_system`, `client_process_pending_map`, `client_request_grab_system`                                             |
| `crates/unnet-plugin/src/systems/client_sync.rs`  | `ClientSnapshotParams`, `SnapshotAppStates`, spawn helpers, `client_apply_snapshots_system`, `delayed_despawn_system`                                                 |
| `crates/unnet-plugin/src/systems/utils.rs`        | `LastSyncedGearState`, `extract_gear_details`                                                                                                                         |
| `crates/unnet-plugin/src/systems/setup.rs`        | `app_setup` — system registration with scheduling, chaining, run conditions                                                                                           |
| `crates/unnet-plugin/src/plugin.rs`               | `UnhaunterNetPlugin` — resource init, calls `systems::setup::app_setup(app)`                                                                                          |
| `crates/unnet-plugin/src/metrics.rs`              | Performance metric constants                                                                                                                                          |
| `crates/unnet-core/src/messages.rs`               | `NetworkMessage` enum, `SnapshotMsg`, `NetworkDataEvent`, `NetworkDisconnectEvent`, `SendNetworkMessage`                                                              |
| `crates/unnet-core/src/network_id.rs`             | `NetworkId(pub u64)`, `ToBeDespawned`                                                                                                                                 |
| `crates/unnet-core/src/resources.rs`              | `LocalPlayer`, `MissionEndRequested`, `HostGone`, `ChangedTiles`                                                                                                      |

### Current `NetworkConn` (the thing we're replacing)

```rust
// File: crates/unnet-plugin/src/resources.rs
#[derive(Resource, Default)]
pub(crate) enum NetworkConn {
    #[default]
    Disconnected,
    Listening(Vec<std::net::TcpListener>),
    Active {
        stream: TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
        installation_id: Option<uuid::Uuid>,
        associated_id: Option<NetworkId>,
        needs_full_sync: bool,
        host_listeners: Vec<std::net::TcpListener>,
    },
}
```

Helper methods: `is_active()`, `send(msg)`.

### How systems use `NetworkConn` today

Every system accesses `NetworkConn` as `ResMut<NetworkConn>` or `Res<NetworkConn>`. There are 8 such sites across the
codebase. Systems that send data use `conn.send(msg)` — there are 10 call sites total. All 10 write to the **single**
stream. On the host side, `network_io_system` **rejects** new connections while one is active (lines 155-168 in
`connection.rs`).

### `NetworkDataEvent` today

```rust
// File: crates/unnet-core/src/messages.rs
#[derive(Debug, Clone, Message)]
pub struct NetworkDataEvent {
    pub message: NetworkMessage,
}
```

This event has **no source identifier** — on the host, it is impossible to know which client sent a particular message.

## Risk Analysis

| Risk                                                         | Severity | Mitigation                                                                                                                                                                        |
| ------------------------------------------------------------ | -------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `NetworkConn` refactor breaks all systems                    | High     | Phase 1 is a pure structural refactor keeping 1-client behavior. Validate before phase 2.                                                                                         |
| `Cargo.toml` edits break workspace                           | High     | After every `Cargo.toml` change, run workspace check. If any `.toml` has errors, stop everything and fix before touching `.rs` files.                                             |
| `conn.send()` call sites missed during migration             | Medium   | grep for `conn.send` after each phase; verify zero occurrences of the old pattern.                                                                                                |
| `NetworkDataEvent` construction sites in OTHER crates missed | High     | There are 3 sites outside `unnet-plugin` (in `untruck-plugin` and `uninteraction-plugin`). Step 2 lists them all. Grep for `NetworkDataEvent {` workspace-wide after editing.     |
| Snapshot broadcast doubles bandwidth                         | Low      | We already send one snapshot per frame. Sending the same snapshot to N clients is N× bandwidth but the snapshot itself is unchanged. Acceptable for up to 9 players on LAN/local. |
| Per-client `needs_full_sync` logic is wrong                  | Medium   | Step 8 uses a two-snapshot approach (full + delta) to avoid the bug of setting `is_full_sync = true` on a delta-only snapshot clone.                                              |
| Duplicate connections from same `installation_id`            | Medium   | Phase 2 adds explicit handling: close the old connection when a new `Hello` arrives with a known UUID.                                                                            |
| Pre-handshake message injection                              | Medium   | `do_client_io` in Step 4 drops all non-`Hello` messages from clients that haven't completed handshake.                                                                            |
| Client-side code accidentally affected                       | Low      | Client-side files (`client_input.rs`, `client_sync.rs`) get minimal changes (just method rename from `send` to `client_send`).                                                    |

## Conventions & Constraints for the Implementing Agent

1. **No terminal commands.** You can only search files, read files, edit files, and run workspace checks (get_errors).
   You cannot run `cargo clippy`, `cargo build`, or any shell command.

2. **Workspace check protocol.** After every phase (and after every `Cargo.toml` edit):
   - Run `get_errors` with no file filter to get ALL errors.
   - If ANY error is in a `Cargo.toml` file, **STOP all other work**. Fix the `Cargo.toml` first, then re-run
     `get_errors` until `Cargo.toml` files are clean.
   - Only then proceed to fix `.rs` errors.
   - Re-run `get_errors` after fixing `.rs` errors. Repeat until zero errors and zero warnings.

3. **No re-exports.** Never use `pub use`. Every symbol has one canonical path.

4. **`mod.rs` and `lib.rs` must contain only `mod` statements.** No code, no imports.

5. **Imports.** Prefer explicit imports. Wildcard `use` is allowed only for `bevy::prelude::*`.

6. **Do not modify files outside the scope of the phase.** If you're working on phase 1, do not touch files that belong
   to phase 2 or 3.

7. **Do not run the game.** You cannot test by running. Validate using workspace checks.

8. **Review the full workspace for warnings** after each phase. Address unused imports, dead code warnings, etc.

---

## Phase 1: Refactor `NetworkConn` to support N clients (structural, keep 1-client behavior)

### Phase 1 Goal

Replace the single-connection `NetworkConn` enum with a structure that **can hold** N client connections on the host
side, while keeping the client side as a single connection. At the end of this phase, the behavior is identical to today
— only the data structures have changed. The host still accepts only one connection at a time (we will change that in
phase 2).

### Phase 1 Overview

The `Active` variant currently serves double duty: it is used both when the host has a connected client AND when a
client has connected to a host. We will split this into two distinct variants: `Host` (with a `Vec` of client
connections) and `Client` (single connection to the host).

### Phase 1, Step 1: Define `ClientConnection` struct and new `NetworkConn` enum

**File to edit:** `crates/unnet-plugin/src/resources.rs`

**Action:** Replace the existing `NetworkConn` enum and its `impl` block with the following. Keep `HandshakeState`,
`PendingMapLoad`, and `PlayerRegistry` unchanged.

**New code for `NetworkConn`:**

```rust
/// Represents a single remote client connection (used on the host side).
pub(crate) struct ClientConnection {
    pub stream: std::net::TcpStream,
    pub read_buffer: String,
    pub write_queue: VecDeque<NetworkMessage>,
    pub handshake: HandshakeState,
    pub installation_id: Option<uuid::Uuid>,
    pub associated_id: Option<unnet_core::network_id::NetworkId>,
    pub needs_full_sync: bool,
}

#[derive(Resource, Default)]
pub(crate) enum NetworkConn {
    /// No network activity.
    #[default]
    Disconnected,
    /// Host: listening for client connections, managing 0..N active clients.
    Host {
        listeners: Vec<std::net::TcpListener>,
        clients: Vec<ClientConnection>,
    },
    /// Client: single connection to the host.
    Client {
        stream: std::net::TcpStream,
        read_buffer: String,
        write_queue: VecDeque<NetworkMessage>,
        handshake: HandshakeState,
    },
}
```

**New helper methods on `NetworkConn`:**

```rust
impl NetworkConn {
    /// Returns true if this is a Host with at least one client, or a connected Client.
    pub(crate) fn is_active(&self) -> bool {
        match self {
            Self::Host { clients, .. } => !clients.is_empty(),
            Self::Client { .. } => true,
            Self::Disconnected => false,
        }
    }

    /// Send a message on the client's connection to the host. Does nothing if not Client.
    pub(crate) fn client_send(&mut self, msg: NetworkMessage) {
        if let Self::Client { write_queue, .. } = self {
            write_queue.push_back(msg);
        }
    }

    /// Broadcast a message to ALL connected clients. Does nothing if not Host.
    pub(crate) fn host_broadcast(&mut self, msg: NetworkMessage) {
        if let Self::Host { clients, .. } = self {
            for client in clients.iter_mut() {
                client.write_queue.push_back(msg.clone());
            }
        }
    }

    /// Send a message to a specific client identified by their NetworkId.
    /// Does nothing if not Host or if the client is not found.
    pub(crate) fn host_send_to(
        &mut self,
        target: unnet_core::network_id::NetworkId,
        msg: NetworkMessage,
    ) {
        if let Self::Host { clients, .. } = self {
            for client in clients.iter_mut() {
                if client.associated_id == Some(target) {
                    client.write_queue.push_back(msg);
                    return;
                }
            }
        }
    }
}
```

**Remove** the old `Listening` and `Active` variants entirely. They no longer exist.

**Remove** the old `send()` method. It is replaced by `client_send()`, `host_broadcast()`, and `host_send_to()`.

### Phase 1, Step 2: Add `source` field to `NetworkDataEvent`

**File to edit:** `crates/unnet-core/src/messages.rs`

**Action:** Add an `Option<NetworkId>` source field to `NetworkDataEvent`:

```rust
#[derive(Debug, Clone, Message)]
pub struct NetworkDataEvent {
    pub message: NetworkMessage,
    /// On the host, identifies which client sent this message.
    /// On the client (and for local passthrough messages), this is always None.
    pub source: Option<NetworkId>,
}
```

**Then** update ALL construction sites in the workspace. There are **4 sites** that construct `NetworkDataEvent`. Grep
for `NetworkDataEvent {` across the entire workspace to find them all. They are:

1. **`crates/unnet-plugin/src/systems/connection.rs`** (line ~214) — inside `network_io_system`. On the host side, set
   `source: client.associated_id`. On the client side, set `source: None`.

2. **`crates/untruck-plugin/src/systems/in_truck_manager.rs`** (2 sites, lines ~25 and ~48) — `on_enter_truck` and
   `on_exit_truck` systems. These are **local passthrough** events: the client writes a `NetworkDataEvent` locally so
   that `client_send_input_system` picks it up and forwards it to the host. Set `source: None` at both sites.

3. **`crates/uninteraction-plugin/src/systems/interactivestuff.rs`** (line ~209) — another **local passthrough** for
   `InteractionRequest`. Set `source: None`.

Additionally, `crates/untruck-plugin/src/journal.rs` **reads** `NetworkDataEvent` via `MessageReader` but does not
construct it — no changes needed there (it only accesses `ev.message`).

**Verification:** After editing, grep for `NetworkDataEvent {` across the entire workspace. Every match must include a
`source:` field. Run `get_errors` — if any crate fails to compile due to a missing `source` field, fix it immediately.

**Design note — why `source: None` is safe for local passthrough:**

On the client side, both "messages received from the host" (emitted in `network_io_system`) and "local passthrough
events" (emitted in `on_enter_truck`, `on_exit_truck`, `interactivestuff`) use `source: None`. This is not ambiguous
because they are consumed by different systems:

- Host-inbound messages (e.g., `Snapshot`, `Welcome`) are consumed by `client_apply_snapshots_system` and
  `handshake_handler_system`, which filter by message variant — they never see `RequestTruckEntry` or
  `InteractionRequest`.
- Local passthrough messages (e.g., `RequestTruckEntry`, `InteractionRequest`) are consumed by
  `client_send_input_system`, which also filters by message variant — it only forwards specific client-action variants
  to the host.

No system uses the `source` field on the client side. The `source` field is only meaningful on the **host** side, where
it identifies which client sent the message.

### Phase 1, Step 3: Rewrite `startup_network_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

**Action:** Update the function to use the new `NetworkConn` variants.

**Changes:**

- `NetMode::Offline` → `*conn = NetworkConn::Disconnected;` (unchanged)
- `NetMode::Host { .. }` → Instead of `*conn = NetworkConn::Listening(listeners)`, use:

  ```rust
  *conn = NetworkConn::Host {
      listeners,
      clients: Vec::new(),
  };
  ```

- `NetMode::Join { .. }` → Instead of `NetworkConn::Active { stream, ..., host_listeners: Vec::new() }`, use:

  ```rust
  *conn = NetworkConn::Client {
      stream,
      read_buffer: String::new(),
      write_queue: VecDeque::new(),
      handshake: HandshakeState::None,
  };
  ```

### Phase 1, Step 4: Rewrite `network_io_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

This is the most complex step. The current function uses `std::mem::replace` to take ownership of the enum, then
pattern-matches on the three variants and puts it back.

**New logic:**

```rust
fn network_io_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_writer: MessageWriter<NetworkDataEvent>,
    mut ev_disconnect: MessageWriter<NetworkDisconnectEvent>,
    mut ev_send: MessageReader<SendNetworkMessage>,
) {
    // 1. Collect outgoing event-based messages
    let send_msgs: Vec<NetworkMessage> = ev_send.read().map(|m| m.0.clone()).collect();

    match &mut *conn {
        NetworkConn::Disconnected => {}

        NetworkConn::Host { listeners, clients } => {
            // 1a. Enqueue event-based messages to ALL clients
            for msg in &send_msgs {
                for client in clients.iter_mut() {
                    client.write_queue.push_back(msg.clone());
                }
            }

            // 2. Accept new connections from listeners
            for listener in listeners.iter() {
                loop {
                    match listener.accept() {
                        Ok((stream, addr)) => {
                            info!("Network: Client connected from {}", addr);
                            // set_nonblocking, set_nodelay
                            // Push new ClientConnection into clients vec
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
                        Err(e) => { error!(...); break; }
                    }
                }
            }

            // 3. Read/write each client
            let mut to_remove = Vec::new();
            for (idx, client) in clients.iter_mut().enumerate() {
                let closed = do_client_io(client, &mut ev_writer);
                if closed {
                    if let Some(id) = client.associated_id {
                        ev_disconnect.write(NetworkDisconnectEvent { id });
                    }
                    to_remove.push(idx);
                }
            }
            // Remove closed connections (reverse order to preserve indices)
            for idx in to_remove.into_iter().rev() {
                let removed = clients.remove(idx);
                info!("Network: Client {:?} disconnected", removed.associated_id);
            }
        }

        NetworkConn::Client { stream, read_buffer, write_queue, .. } => {
            // 1a. Enqueue event-based messages
            for msg in &send_msgs {
                write_queue.push_back(msg.clone());
            }

            // 2. Read/write the single connection
            // (same logic as current Active branch, using source: None for events)
            // If closed, transition to Disconnected
        }
    }
}
```

**Extract a helper** `do_client_io` (a regular function, not a system) that takes a `&mut ClientConnection`,
`&mut MessageWriter<NetworkDataEvent>`, and `&mut PlayerRegistry`, and returns `bool` (true if closed). This helper does
the read loop, JSON parsing, write loop — the same logic currently in the `Active` branch but adapted for
`ClientConnection` fields.

**Message filtering in `do_client_io` (handshake guard):**

When parsing a message from a client's TCP stream, `do_client_io` must apply the following rules:

1. If the message is `Hello` AND `client.handshake == HandshakeState::None`:
   - Extract `installation_id` from the message.
   - Immediately call `player_registry.get_or_assign(installation_id)` to assign a `NetworkId`.
   - Set `client.associated_id = Some(id)` and `client.installation_id = Some(installation_id)`.
   - Emit `NetworkDataEvent { message, source: Some(id) }`.

2. If `client.handshake == HandshakeState::Completed` (post-handshake):
   - Emit `NetworkDataEvent { message, source: client.associated_id }` for ALL message types.

3. **Otherwise** (client has not completed handshake AND message is not `Hello`):
   - **Drop the message.** Log a warning: `"Dropping pre-handshake message from unidentified client"`. This prevents
     malicious or malfunctioning clients from injecting game events before authentication.

On the client side (the `NetworkConn::Client` branch), all messages from the host are emitted as
`NetworkDataEvent { message, source: None }`.

**Important implementation detail:** The current code uses `std::mem::replace` to take ownership of `NetworkConn`. With
the new design, we **do not** need `mem::replace` because we are using `&mut` access through the `match`. The Host
variant stays in place; we just mutate its `clients` vec. If the Client connection closes, set
`*conn = NetworkConn::Disconnected`.

### Phase 1, Step 5: Update `handshake_handler_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

**Current behavior overview:** The system checks if we're in the `Active` variant, does client-side Hello sending, and
host-side Hello/Welcome processing. It accumulates changes in local variables and applies them at the end.

**New behavior:** Split the logic based on the `NetworkConn` variant.

**Client path (`NetworkConn::Client { handshake, .. }`):**

- If `handshake == HandshakeState::None` and `cli.net_mode` is `Join`, call `conn.client_send(Hello { ... })`, set
  handshake to `HelloSent`.
- Process incoming `Welcome` from `ev_reader`, set `local_id`, handshake to `Completed`, etc.
- This is almost identical to the current client-side path.

**Host path (`NetworkConn::Host { clients, .. }`):**

- Process incoming `Hello` events from `ev_reader`. Each event has `source: Some(NetworkId)` because `network_io_system`
  / `do_client_io` already called `player_registry.get_or_assign()` and set `client.associated_id`. Use `ev.source` to
  find the client connection in `clients`:
  ```rust
  if let Some(source_id) = ev.source {
      // Find client by associated_id
      for client in clients.iter_mut() {
          if client.associated_id == Some(source_id) {
              // Re-associate disconnected player entities (same as today)
              // Enqueue Welcome in client.write_queue
              // Set client.handshake = HandshakeState::Completed
              // Set client.needs_full_sync = true
              break;
          }
      }
  }
  ```
- Send `Welcome` directly into the client's `write_queue` (or via `conn.host_send_to(source_id, Welcome { ... })`).
- Remove the `associate_id`, `new_installation_id`, `new_handshake` local variables dance. We now directly match on the
  variant and mutate inline.

### Phase 1, Step 6: Update `host_handle_disconnects_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

**No changes needed.** This system reads `NetworkDisconnectEvent` and marks player entities with `PlayerDisconnected`.
The event is already emitted per-client in the new `network_io_system`. The logic is identical.

### Phase 1, Step 7: Update `client_connection_monitor_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

**Change:** Replace `matches!(*conn, NetworkConn::Disconnected)` with a check appropriate for the client role. The
system already guards on `NetMode::Join`. So the check becomes:

```rust
if !matches!(*conn, NetworkConn::Client { .. }) {
    // Connection lost
    ...
}
```

### Phase 1, Step 8: Update `host_send_snapshots_system`

**File to edit:** `crates/unnet-plugin/src/systems/host_sync.rs`

**Changes:**

- Replace `conn.is_active()` guard with:

  ```rust
  let NetworkConn::Host { clients, .. } = &mut *conn else {
      measure.end_ms();
      return;
  };
  if clients.is_empty() {
      measure.end_ms();
      return;
  }
  ```

- **Two-snapshot approach for per-client `needs_full_sync`:**

  The `needs_full_sync` flag is now per-client. The snapshot's `map_tiles` field is either a full list of all tiles (for
  full sync) or a delta list (only changed tiles). We CANNOT build a delta snapshot and then set `is_full_sync = true`
  on a clone — the clone would still have only delta tiles.

  Instead, build the snapshot data in two stages:
  1. **Check** if ANY client needs full sync:

     ```rust
     let any_needs_full = clients.iter().any(|c|
         c.handshake == HandshakeState::Completed && c.needs_full_sync
     );
     ```

  2. **Build tile lists** — always drain `ChangedTiles` for the delta list. If `any_needs_full`, ALSO build the full
     tile list by querying all map tiles:

     ```rust
     let delta_tiles: Vec<MapTileState> = host_params.changed_tiles.0.drain(..).collect();
     let full_tiles: Option<Vec<MapTileState>> = if any_needs_full {
         Some(host_params.query_map_tiles.iter().map(|(pos, beh)| MapTileState {
             x: pos.x as i32, y: pos.y as i32, z: pos.z as i32,
             tileset: beh.cfg().tileset.clone(),
             tileuid: beh.cfg().tileuid,
             cvo_key: beh.key_cvo().to_key_string(),
         }).collect())
     } else {
         None
     };
     ```

  3. **Build the base snapshot** with `is_full_sync = false` and `map_tiles = delta_tiles`. Build all other fields
     (players, ghosts, gear, etc.) exactly as today.

  4. **Send per-client:**
     ```rust
     for client in clients.iter_mut() {
         if client.handshake != HandshakeState::Completed {
             continue;
         }
         if client.needs_full_sync {
             let mut full_snap = base_snapshot.clone();
             full_snap.is_full_sync = true;
             full_snap.map_tiles = full_tiles.clone().unwrap_or_default();
             client.needs_full_sync = false;
             client.write_queue.push_back(
                 NetworkMessage::Snapshot(Box::new(full_snap))
             );
         } else {
             client.write_queue.push_back(
                 NetworkMessage::Snapshot(Box::new(base_snapshot.clone()))
             );
         }
     }
     ```

  This ensures full-sync clients get the complete tile list, delta clients get only changed tiles, and the expensive
  query runs at most once per frame.

  Note: `SnapshotMsg` must implement `Clone`. It derives `Clone` via `#[derive(Serialize, Deserialize, Debug, Clone)]`
  in `messages.rs`. Verify that all 22 fields are `Clone` compatible — they are (`Vec`, primitives, `Option`, `Box`).
  **Verify this.**

### Phase 1, Step 9: Update `host_send_summary_system`

**File to edit:** `crates/unnet-plugin/src/systems/host_sync.rs`

**Changes:**

- Replace `conn.send(NetworkMessage::MissionSummary { ... })` with
  `conn.host_broadcast(NetworkMessage::MissionSummary { ... })`.

### Phase 1, Step 10: Update `host_apply_input_system`

**File to edit:** `crates/unnet-plugin/src/systems/host_input.rs`

**Changes:**

- The system has `pub network_conn: Option<ResMut<'w, NetworkConn>>` in `HostApplyInputParams`. It accesses
  `needs_full_sync` on the connection in two places (`RequestFullSync` and `RequestTruckInventoryChange`).
- For `RequestFullSync`: use `ev.source` (the `NetworkId` of the requesting client) to find the specific client and set
  its `needs_full_sync = true`:

  ```rust
  NetworkMessage::RequestFullSync { player_id } => {
      if let Some(conn) = &mut params.network_conn
          && let NetworkConn::Host { clients, .. } = &mut **conn
      {
          // Find the client that sent this request
          for client in clients.iter_mut() {
              if client.associated_id == Some(*player_id) {
                  client.needs_full_sync = true;
                  break;
              }
          }
      }
  }
  ```

  Note: We use `player_id` from the message itself (which is the client's NetworkId) to find the right client. We don't
  need `ev.source` here because the message already contains `player_id`.

- For `RequestTruckInventoryChange`: same pattern — find the client by `player_id` and set `needs_full_sync = true` on
  that specific client. Currently it sets the global `needs_full_sync` which was for the single connection. Now it
  should set it on the specific client:

  ```rust
  // After processing the inventory change:
  if let Some(conn) = &mut params.network_conn
      && let NetworkConn::Host { clients, .. } = &mut **conn
  {
      for client in clients.iter_mut() {
          if client.associated_id == Some(*player_id) {
              client.needs_full_sync = true;
              break;
          }
      }
  }
  ```

### Phase 1, Step 11: Update client-side systems

**Files to edit:**

- `crates/unnet-plugin/src/systems/client_input.rs`

**Changes:**

- Every `conn.send(...)` call becomes `conn.client_send(...)`. There are 8 `conn.send()` call sites in
  `client_input.rs`:
  1. `conn.send(NetworkMessage::RequestFullSync { player_id })` → `conn.client_send(...)`
  2. `conn.send(NetworkMessage::PlayerInput { ... })` → `conn.client_send(...)`
  3. `conn.send(ev.message.clone())` (in the CraftRepellent/Truck/Interaction passthrough) →
     `conn.client_send(ev.message.clone())` 4-8. `conn.send(NetworkMessage::GrabRequest(...))`, `DropRequest`,
     `CycleInventoryRequest`, `SwapHandsRequest` → all become `conn.client_send(...)`

- `conn.is_active()` checks remain valid (the method still exists and works for `Client`).

### Phase 1, Step 12: Update `network_io_system` signature

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

Add `ResMut<PlayerRegistry>` to the system signature so it can call `player_registry.get_or_assign()` when processing
`Hello` messages from host-side clients.

```rust
pub(crate) fn network_io_system(
    mut conn: ResMut<NetworkConn>,
    mut ev_writer: MessageWriter<NetworkDataEvent>,
    mut ev_disconnect: MessageWriter<unnet_core::messages::NetworkDisconnectEvent>,
    mut ev_send: MessageReader<unnet_core::messages::SendNetworkMessage>,
    mut player_registry: ResMut<crate::resources::PlayerRegistry>,
) {
```

### Phase 1, Step 13: Workspace check

**Action:** Run `get_errors` (no file filter). Fix all errors. Re-run until clean. Then grep the workspace for:

- `NetworkConn::Active` — should have ZERO matches.
- `NetworkConn::Listening` — should have ZERO matches.
- `conn.send(` — should have ZERO matches (replaced by `client_send`, `host_broadcast`, `host_send_to`).
- `host_listeners` — should have ZERO matches.

Fix any remaining references. Re-run workspace check. Address all warnings (unused imports, dead code, etc.).

### Phase 1, Step 14: Fix `setup.rs` wildcard imports

**File to edit:** `crates/unnet-plugin/src/systems/setup.rs`

**Action:** Replace the 5 wildcard imports with explicit imports:

```rust
use super::client_input::{
    client_process_pending_map, client_request_grab_system, client_send_input_system,
    client_sync_intended_gear_state,
};
use super::client_sync::{client_apply_snapshots_system, delayed_despawn_system};
use super::connection::{
    autostart_net_game, client_connection_monitor_system, handshake_handler_system,
    host_handle_disconnects_system, network_io_system, startup_network_system,
};
use super::host_input::host_apply_input_system;
use super::host_sync::{host_send_snapshots_system, host_send_summary_system};
```

### Phase 1 Validation Checklist

After completing all steps:

- [ ] `get_errors` returns zero errors and zero warnings.
- [ ] Grep for `NetworkConn::Active` returns zero matches.
- [ ] Grep for `NetworkConn::Listening` returns zero matches.
- [ ] Grep for `\.send\(` in `crates/unnet-plugin/src/` returns zero matches (only `client_send`, `host_broadcast`,
      `host_send_to` should exist).
- [ ] Grep for `host_listeners` returns zero matches.
- [ ] `NetworkDataEvent` has a `source: Option<NetworkId>` field.
- [ ] All `NetworkDataEvent` construction sites set `source` appropriately.

---

## Phase 2: Enable N simultaneous client connections

### Phase 2 Goal

Remove the single-client restriction. The host now accepts and maintains multiple simultaneous TCP connections. Each
client independently completes handshake, receives snapshots, and has its inputs applied.

### Phase 2 Prerequisites

Phase 1 must be complete and validated. Phase 2 must NOT begin until Phase 1 passes all validation checks.

### Phase 2, Step 1: Remove single-client restriction from `network_io_system`

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

In Phase 1, the host's `network_io_system` might still have a guard that limits to one client (e.g.,
`if !clients.is_empty() { reject... }`). If so, **remove it**. The accept loop should always push new connections into
`clients` regardless of how many already exist.

The accept loop (inside `NetworkConn::Host`) should look like:

```rust
for listener in listeners.iter() {
    loop {
        match listener.accept() {
            Ok((stream, addr)) => {
                info!("Network: Client connected from {}", addr);
                if let Err(e) = stream.set_nonblocking(true) {
                    error!("Failed to set client stream non-blocking: {}", e);
                    continue;
                }
                if let Err(e) = stream.set_nodelay(true) {
                    error!("Failed to set TCP_NODELAY for client: {}", e);
                }
                clients.push(ClientConnection {
                    stream,
                    read_buffer: String::new(),
                    write_queue: VecDeque::new(),
                    handshake: HandshakeState::None,
                    installation_id: None,
                    associated_id: None,
                    needs_full_sync: false,
                });
            }
            Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => break,
            Err(e) => {
                error!("Network: Accept error: {}", e);
                break;
            }
        }
    }
}
```

No rejections. No limits.

### Phase 2, Step 2: Handle duplicate connections (same `installation_id`)

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

**Problem:** If a player disconnects and reconnects quickly (or opens a second client with the same
`--installation-id-file`), `player_registry.get_or_assign()` returns the same `NetworkId` for both connections. Two
entries in `clients` would share the same `associated_id`, causing confusion: if one disconnects, the player entity gets
marked as disconnected even though the other stream is active.

**Solution:** In `do_client_io`, when processing a `Hello` message and assigning an `associated_id` via
`player_registry.get_or_assign()`, **check if any existing client already has that `associated_id`**. If so, mark the
OLD connection for closure:

```rust
// After: let id = player_registry.get_or_assign(installation_id);
// Check for duplicate connections with the same NetworkId
for other in existing_clients.iter_mut() {
    if other.associated_id == Some(id) {
        warn!("Network: Closing old connection for {:?} (reconnect)", id);
        // Mark for closure — the IO loop will handle cleanup
        other.mark_closed = true;
    }
}
client.associated_id = Some(id);
```

This requires adding a `pub mark_closed: bool` field to `ClientConnection` (default `false`). In the IO loop, treat
`mark_closed` the same as a TCP read error — emit `NetworkDisconnectEvent` and remove the client. **But do NOT emit
`NetworkDisconnectEvent`** for the old connection IF the new connection has the same `associated_id` — the player is
reconnecting, not disconnecting.

**Alternative simpler approach:** Since `do_client_io` processes one client at a time and doesn't have access to the
other clients, handle this in `network_io_system` **after** the `do_client_io` loop completes:

```rust
// After all client IO, check for duplicate associated_ids
let mut seen_ids: std::collections::HashMap<NetworkId, usize> = std::collections::HashMap::new();
let mut duplicates_to_remove = Vec::new();
for (idx, client) in clients.iter().enumerate() {
    if let Some(id) = client.associated_id {
        if let Some(&prev_idx) = seen_ids.get(&id) {
            // Keep the newer connection (higher index), close the older one
            duplicates_to_remove.push(prev_idx);
            warn!("Network: Closing stale connection at index {} for {:?} (superseded)", prev_idx, id);
        }
        seen_ids.insert(id, idx);
    }
}
for idx in duplicates_to_remove.into_iter().rev() {
    clients.remove(idx);
    // Do NOT emit NetworkDisconnectEvent — the player is reconnecting
}
```

Run this cleanup AFTER the main IO loop but BEFORE removing truly-closed connections.

### Phase 2, Step 3: Verify `handshake_handler_system` works per-client

**File to edit:** `crates/unnet-plugin/src/systems/connection.rs`

The host-side handshake logic processes `Hello` events from `ev_reader`. With N clients, multiple `Hello` events may
arrive in the same frame. Verify that:

1. Each `Hello` event has a unique `source: Some(NetworkId)` (assigned in `network_io_system` via
   `player_registry.get_or_assign()`).
2. The system iterates ALL events (the `for ev in ev_reader.read()` loop), not just one.
3. `Welcome` is sent via `conn.host_send_to(source_id, Welcome { ... })`, targeting only the client that sent the
   `Hello`.
4. Each client's `handshake` and `needs_full_sync` are set independently.

If the Phase 1 implementation already handles this correctly, no code changes are needed. Just verify by reading the
code.

### Phase 2, Step 4: Verify `host_send_snapshots_system` broadcasts correctly

**File to edit:** `crates/unnet-plugin/src/systems/host_sync.rs`

Already changed in Phase 1 to iterate all clients and send per-client snapshots. Verify that:

1. Clients with `handshake != Completed` do NOT receive snapshots.
2. Each client gets its own `is_full_sync` value.
3. The `changed_tiles` are cleared ONCE (not per-client). The full tile list is built once if any client needs it.

### Phase 2, Step 5: Verify `host_apply_input_system` handles N clients

**File to edit:** `crates/unnet-plugin/src/systems/host_input.rs`

This system reads `NetworkDataEvent` and processes `PlayerInput`, `InteractionRequest`, `RequestTruckEntry`, etc. With N
clients, events from different clients will be interleaved. Verify:

1. The `for ev in params.ev_reader.read()` loop processes ALL events.
2. Each event's `player_id` correctly identifies which player entity to modify.
3. No system assumes a single source client.

This should already work correctly because the system uses `player_id` from the message to find the right entity, not
any connection-level identity.

### Phase 2, Step 6: Workspace check

**Action:** Run `get_errors`. Fix all errors and warnings. Verify no regressions.

### Phase 2 Validation Checklist

- [ ] `get_errors` returns zero errors and zero warnings.
- [ ] Grep for `"Rejecting connection"` returns zero matches (no more rejection logic).
- [ ] The host's accept loop in `network_io_system` pushes ALL new connections into `clients` without restriction.
- [ ] `handshake_handler_system` sends `Welcome` via `host_send_to`, not `host_broadcast`.
- [ ] `host_send_snapshots_system` only sends to clients with `handshake == Completed`.
- [ ] When a second connection arrives with the same `installation_id`, the old connection is removed and no
      `NetworkDisconnectEvent` is emitted for it.

---

## Phase 3: Per-client snapshot optimization and correctness

### Phase 3 Goal

Ensure per-client state is correctly managed: each client has independent `needs_full_sync`, each client receives
snapshots only after completing handshake, and the changed-tiles delta system works correctly with N clients.

### Phase 3, Step 1: Fix changed-tiles delta logic for N clients

**File to edit:** `crates/unnet-plugin/src/systems/host_sync.rs`

**Problem:** `ChangedTiles` is a single `Vec` that accumulates tile changes between frames. Currently, it is drained
once per frame when building the snapshot. With N clients, if client A needs a full sync and client B doesn't:

- Client A gets the full tile list (correct).
- Client B gets the delta tiles from `ChangedTiles` (correct).
- But `ChangedTiles` is drained/cleared for the frame, so it works.

The issue arises ONLY if a new client connects mid-game. They need a full sync, which is handled by
`needs_full_sync = true`. So the current architecture is fine.

**No changes needed** if the Phase 1 implementation correctly:

1. Clears `changed_tiles` once per frame (not per client).
2. Builds the full tile list separately for clients needing full sync.
3. Uses the delta list for clients that don't need full sync.

Verify this by reading the code.

### Phase 3, Step 2: Guard snapshot sending on handshake completion

**File to edit:** `crates/unnet-plugin/src/systems/host_sync.rs`

In the snapshot sending loop, add a guard:

```rust
for client in clients.iter_mut() {
    if client.handshake != HandshakeState::Completed {
        continue;
    }
    // ... send snapshot ...
}
```

This prevents sending snapshots to clients that are still in the handshake process.

### Phase 3, Step 3: Workspace check

**Action:** Run `get_errors`. Fix all errors and warnings.

### Phase 3 Validation Checklist

- [ ] `get_errors` returns zero errors and zero warnings.
- [ ] Only clients with `handshake == Completed` receive snapshots.
- [ ] `ChangedTiles` is cleared once per frame, not per client.

---

## Summary of all files modified

| File                                                          | Phases | Change type                                   |
| ------------------------------------------------------------- | ------ | --------------------------------------------- |
| `crates/unnet-core/src/messages.rs`                           | 1      | Add `source` field to `NetworkDataEvent`      |
| `crates/unnet-plugin/src/resources.rs`                        | 1      | New `ClientConnection`, rewrite `NetworkConn` |
| `crates/unnet-plugin/src/systems/connection.rs`               | 1, 2   | Rewrite IO, handshake, startup systems        |
| `crates/unnet-plugin/src/systems/host_sync.rs`                | 1, 3   | Two-snapshot approach, per-client send        |
| `crates/unnet-plugin/src/systems/host_input.rs`               | 1      | Per-client `needs_full_sync`                  |
| `crates/unnet-plugin/src/systems/client_input.rs`             | 1      | `send` → `client_send`                        |
| `crates/unnet-plugin/src/systems/setup.rs`                    | 1      | Fix wildcard imports                          |
| `crates/untruck-plugin/src/systems/in_truck_manager.rs`       | 1      | Add `source: None` to `NetworkDataEvent`      |
| `crates/uninteraction-plugin/src/systems/interactivestuff.rs` | 1      | Add `source: None` to `NetworkDataEvent`      |

**Files NOT modified** (verify these remain untouched):

- `crates/unnet-plugin/src/systems/client_sync.rs` — No changes needed. This file reads `NetworkDataEvent` (which now
  has `source` but client_sync ignores it) and uses `ClientSnapshotParams`. No `conn.send()` calls.
- `crates/unnet-plugin/src/systems/utils.rs` — Utility code, no connection access.
- `crates/unnet-plugin/src/systems/mod.rs` — Only `mod` statements.
- `crates/unnet-plugin/src/plugin.rs` — Resources and plugin setup, no changes.
- `crates/unnet-plugin/src/metrics.rs` — Metric constants, no changes.
- `crates/unnet-core/src/network_id.rs` — No changes.
- `crates/unnet-core/src/resources.rs` — No changes.
- `crates/unnet-core/Cargo.toml` — No changes (no new dependencies).
- `crates/unnet-plugin/Cargo.toml` — No changes (no new dependencies).
- `crates/untruck-plugin/src/journal.rs` — Only reads `NetworkDataEvent.message`, does not construct it. No changes
  needed.

## Dependency notes

No new crate dependencies are needed. `uuid` is already in both `unnet-core` and `unnet-plugin` Cargo.toml files.
`serde_json`, `bevy`, `rand` — all already present.

## What this plan does NOT cover (future work)

- Player spawn positions for >2 players (map/spawn system changes)
- Lobby / waiting room UI
- Dedicated server binary
- NAT traversal / relay
- Server browser / discovery
- Transport abstraction (WebSocket, QUIC)
