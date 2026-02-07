# Deep Dive: Truck Items Not Despawning and EMF Pinging

## 1. Problem Description

In multiplayer matches, items in the truck seem to linger or fail to despawn correctly. This is most noticeable with EMF
meters, which continue to "ping" or chirp in the truck even after players have attempted to remove or use them.

## 2. Facts (100% Certain)

- **Local Spawning**: The `untruck-plugin` (see [truckgear.rs](crates/untruck-plugin/src/truckgear.rs)) contains a
  system `initialize_truck_gear` that spawns a full set of mission gear upon receiving a `LevelLoadedEvent`.
- **Global Execution**: This system is registered to run on all instances (Host and Client).
- **Network Sync**: `unnet-plugin` [syncs gear entities](crates/unnet-plugin/src/systems.rs) from Host to Client using
  `NetworkId` and snapshots.
- **Divergent Paths**: Gear spawned by the Host has a `NetworkId` and is synced. Gear spawned locally by the Client's
  `initialize_truck_gear` DOES NOT have a `NetworkId` and is completely unknown to the Host.
- **Sound Logic**: The `update_emfmeter` system (in
  [ungearitems-plugin](crates/ungearitems-plugin/src/components/emfmeter.rs)) plays chirping sounds based on the item's
  `emf` value. While the _readings_ update only on the Host (`if is_host`), the _sound effect_ logic is NOT gated and
  runs on all instances.

## 2. Facts (100% Certain) (Extended)

- **Network-Driven Despawning:**
  - In [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs#L2250), the system processes
    `TruckInventoryChange` messages (e.g., `RemoveRightHand`, `RemoveInventoryIndex`).
  - These messages call `commands.entity(e).despawn()`.
  - **RESEARCH FINDING:** If the Client and Host disagree on the inventory index or state of the truck (e.g., due to the
    local orchestrator having spawned its own gear), the Host might send a "Remove" message that the Client erroneously
    applies to the local gear it thinks is in the truck.

- **Grid and Bounds Interaction:**
  - While no "out-of-bounds" despawner was found in `unmapload-plugin`, there is a significant interaction with
    `FloorItemCollidable`.
  - When gear is "synced" by the network, `unnet-plugin` manages the addition/removal of `FloorItemCollidable`.
  - If an item is moved from the truck (not a "grid" room) to a room, it becomes part of the grid. If the Host and
    Client are de-synced on _where_ the truck is or the player's presence in it, the item might be dropped into an
    "invalid" grid location and effectively lost.

- **Initialization Race:**
  - `untruck-plugin/src/truckgear.rs` drains the truck inventory on `LevelLoadedEvent`.
  - In multiplayer, `LevelLoadedEvent` might fire at different times relative to the initial network snapshot.
  - If the snapshot arrives _before_ the local level finish, the local orchestrator might "wipe" the network-synced
    gear.

## 3. Guesses and Theories

### Theory A: The "Ghost Gear" Duplicate (High Confidence)

Due to the `initialize_truck_gear` system running on both Host and Client, every multiplayer session starts with a
double set of gear in the truck:

1. **Host's Gear (Synced)**: These items have `NetworkId`s. When a player grabs one, the Host updates its state, and all
   clients see it move/disappear.
2. **Client's Gear (Local)**: These items appear in the exact same spot but have no `NetworkId`. The Host doesn't know
   they exist. When a player tries to "grab" them, the `GrabRequest` (which uses `NetworkId`) will likely fail or target
   the synced version instead.
3. **The Lingering Effect**: The Local Gear remains on the truck shelf indefinitely. Since the Client is running the
   `update_emfmeter` sound logic locally, if these "ghost" EMF meters have any registered EMF value (or default noise),
   they will chirp in the truck for the entire mission.

### Theory B: EMF Sound Logic Loop

Even if gear is grabbed, the client-local logic for playing the "chirp" sound might be using stale data. If an item is
grabbed and made "invisible" on the Host (by removing `Sprite`/`Transform`), those component removals are NOT currently
synced to the Client. The Client's `sync_held_gear_position` keeps moving the entity to the player's position, but if it
remains "active" in the Client's eyes, it keeps making noise.

## 4. Proposed Solutions

- **Gato Spawning by Role**: The `initialize_truck_gear` system should be restricted to run only on the Host
  (`run_if(is_host)`). The Client should rely entirely on snapshots to see the truck's equipment.
- **Sync Component Removals**: The network snapshot system should handle the removal of components (like `Sprite`,
  `Transform`, `Visibility`) when an item is grabbed, ensuring it truly "disappears" on the Client.
- **Gate Audio by Local State**: Ensure that gear chirps/sounds only play if the item is held by a player or is actively
  being used, and potentially gate more of the audio logic behind `is_host` if the readings are server-authoritative.

## 5. Further Research Needed

1. **Check Default EMF**: Verify if a newly spawned `EMFMeter` has a non-zero `emf` value that would cause chirping.
2. **Snapshot Component Visibility**: Investigate if snapshots can be used to sync "missing" components (Entity exists
   but should not have a Sprite).
3. **GrabRequest for Local Entities**: Confirm what happens when a client tries to grab one of their own "local-only"
   entities in a networked game.
