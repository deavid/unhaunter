# Multiplayer Breadth & Interaction Refinement - Round 7

This document continues exploring the broad architectural implications of multiplayer, focusing on shared state, the
death/spectate loop, and session transitions.

## User Feedback & Decisions

- **Vision Sharing**: Individual. Players only see ghost orbs, breaches, or ghosts if they are directly looking at them
  (local visibility simulation).
- **Backpack Logic**:
  - On disconnect, items are stored in a **Persistent Backpack** entity.
  - Backpack disappears when empty.
  - Interaction (`[E]`) automatically transfers as many items as possible to the player's inventory.
- **NPC Dialogs**: Local. Only the interacting player is "stalled" in the dialog menu.
- **Death/Spectating**: Dead players move to a spectate mode.
- **Visual Sync**: Rough equivalence is prioritized over perfect matches (e.g., flickering lights can be out of sync
  across clients).
- **Connectivity**: 256kbps upload budget per client (Home Fiber assumption).

## Broad Architecture: Shared Session State

### 1. The Collaborative Journal (`GhostGuess`)

Currently, `GhostGuess` (found in
[crates/unghost-core/src/resources/ghost_guess.rs](crates/unghost-core/src/resources/ghost_guess.rs)) is a local
resource.

- **Sync Requirement**: Every discovered evidence piece marked by any player must be reflected in the truck's terminal
  for everyone.
- **Broadcast**: When a player clicks the UI to mark an evidence, a message is sent to the Host, who updates the
  session-wide `GhostGuess` and broadcasts the update.

### 2. Thermal & Evidence Grids

The `ThermalGrid` ([crates/unthermal-core/src/resources.rs](crates/unthermal-core/src/resources.rs)) and `SoundGrid`
([crates/unsound-core/src/resources.rs](crates/unsound-core/src/resources.rs)) are 3D fields.

- **Strategy**:
  - **Host**: Authoritative simulation of heat diffusion and sound propagation.
  - **Client**: Periodically receives the Host's grid state.
  - **UI/Gear**: Thermometers and EMF meters read from the _synced_ grid, ensuring Player A and Player B see the same
    temperature in the same spot.

### 3. Spectator Loop & Camera Focus

When a player dies:

- **State Change**: Their entity is hidden (or replaced with a corpse), and their `Viewer` component is removed or
  disabled.
- **Spectate Mechanism**: The local camera logic needs to switch to a "Free Cam" or "Player-locked Cam".
- **Implementation**: We can repurpose the `Viewer` logic to focus on a remote player's position, allowing the dead
  player to still see the investigation through their teammates' eyes.

### 4. Audio: Global vs. Local Events

`SoundEmitter` ([crates/unsound-core/src/emitter.rs](crates/unsound-core/src/emitter.rs)) triggers sounds.

- **Networked Sounds**: Dropped items, ghost roars, doors opening. These must be broadcast as a `NetworkedSoundEvent`.
- **Local Sounds**: UI clicks, menu music, personal walkie-talkie static (intro/outro zig-zags). These should remain
  local-only to save bandwidth.

## Session Transitions (The "Breadth" of Joining)

### 1. Main Menu to Lobby

- **New State**: `AppState::Lobby`.
- **Functionality**:
  - **Host**: Generates a Signaling ID (Room Code).
  - **Peers**: Input Room Code to join.
  - **Sync**: Once connected, peers appear in a "Ready Room" list.

### 2. Starting the Mission

- **Flow**:
  1. Host selects Mission + Difficulty.
  2. Host presses "Start".
  3. All clients transition to `AppState::Loading`.
  4. Clients load the same map + seed for static objects (deterministic generation).
  5. Clients signal `MapReady` once assets are prepared.
  6. Match begins when all `MapReady` signals are received.

## Conflict Resolution: The "Grab" Logic (Refined)

The User clarified the Backpack logic:

- **Interaction**: Pressing `[E]` on a backpack should "batch" load items.
- **Implementation**:
  1. Client sends `InteractBackpack(EntityId)` to Host.
  2. Host iterates through `Backpack.contents` and `PlayerGear.inventory`.
  3. Host moves as many item entities as fit.
  4. Host deletes the backpack entity if `contents.is_empty()`.
  5. Host broadcasts the inventory changes to all clients.

## Remaining Questions & Topics

1. **Host-only UI**: Are there any UI elements (like a "Pause Match") that only the Host should control?
2. **Ping Visibility**: Do we need a "Ping" or "Signal" system so players can point at items without using voice chat?
3. **Ghost Sanity Drain**: Sanity loss is currently handled in
   [crates/unplayer-plugin/src/systems/sanityhealth.rs](crates/unplayer-plugin/src/systems/sanityhealth.rs). Does the
   Host handle all sanity drains and replicate them, or do clients calculate their own based on distance to the ghost?
   (The user mentioned the ghost reacts to hiding, so host-side calculation for sanity makes sense for consistency).
4. **Tutorial Triggers**: If a player triggers a tutorial walkie message (e.g. "You're struggling to hide"), should it
   only play for that player or the whole team?
5. **Item States (Durability/Battery)**: For items with batteries or specific charges, these must be synced. Should the
   Host be the authority for battery drain?
