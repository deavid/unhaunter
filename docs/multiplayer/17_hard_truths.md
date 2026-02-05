# Hard Truths: A Self-Reflection on the Multiplayer Architecture (Document 17)

This document is a brutally honest post-mortem of the multiplayer implementation journey. It identifies what went wrong,
why it went wrong, and what the path forward should look like. No sugar-coating, no excuses.

---

## The Core Problem: We Solved the Wrong Problem

### What We Built

We built **a networking layer** — TCP connections, JSONL serialization, handshakes, snapshot transmission, input
forwarding.

### What We Needed

We needed **an authority model** — clear rules about who owns state, who can mutate it, and how changes propagate.

### The Symptom-Chasing Trap

Every document from 01 to 16 reveals the same pattern:

1. Bug discovered: "Flashlights don't sync" / "Doors don't update" / "Van entry breaks both players"
2. Root cause identified: "State X isn't in the snapshot" / "Client runs the same logic as Host"
3. "Fix" proposed: Add more fields to snapshot, add more message types, add more sync logic
4. New bug emerges from the "fix"
5. Repeat

We were treating **symptoms**, not the **disease**. The disease is that the game was designed as single-player with
networking bolted on afterward.

---

## Hard Truth #1: The Snapshot Is a Kitchen Sink

The `SnapshotMsg` struct now has **22 fields**:

```rust
pub struct SnapshotMsg {
    pub tick: u64,
    pub is_full_sync: bool,
    pub app_state: AppState,
    pub game_state: GameState,
    pub players: Vec<PlayerState>,
    pub ghosts: Vec<GhostState>,
    pub rooms: Vec<RoomSync>,
    pub map_tiles: Vec<MapTileState>,
    pub gear: Vec<GearSyncState>,
    pub player_gear: Vec<PlayerGearState>,
    pub events: Vec<TransientEvent>,
    pub evidences_found: Vec<Evidence>,
    pub evidences_missing: Vec<Evidence>,
    pub ghost_type_guess: Option<GhostType>,
    pub ghosts_discarded: Vec<GhostType>,
    pub mission_result: Box<Option<MissionResult>>,
    pub repellent_crafted_count: u32,
    pub breach_position: Option<[f32; 3]>,
    pub ghost_type: Option<GhostType>,
    pub haunted_objects: Vec<HauntedObjectSync>,
    pub movable_objects: Vec<MovableObjectSync>,
}
```

Every time something didn't sync, we added another field. This is **design by accretion**, not architecture.

**Why It's a Problem:**

- No coherent model of what should be authoritative vs. derived
- No clear ownership — snapshot is assembled by reading random queries
- Impossible to reason about what's synced vs. what isn't
- Adding a new feature requires modifying the snapshot (tight coupling)

**The Truth:** We should have defined **one canonical authority model** upfront, not grown a snapshot organically.

---

## Hard Truth #2: State Triplication Creates Unsolvable Bugs

Item ownership is stored in **three places**:

1. `PlayerGear` on the player entity — `left_hand: Option<Entity>`, `right_hand: Option<Entity>`,
   `held_item: Option<Entity>`
2. `EquipmentPosition` on the gear entity — `In(Slot::LeftHand)`, `On(BoardPosition)`, etc.
3. `FloorItemCollidable` on the gear entity — present when on floor, absent when held

This is classic **normalization failure**. If any of these disagree, the game is in an inconsistent state.

**Current "solution":** Sync all three independently and hope they stay consistent.

**Why It Fails:**

- Race conditions: Client updates `EquipmentPosition` before Host updates `PlayerGear`
- Partial failures: One sync succeeds, another fails
- No single source of truth: Which one do you trust when they conflict?

**The Truth:** There should be **exactly one** authoritative representation. Everything else should be derived.

```txt
   Authoritative                    Derived
   ─────────────                    ───────
   PlayerGear.left_hand = Some(E)   EquipmentPosition::In(LeftHand)  ← computed
                                    FloorItemCollidable: absent      ← computed
```

We never built this derivation layer. We just synced everything and prayed.

---

## Hard Truth #3: The 2300-Line God File

[systems.rs](../../crates/unnet-plugin/src/systems.rs) contains **2295 lines** with 12 public functions doing
everything:

- TCP connection management
- Handshake protocol
- Snapshot assembly (22 fields worth)
- Snapshot application (inverse of assembly)
- Input forwarding
- Grab/drop request handling
- Truck inventory sync
- Mission summary sync
- Disconnection handling

**Why It's a Problem:**

- Impossible to understand without reading the whole file
- Changes have unpredictable ripple effects
- No separation of concerns — transport, protocol, game logic all intertwined
- The `HostSnapshotParams` and `ClientSnapshotParams` have **22+ queries each**

**The Truth:** This should have been split into:

- `transport.rs` — Raw TCP I/O, connection lifecycle
- `protocol.rs` — Message serialization, handshake state machine
- `sync/*.rs` — Per-domain sync (players, ghosts, gear, map tiles, etc.)
- `authority.rs` — Who owns what, permission checking

We didn't do this because "we'll refactor later." Later never came.

---

## Hard Truth #4: We Sync Derived State Instead of Source State

Example: Door synchronization.

**What We Do:**

1. Host opens door → Host's `Behavior` component changes tile UID
2. Host reads `Behavior` → Puts tile UID in `MapTileState`
3. Host sends snapshot with `MapTileState`
4. Client receives snapshot → Finds entity at position → Calls `execute_interaction()`
5. Client's `execute_interaction()` re-runs the door logic, playing sounds, emitting events

**What We Should Do:**

1. Host opens door → Host records "Door at (5,3,0) is now open"
2. Host sends "Door(5,3,0) = open" to Client
3. Client receives → Sets door state → Derives visuals from state

We're syncing the **visual tile UID** instead of the **semantic state**. This means:

- Client runs door-opening logic again (wrong sounds, duplicate events)
- No rollback possible (we don't know "previous state")
- Position-based lookups are fragile (what if two doors at same position?)

**The Truth:** Network sync should operate on **state**, not **side effects**.

---

## Hard Truth #5: "Client Runs Everything, Host Overwrites" Is Not Authority

The current model:

1. Client loads map → Runs `classic_mode_orchestrator` → Spawns ghost, picks haunted objects
2. Host sends snapshot → Overwrites ghost position
3. Client's breach location and haunted objects are **never corrected**

This is "eventual consistency" without the "eventual."

**Why It's a Problem:**

- Client has entities with wrong state that never get fixed
- Wasted computation (Client makes decisions that are immediately discarded)
- Hard to debug (is this entity's state from Client or Host?)

**What Authority Should Mean:**

- Client should **never** run selection logic
- Client should **wait** for Host to tell it where things are
- Client state should be **empty until populated by Host**

Document 16 proposed "all joins are late joins." This is correct. We never implemented it.

---

## Hard Truth #6: The MVP Plan Was Ignored

Document 11 laid out a clear plan:

> **Phase 3: Snapshot Synchronization**
>
> - [ ] Create data structures for `Snapshot` (players, ghosts, gear)
> - [ ] Implement `host_send_snapshot_system` (collect world state)
> - [ ] Implement `client_apply_snapshots_system` (reconcile local state)
> - [ ] Support full sync on join, incremental sync after

We did this... sort of. But then:

- Every bug led to "add more to snapshot"
- No refactoring, just more fields
- No testing, just "does it work on my machine?"

The MVP was supposed to be **minimal**. Instead:

- 22-field snapshot
- 6+ request/response message pairs
- Complex permission flows for grab/drop
- Still broken

**The Truth:** We violated the MVP principle. We kept adding features without stabilizing the foundation.

---

## Hard Truth #7: No Abstractions, Just Repetition

Look at how we handle gear in snapshots:

**Host side (assembly):**

```rust
for (entity, net_id, pos, toggleable, gear_marker, ...) in gear_query.iter() {
    gear.push(GearSyncState {
        id: net_id.0 as u32,
        position: [pos.x, pos.y, pos.z],
        is_on: toggleable.map_or(false, |t| t.is_on),
        // ... more fields
    });
}
```

**Client side (application):**

```rust
for g_sync in &snapshot.gear {
    if let Some((entity, ...)) = gear_query.iter().find(|(_, nid, ..)| nid.0 as u32 == g_sync.id) {
        // ... update components manually
    }
}
```

This pattern repeats for: players, ghosts, rooms, map tiles, player gear, movable objects, haunted objects, etc.

**No abstraction.** Every entity type has bespoke assembly and application code.

**What We Should Have:**

```rust
trait NetworkSync {
    type SyncData: Serialize + DeserializeOwned;
    fn to_sync(&self) -> Self::SyncData;
    fn from_sync(&mut self, data: &Self::SyncData);
}
```

Then: `snapshot.entities = world.query::<&dyn NetworkSync>().map(|s| s.to_sync()).collect()`

We never built this abstraction because "it's just a few entity types." Now it's a dozen.

---

## Hard Truth #8: The Documentation Is Better Than the Code

We wrote 16 detailed documents:

- Architecture analysis
- Problem definitions
- Implementation plans
- Phase breakdowns

Yet the code doesn't reflect this understanding.

**Document 12** clearly identified:

> "Most game systems were designed for single-player and are not authority-aware."

**Document 15** clearly defined:

> "The Host is the single source of truth for the world state."

**Document 16** provided:

> Detailed phases M.1 through M.5 for fixing map sync.

We have the knowledge. We didn't apply it systematically.

**Why?**

- Pressure to "just fix the bug"
- Each session started with "what's broken now?" instead of "what's next on the plan?"
- No discipline to follow the roadmap

**The Truth:** We knew what to do. We didn't do it.

---

## Hard Truth #9: Grab/Drop Is the Symptom, Not the Cause

Multiple sessions focused on making grab/drop work:

- M.4: Add `MovableObjectSync` to snapshots
- M.5: Add explicit `GrabRequest` / `GrabResponse` messages
- Query conflict fixes
- `EquipmentPosition` sync additions

After all this work: **Grab/drop still doesn't work.**

Because we were fixing the **mechanism** without fixing the **model**.

**The Model Problem:**

1. Who decides if a grab succeeds? (Host)
2. What state changes on success? (PlayerGear.held_item, EquipmentPosition, Position, FloorItemCollidable)
3. How does Client learn about the change? (Snapshot? Response message? Both?)
4. What if Client's view is stale? (Entity moved, already grabbed by someone else)

We answered these with ad-hoc code instead of a coherent design.

**The Truth:** Before writing another line of grab/drop code, we need a **state machine diagram** of item ownership
transitions.

---

## Hard Truth #10: We Have Technical Debt We Can't Pay

The codebase now has:

- Two `NetworkId` types (u32 in `untags-core`, u64 in `unnet-core`)
- Entity lookup by position (fragile, O(n))
- `OriginalMapPosition` component (added to correlate entities that should have been tracked from spawn)
- Multiple overlapping query filters that sometimes work, sometimes don't
- Suppressed Clippy warnings for type complexity

Every workaround created more workarounds.

**Document 9** identified early refactors needed:

> - Remove `.single()` assumptions
> - Make `PlayerInput` a component, not a resource
> - Unify `NetworkId` types

We skipped these. Now they compound every new bug.

**The Truth:** We can't build features on a broken foundation. We need to pay the debt first.

---

## The Path Forward

### Option A: Continue Patching (Not Recommended)

Keep adding fields to snapshot, keep chasing symptoms, hope it eventually works.

**Prediction:** We'll write documents 18, 19, 20... with the same patterns.

### Option B: Architectural Reset (Recommended)

1. **Define the authority model formally** — What state exists? Who owns it? How does it replicate?

2. **Eliminate state triplication** — One source of truth for item ownership, derive everything else.

3. **Split the god file** — Transport, protocol, sync per-domain.

4. **Build abstractions** — `NetworkSync` trait, entity registration, automatic serialization.

5. **Suppress Client-side logic** — Client should not run game logic, only rendering.

6. **Test before adding features** — Two instances, verify sync before adding complexity.

### Option C: Simpler Networking Model

Consider: Do we need full host-authority with 60Hz snapshots?

Alternative: **Lockstep simulation** — Both instances run the same logic with the same inputs. Only sync inputs, not
state. Determinism guarantees consistency.

**Trade-offs:**

- Lockstep requires determinism (no system time, seeded RNG)
- Lockstep has input delay instead of state correction
- Lockstep is simpler to reason about

This might be overkill, but it's worth considering whether our current model is the right fit.

---

## Conclusion

We built a lot. We learned a lot. But we didn't build the **right things** in the **right order**.

The multiplayer implementation is not "almost working." It's architecturally broken. The snapshot-based replication
model can work, but only if:

1. There's a single source of truth for each piece of state
2. Sync operates on state, not side effects
3. Client is purely reactive (no independent logic)
4. Code is organized by concern, not by "what bug am I fixing today"

Until we address these fundamentals, we'll keep playing whack-a-mole.

**The hardest truth:** We need to stop adding features and start fixing architecture. That might mean reverting to a
simpler working state and rebuilding properly.

---

## Appendix: Questions We Never Answered

1. What is the complete list of state that needs to sync?
2. For each piece of state, who owns it?
3. What happens when ownership transfers (grab/drop, player joins, player leaves)?
4. How do we handle conflicts (two players grab same object)?
5. What's the latency budget? (How much delay is acceptable?)
6. What's the bandwidth budget? (How big can snapshots be?)
7. How do we test this systematically?

These questions appear in various documents but were never answered definitively. They should be answered **before** the
next line of networking code is written.
