# Multiplayer MVP: Detailed Execution Plan

This document expands on the high-level goals of the MVP into a concrete, step-by-step implementation guide.

## Phase 1: Foundation & Scaffolding

### 1.1 New Crates Creation

We need to create two new crates to house the networking logic, keeping dependencies clean.

- **`crates/unnet-core`**:
  - **Purpose**: Shared types, protocol definitions, serialization logic.
  - **Dependencies**: `serde`, `bevy` (for Component derives), `thiserror`.
- **`crates/unnet-plugin`**:
  - **Purpose**: Systems, Resources, TCP Socket management, Bevy integration.
  - **Dependencies**: `unnet-core`, `bevy`, `std::net` (TCP), `crossbeam-channel` or `flume` (for non-blocking IO
    signaling), `serde_json`.

### 1.2 Workspace Update

- Update `Cargo.toml` in the root to include the two new crates.
- Add `unnet-plugin` to `unhaunter/src/app.rs` (initially disabled or behind the configured flag).

### 1.3 CLI & Config Mode

- **Modify** `crates/untypes-core/src/cli.rs`:
  ```rust
  #[derive(Clone, Debug, Serialize, Deserialize)]
  pub enum NetMode {
      Offline,
      Host { port: u16 },
      Client { address: String },
  }
  // Add `pub net_mode: NetMode` to CliOptions
  ```
- **Parse Flags**: Update `unhaunter/src/lib.rs` (or where Clap is used) to parse `--host <PORT>` and `--join <ADDR>`.

---

## Phase 2: The Wire (Protocol & Transport)

### 2.1 The Protocol (`unnet-core`)

Define the `NetworkMessage` enum (using `serde`).

```rust
#[derive(Serialize, Deserialize, Debug)]
pub enum NetworkMessage {
    // Handshake
    Hello { version: String, player_name: String },
    Welcome { your_id: u64, map_seed: u64 },

    // Gameplay (Reliable - TCP guarantees order)
    PlayerInput {
        x: f32,
        y: f32,
        interact: bool
    },

    // State (Snapshot)
    Snapshot {
        tick: u64,
        players: Vec<PlayerState>,
        ghosts: Vec<GhostState>,
    }
}

pub struct PlayerState {
    pub id: u64,
    pub pos: (f32, f32),
    pub dir: (f32, f32),
    pub anim_state: String, // Simplified
}
```

### 2.2 The Socket Plugin (`unnet-plugin`)

Since Bevy systems run once per frame, we cannot block on `socket.read()`.

- **IO Threads**:
  - `TcpListener` (Host) and `TcpStream` (Client) should ideally run in a separate thread or use non-blocking methods.
  - For MVP simplicity: Use **non-blocking** mode with `std::net::TcpStream::set_nonblocking(true)`. In the `Update`
    system, try allowed reads until `WouldBlock`.
- **Resources**:
  - `NetworkConn`: Wrapper around `TcpStream`.
  - `NetworkConfig`: Stores `NetMode`.

---

## Phase 3: The Game Loop Integration

### 3.1 Connection State Machine

Implement a `NetworkState` state in Bevy (or just a Resource) to track connection progress.

- **Client**: `Connecting` -> `Handshaking` -> `Connected`.
- **Host**: `Listening` -> `ClientConnected` (1:1 limit for MVP) -> `InGame`.

### 3.2 Spawning Logic (`classic_mode_orchestrator`)

This is the most critical logic change.

- **Current**: Spawns 1 Player with `MainPlayer`.
- **New Logic (Host)**:
  - Spawn Local Player: `MainPlayer`, `NetworkId(1)`.
  - Wait for Client Join.
  - Spawn Remote Player (Client): `NetworkId(2)`, `PlayerSprite`. **NO `MainPlayer`**.
- **New Logic (Client)**:
  - Connect. Receive `Welcome(id=2)`.
  - Spawn Local Player: `MainPlayer`, `NetworkId(2)`.
  - Spawn Remote Player (Host): `NetworkId(1)`, `PlayerSprite`. **NO `MainPlayer`**.

### 3.3 State Synchronization Systems

- **Host (Send Snapshot)**:
  - Query all `Position`, `Direction`, `NetworkId`.
  - Construct `NetworkMessage::Snapshot`.
  - Serialize to JSONL (`json + \n`).
  - Write to TCP Stream.
- **Client (Receive Snapshot)**:
  - Read JSONL from TCP.
  - Parse `NetworkMessage::Snapshot`.
  - Query `(Entity, &NetworkId)`. Match IDs.
  - Update `Position` and `Direction` (Interpolation: None for now, just teleport).

- **Client (Send Input)**:
  - Query `&PlayerInput` from `MainPlayer`.
  - Send `NetworkMessage::PlayerInput`.
- **Host (Receive Input)**:
  - Read `PlayerInput` msg.
  - Apply to the `PlayerInput` component of the entity with `NetworkId(2)`.

---

## Phase 4: Verification Steps

1. **Verify CLI**: Run `./target/debug/unhaunter --host 5000` and ensure it doesn't crash.
2. **Verify TCP**: Run Host, then Client. Use `netcat` or logs to confirm connection.
3. **Verify Ghosting**:
   - Host moves. Client sees "Ghost Player" move.
   - Client moves. Host sees "Ghost Player" move.
   - **Success Condition**: Two screens, two moving sprites.

## Implementation Details & "Gotchas"

- **MainPlayer Component**: This component _only_ exists on the locally controlled entity. Input systems
  (`keyboard_input_system`) write to it.
- **Remote Entities**: They have `PlayerInput` component too! But it is written by the _Network System_, not the
  Keyboard System. The `player_movement_system` runs on _all_ entities with `PlayerInput`, so it will naturally animate
  and move the remote player based on the received input data (Host Side) or just sync Position (Client Side).
  - On the Client side, the Remote Player (Host) is purely visual. We force overwrite `Position` from snapshots.
  - **Decision**:
    - **Host**: Simulates EVERYTHING. Receives Input from Client. Runs physics for both.
    - **Client**: Does NOT simulate. Sends input to Host. Receives snapshots. Renders what Host says.
    - **Latency**: Client will experience round-trip latency on their own movement. This is acceptable for MVP.

## Phase 5: MVP Refinements

- **Smoothness**: Add basic `Transform::lerp` between snapshot updates (post-MVP).
- **Interaction**: Add `NetworkMessage::Interact` to sync button presses (e.g. `E`).
- **Events**: Ensure `RoomChangedEvent` or `GhostSighted` are somewhat consistent, though strict sync isn't needed for
  MVP walking test.

---

## Self-Review & Critique

### Issues Identified & User Decisions

#### 1. **60Hz JSONL over TCP**

- **Original Concern**: 30KB/s bandwidth might be excessive.
- **User Decision**: **Not a problem.** Modern connections handle MiB/s easily. 30KB/s is nothing. Do not prematurely
  optimize. Keep 60Hz for MVP.

#### 2. **Client-Side Prediction**

- **Original Concern**: Prediction + reconciliation is complex.
- **User Decision**: **No prediction. No rollback. No reconciliation.** Client sends input, waits for Host snapshot,
  renders that. Higher latency is acceptable for MVP.

#### 3. **Spawning Timing / Mid-Session Join**

- **Original Concern**: Race conditions if client joins mid-game.
- **User Decision**: **Client CAN join mid-session.** On new connection:
  - Server sends a full snapshot (or client requests one).
  - Client agrees to everything the server says.
  - No hard lobby sync required. CLI flags specify map/difficulty directly—both instances jump straight to the mission
    with no menu.

#### 4. **Disconnection Handling**

- **User Decision**: **Not needed for MVP.** If disconnect happens, server freezes or crashes. That's fine for now.

#### 5. **Map Selection**

- **User Decision**: **CLI flags specify everything.** Server specifies campaign number, map, or difficulty via flags.
  Client must match or the `Hello` handshake fails. No runtime map selection UI needed.

#### 6. **Ghost AI State**

- **User Decision**: **Option B confirmed.** Ghost is purely visual on Client. No Ghost AI runs on Client. Host sends
  Position + Animation. Client renders.

#### 7. **Version/Protocol Check**

- **User Decision**: **Simple `Hello` message check.** Both must be running "Unhaunter" with reasonably similar
  versions. A few strings must match. No complex negotiation.

---

## Simplified Phase Order (MVP-Focused)

### Phase 1: Foundation

- Create `unnet-core` and `unnet-plugin` crates.
- Add CLI flags (`--host <PORT>`, `--join <ADDR>`, `--map <PATH>`, `--difficulty <ID>`).
- Wire up plugin in `app.rs`.

### Phase 2: Connection & Handshake

- Implement TCP connect/accept (non-blocking).
- Implement `Hello` / `Welcome` exchange.
- Validate version strings match.
- **Exit Criteria**: Two instances connect and log "Player connected!".

### Phase 3: Walking Together

- Host spawns both players on mission start.
- Client connects mid-mission, requests full snapshot.
- Client spawns entities based on snapshot.
- Client sends `PlayerInput` every frame.
- Host applies input, runs physics, sends `Snapshot` at 60Hz.
- Client overwrites all positions from snapshot (no prediction).
- **Exit Criteria**: Two screens. Movement on either side visible on both.

### Phase 4: Ghost Sync

- Host includes Ghost position in snapshot.
- Client disables Ghost AI systems (guard with `is_host` check).
- Client renders Ghost at received position.
- **Exit Criteria**: Ghost moves identically on both screens.

### Phase 5: Interactions

- Sync `Toggleable` state changes (lights, doors).
- Sync item pickups/drops.

---

## Remaining Risks

| Risk                     | Likelihood | Impact | Notes                                                               |
| ------------------------ | ---------- | ------ | ------------------------------------------------------------------- |
| TCP message framing bugs | Medium     | Medium | JSONL newline parsing edge cases. Test thoroughly.                  |
| Entity ID mismatch       | Medium     | High   | `NetworkId` must be assigned consistently. Host is source of truth. |
| Client joins mid-hunt    | Low        | Medium | Full snapshot should handle this, but edge cases may exist.         |
| High latency feel        | Medium     | Low    | Acceptable for MVP. Can add interpolation later.                    |

---

## Second Review Pass

After incorporating user feedback, the plan is now significantly simpler:

### What Was Removed

- ❌ Client-side prediction and reconciliation
- ❌ Lobby state machine with `MapReady` synchronization
- ❌ Disconnection handling
- ❌ Bandwidth optimization concerns
- ❌ Phase 0 spike (unnecessary complexity)

### What Remains (Core MVP)

- ✅ TCP transport with JSONL at 60Hz
- ✅ Simple `Hello`/`Welcome` handshake with version check
- ✅ CLI-driven connection (no menu UI)
- ✅ Host runs all simulation; Client is a "dumb terminal"
- ✅ Ghost is purely visual on Client
- ✅ Mid-session join via full snapshot

### Open Questions & Decisions for Implementation

1. **Full Snapshot Contents**: What exactly goes in the "full snapshot" for mid-session join?
   - **Decision**: To be determined during development. The contents must ensure consistency based on what state is
     critical at join-time.

2. **Entity Spawning on Client**:
   - **Decision**: **Both load the same map.** Configuration (Map + Difficulty) is defined via CLI flags.

3. **NetworkId Assignment**: Who assigns `NetworkId` values?
   - **Decision**: **Host assigns all IDs.**
   - Fixed values: Host Player `NetworkId(1)`, Client Player `NetworkId(2)`, Ghost `NetworkId(100)`.
   - Other entities (doors, lights, items) will need a deterministic or host-assigned ID mapping.
