# Multiplayer Breadth & Interaction Conflicts - Round 5

This document shifts focus from deep technical simulations (RNG/Voice) to broader game systems: Lobby management, shared
inventory, and level synchronization.

## Broad Scope Requirements

- **Lobby & Discovery**: A system to allow players to join a session (Shared secret/Room ID).
- **Consensus-Based Start**: All players must signal "Ready" before the level transition occurs.
- **Authority-Gated Grabbing**: Resolving conflicts when multiple players attempt to grab the same single-instance item
  (like the EMF meter).
- **Shared Mission Rewards**: Ensuring all participants receive consistently calculated XP and Money at the end of a
  mission.

## Architectural Breadth: Identified Areas

### 1. From Menu to Gameplay (The Lobby)

The current menu flow in [crates/unmainmenu-plugin/src/mainmenu.rs](crates/unmainmenu-plugin/src/mainmenu.rs) goes
directly to `MissionSelect`.

- **Change**: A new "Multiplayer" intermediate state is needed.
- **Discovery**: Since we are using WebRTC (browser-friendly), a signaling server will provide a "Room ID". One player
  generates the ID, others enter it.
- **Lobby State**: A synchronized list of players, their "Ready" status, and potentially their chosen equipment colors.

### 2. Level Loading & Synchronization

The `load_level_handler` in [crates/unmapload-plugin/src/level_setup.rs](crates/unmapload-plugin/src/level_setup.rs) is
currently fire-and-forget for a single player.

- **Network Flow**:
  1. Host selects map.
  2. Host broadcasts `LoadLevel(MapPath)` to all peers.
  3. Each client loads assets locally.
  4. Clients send `LevelLoaded` back to Host once their `LevelLoadingStatus` reaches completion.
  5. Host waits for all `LevelLoaded` signals before broadcasting `StartMatch`.

### 3. Inventory Conflict Resolution (The Grab Logic)

The `grab_object` system in
[crates/unplayer-plugin/src/systems/grabdrop.rs](crates/unplayer-plugin/src/systems/grabdrop.rs) uses a simple distance
check and immediately modifies `PlayerGear`.

- **The Problem**: Latency can lead to "Double Grabs" where two players think they hold the same item.
- **The Solution (Authority)**:
  - **Client**: Sends `RequestGrab(EntityId)` to Host.
  - **Host**: Verifies distance and if the entity is already attached to another `PlayerGear`.
  - **Host**: If successful, removes `FloorItemCollidable` and broadcasts `GrabSuccess(PlayerId, EntityId)`.
  - **Client**: Updates local UI/Sprite only after receiving `GrabSuccess`.

### 4. Partitioning Ghost Logic

Ghost interactions are currently driven by the `ghost_interaction_selection_system`.

- **Host-Only**: Only the Host runs the _selection_ logic (RNG-heavy, deciding _what_ to do).
- **Shared Execution**: The _execution_ logic (moving the actual door/light entity) remains in all clients but is
  triggered by a networked `GhostInteractionEvent`.

### 5. Summary & Progression Consistency

The `SummaryData` in [crates/unsummary-core/src/summary.rs](crates/unsummary-core/src/summary.rs) tracks many statistics
(time taken, sanity, etc.).

- **Consistency**: The Host calculates the final `Grade` and `MoneyEarned` and broadcasts it.
- **Individual XP**: Each client applies the received total to their local `PlayerProfileData`.

## Technical Research - Breadth

| Topic              | Primary Challenge                                                        | Crate Affected      |
| :----------------- | :----------------------------------------------------------------------- | :------------------ |
| **Session ID**     | Passing a string "Room Code" to `bevy_matchbox` or similar.              | `unmainmenu-plugin` |
| **Ready Check**    | Managing a bitfield or map of `PlayerId -> IsReady`.                     | `uncampaign-plugin` |
| **Remote Players** | Mapping `NetworkId` to spawning `PlayerSprite` without `MainPlayer` tag. | `unplayer-plugin`   |
| **Sync Map State** | Replicating `Door` open/closed state for players joining late (JIP).     | `untmxmap-plugin`   |

## Questions for the User

1. **Late Join**: Should players be allowed to join a mission that has already started (Join-In-Progress), or should
   sessions be locked once the mission begins?
2. **Item Ownership**: If a player disconnects, should their gear drop to the floor at their last location, or should it
   be "returned to truck"?
3. **Friendly Fire**: Should player interactions (like throwing items) affect other players (nudge them, cause sanity
   loss)?
4. **Shared Journal**: Does everyone share one journal in the truck, or does each player have their own "Evidence
   Record"? (Currently `HauntState` is global).
5. **Campaign Progress**: If 4 players complete a mission, does it unlock the next map for all of them, or only for the
   Host?
