# Multiplayer Architectural Deep Dive - Round 2

This document records user feedback and explores deeper architectural changes required for Unhaunter's multiplayer.

## User Feedback & Decisions

- **Network Model**: Hybrid exploration of Peer-to-Peer (P2P) and Client-Server.
  - **P2P Interest**: Utilizing a "dumb middleman" (signaling server) for session coordination where the "truth" is
    shared/agreed upon by players. This minimizes server-side costs.
- **Cross-Play**: Full cross-play support between **WASM** and **Native** is required.
- **Simulation Style**: **Snapshot Interpolation** is the chosen approach (no rollback/determinism strictly required).
- **Player Scale**:
  - Recommended: 4 players.
  - Absolute Maximum: 16 players.
- **Technology Stack**:
  - `bevy_matchbox` for WebRTC-based networking.
  - NAT hole punching using browser standards.

## Architectural Analysis

### 1. From Global Input to Per-Player Input

The current input system (e.g.,
[crates/unplayer-plugin/src/systems/input/keyboard.rs](crates/unplayer-plugin/src/systems/input/keyboard.rs)) writes to
a global `PlayerInput` resource.

- **Change Required**: Movement input must be localized to the player entity.
- **New Component**: `NetworkInput` or similar, carrying the movement vector and interaction requests.
- **Remote Players**: For remote players, this component will be populated by incoming network packets instead of local
  keyboard/mouse events.

### 2. Authority & "The Truth"

Even in P2P, one client usually acts as the **Host** (Authority) to ensure consistent world state.

- **Host Authority**:
  - **Ghost AI**: Ghost movement ([crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs)) and state
    logic.
  - **Map State**: Which lights are on, door states, placement of items.
  - **Miasma/Fog**: Growth and distribution.
- **Sync Strategy**:
  - Host sends snapshots of `Position`, `Direction`, `Toggleable`, and `HauntState`.
  - Clients interpolate positions of remote players and the ghost.

### 3. Component Replication List

Based on current data types, the following need snapshot replication:

- **Core Entities**:
  - `PlayerSprite`: `health`, `crazyness`, `mean_sound`.
  - `Position` & `Direction`: For all movable entities (Players, Ghost, thrown items).
  - `Stamina`: To sync breathing sounds/stuttering.
- **Environment**:
  - `Toggleable`: To sync lights, gear, and switches.
  - `RoomState`: Syncing room-specific conditions.
- **Resources**:
  - `BoardTopology`: Primarily used for collision; needs to be consistent, but maps are static once loaded.
  - `HauntState`: Syncing the `ghost_warning_intensity` and `ghost_warning_position`.

### 4. Cross-Play (WASM/Native)

Since WASM cannot use standard UDP/TCP sockets easily, `bevy_matchbox` with WebRTC is the correct path.

- **Native**: Will also use WebRTC to communicate with WASM clients.
- **Signaling**: A signaling server (e.g., Matchbox) will handle initial handshakes.

### 5. Determinism & Randomness

- **Issue**: `unfoundation-core/src/random_seed.rs` uses `Instant::now()` which is non-deterministic.
- **Solution**: The Host should generate a `Seed` resource and replicate it to all clients at mission start. Systems
  like ghost behavior can then be partially predictable if they use this shared seed, reducing the need for
  high-frequency position updates for every micro-movement.

## Key Challenges Identified

- **Physics & Collisions**: `CollisionHandler` in `unnavigation-core` is currently local. If a client lags, they might
  "walk through walls" on their screen but be blocked on the host. Snapshot interpolation will need to handle
  corrections smoothly.
- **Entity ID Mapping**: Bevy `Entity` IDs are not stable across different runs or clients. A stable `NetworkId`
  component will be needed to map network entities to local ECS entities.
- **Bandwidth at 16 Players**: Snapshotting 16 players + the ghost + environmental objects might saturate WebRTC
  channels if not optimized (e.g., using interest management or delta compression).

## Questions for the User

1. **Host Migration**: If the Host (who started the session) leaves, should the session end, or should we implement Host
   Migration? (The latter is significantly more complex).
2. **Authority Model**: Are you okay with the "Session Creator" being the authority for the Ghost AI, or should we
   explore a "Shared Simulation" where everyone runs the same logic based on a shared seed (which might drift)?
3. **Voice Chat**: Is in-game proximity voice chat a feature you'd like to eventually support? (This heavily influences
   the transport layer and bandwidth needs).
4. **Latency Buffering**: Since we are using snapshot interpolation, how much "interp delay" (e.g., 100ms) are you
   comfortable with for other players' movement?

## Next Steps for Investigation

- Analyze how to replace the global `PlayerInput` resource with a component-based approach without breaking
  `unplayer-plugin`.
- Research how to bridge `unevents-core` (which are Bevy events) to network messages.
- Investigate audio backend compatibility for real-time voice capture in WASM/Native.
