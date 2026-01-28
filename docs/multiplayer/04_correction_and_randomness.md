# Multiplayer Correction & Randomness - Round 4

This document synthesizes the latest user feedback on authority, randomness, and the radio mechanic.

## User Feedback & Critical Decisions

- **Snapshot Strategy**: Confirmed. Snapshots will be used for ghost AI and world state synchronization as they are more
  robust than event-based updates for correction.
- **Randomness Approach**:
  - **Rejection of Common Seeds**: Shared seeds are insufficient due to platform differences, non-deterministic
    multithreading, and FPS-dependent sampling.
  - **No Active Refactor**: Trying to make the current randomness deterministic is deferred as "futile."
  - **Strategy**: Random gameplay elements will be simulated on the **Authority (Host)** and the results will be
    replicated to clients via snapshots. The "drift" caused by local RNG on clients will be corrected by incoming
    authoritative snapshots.
- **Radio & EMI**:
  - **Feature**: Radio is "Talk to Everyone" (Global) but is susceptible to **Electromagnetic Interference (EMI)** from
    ghosts.
  - **Research Goal**: How to detect ghost proximity to the speaker/listener to apply audio distortion (static) to the
    voice channel.
- **Hiding Mechanic**: Confirmed. Hiding state must be verified on the server-side as the ghost's behavior (visibility
  checks) depends on it.

## Technical Deep Dive: Randomness Drift

### 1. Identifying RNG Dependencies

The following systems use `random_seed::rng()` and will experience drift:

- **Ghost Movement**: [crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs) uses RNG for selecting
  warder points and warping triggers.
- **EMI Effects**:
  [crates/ungearitems-plugin/src/components/recorder.rs](crates/ungearitems-plugin/src/components/recorder.rs) uses RNG
  for display glitches and sound chirps.
- **Sound Propagation**: [crates/unsound-plugin/src/systems.rs](crates/unsound-plugin/src/systems.rs) uses RNG for
  "ghost talk" frequency and sound diffusion vectors.

### 2. Implementation of Snapshot Correction

To resolve drift without making RNG deterministic:

- **Host**: Runs the simulation as-is.
- **Client**: Runs a parallel simulation, but when a snapshot arrives, it overwrites the local state (e.g.,
  `ghost.target_point`, `ghost.hunting`).
- **Smoothing**: We will use **Hermite Interpolation** or similar for positions to prevent visually jarring "teleports"
  when RNG paths diverge significantly before the next snapshot.

## Voice Chat & EMI Integration

### 1. Proximity EMI

The radio mechanic needs to check distance to the ghost.

- **Logic**: If `VoicePacket.is_radio == true`, calculate `distance(Ghost.Position, Speaker.Position)` and
  `distance(Ghost.Position, Listener.Position)`.
- **Effect**: If either is within the ghost's influence radius, apply a static/crackle filter to the Opus stream before
  playback.

## Server-Side Verification of Hiding

Current logic in [crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs):

```rust
let player_pos_l: Vec<(&Position, bool)> = qp
    .iter()
    .filter(|(_, p, _)| p.health > 0.0)
    .map(|(pos, _, hiding)| (pos, hiding.is_some()))
    .collect();
// ... ghost targets based on 'hiding' bool
```

- **Requirement**: The `Hiding` component must be replicated from the Client to the Host. The Host then uses this
  replicated data to run the Ghost AI.

## Remaining Challenges & Reminders

- **Problem 1 (RNG)**: Note for later: Develop a way to "re-seed" or "align" local RNG generators if drift causes too
  much correction overhead (low priority).
- **Problem 2 (Audio)**: Capture API for WASM. Web browsers require a user interaction (like a click) before audio
  capture can start.
- **Problem 3 (Entity Scale)**: Snapshotting 16 players + ghost + environmental EMI sources might require "Delta
  Snapshots" (only send what changed) to stay within bandwidth limits.

## Questions for the User

1. **EMI Severity**: Should the Radio be _completely_ unusable during a Ghost Hunt, or just very distorted?
2. **Authority Selection**: Should the room creator always be the Host, or should we pick the player with the lowest
   ping/best hardware?
3. **Ghost "Teleporting"**: If the Host's RNG makes the ghost Warp and the Client's RNG doesn't, the snapshot will force
   a teleport. Are you okay with this being smoothed over, or should the "Warps" be specific events?
4. **Bandwidth**: Are we targeting players on mobile/data with strict limits, or is "Home Fiber" the assumption?
   (Affects Voice compression settings).
