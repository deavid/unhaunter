# Multiplayer Voice & Shared Simulation - Round 3

This document explores the long-term goal of in-game voice chat and the technical requirements for a shared authority
simulation in Unhaunter.

## Updated User Requirements

- **Host Migration**: Required for the long term. The design must be extensible to support moving authority from one
  peer to another mid-session.
- **Authority Model**: **Shared Simulation**. One player is the designated "server" (Authority), and other peers correct
  their local state towards this authority.
- **Voice Chat**: **Priority!** (Long term). Must support:
  - **Proximity Chat**: Audio volume/spatialization depends on distance between players.
  - **Radio Chat**: Global (or group-based) communication with potential audio filters (walkie-talkie effect).
- **Latency Buffering**: Target < 100ms for smooth experience; < 30ms is optimal.

## Shared Simulation & Correction Logic

### 1. Snapshot-Based Correction

Since we are using **Snapshot Interpolation**, the authority ("server") will periodically broadcast the state of all
replicated entities.

- **Local Simulation**: Clients will simulate their own character locally ("Client-Side Prediction") to ensure
  zero-latency movement.
- **Reconciliation**: When a snapshot arrives from the server for a past frame, the client compares their predicted
  position with the server's authoritative position.
- **Correction**: If the difference exceeds a threshold, the client "snaps" or smoothly blends towards the server's
  position.

### 2. Identifying Replicated Updates

Systems that currently modify state based on global resources need to be updated:

- **Movement**: [crates/unplayer-plugin/src/systems/movement.rs](crates/unplayer-plugin/src/systems/movement.rs)
  currently reads from a global `PlayerInput`.
  - **Action**: Move `PlayerInput` from `Resource` to a `Component` on each player entity.
- **Ghost AI**: [crates/unghost_plugin/src/ghost.rs](crates/unghost_plugin/src/ghost.rs) should only run on the
  Authority player. Other players should just receive position/animation updates for the ghost.

## Voice Chat Architectural Research

### 1. Audio Capture & Transport

- **Capture**: Requires integration with audio input APIs (likely via `cpal` or a dedicated Bevy plugin for mic input).
- **Compression**: Raw audio is too large. **Opus** is the industry standard for low-latency voice.
- **Transport**: WebRTC (via `bevy_matchbox` or similar) is ideal as it handles unreliable/out-of-order packets
  (UDP-like) which is crucial for real-time voice.

### 2. Proximity Implementation

- **Spatial Positioning**: Voices should be spawned as `AudioPlayer` entities at the position of the remote player's
  entity.
- **Attenuation**: Use Bevy's spatial audio features (`SpatialScale`, `PlaybackSettings::spatial`) to handle volume
  drop-off.
- **Radio Override**: When a player uses the "Radio" (Walkie-Talkie), the spatial attenuation is ignored, and a filter
  (low-pass + distortion) can be applied to simulate radio static.

## Design for Host Migration

To avoid locking out host migration, we need:

1. **Serialized State**: A way to package the _entire_ authoritative state (Ghost AI variables, map toggle states,
    etc.) into a message.
2. **Election Protocol**: A way for clients to agree on who the new host is if the current one disconnects.
3. **State Handoff**: The new host initializes their local AI logic using the last known snapshot from the previous
    host.

## Refactoring Targets

| Feature            | Current State                    | Target State                                |
| :----------------- | :------------------------------- | :------------------------------------------ |
| **Player Input**   | Global `Resource`                | Per-entity `Component`                      |
| **Randomness**     | Local `Instant::now()`           | Shared `Seed` from Authority                |
| **Ghost AI**       | Runs on all clients              | Runs on Authority only; syncs via snapshots |
| **Sound Playback** | Global `SoundEvent`              | Network-aware `SoundEvent`                  |
| **Evidence**       | Global `CurrentEvidenceReadings` | Potentially per-player or synced if shared  |

## Questions for the User

1. **Snapshots vs Events**: Would you prefer the Ghost AI state to be sent as a full snapshot every tick, or just send
   "State Change" events (e.g., "Ghost started hunting at X")? Snapshots are more robust but use more bandwidth.
2. **Deterministic Seed**: Should we move towards a more deterministic random system now to make the "corrections"
   smaller? (e.g., if we both know the ghost will turn left at the next junction, the correction is minimal).
3. **Radio Mechanic**: Does the radio have "channels" or is it a simple "Talk to everyone" button?
4. **Hiding & Stealth**: If a player hides
   ([crates/unplayer-plugin/src/systems/hide.rs](crates/unplayer-plugin/src/systems/hide.rs)), should this state be
   purely local for visual effects, or must it be verified by the host for ghost detection?
