# Map Synchronization: Design (Document 16)

This document designs the solution for map synchronization, building on the problem definition in Document 15.

---

## Design Principles

Based on discussion:

1. **Host authority** — Host decides everything, Client receives corrections
2. **Robust over clever** — If entity matching is fragile, don't rely on it
3. **Incremental** — Each phase should be testable independently
4. **Simple** — More network traffic is acceptable; correctness matters more

---

## High-Level Approach

### The Entity Correlation Problem

When both Host and Client load the same map, they create the same entities. But Bevy assigns `Entity` IDs independently,
so Host's Entity(42) is not the same as Client's Entity(42).

For entities that need syncing, we need a **stable identifier** that both sides can agree on.

**Current solution for players/ghosts/gear:** `NetworkId` assigned at spawn time, synced explicitly.

**Problem for map entities:** They're spawned during map load, not through network messages.

### Solution: Position-Based Identity

Map entities have a **canonical position** — where they appear in the TMX file. This is deterministic: both Host and
Client will create an entity at the same (x, y, z) with the same tileset and tile UID.

We can use this as a correlation key: `(x, y, z, tileset, tileuid)`

**For the MVP:**

- We don't give `NetworkId` to every map entity (too much overhead)
- Instead, we address map entities by their original position in sync messages
- This works because map entities that move are rare (only `Movable` furniture)

**For movable entities:**

- When Host assigns `NetworkId`, it also records the original position
- Host sends a mapping: "The entity originally at (5,3,0) with tileset X has NetworkId(100)"
- Client looks up that position, finds its local entity, assigns the same NetworkId
- From then on, sync uses NetworkId

---

## Data Structures

### New Snapshot Fields

Add to the `Snapshot` message:

```rust
// Breach synchronization
breach_position: Option<[f32; 3]>,  // None if not yet spawned

// Ghost type (actual, not player's guess)
ghost_type: Option<GhostType>,

// Haunted object assignments
// Identified by original board position
haunted_objects: Vec<HauntedObjectSync>,

// Movable objects that have moved or need NetworkId sync
movable_objects: Vec<MovableObjectSync>,
```

### New Message Types

```rust
/// Haunted object state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct HauntedObjectSync {
    /// Original position in map (for correlation)
    pub original_position: [i32; 3],
    /// Tileset name (for disambiguation if multiple entities at same pos)
    pub tileset: String,
    /// Tile UID (for disambiguation)
    pub tileuid: u32,
    /// What influence type was assigned
    pub influence_type: InfluenceType,  // Attractive, Repulsive, or Neutral
}

/// Movable object state
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MovableObjectSync {
    /// NetworkId assigned by Host
    pub id: NetworkId,
    /// Original position in map (for initial correlation)
    pub original_position: [i32; 3],
    pub tileset: String,
    pub tileuid: u32,
    /// Current position (may differ from original if moved)
    pub current_position: [f32; 3],
    /// Who is holding this object (if anyone)
    pub held_by: Option<NetworkId>,
}

/// Request from Client to grab an object
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrabRequest {
    pub player_id: NetworkId,
    pub target_id: NetworkId,
}

/// Response from Host
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct GrabResponse {
    pub player_id: NetworkId,
    pub target_id: NetworkId,
    pub success: bool,
}
```

---

## Implementation Phases

### Phase M.1: Breach Position Sync

**Goal:** Client sees breach in the correct location.

**Changes:**

1. **Add `breach_position` to Snapshot** (`unnet-core/src/messages.rs`)

   ```rust
   Snapshot {
       // ... existing fields ...
       breach_position: Option<[f32; 3]>,
   }
   ```

2. **Host populates breach position** (`unnet-plugin/src/systems.rs`, `host_send_snapshot_system`)
   - Query for entity with `GhostBreach` component
   - Get its `Position`, add to snapshot

3. **Client applies breach position** (`unnet-plugin/src/systems.rs`, `client_apply_snapshots_system`)
   - Find local entity with `GhostBreach`
   - Update its `Position` to match snapshot
   - If no breach exists locally but snapshot has position → spawn breach
   - If breach exists locally but snapshot has None → this is an error state, log warning

**Testing:**

- Host starts mission, breach appears at position A
- Client joins, Client's breach moves to position A
- Both players see breach in same location

**Edge cases:**

- Client loads faster than Host decides breach → Client's breach will be corrected on first snapshot
- Multiple breaches → Current design assumes one breach; extend later if needed

---

### Phase M.2: Ghost Type Sync

**Goal:** Client shows the same ghost type as Host selected.

**Changes:**

1. **Add `ghost_type` to Snapshot** (`unnet-core/src/messages.rs`)

   ```rust
   ghost_type: Option<GhostType>,
   ```

2. **Host populates ghost type** (`host_send_snapshot_system`)
   - Query entity with `GhostSprite` component
   - Get `ghost_sprite.class`, add to snapshot

3. **Client applies ghost type** (`client_apply_snapshots_system`)
   - Find local entity with `GhostSprite`
   - If `ghost_sprite.class != snapshot.ghost_type`, update it
   - This may require updating related components (evidence types, behaviors)

**Complexity note:** Ghost type affects many derived properties (which evidences are possible, behavior patterns). When
correcting the ghost type, we need to reinitialize these. Consider calling `GhostSprite::new()` with the correct type,
or adding a `set_ghost_type()` method.

**Testing:**

- Host starts mission, ghost is Phantom
- Client joins, Client's ghost changes from (whatever) to Phantom
- Evidence collection works correctly for both players

---

### Phase M.3: Haunted Object Sync

**Goal:** Client sees the same objects marked as haunted (ghost-attracting/repelling).

**Changes:**

1. **Add `haunted_objects` to Snapshot** (`unnet-core/src/messages.rs`)

   ```rust
   haunted_objects: Vec<HauntedObjectSync>,
   ```

2. **Host populates haunted objects** (`host_send_snapshot_system`)
   - Query all entities with `GhostInfluence` component
   - For each, record original position, tileset, tileuid, and influence type
   - Add to snapshot

3. **Client applies haunted objects** (`client_apply_snapshots_system`)
   - For each `HauntedObjectSync`:
     - Find local entity at that position with matching tileset/tileuid
     - Insert/update `GhostInfluence` component with correct influence type
   - For any local entity with `GhostInfluence` NOT in the snapshot:
     - Remove the `GhostInfluence` component (Client had wrong selection)

**Entity lookup helper:**

```rust
fn find_entity_by_map_position(
    board_field: &BoardEntityField,
    query: &Query<(&Position, &Behavior)>,
    target_pos: [i32; 3],
    target_tileset: &str,
    target_tileuid: u32,
) -> Option<Entity> {
    let bpos = BoardPosition { x: target_pos[0] as i64, y: target_pos[1] as i64, z: target_pos[2] as i64 };
    let entities = &board_field.0[bpos.ndidx()];
    for &entity in entities {
        if let Ok((_, behavior)) = query.get(entity) {
            if behavior.cfg().tileset == target_tileset && behavior.cfg().tileuid == target_tileuid {
                return Some(entity);
            }
        }
    }
    None
}
```

**Testing:**

- Host starts mission, objects X, Y, Z are haunted
- Client joins, Client's haunted objects change to X, Y, Z
- Moving haunted object affects ghost on both clients

---

### Phase M.4: Movable Object NetworkId + Position Sync

**Goal:** Movable objects have consistent identity and position across Host and Client.

**Changes:**

1. **Assign NetworkId to Movable entities on Host** (`classic_mode_orchestrator` or new system)
   - After map loads, query all entities with `Movable` component
   - Assign `NetworkId` to each
   - Store mapping: original position → NetworkId

2. **Add `movable_objects` to Snapshot**

   ```rust
   movable_objects: Vec<MovableObjectSync>,
   ```

3. **Host populates movable objects** (`host_send_snapshot_system`)
   - For full sync: include ALL movable objects
   - For delta sync: include only those that moved since last sync

4. **Client applies movable objects** (`client_apply_snapshots_system`)
   - For each `MovableObjectSync`:
     - If Client doesn't have entity with that NetworkId:
       - Find entity by original position/tileset/tileuid
       - Assign the NetworkId to it
     - Update entity's position to `current_position`
     - Update `held_by` state

5. **Track original positions on Host**
   - New component or resource to remember where each movable started
   - Needed for the correlation step

**New component:**

```rust
/// Records the original map position of a movable entity for network correlation
#[derive(Component)]
pub struct OriginalMapPosition {
    pub position: BoardPosition,
    pub tileset: String,
    pub tileuid: u32,
}
```

**Testing:**

- Host moves chair from A to B
- Client sees chair at B
- Client's chair entity has same NetworkId as Host's

---

### Phase M.5: Grab Permission Flow

**Goal:** Client can grab/move furniture, with Host validation.

**Changes:**

1. **Add messages** (`unnet-core/src/messages.rs`)

   ```rust
   NetworkMessage::GrabRequest { player_id: NetworkId, target_id: NetworkId },
   NetworkMessage::GrabResponse { player_id: NetworkId, target_id: NetworkId, success: bool },
   NetworkMessage::DropRequest { player_id: NetworkId },
   ```

2. **Modify grab system for Client** (`grabdrop.rs`)
   - If Client and input.grab:
     - Find target entity (must have NetworkId now)
     - Send `GrabRequest` to Host
     - Set local state to "grab pending" (optional: show visual feedback)
     - Do NOT actually grab locally yet

3. **Host handles GrabRequest** (new handler)
   - Validate: Is object not already held? Is player close enough?
   - If valid:
     - Update Host's state (object is now held by player)
     - Send `GrabResponse { success: true }`
   - If invalid:
     - Send `GrabResponse { success: false }`

4. **Client handles GrabResponse**
   - If success: local grab completes (object follows player)
   - If failure: clear "grab pending" state, optionally show feedback

5. **Position sync via snapshot**
   - When object is held, its position follows the holding player
   - `MovableObjectSync.held_by` tells Client who is holding
   - Client can derive position from holder's position

**Testing:**

- Client walks to chair, presses grab
- Brief delay (network round-trip)
- Chair attaches to Client player
- Host sees Client holding chair
- Chair position synced on both sides

---

## Suppressing Client's Random Selections

After implementing the above phases, the Client's independent selections are overwritten by sync. However, for
cleanliness, we should eventually suppress them:

**Option A (Quick):** Add a check in `classic_mode_orchestrator`:

```rust
if cli.net_mode.is_client() {
    // Skip random selections — will be synced from Host
    return;
}
```

**Option B (Cleaner):** Split orchestrator into "setup" (both) and "selection" (Host only) systems.

For MVP, Option A is sufficient. The Client's wrong selections exist briefly but are corrected by the first snapshot.

---

## Summary: What Changes Where

| File                                                | Phase   | Change                               |
| --------------------------------------------------- | ------- | ------------------------------------ |
| `unnet-core/src/messages.rs`                        | M.1-M.5 | Add new snapshot fields and messages |
| `unnet-plugin/src/systems.rs`                       | M.1-M.5 | Populate and apply new sync data     |
| `unclassic-mode-plugin/src/systems/orchestrator.rs` | M.4     | Assign NetworkId to Movables         |
| `unplayer-plugin/src/systems/grabdrop.rs`           | M.5     | Permission-based grabbing            |
| `untruck-core` or `unnet-core`                      | M.4     | New `OriginalMapPosition` component  |
| `unghost-core/src/components/ghost_sprite.rs`       | M.2     | Possibly add `set_ghost_type()`      |

---

## Open Questions

1. **Ghost type correction complexity** — When Client's ghost type is wrong, how much state needs resetting? Need to
   investigate what depends on `GhostSprite.class`.

2. **Full sync frequency** — Should movable objects always be in full sync, or use delta? For MVP, always include all in
   snapshot (simple). Optimize later if needed.

3. **Visual feedback during grab request** — Should Client see "grabbing..." animation during round-trip? Or just delay?
   For MVP, simple delay is fine.

4. **Multiple entities at same position** — The tileset+tileuid should disambiguate, but verify this assumption holds in
   actual maps.

---

## Testing Strategy

After each phase, test with this scenario:

1. Host starts mission on "Maple Lodge" (or any map)
2. Wait 5 seconds (Host makes all selections, maybe moves something)
3. Client connects
4. Verify Client sees same state as Host for that phase's elements

Specific checks:

- **M.1:** Breach marker visible in same room for both
- **M.2:** Ghost type in journal matches on both (if visible)
- **M.3:** Haunted objects glow/highlight correctly for both
- **M.4:** Any moved furniture is in same position for both
- **M.5:** Client can grab chair, Host sees it

---

## Appendix: Why Not Seeded Determinism?

We considered having both Host and Client run the same RNG with a shared seed. Rejected because:

1. **Fragile** — Any difference in code path (iteration order, floating point) causes silent divergence
2. **Hard to debug** — When it fails, you don't know why
3. **Doesn't help late-join** — Still need full state sync for reconnection

Host authority is more network traffic but eliminates an entire class of bugs.
