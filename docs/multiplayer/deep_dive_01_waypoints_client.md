# Deep Dive: Mouse Clicking Waypoints Not Working on Client

## 1. Problem Description

In multiplayer matches, when playing as a client, clicking on the ground or interactive objects to create waypoints for
movement/interaction does not function as expected. The player character fails to follow the clicked path or interact
with the clicked objects.

## 2. Facts (100% Certain)

- **Local Logic**: Both `waypoint_creation_system` and `waypoint_following_system` (located in
  [crates/unplayer-plugin/src/systems/waypoint.rs](crates/unplayer-plugin/src/systems/waypoint.rs)) are registered to
  run on both Host and Client.
- **Component Requirements**: These systems use a query that requires the `MainPlayer` component to identify the local
  player.
- **Spawning Logic A**: `unclassic-mode-plugin` (specifically
  [classic_mode_orchestrator](crates/unclassic-mode-plugin/src/systems/orchestrator.rs)) spawns a player entity and
  inserts the `MainPlayer` component when it detects the local player's role (Offline, Host, or Joiner).
- **Spawning Logic B**: `unnet-plugin` (specifically
  [client_apply_snapshots_system](crates/unnet-plugin/src/systems.rs)) also spawns player entities when it receives
  snapshots for unknown `NetworkId`s.
- **Missing Component**: The `spawn_remote_player` function in [unnet-plugin](crates/unnet-plugin/src/systems.rs) DOES
  NOT insert the `MainPlayer` component, even if the ID matches the local player's ID.
- **Input Sync**: The client sends its `PlayerInput` to the host via
  [client_send_input_system](crates/unnet-plugin/src/systems.rs), which queries for `With<MainPlayer>`.
- **Early Exit**: `waypoint_creation_system` returns early if the entity with `MainPlayer` also has an `InTruck`
  component.
- **Truck Sync Bug**: `client_apply_snapshots_system` explicitly skips updating the `InTruck` component for entities
  that have `MainPlayer`.

## 3. Facts (100% Certain) (Extended)

- **The Identity Conflict (MainPlayer vs NetworkId):**
  - The waypoint systems (`waypoint_creation_system` and `waypoint_following_system`) are gated by `With<MainPlayer>`.
  - The networking system `spawn_remote_player` in
    [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs#L1058) spawns player entities with
    `PlayerSprite`, `NetworkId`, and `PlayerInput`, but **omits** `MainPlayer`.
  - **RESEARCH FINDING:** On the Client, the entity that the user _should_ be controlling is often the one
    spawned/updated by the network (to maintain sync with the Host). Because this entity lacks the `MainPlayer` tag, the
    waypoint systems ignore it entirely. Clicks are either ignored or attempt to control a "phantom" local-only entity
    that is immediately snapped back or overriden by the network-synced entity.

- **Coordinate Systems (Screen to World):**
  - `waypoint_creation_system` uses [perspective::screen_to_world](crates/unspatial-core/src/perspective.rs) to convert
    mouse clicks into target positions.
  - This calculation depends on the player's current `Z` (floor) position.
  - If the client's local player entity is de-synced from the network-synced entity, the `Z` value used for the
    projection might be incorrect, leading to clicks "hitting" the wrong floor or being projected into the void.

- **In-Truck Filtering:**
  - The waypoint systems explicitly check for `InTruck` and return early if found.
  - If the `InTruck` component is not correctly synced to the client's representation of themselves, they may be able to
    set waypoints while in the truck (which likely breaks when they "leap" out of the truck bounds).

## 4. Guesses and Theories

### Theory A: The "Double Entity" Race Condition (High Confidence)

There is a fundamental race condition between the map loading systems and the network snapshot systems.

1. **The Snap:** A network snapshot arrives before the map is fully ready. `unnet-plugin` checks its local
   `net_to_entity` map and finds nothing for ID 2 (the client). It calls `spawn_remote_player`, creating **Entity A**.
   This entity has `NetworkId(2)` but NO `MainPlayer` tag.
2. **The Spawn:** The map finishes loading, triggering `MapEntitiesReadyEvent`. `classic_mode_orchestrator` runs. It
   sees it is a `NetMode::Join` and spawns a player for ID 2 using its own logic. This creates **Entity B** with
   `NetworkId(2)` AND the `MainPlayer` tag.
3. **The Duality:** The client now has two entities representing the same network ID.
4. **The Update Loop:** Every frame, `client_apply_snapshots_system` rebuilds `net_to_entity`. Since both entities have
   `NetworkId(2)`, the hashmap will only keep one. If it keeps **Entity A** (the one without `MainPlayer`), then
   snapshots for the local player's position/animation are applied to the "ghost" entity.
5. **The Waypoint Failure:**
   - `waypoint_creation_system` queries for `MainPlayer`, so it interacts with **Entity B**.
   - It creates waypoints near **Entity B**'s position (likely a stagnant spawn point).
   - It updates `PlayerInput` on **Entity B**.
   - `client_send_input_system` sends the input from **Entity B**'s component to the Host.
   - The Host moves ID 2 on its side and sends back a snapshot.
   - The Client applies the snapshot to **Entity A**.
   - **Result**: The player's camera (on Entity B) stays stagnant at the spawn point while Entity A (invisible or
     separate) moves around. The player sees waypoints being created but the "character" they are controlling (Entity B)
     never moves, even though the server thinks they are.

### Theory B: Stale "InTruck" Component (Medium Confidence)

Even if only one entity exists, it might be stuck with the `InTruck` component due to a logic flaw in snapshot
application.

1. The `client_apply_snapshots_system` in `unnet-plugin` has an explicit check:
   `if main_player.is_none() { ... update InTruck ... }`.
2. This means if an entity has the `MainPlayer` tag, its `InTruck` status is **NEVER** updated via network snapshots.
3. If the client somehow starts with or gains the `InTruck` component (e.g. during lobby or a local state transition),
   and the Host later decides they are NOT in the truck (transition to mission), the Host's snapshot will say
   `is_in_truck: false`, but the client will **ignore** this for the `MainPlayer`.
4. Since `waypoint_creation_system` has an early return `if !q_in_truck.is_empty()`, all mouse clicks are silently
   ignored.

## 4. Proposed Solutions

- **Single Entity Authority**: `unnet-plugin` should be the sole authority for spawning networked entities.
  `classic_mode_orchestrator` should skip spawning players if in multiplayer mode, or at least should not spawn its own
  `MainPlayer` entity if one already exists for that ID.
- **InTruck Sync**: Remove the `if main_player.is_none()` guard around the `InTruck` and `Hiding` component updates in
  `client_apply_snapshots_system`. The Host should be authoritative over these states even for the local player to
  ensure synchronization.
- **Identity Adoption**: When receiving the `Welcome` message with a local ID, the client should search for any existing
  player entity and "adopt" it as the `MainPlayer`, rather than waiting for a separate orchestrator.

## 5. Further Research Needed

1. **Verify Entity Count**: Check if two entities with the same `NetworkId` actually exist on the client during a
   session.
2. **Event Propagation**: Confirm that `Pointer<Click>` events are actually being fired on the client for map sprites.
3. **InTruck State**: Verify if the `MainPlayer` on the client keeps the `InTruck` component after the mission starts.
4. **Log Analysis**: Search for "Spawning remote player" logs on the client to see if it spawns itself.
