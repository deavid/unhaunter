# Hub Service: Design Proposal (v1)

- **Date:** 2026-02-15
- **Status:** Draft / Proposal
- **Role:** Discovery, Social, and Orchestration Layer

---

## 1. The Problem

Currently, Unhaunter multiplayer requires players to share IP addresses. This is a high-friction barrier that prevents
"spontaneous" play and community growth. We need a central point of discovery that allows players to find each other
using human-readable codes.

## 2. Core Philosophy

- **Frictionless:** No accounts, no passwords, no email. Identity is based on Installation UUID.
- **Lightweight:** Non-Bevy, async Rust (Axum + In-memory state). Must handle thousands of concurrent connections on
  minimal hardware (e.g., 512MB RAM VPS).
- **Dedicated Server Architecture:** `unhub` doesn't run game code. It routes players to `undedicated` game server
  instances managed by `unprocman`. The Hub never transmits player IP addresses in its data payloads.
- **FOSS Values:** The game must still work without the Hub (direct IP connect), but the Hub is the "official"
  experience.

---

## 3. Functional Requirements & User Experience

The Hub is the "connective tissue" of the Unhaunter community. Its primary goal is to remove the technical burden of
networking from the player and replace it with a social, discovery-focused interface.

### 3.1 The "Frictionless Connection" (Must-Have)

The most critical requirement is the transition from "I want to play with friends" to "We are in a lobby together."

- **Room Codes:** Instead of sharing IP addresses (which are scary, dynamic, and often hidden behind NAT), players share
  a 4-5 character mnemonic code. These codes are designed to be "Safe-Vocal": easy to read, easy to type, and
  phonetically distinct over voice chat.
- **Automatic Handoff:** When a player enters a code, the Hub performs the handshake. It checks if the room exists, if
  it's full, and what version of the game it's running. If everything matches, it hands the client the address of the
  `undedicated` instance hosting that room, and the client connects automatically.
- **Late-Join Support:** The Hub must track if a mission is already in progress. It should warn players if they are
  joining a game that has already started, or prevent it if the host has locked the room.

### 3.2 Discovery & The "Living World" (Future — Not Part of v1)

Not part of the MVP. Mentioned here only for future context.

- **Public Room Browser:** A list of rooms marked "Public" for solo players to find groups. Requires public/private room
  tagging, live metadata (map, difficulty, player count), and server health monitoring to avoid stale entries.
- This feature is deferred until the core private-room flow is proven in production.

### 3.3 Identity without Accounts

We want to recognize players without forcing them to manage another password.

- **Installation Persistence:** By using the Installation UUID, the Hub can remember a player's preferred nickname.
- **Moderation:** The Hub provides the infrastructure for "Soft Bans." If a player is reported for trolling, the Hub can
  ban their UUID from all Hub services (room creation, joining, browsing). Banned players can still play single-player
  and direct-IP multiplayer. Ban evasion by regenerating a UUID is acknowledged as possible but accepted — in the
  process, the player loses all their progress.

### 3.4 The "Mixer" (The Social Goal)

The ultimate goal of the Hub is to solve the "5-player problem." If there are 5 people globally wanting to play, but
they are all in their own private lobbies, nobody plays.

- **Matchmaking/Queueing:** The Hub can act as a meeting point. "I don't have a group, just put me with whoever is
  ready."
- **Orchestration:** The Hub can see that 4 people are in the queue and that an Official Dedicated Server is idle. It
  can "command" the server to open a room and "invite" the 4 players to it simultaneously.

---

## 4. Technical Architecture

### 4.1 Stack

- **Language:** Rust
- **Web Framework:** [Axum](https://github.com/tokio-rs/axum) (Tokio-based)
- **State Management:** In-memory `DashMap` for active rooms and heartbeats (ultra-low latency).
- **Persistence:** High-performance File-based Persistence using **JSON** or **RON**.
  - No database engine to manage.
  - Schema is code-first (Rust structs) and migrated at load time via versioned compatibility logic.
  - "Cold" data (bans, official server registry) is loaded into RAM on startup and saved asynchronously on change.
- **Communication:** REST API for metadata; potentially WebSockets for real-time presence.

### 4.2 Data Model (Rust Structs)

Since we are avoiding SQL, the "Schema" is simply the Rust data structures.

**Persistent State (Saved to disk):**

```rust
struct HubConfig {
    // List of UUIDs that are allowed to host "Official" ranked games
    official_server_keys: HashMap<Uuid, String>,
    // UUIDs banned from the master server
    banned_uuids: HashSet<Uuid>,
}
```

**Transient State (In-Memory only):**

```rust
struct HubState {
    // The "World State"
    servers: DashMap<Uuid, ServerInfo>,
    // The active lobbies
    rooms: DashMap<String, RoomInfo>, // Key is the 5-char code
}

struct RoomInfo {
    server_id: Uuid,
    player_count: u8,
    state: RoomState, // Lobby, InGame
    metadata: RoomMetadata, // Map, Difficulty
}
```

### 4.3 Room Code Generation (Safe-Vocal)

To minimize friction in voice chat and prevent accidental slurs, the Hub uses a "Safe-Vocal" alphabet for room codes.

- **The Alphabet:** `C, D, F, G, H, J, K, L, M, P, R, S, T, V, W, X, 2, 4, 7, 9` (Base-20).
- **Design Principles:**
  - **Phonetic Distinctness:** Excludes the "E-group" (B, E, P, T, V, Z) where possible and avoids visually/aurally
    ambiguous pairs like `1/I/L`, `0/O/Q`, or `M/N`.
  - **Slur Prevention:** By removing all vowels (A, E, I, O, U) and the pseudo-vowel Y, it is mathematically impossible
    to form the vast majority of offensive words.
  - **Numbers:** Only includes `2, 4, 7, 9` as they are phonetically unique compared to the chosen consonants.
- **Entropy:**
  - A 4-character code provides $20^4 = 160,000$ combinations.
  - A 5-character code provides $20^5 = 3,200,000$ combinations.
- **Recommendation:** Use **5 characters** (e.g., `K7WRP`) to virtually eliminate collisions while remaining easy to
  "chunk" and communicate.

### 4.4 Persistence Tradeoffs & Migration Story

**Why RON/JSON for v1 (Ultra-Lean Goal):**

- **Operational Simplicity:** No DB daemon, no SQL migration files, easy backup/restore via file copy.
- **Admin-Friendly:** Operators can inspect and edit configuration-like state directly.
- **Cost Efficiency:** Fits the low-RAM, low-CPU VPS target when active state remains in-memory.

**What We Give Up vs SQLite:**

- **Weaker Integrity Guarantees:** No relational constraints or transactional query semantics.
- **Less Query Power:** Analytics and ad-hoc querying are harder than SQL.
- **Manual Edit Risk:** Human edits can introduce invalid data unless validated on load.

**Schema/Migration Policy (Not Eliminated, Just Different):**

1. Persist a top-level `version` field in the RON/JSON file.
2. Use backward-compatible deserialization (`Option<T>`, defaults) for additive changes.
3. On startup, load legacy versions and up-convert to the current Rust structs.
4. Rewrite the file in current format using atomic write (`tmp` + rename), keeping a `.bak`.
5. Keep only a small migration window (e.g., current + previous 2 versions) to limit complexity.

This approach keeps the Hub lean while still having an explicit, maintainable schema evolution story.

### 4.5 Client Configuration & Universe Discovery

The game client discovers the Hub via a static configuration asset:

**`assets/config/client.ron`:**

```ron
UnhaunterClientConfig(
    universe: Universe(
        codename: "Unhaunter.com",
        uri: "https://hub.unhaunter.com/v1",
    ),
)
```

- **Universe:** A logical grouping of `unhub` instances, `unprocman`/`undedicated` hosts, rooms, and stats that form a
  cohesive ecosystem. Multiple Hub instances can serve the same Universe (via DNS round-robin or multiple A/NS records)
  for redundancy.
- **Override:** The binary accepts `--universe https://domain.tld/path` to override the default.
- **Non-Blocking:** The Hub connection happens in the background on game boot. If the Hub is unreachable, single-player
  and direct-IP multiplayer remain fully functional. The UI disables or shows errors on Hub-dependent actions (Create
  Room, Join Room) when the connection is unavailable.
- **TLS:** Production Hubs should serve HTTPS (required for WASM clients). HTTP is supported for development. TLS
  certificate provisioning (e.g., Let's Encrypt) is the responsibility of the sysadmin deploying the Hub.

### 4.6 System Components

The multiplayer infrastructure consists of three separate binaries:

| Binary        | Name                      | Role                                                                                                                                                                                                                                                                                                                        |
| ------------- | ------------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unhub`       | **Hub**                   | Central discovery service. REST API. Routes players to game servers via room codes. Manages room code ↔ server mappings in memory. Persists bans and config to RON files.                                                                                                                                                   |
| `unprocman`   | **Process Manager**       | Co-located with game server binaries on a hosting machine. Spawns, monitors, and kills game server processes. Maintains a configurable pool of idle servers (default: 1). One ProcMan per machine. Manages a reserved TCP port range (e.g., 11000–11999). Can manage multiple `undedicated` binary versions simultaneously. |
| `undedicated` | **Dedicated Game Server** | The actual game server. One process = one room = one TCP port. Headless Bevy. Runs the game simulation. Players connect to it directly via `unnet` after the Hub handoff.                                                                                                                                                   |

**Communication Topology:**

```
Players ──HTTPS──► Hub ◄──persistent TCP──┬── ProcMan A (machine 1, ports 11000-11999)
                                          │      ├── undedicated :11000 (idle, no room)
                                          │      ├── undedicated :11001 (room H4KRP, 3 players)
                                          │      └── undedicated :11002 (room X9FDC, in-game)
                                          │
                                          └── ProcMan B (machine 2, ports 11000-11999)
                                                 └── undedicated :11000 (idle, no room)

Players ──TCP──► undedicated (direct game traffic, after Hub handoff)
```

**Key Properties:**

- **ProcMan connects TO the Hub** (outbound), not the other way around. This means the Hub doesn't need to know ProcMan
  IPs upfront — ProcMans self-register by connecting. Firewalls and NAT are not an issue for the ProcMan→Hub link.
- **Hub can talk to multiple ProcMans** across different machines/VPS. The Hub selects one with idle capacity when a
  room is requested.
- **ProcMan manages the port range.** Configured with a range (e.g., `port_range: 11000..12000`). It assigns the next
  free port when spawning an `undedicated` process. When an `undedicated` exits, the port is reclaimed.
- **ProcMan maintains the idle pool.** Configured with `idle_pool_size` (default: 1). When an idle `undedicated` gets
  assigned a room, ProcMan spawns a replacement. When an `undedicated` exits after its room empties, ProcMan replenishes
  the pool as needed.
- **Room ID on `undedicated` is `Option<String>`.** An idle server has `None`. The ProcMan (or Hub, via ProcMan) assigns
  a room code to activate it. The room code can be changed or stripped while the server is live — players already
  connected stay, but no new players can join via the old code.

**"Create Room" Flow:**

1. Player → Hub: `POST /v1/rooms/create`
2. Hub selects a ProcMan with idle capacity.
3. Hub → ProcMan (over persistent connection): "Allocate a room."
4. ProcMan assigns an idle `undedicated`, tells it: "You are now room `H4KRP`."
5. ProcMan → Hub: "Room ready at `203.0.113.5:11003`."
6. Hub → Player: `{ code: "H4KRP", addr: "203.0.113.5:11003" }`
7. Player → `undedicated`: direct TCP connection via `unnet`.

**`undedicated` Lifecycle:**

```
Spawned (idle, no room) → Assigned (room code set, lobby) → In-Game → Back to Lobby → ...
    → All players leave → 5min timeout → Exit
                                           ↑
                            ProcMan detects exit, reclaims port,
                            spawns replacement if pool < idle_pool_size
```

---

## 5. The "Mixer" Mode (Phase 2)

The Mixer is not a matchmaking queue. It is a simpler concept: rooms can be tagged as "mixer" rooms. A player clicking
"Mixer" is dropped into a random mixer-tagged room that is in lobby state with available slots. If none exist, a new
mixer room is created on an available `undedicated` via `unprocman`. After a mission ends and players return to the
lobby, they may be re-shuffled into different rooms to keep the social mix fresh.

---

## 6. API Sketch

- `POST /v1/rooms/create` -> Returns `{ code: "XJ92" }`
- `GET /v1/rooms/join/:code` -> Returns `{ addr: "1.2.3.4:1234" }` (`undedicated` address)
- `GET /v1/browser/list` -> Returns a list of active public rooms. _(Future — not part of v1.)_

---

## 7. Open Questions

1. **Lobby Chat:** Should `unhub` handle lobby chat, or should that happen on the `undedicated` once connected? (Leaning
   towards `undedicated` to keep `unhub` "thin").
2. **WASM Multiplayer:** Out of scope for short/mid term. The current game networking uses raw TCP, which is unavailable
   in browsers. WASM multiplayer will require a transport change (WebSocket/WebRTC) and is deferred until the desktop
   multiplayer reaches production-ready state. The Hub itself will be reachable via HTTPS from WASM clients for
   browsing/discovery purposes.

---

## 8. Next Steps

1. Initialize `crates/tools/unhub` as a standalone binary.
2. Implement basic Room Code generation and lookup.
3. Add Hub client logic in a new `unhub-*` crate family (separate from `unnet-plugin`).

---

## 9. Player Journeys: End-to-End

These journeys describe the flow from the moment the user launches the executable until they are active in a mission.
All multiplayer through the Hub uses `undedicated` (Dedicated Game Server) instances managed by `unprocman` (Process
Manager). Players never host game servers directly via the Hub.

### 9.1 Journey 1: Creating a New Room

1. **Game Boot:** Player launches `unhaunter`. The game initializes and enters `AppState::MainMenu`. The Hub client (in
   `unhub-*` crate) begins connecting to the Universe endpoint in the background.
2. **Identity:** The game reads the local Installation UUID from the player profile.
3. **Create Action:** Player clicks **"Play Online"** and then **"Create Room"**.
4. **Hub Request:** The game sends `POST /v1/rooms/create` to `unhub`, including the player's UUID and game version.
5. **Hub Processing:**
   - `unhub` validates the request (checks UUID is not banned, applies rate limits, checks version).
   - `unhub` selects a connected `unprocman` with idle capacity.
   - `unhub` asks the `unprocman` to allocate a room.
   - `unprocman` assigns an idle `undedicated` process and sets its room code.
   - `unprocman` reports back: room ready at `addr:port`.
   - `unhub` generates a 5-character Safe-Vocal code (e.g., `H4KRP`), maps it to the `undedicated` address, and returns
     both to the player.
6. **Connection:** The game connects directly to the `undedicated` instance using the existing `unnet` protocol.
7. **Lobby State:** The player enters the Lobby UI. The code `H4KRP` is displayed prominently for sharing with friends.
   As the first player to join, they are designated the **"host"** and can select map and difficulty.
8. **Preparation:** Other players join (see Journey 2). The host selects a map and difficulty.
9. **Launch:** The host clicks **"Start Mission"**.
10. **Transition:** The `undedicated` transitions the room to `InGame` state and notifies `unhub` (via `unprocman`). All
    connected clients switch to `AppState::Loading`, then `AppState::InGame`.
11. **Gameplay:** All players are dropped into the mission board.

### 9.2 Journey 2: Joining an Existing Room

1. **Game Boot:** Player launches `unhaunter`. The game initializes and enters `AppState::MainMenu`. The Hub client
   connects in the background.
2. **Join Action:** Player clicks **"Play Online"** and then **"Join Room"**. They are prompted for a code.
3. **Code Entry:** Player types or pastes the 5-character code provided by the creator (e.g., `H4KRP`).
4. **Hub Lookup:**
   - The game sends `GET /v1/rooms/join/H4KRP` to `unhub`.
   - `unhub` looks up `H4KRP` in its `DashMap`. It checks if the room is full or version-incompatible.
   - `unhub` returns the `undedicated` instance's address.
5. **Connection:** `unhub`'s job is done. The game connects directly to the `undedicated` instance using `unnet`.
6. **Lobby Sync:** The `undedicated` admits the player. They enter the Lobby UI and see the other players, selected map,
   and difficulty.
7. **Loading:** When the host starts the mission, the client receives the "Start" message and transitions to
   `AppState::Loading`.
8. **Gameplay:** The client finishes loading assets and is dropped into the mission board alongside the other players.

### 9.3 Journey 3: Post-Mission

1. **Mission End:** The mission concludes (victory, defeat, or player vote to end).
2. **Return to Lobby:** The `undedicated` transitions the room back to "Lobby" state. `unhub` is notified.
3. **Another Round:** The room code remains valid. The host can select a new map and start another mission.
4. **Disconnect:** Players can leave at any time. When all players have disconnected, the `undedicated` closes the room
   after a timeout (default: 5 minutes) and exits. `unprocman` detects the exit, reclaims the port, and spawns a
   replacement if the idle pool is below target. `unhub` reclaims the room code.

---

## 10. Open Questions & Remaining Blind Spots

This section contains items that still need design decisions or further discussion. Fully resolved items from the
original blind-spot analysis have been integrated into the proposal above.

### 10.1 Trust Model & Authentication

The three-component architecture introduces trust boundaries. The baseline assumption is a **trusted network** (all
components on the same machine or private VLAN). Untrusted deployments require TLS — see below.

**Baseline (Trusted Network):**

1. **ProcMan → Hub Authentication:** Each `unprocman` instance has its own Installation UUID (generated on first run,
   persisted to disk). On connect, ProcMan sends its UUID. The Hub checks it against a list of allowed UUIDs in its
   config (`allowed_procman_uuids: [...]`). Unknown UUIDs are rejected and the connection is dropped. This is sufficient
   when all components share a trusted network where eavesdropping is not a concern.
2. **Player → Hub API Abuse:** Rate limiting per source IP (~5 requests/minute, ~20 requests/hour). UUID-based
   throttling as an additional layer. Auto-banning (both IP and UUID) for detected abuse patterns. Exact thresholds TBD.
3. **Player → `undedicated` Admission:** The player presents the room code in the `unnet` Hello message. The
   `undedicated` checks it matches its assigned code. The code is shared openly among friends, but the 3.2M combination
   space plus rate limiting on the Hub makes brute-force impractical. This is the v1 approach.

**Untrusted Network (Cross-DC, Internet-Facing ProcMan→Hub):**

If the ProcMan→Hub connection crosses an untrusted network, plain TCP with a UUID is vulnerable to eavesdropping and
replay. The solution is **TLS with mutual TLS (mTLS)**:

- Hub presents a server certificate; ProcMan validates it.
- ProcMan presents a client certificate; Hub validates it.
- The UUID-based allow-list still applies as an application-layer check on top of mTLS.
- This is a transport-layer change only — the message protocol remains identical.

mTLS is not required for v1 (trusted network assumption) but the protocol should be designed so TLS can be wrapped
around the connection without message format changes.

### 10.2 Hub ↔ ProcMan Protocol

The persistent TCP connection between `unhub` and `unprocman` uses JSONL (newline-delimited JSON), matching the existing
game protocol in `unnet`. Each line is a self-contained JSON object with a `type` field. The protocol is designed to be
TLS-wrappable without message format changes (see 10.1).

#### 10.2.1 Connection Handshake

On connect (or reconnect after Hub restart), ProcMan sends its full state so the Hub can rebuild its room map:

```
PM → Hub: ProcManHello
  - uuid: ProcMan's Installation UUID
  - version: ProcMan binary version
  - game_versions: ["0.15", "0.16"]    // all undedicated versions this ProcMan can spawn
  - port_range: [11000, 12000]
  - public_addr: "203.0.113.5" (the IP players will connect to)
  - idle_pool: { "0.15": 1, "0.16": 0 } // idle servers per version
  - rooms: [                           // current active rooms (empty if fresh start)
      { code: "H4KRP", port: 11001, game_version: "0.15", secret: "QWER-123133",
        state: "Lobby", players: [uuid1, uuid2], metadata: { map: "...", difficulty: "..." } },
      ...
    ]
  - idle_servers: [{ port: 11000, game_version: "0.15" }]
```

Hub validates the UUID against `allowed_procman_uuids`. If rejected, Hub sends `AuthRejected` and closes the connection.
If accepted, Hub sends `ProcManAccepted` and integrates the ProcMan's rooms into its state.

#### 10.2.2 ProcMan → Hub Messages

| Message               | Fields                                                                                                | Trigger                                  |
| --------------------- | ----------------------------------------------------------------------------------------------------- | ---------------------------------------- |
| **Heartbeat**         | idle_capacity (int), idle_servers (int), rooms summary (code, player count, state, metadata per room) | Periodic, every 60s                      |
| **PlayerJoined**      | room_code, player_uuid, new_player_count                                                              | Forwarded from undedicated               |
| **PlayerLeft**        | room_code, player_uuid, new_player_count                                                              | Forwarded from undedicated               |
| **RoomStateChanged**  | room_code, old_state, new_state, metadata                                                             | Forwarded (Lobby→InGame→Lobby)           |
| **RoomReady**         | room_code, port, secret                                                                               | Response to CreateRoom                   |
| **RoomRenameRequest** | room_code, reason ("host_kick")                                                                       | Host kicked a player, wants new identity |
| **RoomRenamed**       | old_code, new_code, port, new_secret                                                                  | Confirmation of RenameRoom               |
| **RoomClosed**        | room_code, port, reason ("timeout", "empty", "killed", "crash")                                       | undedicated exited                       |
| **CreateRoomFailed**  | room_code, reason ("no_capacity", "spawn_failed")                                                     | Could not fulfill CreateRoom             |

#### 10.2.3 Hub → ProcMan Messages

| Message             | Fields                               | Trigger                                                                                                           |
| ------------------- | ------------------------------------ | ----------------------------------------------------------------------------------------------------------------- |
| **ProcManAccepted** | hub_version                          | Response to ProcManHello                                                                                          |
| **AuthRejected**    | reason                               | Bad UUID                                                                                                          |
| **CreateRoom**      | room_code, secret                    | Player requested a new room via API                                                                               |
| **RenameRoom**      | old_code, new_code, new_secret       | Response to RoomRenameRequest                                                                                     |
| **KillRoom**        | room_code, reason ("admin", "abuse") | Admin action or abuse detection                                                                                   |
| **Drain**           | —                                    | Stop accepting new rooms; let existing ones finish; then disconnect. For rolling updates of undedicated binaries. |

#### 10.2.4 Room Secret Flow

Each room has a secret generated by the Hub at creation time (and rotated on rename). The flow:

1. Hub generates a room code + secret, sends `CreateRoom { code, secret }` to ProcMan.
2. ProcMan passes the secret to the `undedicated` process.
3. When a player calls `GET /v1/rooms/join/:code`, the Hub returns both the address **and** the secret.
4. The player presents the room code + secret in the `unnet` Hello message to `undedicated`.
5. `undedicated` checks both match. This prevents players from bypassing the Hub entirely (e.g., port-scanning and
   connecting directly without a valid secret).

The secret is not meant to be cryptographically strong — it only needs to prove the player went through the Hub. A short
random string (e.g., 16 alphanumeric characters) is sufficient.

#### 10.2.5 Reconnection & Hub Restart

- ProcMan auto-reconnects on connection loss with exponential backoff (1s, 2s, 4s, ... capped at 60s).
- On reconnect, ProcMan re-sends `ProcManHello` with its full current state.
- The Hub rebuilds its room map from what ProcMans report. Rooms that existed before restart but are not reported by any
  ProcMan are considered gone — their codes are reclaimed.
- During Hub downtime, active game sessions on `undedicated` instances continue uninterrupted. Players already connected
  stay connected. Only new room creation and joining via code are unavailable.

#### 10.2.6 Heartbeat & Metadata Sync

Heartbeats flow over the persistent ProcMan→Hub TCP connection (not as separate HTTP calls from each `undedicated`).

- **Interval:** 60 seconds.
- **Expiry:** If the Hub receives no heartbeat (and no other message) from a ProcMan for 5 minutes, it considers that
  ProcMan dead and removes all its rooms from the active state.
- **Payload:** The heartbeat carries a summary of all rooms (code, player count, state, metadata) so the Hub's in-memory
  state stays current. This also serves as an implicit consistency check — if the Hub has a room that the ProcMan
  doesn't report, it's stale and gets cleaned up.

### 10.2.7 ProcMan ↔ undedicated Communication

ProcMan spawns each `undedicated` as a child process and communicates via **stdin/stdout** using the same JSONL format.
Command-line arguments provide initial configuration; runtime messages handle room assignment and lifecycle.

**Spawn arguments:**

```
undedicated --port 11001 --procman-channel stdin
```

**ProcMan → undedicated messages:**

| Message        | Fields               | Effect                                                                   |
| -------------- | -------------------- | ------------------------------------------------------------------------ |
| **AssignRoom** | room_code, secret    | Server transitions from idle to active with this room code and secret    |
| **RenameRoom** | new_code, new_secret | Server updates its room code and secret; existing players stay connected |
| **Shutdown**   | reason               | Graceful shutdown: disconnect players, then exit                         |

**undedicated → ProcMan messages:**

| Message               | Fields                       | Trigger                                                  |
| --------------------- | ---------------------------- | -------------------------------------------------------- |
| **Ready**             | port                         | Process started, listening, ready to accept rooms        |
| **PlayerJoined**      | player_uuid                  | Player connected and passed room code + secret check     |
| **PlayerLeft**        | player_uuid, remaining_count | Player disconnected                                      |
| **StateChanged**      | new_state, metadata          | Lobby→InGame→Lobby transitions, map/difficulty selection |
| **RoomRenameRequest** | reason                       | Host kicked a player, requesting new identity            |
| **Exiting**           | reason                       | About to exit (empty timeout, crash, shutdown command)   |

ProcMan forwards relevant events to the Hub (translating them into the Hub↔ProcMan protocol messages). This keeps
`undedicated` unaware of the Hub — it only talks to its parent ProcMan.

### 10.3 Game Version Compatibility (Resolved)

**Version matching:** Exact `major.minor` match. The version string comes from `Cargo.toml`
(`env!("CARGO_PKG_VERSION")`). Patch versions are ignored — `0.15.0` and `0.15.3` can play together; `0.15.x` and
`0.16.x` cannot.

**Multi-version hosting:** One Hub serves all game versions simultaneously. Each ProcMan can manage multiple
`undedicated` binary versions on the same machine and reports each version's capacity in its `ProcManHello`. When
creating a room, the Hub picks a ProcMan that has idle capacity for the requesting player's version. ProcMan's
configuration specifies which binary paths correspond to which versions, and how many idle servers to keep per version.

**Version mismatch UX:** When a player tries to join a room running a different `major.minor`, the Hub rejects the
request and returns the room's version so the client can display: _"This room requires version 0.16. You are on 0.15.
Please update."_ No auto-update — the player updates through their normal channel (itch.io, GitHub Releases, package
manager).

**Future: protocol-level compatibility.** The `unnet` protocol is designed to be backward-compatible. If a future
version wants to allow cross-version play, a separate `protocol_version` field can be introduced alongside
`game_version`. The Hub would match on `protocol_version` instead. This requires no Hub changes — only the matching
logic changes.

### 10.4 API Design (Resolved)

**Request/Response format:** JSON everywhere. The Hub handles tens of requests per second at most — JSON parsing is not
a bottleneck. Debuggability wins over marginal performance.

**Error responses:** All errors return a JSON object:

```json
{ "error": "room_full", "message": "Room H4KRP is full (4/4 players).", "details": { ... } }
```

| HTTP Status | Error Code         | Meaning                                                                             |
| ----------- | ------------------ | ----------------------------------------------------------------------------------- |
| 404         | `code_not_found`   | Room code does not exist or has expired                                             |
| 409         | `room_full`        | Room is at capacity                                                                 |
| 409         | `version_mismatch` | Player's `major.minor` doesn't match the room's. `details` includes `room_version`. |
| 403         | `banned`           | Player UUID is banned from Hub services                                             |
| 429         | `rate_limited`     | Too many requests. `details` includes `retry_after_seconds`.                        |
| 503         | `no_capacity`      | No ProcMan has idle capacity for the requested game version                         |

**API versioning:** `/v1/` prefix. Additive changes (new optional fields) do not require a version bump. If `/v1/` is
ever deprecated, old clients receive `410 Gone` with a body pointing to the new version. In practice, `/v2/` is not
expected for a long time.

**Endpoints (v1 MVP):**

- `POST /v1/rooms/create` — Create a room. Returns `{ code, secret, addr }`.
- `GET /v1/rooms/join/:code` — Join a room. Returns `{ addr, secret }`.
- `GET /health` — Returns `{ hub_version, uptime_seconds }`. No auth required.
- `GET /v1/browser/list` — _(Future — not part of v1.)_

Ban management is done via config file edits for v1. Admin API endpoints are future work.

### 10.5 Operational Resilience (Resolved)

**Hub restart recovery:** A Hub restart does **not** disrupt active game sessions — those live on `undedicated`
instances and players stay connected. The impact is that room creation and joining via code are unavailable during
downtime. Recovery is automatic:

1. ProcMan detects the TCP connection drop.
2. ProcMan reconnects with exponential backoff (1s, 2s, 4s, ... capped at 60s). The Hub address is in ProcMan's config;
   if it's a DNS name, ProcMan re-resolves on each attempt (handles IP changes from restarts/redeployments).
3. On reconnect, ProcMan sends `ProcManHello` with its full current state: all active rooms (codes, secrets, player
   lists, states, metadata) and idle servers.
4. The Hub rebuilds its room map from what ProcMans report. Room codes that existed before restart but are not reported
   by any ProcMan are considered gone — their codes are reclaimed.
5. Within seconds of a ProcMan reconnecting, all its rooms are joinable again via their original codes.

**Deployment strategy:** For v1, plain binaries managed by **systemd** unit files. One service each for `unhub` and
`unprocman`. Example `.service` files provided in the repo under `pkg/systemd/`. Docker is future work.

**Monitoring:** `GET /health` endpoint (defined in 10.4). Structured logging to stdout via `tracing` +
`tracing-subscriber` (JSON format in production, human-readable in dev). Further observability (metrics, dashboards) is
future work.

**Memory bounds:** ProcMan config has `max_rooms` (default: 20). Hub config has `max_procmans` (default: 10). These cap
the total possible rooms at `max_procmans × max_rooms` (200 by default). Hub rejects `rooms/create` with
`503 no_capacity` when the limit is hit.

**Hub as SPOF — future considerations:** The Hub is a single point of failure for room creation/joining (not for active
gameplay). Potential future mitigations:

- **Multiple Hub instances** behind DNS round-robin or a load balancer, sharing state via a lightweight replication
  protocol or a shared backing store (e.g., Redis). Room codes are designed to be globally unique (3.2M combinations,
  short-lived), which simplifies multi-Hub coordination.
- **Community-contributed ProcMan capacity:** ProcMan is the safer layer to federate — ProcMans are trusted with less
  data (no player UUIDs in their config, only room-level state). Community members could run ProcMans that connect to
  the official Hub, contributing server capacity without access to Hub-level data (ban lists, rate-limit state).
- **Community-operated Hub instances:** Riskier — a Hub operator can observe room creation patterns, player UUIDs, and
  connection metadata. If pursued, the trust model and data-access implications need explicit design. The Universe
  concept already supports this: a community Hub is simply a different Universe endpoint.

These are not part of v1 but the architecture does not block them.

### 10.6 Identity & Moderation Workflow (Resolved)

**Nickname system:** Players are identified by randomly generated codenames, not free-text input. This avoids the need
for a text input UI and eliminates profanity concerns by construction.

- The player chooses a **single letter** (A–Z).
- The system generates a two-word codename: **adjective + noun**, starting with that letter (e.g., choosing "K" might
  yield "Keen Kestrel").
- The word lists are curated and profanity-free. Duplicates across players are allowed.
- If the player doesn't like their codename, they can re-roll to get another random one from the same letter.
- Generation can happen locally (deterministic from UUID + letter + attempt number, using the same word lists) or
  server-side. Exact mechanism TBD — the important decision is the codename approach, not where it runs.
- The codename is sent alongside the UUID in the `unnet` Hello message. The Hub passes it through without storing it.
- Full design of the word lists, UI flow, and persistence is deferred to a dedicated discussion.

**Report/Ban workflow:** For v1, admin-only. The admin edits the Hub's RON config file to add a UUID to `banned_uuids`,
then sends `SIGHUP` (or restarts) to reload. In-game reporting UI, ban appeals, and temporary bans are all future work.

### 10.7 Game-Side Integration (Resolved)

**Bevy state machine:** Hub interaction happens within the existing `AppState::MainMenu`. No new `AppState` variant
needed. The Hub client (`unhub-*` crate) runs as a background async task (spawned via `IoTaskPool`) that starts on game
boot and exposes its state via a Bevy `Resource` (e.g.,
`HubConnection { status: Connecting | Connected | Unavailable, ... }`). The "Play Online" menu reads this resource to
decide what to show. Room creation/joining are async requests — the UI shows a spinner while waiting for the Hub
response, then transitions to the lobby.

**Hub unavailable UX:** The "Play Online" button is always visible. If `HubConnection.status` is `Unavailable`, clicking
"Create Room" or "Join Room" shows an inline error: _"Hub is unreachable. Check your internet connection or try again
later."_ Direct-IP connect remains available as a separate option regardless of Hub status. No modal popups — just a
status indicator in the multiplayer menu (e.g., a small "Hub: Connected" / "Hub: Offline" label).

### 10.8 Player-Hosted Rooms via Hub (Future Idea)

1. **Concept:** A player running `undedicated` (or the game itself in host mode) on their own machine could optionally
   register with an `unprocman` or directly with `unhub`, getting a room code so friends can join via code instead of
   IP:Port. NAT/port forwarding would be the player's responsibility. Future consideration, not part of v1.

### 10.9 Data Privacy & Legal

1. **Privacy Policy Required:** A `PRIVACY.md` in the repository and a link in the game UI are needed. Layered notice:
   - **In-Game (Layer 1):** Small text in the multiplayer menu.
   - **Full Policy (Layer 2):** Hosted at `PRIVACY.md` or on the website.
   - **WASM:** Footer link on the page hosting the game.
2. **Data Minimization:** No IP↔UUID association stored. IPs for rate limiting kept transiently in memory only. Ban
   records store UUID only. Server metrics are aggregate-only.
3. **UUID as Personal Data:** Acknowledged as pseudonymous identifier under GDPR. Lawful basis: "Legitimate Interest."
   Players can reset their UUID (losing progress) to exercise "right to erasure."
4. **In-Game Legal Notice:** A "Privacy Info" link in Settings/About that opens the policy URL is the minimum.
