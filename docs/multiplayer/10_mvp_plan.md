# Multiplayer MVP Plan - Round 10

This document outlines the current state of refactors and the proposed plan for a Minimum Viable Product (MVP) of
Unhaunter's multiplayer.

## Current Readiness Assessment

Several "Early Refactor" targets identified in [09_early_refactors.md](../multiplayer/09_early_refactors.md) have
already been partially or fully implemented.

### ✅ Completed Refactors

- **Component-Based Input**: `PlayerInput` is now a `Component` on player entities instead of a global `Resource`.
  Systems in `unplayer-plugin` correctly read from/write to this component.
- **Multi-Player Support (Iteration)**: Most systems targeting the player (movement, animation, sanity) have been
  refactored from `query.single()` to `query.iter()`, allowing the game to handle multiple player entities without
  crashing.
- **MainPlayer Tag**: The `MainPlayer` component is used to distinguish the local player from others, providing a
  foundation for remote player rendering.

### 🚧 Pending Refactors (Necessary for MVP)

- **Networking Crates**: No crates currently exist for transport (e.g., `unmatchbox-plugin` or `unnet-core`).
- **Entity ID Mapping**: We need a stable `NetworkId` to map entities between peers as Bevy's `Entity` IDs are not
  stable across different runs.
- **Snapshot Selection**: Deciding which components (Position, Animation, Hiding) are part of the "core snapshot".
- **Spawn Logic**: `classic_mode_orchestrator` still hardcodes the spawning of exactly one player.

### 🛑 Deferred Refactors (Post-MVP)

- **HauntState as Component**: Remains a global resource for now.
- **Deterministic RNG**: Still using global seeds; snapshots will force-correct drifts.
- **Multi-Viewer Visibility**: `VisibilityData` is still largely single-viewer-centric.

---

## The MVP Goal

**"The CLI Duo"**: A 2-player session where two Linux/Native instances can connect via CLI flags, walk together, and see
the same ghost behavior.

### 1. Transport Layer: Simple TCP

- **Selection**: A simple TCP-based transport for Native-to-Native communication (MVP stage).
- **Serialization**: JSONL (JSON lines) using `serde`. All network-enabled data must be `Serialize` + `Deserialize`.
- **Encapsulation**: Protocol details and transport logic must be strictly confined to `unnet-plugin` and `unnet-core`.
  No other crates should know about the wire format or socket implementation.
- **Scope**: Reliable interactions (events) and high-frequency snapshots (positions).

NOTE: Initial research suggested `bevy_matchbox`, but it is currently pinned to Bevy 0.17. We will revisit WebRTC/WASM
support once the MVP is stable on Native.

### 2. CLI-Driven Connection

To avoid building complex UI menus immediately, the MVP will use flags:

- `--host`: Starts the signaling and waits for a peer.
- `--join <ROOM_ID>`: Connects to an existing host.

### 3. Core Synchronization Targets

For the MVP, we will only sync the "Bare Minimums":

- **Synchronized Entities**:
  - **Players**: Position, Direction, Animation State, Hiding state.
  - **Ghost**: Position, Current Alpha (Visibility), and Hunting state.
- **Synchronized Interactions**:
  - **Toggleables**: Light switches and door states (synced via state change events).
  - **Grabs**: Request-style interaction for items in the truck.

### 4. Authority Model: Host-Authoritative

- **Host**: Runs RNG, Ghost AI, Miasma growth, and Thermal diffusion.
- **Guest**: Sends local input to the Host; predicts local movement; corrects position based on Host snapshots.

---

## Immediate Implementation Path (The MVP Roadmap)

### Step 1: Networking Core (`unnet-core`)

- Define `NetworkId` component.
- Define `NetworkMessage` enum (Snapshots, Events, Input).

### Step 2: Transport Plugin (`unnet-plugin`)

- Implement non-blocking TCP sockets using `std::net` in a Bevy-friendly manner.
- Implement basic "Lobby" logic that exchanges `NetworkId` for player entities.

See [11_detailed_mvp_implementation_plan.md](./11_detailed_mvp_implementation_plan.md) for full implementation details.

### Step 3: Update CliOptions

- Extend `untypes-core/src/cli.rs` to include `MultiplayerMode { Solo, Host, Join(String) }`.

### Step 4: Spawning Refactor

- Update `classic_mode_orchestrator` to spawn a `PlayerSprite` without `MainPlayer` for every connected peer.

---

## MVP Configuration Decisions

1. **Snapshots**: 60 ticks per second.
2. **Entity Buffer**: Mid-game spawns (e.g., items dropped from a backpack) are broadcast as events and integrated into
   the simulation on the following tick.
3. **Ghost Teleport**: Visual corrections (interpolation) will be implemented using the most idiomatic patterns in the
   codebase, with specific smoothing thresholds determined through playtesting.
