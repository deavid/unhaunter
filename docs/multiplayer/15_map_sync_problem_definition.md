# Map Synchronization: Problem Definition (Document 15)

This document defines the map synchronization problem for multiplayer. It does **not** propose solutions — only
articulates what we want, what we have, and why there's a gap.

---

## Part 1: Intended Behavior (The Vision)

### 1.1 Core Principle: Host Authority

The Host is the single source of truth for the world state. The Host decides everything:

- Where the ghost breach is located
- Which objects are "haunted" (ghost-attracting/repelling)
- What ghost type was selected
- Current state of all doors (open/closed)
- Current state of all room lights (on/off)
- Position of all movable objects
- Who is holding what

Clients do not make independent decisions. They receive the world state from the Host and render it.

### 1.2 All Joins Are "Late Joins"

There is no special "initial join" vs "late join" distinction. The Host always loads the map first and makes all
decisions. When a Client connects (whether 3 seconds after Host or 30 minutes later), the Client receives:

1. The full world state as it currently exists
2. Not "here's the seed, figure it out yourself"

This means even the first Client to join is technically "late" — the Host has already decided everything.

### 1.3 Functionally Identical Worlds

Both players must see a world that is **functionally indistinguishable**. This means:

**Must be identical:**

- Ghost breach location and visual state
- Haunted object selections and their positions
- Door states (open/closed)
- Room light states (on/off)
- Positions of all movable furniture
- Ghost behavior and position
- Evidence manifestations (temperature readings, EMF, etc.)

**Acceptable differences (cosmetic only):**

- Exact positions of particle effects (miasma, smoke)
- Sprite animation frames for ambient effects
- Minor visual flourishes that don't affect gameplay

### 1.4 Why This Matters for Gameplay

| Element         | If Out of Sync                          | Gameplay Impact                                                                     |
| --------------- | --------------------------------------- | ----------------------------------------------------------------------------------- |
| Breach location | Client sees breach in wrong room        | Temperature readings make no sense; "12°C on top of breach" but breach is elsewhere |
| Haunted objects | Client sees wrong objects highlighted   | Moving objects has no effect on ghost; advanced tactics broken                      |
| Doors           | Client sees door closed, Host sees open | Player walks through "closed" door; collision/visibility broken                     |
| Movable objects | Client can't move furniture             | Core mechanic non-functional; can't reposition haunted objects                      |

For experienced players, haunted objects are critical for manipulating ghost psychology. For all players, door sync is
critical for basic collision and visibility.

---

## Part 2: Current Behavior (What The Code Does)

### 2.1 Map Loading Flow

**Host side:**

1. Host selects map from CLI or menu
2. Host loads map via `unmapload-plugin`
3. `MapEntitiesReadyEvent` fires
4. `classic_mode_orchestrator` runs:
   - Uses `random_seed::rng()` (non-deterministic) to pick player spawn
   - Uses `random_seed::rng()` to pick ghost spawn from `HostileSpawnPoint` entities
   - Uses `random_seed::rng()` to select haunted objects via `selection.rs`
5. Ghost, breach, and players are spawned

**Client side:**

1. Client sends `Hello` message
2. Host responds with `Welcome { id, map_seed, map_filepath, difficulty_id }`
3. Client loads the same map file
4. `MapEntitiesReadyEvent` fires on Client
5. `classic_mode_orchestrator` runs **again** on Client:
   - Uses `random_seed::rng()` — **different seed, different results**
   - Picks **different** ghost spawn, **different** haunted objects
6. Client now has a divergent world state

**The `map_seed` in the Welcome message is received but never used.** It's stored but the selection systems don't read
from it.

### 2.2 What IS Currently Synced

The snapshot system (`host_send_snapshot_system` → `client_apply_snapshots_system`) syncs:

| Data                      | How                                    | Notes           |
| ------------------------- | -------------------------------------- | --------------- |
| Player positions          | `PlayerState` in snapshot              | Works           |
| Player hiding/truck state | `PlayerState.is_hiding`, `is_in_truck` | Works           |
| Ghost position & state    | `GhostState` in snapshot               | Works           |
| Room light states         | `RoomSync` in snapshot                 | Works           |
| Door tile states          | `MapTileState` in snapshot             | Partially works |
| Gear positions & state    | `GearSyncState` in snapshot            | Works           |
| Player gear assignments   | `PlayerGearState` in snapshot          | Works           |
| Evidence found            | `evidences_found` in snapshot          | Works           |

### 2.3 What Is NOT Synced

| Data                      | Why Not                         | Impact                           |
| ------------------------- | ------------------------------- | -------------------------------- |
| Ghost breach location     | Never sent to Client            | Client picks own breach location |
| Haunted object selections | Never sent to Client            | Client picks own haunted objects |
| Ghost type                | Never explicitly sent           | Client picks own ghost type      |
| Movable object positions  | No `NetworkId`, not in snapshot | Client can't see moved furniture |
| Held objects (non-gear)   | `held_item` partially synced    | Furniture grabbing broken        |

### 2.4 The Snapshot's Map Tile Sync

The snapshot includes `map_tiles: Vec<MapTileState>`:

```rust
pub struct MapTileState {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub tileset: String,
    pub tileuid: u32,
}
```

This syncs **tile visual state** (e.g., door open vs closed) by board position. It works for:

- Doors changing from closed→open tile
- Switches changing visual state

It does **NOT** work for:

- Objects that have moved from their original position
- Objects that need more than just tileset/tileuid to describe their state
- Identifying "which entity" when multiple entities could be at a position

### 2.5 The Selection System

In `unclassic-mode-plugin/src/selection.rs`:

```rust
pub(crate) fn generate_scored_ghost_setup(...) {
    // ...
    let mut rng = random_seed::rng();  // <-- Non-deterministic!

    for _ in 0..simulation_count {
        let spawn_point = select_ghost_spawn_point(ghost_spawn_points, &mut rng);
        let influence_assignments = select_influence_objects(&objects_by_floor, board_toplogy, &mut rng);
        // ...
    }
}
```

Every call to `random_seed::rng()` creates a new RNG seeded from system time. Host and Client will get different results
even if they run the same code.

### 2.6 Movable Objects

Movable objects are spawned during map hydration with the `Movable` component:

```rust
// unmapload-plugin/src/hydration_generic.rs
if behavior.p.object.movable {
    cmd.insert(components::Movable);
}
```

They do **NOT** receive a `NetworkId`. The snapshot system only syncs entities that have `NetworkId`:

- Players (assigned ID on spawn)
- Ghosts (assigned ID on spawn)
- Gear items (assigned ID on spawn)

Movable furniture has no network identity and is invisible to the sync system.

### 2.7 Grabbing System

In `unplayer-plugin/src/systems/grabdrop.rs`:

```rust
fn grab_object(...) {
    for (mut player_gear, player_pos, player_input) in players.iter_mut() {
        if player_input.grab {
            // Find closest pickable object
            // Directly modify local components
            // No network message sent
        }
    }
}
```

Grabbing is purely local. When a Client grabs an object:

1. Client's local `PlayerGear.held_item` is set
2. No message is sent to Host
3. Host doesn't know Client grabbed anything
4. Object position diverges

---

## Part 3: The Gap

### 3.1 Root Cause: Client Runs Selection Independently

The fundamental problem is that `classic_mode_orchestrator` runs on **both** Host and Client, and both make independent
random selections. The `map_seed` sent in `Welcome` is never used.

```text
Host                              Client
────                              ──────
Load map
Select ghost spawn (RNG)
Select haunted objects (RNG)
Spawn ghost at position A
                                  Receive Welcome (map_seed unused)
                                  Load map
                                  Select ghost spawn (RNG) → position B
                                  Select haunted objects (RNG) → different set
                                  Spawn ghost at position B (ignored by sync)
```

The Client's ghost entity is immediately overwritten by snapshot sync, but the **haunted object assignments** and
**breach entity position** are never corrected because they're not in the snapshot.

### 3.2 Root Cause: No Network Identity for Map Objects

Movable objects exist only as "map tiles" with a board position. They have no `NetworkId`, so:

1. The snapshot system doesn't know they exist
2. There's no way to say "object X moved from A to B"
3. There's no way to say "Player 2 is holding object X"

### 3.3 Root Cause: No Authority Model for Grabbing

The grab system assumes single-player. It directly modifies local state without:

1. Checking who has authority
2. Sending a request to Host
3. Receiving confirmation before acting

### 3.4 Summary of Gaps

| Intended                              | Current                               | Gap                                                              |
| ------------------------------------- | ------------------------------------- | ---------------------------------------------------------------- |
| Host decides breach location          | Both decide independently             | Client selection not suppressed; Host selection not sent         |
| Host decides haunted objects          | Both decide independently             | Same as above                                                    |
| Client receives full world state      | Client only receives partial snapshot | Breach, haunted objects, movable positions missing from snapshot |
| Movable objects have network identity | Movable objects have no NetworkId     | Cannot be addressed in sync messages                             |
| Grabbing requires Host permission     | Grabbing is local-only                | No request/response flow                                         |

---

## Part 4: Scope Clarification

### 4.1 What's In Scope for MVP

Based on discussion:

- **Must work:** Breach location, haunted objects, doors, room lights, movable object positions
- **Must work:** Permission-based grabbing for furniture
- **Must work:** Full world state on any join (first join = late join)

### 4.2 What Can Be Deferred

- **Mouse interaction for Client** (picking objects with mouse) — minor, tackle later
- **Seeded determinism** — not pursuing; Host authority is simpler and more robust
- **Optimized delta sync** — full state sync is fine for MVP; optimize later if needed

### 4.3 Complexity Budget

David's guidance: "Host authority is what I am going for. I want simple and robust. If it means more traffic, more lag,
more network... I'm fine with it for the MVP stage."

This means:

- Prefer explicit state transfer over clever algorithms
- Prefer correctness over performance
- Prefer working now over elegant later

---

## Part 5: Open Questions for Solution Design

These questions should be answered in a follow-up solution document:

1. **How should NetworkId be assigned to movable objects?**
   - Host assigns during map load?
   - How does Client know which local entity maps to which NetworkId?

2. **What new data needs to be in the snapshot?**
   - Breach position?
   - Haunted object assignments (entity ID + influence type)?
   - Movable object positions?

3. **Should Client skip `classic_mode_orchestrator` entirely?**
   - Or run it but ignore certain parts?
   - How to suppress local ghost/breach spawning on Client?

4. **What's the grab request/response flow?**
   - Client sends `GrabRequest { target_entity: NetworkId }`?
   - Host validates and responds?
   - What happens during the round-trip delay?

5. **How to handle the "Welcome" → "Full State" transition?**
   - Is `Welcome` enough, or do we need a separate `WorldState` message?
   - When does Client consider itself "ready"?

---

## Appendix: Relevant Code Locations

| Purpose                          | File                                                                   |
| -------------------------------- | ---------------------------------------------------------------------- |
| Selection logic                  | `crates/unclassic-mode-plugin/src/selection.rs`                        |
| Orchestrator (spawns everything) | `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`             |
| Snapshot sending                 | `crates/unnet-plugin/src/systems.rs` (`host_send_snapshot_system`)     |
| Snapshot receiving               | `crates/unnet-plugin/src/systems.rs` (`client_apply_snapshots_system`) |
| Grab/drop logic                  | `crates/unplayer-plugin/src/systems/grabdrop.rs`                       |
| Network messages                 | `crates/unnet-core/src/messages.rs`                                    |
| Movable component                | `crates/unbehavior/src/components.rs`                                  |
| Map hydration                    | `crates/unmapload-plugin/src/hydration_generic.rs`                     |
| Welcome message handling         | `crates/unnet-plugin/src/systems.rs` (`handshake_handler_system`)      |
