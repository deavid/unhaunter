# Multiplayer Investigation - Round 1

This document outlines the initial findings on adding networking multiplayer to Unhaunter.

## Current State Analysis

### Architecture & Modularization

The codebase is highly modularized, which is beneficial for adding a dedicated networking plugin.

- **Crate Structure**: Separation between `*-core` and `*-plugin` crates simplifies where to add networking logic
  without introducing circular dependencies.
- **Event System**: The game already uses a custom event system (`unevents-core`). Many gameplay-critical events (Ghost
  interactions, level loading, room changes) are already encapsulated in messages.

### Player Management

- **Single-Player Centric**: Current spawning logic in `unclassic-mode-plugin` (specifically
  `classic_mode_orchestrator`) assumes a single local player. It inserts the `MainPlayer` marker component into the
  spawned entity.
- **Player Input**: The `PlayerInput` resource is a global that aggregates input for "the" player. This needs to move to
  per-entity component storage for multiplayer.
- **Multiple IDs**: `PlayerSprite` already has an `id` field, and systems like `player_movement_system` iterate over all
  entities with `PlayerSprite`, suggesting some groundwork for multiple characters might exist.

### Game State & Determinism

- **Global Resources**: Much of the game logic depends on global resources like `HauntState`, `SoundGrid`, and
  `MiasmaGrid`. These will need to be either:
  - Replicated from a server.
  - Computed deterministically on all clients from a shared seed.
- **Randomness**: The `unfoundation-core` provides `heavy_rng()` which uses system time. This is non-deterministic
  across clients.

### Tiled Integration

- **Map Loading**: Maps are loaded via `unmapload-plugin`. Synchronization of the initial map state (which door is open,
  where items are) will be required upon client connection.

## Technical Considerations

### Platform: WASM

Since the game targets WASM, networking is restricted to:

- **WebSockets**: Standard for Client-Server.
- **WebRTC**: Preferred for Peer-to-Peer due to lower latency (UDP-like).
- **Libraries**: `bevy_matchbox` (P2P/WebRTC) or `bevy_renet` (CS via WebSockets/UDP) are the primary candidates.

### Synchronization Requirements

- **High-Frequency**: Player positions, rotations, and animations.
- **Low-Frequency/Reliable**: Interaction events (toggling lights, opening doors), mission state changes, inventory
  changes.
- **Environment**: Sound levels, temperature readings, and ghost clues (e.g., EMF readings).

## Key Challenges

1. **Authority Model**: Deciding which client (or a dedicated server) owns the Ghost AI, the Map State, and the Miasma
   growth.
2. **Component Replication**: Identifying which of the many components (e.g., `Position`, `Direction`, `Stamina`,
   `Health`) should be synced and at what frequency.
3. **Ghost AI**: The ghost currently queries player positions and updates its own state. This logic needs to stay
   server-side to prevent "ghost teleporting" or inconsistent behavior between clients.

## Questions for the User

1. **Network Model**: Do you prefer a **Peer-to-Peer** (P2P) model (easier for small groups, no server cost) or a
   **Client-Server** model (more authoritative, prevents cheating, better for matchmaking)?
2. **WASM Priority**: Is WASM-to-WASM multiplayer the primary goal, or is Native-to-Native also a priority? This affects
   the choice of transport layer (WebRTC vs Standard UDP).
3. **Determinism**: Are you open to making the simulation more deterministic to use **Rollback** (GGRS-style) or do you
   prefer a **Snapshot Interpolation** approach?
4. **Player Count**: What is the target maximum player count for a single session?

## Next Steps for Investigation

- Analyze `ungear-plugin` to see how item interactions (grabbing/dropping) are handled and how to replicate them.
- Investigate `unboard-core` to see if collision detection can be moved to a shared library for server-side validation.
- Explore how to wrap `unevents-core` messages into network packets.
