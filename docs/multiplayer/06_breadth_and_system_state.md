# Multiplayer Breadth & System State - Round 6

This document expands the research to include environmental simulations, player progression, and the complexities of
multi-viewer rendering.

## User Feedback & Decisions

- **Late Join**: Low priority but should be architecturally possible.
- **Item Persistence**: On disconnect, items drop to the ground. Multiple items spawn a "backpack" container.
- **Journal/Evidence**: Fully shared and synchronized. One journal per session.
- **Campaign Unlocks**: Individual. Each player earns XP/Money and unlocks content for their own local profile.
- **Inter-player Collision**: Disabled. Players pass through each other to avoid griefing and navigation bottlenecks.

## Environmental & System State Synchronization

### 1. Miasma (Fog) Simulation

The Miasma system ([crates/unfog-plugin/src/systems.rs](crates/unfog-plugin/src/systems.rs)) uses a grid-based fluid
simulation for pressure and velocity.

- **Problem**: Running this simulation independently on all clients will lead to divergence due to RNG and float
  precision.
- **Breadth Solution**:
  - The **Host** runs the authoritative simulation.
  - Clients receive periodic low-resolution updates of the `pressure_field`.
  - Clients use local interpolation to fill in the gaps for visual rendering.
  - Sanity/Health effects are calculated by the Host based on the authoritative grid.

### 2. Main Power & Breakers

The Ghost can trip the main breaker
([crates/unghost-plugin/src/systems/gis/execution.rs](crates/unghost-plugin/src/systems/gis/execution.rs)).

- **Sync Requirement**: The `Toggleable` state of the breaker entity must be strictly synchronized. If the breaker is
  "Off," the Host must ensure `unlight-plugin` reflects this for all clients.

### 3. Shared Truck Resources

- **Repellent Crafting**: The `RepellentCraftTracker` in
  [crates/untruck-plugin/src/systems/truck_ui_systems.rs](crates/untruck-plugin/src/systems/truck_ui_systems.rs) is
  currently a local resource.
- **Sync Requirement**: This should move to a shared state. If the difficulty limit is 3 bottles, and Player A crafts 2,
  Player B should only see 1 remaining.

## Rendering & Visibility Challenges

### 1. Multi-Viewer Visibility

Currently, `VisibilityData` is calculated relative to a single `player_id` in
[crates/unlight-plugin/src/maplight.rs](crates/unlight-plugin/src/maplight.rs).

- **Challenge**:
  - **Local Client**: Needs to calculate visibility relative to the local player for rendering (fog of war).
  - **Host**: Needs a "Total Visibility" map representing everything seen by _any_ player, to determine if the Ghost is
    "sighted" and to update `HauntState`.
- **Breadth Solution**: The Host maintains a bitmask or composite grid of all `Viewer` positions.

### 2. Proximity-Based Optimization

`sync_map_entity_field`
([crates/unrender-plugin/src/systems/board_sync.rs](crates/unrender-plugin/src/systems/board_sync.rs)) optimizes entity
updates by only looking at a radius around the player.

- **Multiplayer Fix**: The system must be updated to loop through all active `PlayerSprite` positions to ensure entities
  near _any_ player are correctly synchronized and rendered.

## Disconnect & Cleanup Flow

### 1. The "Backpack" System

When a player disconnects:

- Iterate through their `PlayerGear` (left hand, right hand, inventory).
- If one item: Spawn it as a `FloorItemCollidable` at the last known `Position`.
- If multiple items: Spawn a `Backpack` entity (a new container type) containing all items.

## Individual Rewards Calculation

- **Consistency**: At mission end, the Host calculates the `SummaryData`.
- **Distribution**: The Host broadcasts a `MissionCompleteEvent(Score, Grade)` to all clients.
- **Local Application**: Each client's `unprofile-plugin` receives this event and updates their local
  `PlayerProfileData.progression`. This ensures everyone gets the same reward without needing to share private profile
  data (like current total money).

## Questions for the User

1. **Viewer Proximity**: Should players "share" vision? (e.g., If Player A sees a Ghost Orb, does Player B also see it
   on their screen even if they are in another room?)
2. **Backpack Interaction**: Should a dropped backpack be "unpackable" (spilling items on the floor) or act as a mobile
   storage unit?
3. **NPC Dialogs**: When a player talks to an NPC
   ([crates/unnpc-plugin/src/npchelp.rs](crates/unnpc-plugin/src/npchelp.rs)), should the game pause for everyone, or
   just the person in the menu? (Currently, it uses a global `GameState::NpcHelp`).
4. **Spectating**: If a player dies, should they be able to spectate others through their cameras?
5. **EMI Source Sync**: Should non-ghost EMI sources (e.g., faulty equipment or environmental hazards) be synchronized
   identically for everyone?
