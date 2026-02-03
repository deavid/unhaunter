# Fresh Investigation: Multiplayer Network Issues (Document 13)

This document supersedes the previous roadmap (document 12). After completing all 8 phases outlined there, most original
symptoms persist. This fresh investigation identifies the **root causes** and proposes a new, more targeted plan.

---

## Part 1: Symptom Re-Statement (With Current Test Results)

### Symptom 1: Van Entry Opens Host's Truck UI

**Observed Behavior:** When the _Client_ player walks near the van and triggers entry, the _Host's_ screen transitions
to `GameState::Truck` and displays the truck UI.

**Expected Behavior:** Only the Client's screen should transition; each player's truck UI is independent.

**Status After Phase 8:** STILL BROKEN

### Symptom 2: Client's Flashlight Doesn't Work

**Observed Behavior:** The Client cannot use their flashlight. No light appears when toggled on.

**Expected Behavior:** Client should be able to turn on their flashlight and see light locally.

**Status After Phase 8:** UNTESTABLE — Gear never spawns on the client

**New Finding:** Client's initial gear (spawned during game start) doesn't exist on the Client. The `PlayerGear`
component references entities that don't exist locally.

### Symptom 3: Host's Flashlight Not Visible to Client (Partially Fixed)

**Observed Behavior (Before):** The Host's flashlight provided no illumination on the Client's screen.

**Observed Behavior (Now):**

- Regular flashlight IS now visible, but the **light direction points downward** instead of matching the Host's facing
  direction.
- **UV Torch, Red Torch, and Night Vision lights** are NOT visible at all on the Client.

**Expected Behavior:** All light types should be visible on the Client with correct direction/position.

**Status After Phase 8:** PARTIALLY FIXED — regular flashlight visible but wrong direction; other lights invisible

### Symptom 4: Door State Not Synced

**Observed Behavior:** When the Host opens/closes a door, the Client doesn't see the change (and vice versa).

**Expected Behavior:** Door state changes should sync bidirectionally.

**Status After Phase 8:** STILL BROKEN

---

## Part 2: Root Cause Analysis

### Root Cause A: Gear Entities Only Exist on Host

**Location:**
[crates/unclassic-mode-plugin/src/systems/orchestrator.rs](../../crates/unclassic-mode-plugin/src/systems/orchestrator.rs)

**Problem:** During game initialization, gear is only spawned for the `MainPlayer`:

```rust
// Line ~108-140 in orchestrator.rs
if is_main_player {
    // Spawn gear entities
    for kind in player_loadout.gear.iter() {
        let gear_entity = gear_registry.spawn(&mut commands, *kind);
        // ... assigns to PlayerGear
    }
} else {
    // Remote players get empty PlayerGear
    commands.entity(player_entity).insert(PlayerGear::default());
}
```

**Impact:**

- Client's `PlayerGear.left_hand`, `right_hand`, and `inventory` are all `None`.
- The `gather_flashlights_system` in `unlight-plugin` queries `PlayerGear` and iterates through gear entities. Since
  Client has no gear entities, it finds no flashlights for itself.
- When snapshots arrive with `PlayerGearState` containing `NetworkId` references, the `net_to_entity` lookup fails
  because those gear entities were never spawned on the Client.

### Root Cause B: `spawn_remote_gear()` Called But Not Effective

**Location:** [crates/unnet-plugin/src/systems.rs](../../crates/unnet-plugin/src/systems.rs) (line ~827, ~1107)

**Problem:** The function `spawn_remote_gear()` exists and IS called when processing gear sync:

```rust
for g_sync in gear {
    let g_entity = if let Some(e) = net_to_entity.get(&g_sync.id) {
        *e
    } else {
        spawn_remote_gear(&mut params, g_sync)  // <-- Called here!
    };
    // ... sync is_on, position, etc.
}
```

However, the `spawn_remote_gear()` implementation only spawns a basic entity:

```rust
fn spawn_remote_gear(params: &mut ClientSnapshotParams, g_sync: &GearSyncState) -> Entity {
    let entity = params
        .gear_registry
        .spawn(&mut params.commands, g_sync.kind);
    params.commands.entity(entity).insert(g_sync.id);
    entity
}
```

**Investigation Needed:** Is `spawn_remote_gear()` actually being called? Possible issues:

1. The gear might already be in `net_to_entity` from a previous frame (stale mapping).
2. The `gear` vector in snapshots might be empty.
3. There may be an ordering issue where `PlayerGearState` is processed before gear entities are spawned.

### Root Cause C: Light Direction Not Synced

**Location:**
[crates/unlight-plugin/src/maplight/systems/gathering.rs](../../crates/unlight-plugin/src/maplight/systems/gathering.rs)

**Problem:** The `gather_flashlights_system` uses the PLAYER's `Direction` component for flashlight direction:

```rust
for (pos, direction, gear) in qp.iter() {  // direction comes from player
    // ... later:
    let mut fldir = *direction;  // Uses player's direction
    flashlights.push(FlashlightData {
        pos: *pos,
        dir: fldir,  // <-- Player direction, not gear-specific
        // ...
    });
}
```

For the **local player**, this works because their `Direction` component is updated via input.

For **remote players**, the `Direction` component IS synced via snapshot:

```rust
// In client_apply_snapshots_system
dir.dx = f32::cos(orientation);
dir.dy = f32::sin(orientation);
```

But wait — this should work. The `orientation` field is `p_state.position[3]`. Let's verify that orientation is being
sent correctly from the Host and applied to remote player's `Direction` component.

**Hypothesis:** Either:

1. The `orientation` value in `PlayerState` is not being populated correctly on the Host, OR
2. The `Direction` component update on the Client isn't being applied to remote players (only to the local player?), OR
3. There's a timing issue where light gathering runs before direction sync completes.

### Root Cause D: UV/Red/NV Lights Not Visible

**Location:** [crates/unnet-core/src/messages.rs](../../crates/unnet-core/src/messages.rs) — `GearSyncState`

**Problem:** The `GearSyncState` does NOT include `LightEmitter` data:

```rust
pub struct GearSyncState {
    pub id: NetworkId,
    pub kind: GearKind,
    pub position: [f32; 3],
    pub is_on: bool,
    pub details: GearDetails,  // Only has FlashlightStatus, Sage, RepellentFlask
    pub battery: f32,
}
```

When `spawn_remote_gear()` calls `gear_registry.spawn()`, the registry DOES create entities with `LightEmitter`
components (see `registration.rs` — UV Torch, Red Torch, etc. all have `LightEmitter` inserted).

**Hypothesis:** The lights might actually be created correctly, but:

1. The `LightType` filtering in the rendering/gathering code might exclude them, OR
2. There's an issue with how the `power` field is synced (it might be 0), OR
3. The `Toggleable.is_on` state isn't being applied correctly.

**Verification:** Add debug logging to confirm:

- Whether gear entities with `LightEmitter` components are being spawned for remote gear
- What `LightType` and `power` values those components have
- Whether `Toggleable.is_on` is `true` for those entities

### Root Cause E: Door Sync Architecture is Flawed

**Current Flow:**

1. Player presses interact → `ExecuteInteractionEvent` fires locally
2. `interaction_event_handler` processes the event with `Authority::Host` or `Authority::Client`
3. For doors, `execute_interaction()` changes the tile state on the Host only

**Problem:** There's no `NetworkMessage` for door interactions. The Host changes door state, but there's no mechanism
to:

1. Send door interaction requests from Client to Host
2. Broadcast door state changes from Host to all Clients

**Current Mitigation (Partial):** The snapshot includes `map_tiles: Vec<MapTileState>` which syncs tile state. However:

1. `MapTileState` only contains tiles that have changed
2. The client processes `MapTileState` by firing `ExecuteInteractionEvent` with `force_tuid`, but with
   `Authority::Client`, which may not actually change the state

### Root Cause F: Aim Direction Not Synced from Client to Host

**Problem:** The `mouse_aim_system` (in `unplayer-plugin/src/systems/mouse.rs`) correctly updates the local player's
`Direction` component based on mouse position. However, this direction is never sent to the Host.

**The Feedback Loop of Doom:**

1.  **Client:** `mouse_aim_system` updates `Direction` for the local player.
2.  **Host:** The remote player entity (representing the Client) has a default/stale `Direction` component because
    `mouse_aim_system` only runs for `MainPlayer`.
3.  **Host Snapshot:** `host_send_snapshots_system` packages the Host's version of the Client's direction (`atan2`) into
    the snapshot.
4.  **Client Snapshot Apply:** `client_apply_snapshots_system` receives the snapshot and applies the Host's stale
    direction to the Client's local `Direction` component, **overwriting** the local mouse-controlled direction.

**Impact:**

- The Client cannot aim their flashlight; it keeps snapping back to a default direction or points "down" (constant).
- The Host sees the Client's flashlight pointing in a constant direction.
- The Host's aiming works because they are the authority on their own `Direction` and the snapshot they send is the one
  they generated.

### Root Cause G: Mouse Interaction Isolation Issues

**Observed Behavior:** The user reports that mouse pointing "seems to affect a bit to the client too" and there's "no
good isolation".

**Potential Problem:** Some mouse-related resources or systems might be shared or incorrectly gated.

1.  **`MouseVisibility`:** This resource controls whether `mouse_aim_system` runs. It is updated in
    `unengine-plugin/src/hide_mouse.rs`. While it should be local, if any code accidentally uses it to gate network
    messages or global state, it could cause issues.
2.  **`player_gear_usage_system`:** (in `unplayer-plugin/src/systems/input/mouse_interaction.rs`) This system iterates
    over **all** players with `PlayerSprite` and `PlayerInput`. On a Client, this includes the remote Host player. If
    the remote Host player's `PlayerInput` component somehow gets populated with values (it shouldn't, as snapshots
    don't carry `PlayerInput`), then the Client would try to toggle the Host's gear locally.

---

## Part 3: New Implementation Plan

### Phase A: Fix Gear Spawning on Client (Priority: Critical)

**Status:** COMPLETED. Client now has starter gear and can see all light types.

### Phase B: Fix Aim/Light Direction Sync (Priority: High)

**Goal:** Ensure Client's aiming is sent to Host and correctly reflected in snapshots.

#### B.1: Update `PlayerInput` to include `aim_direction`

**File:** `crates/unplayer-core/src/components.rs`

Add `aim_direction: Vec2` to `PlayerInput`.

#### B.2: Send `aim_direction` in Network Messages

**File:** `crates/unnet-core/src/messages.rs`

Update `NetworkMessage::PlayerInput` to include `aim_direction: [f32; 2]`.

#### B.3: Populate `aim_direction` on Client

**File:** `crates/unplayer-plugin/src/systems/mouse.rs`

In `mouse_aim_system`, update `PlayerInput.aim_direction` as well as the `Direction` component.

#### B.4: Apply `aim_direction` on Host

**File:** `crates/unnet-plugin/src/systems.rs`

In `host_apply_input_system`, when receiving `PlayerInput`, update the `Direction` component of the remote player entity
using the received `aim_direction`.

#### B.5: Verify Orientation Encoding/Decoding

Ensure `atan2(y, x)` on Host and `cos/sin` on Client use a consistent coordinate system.

### Phase C: Fix Other Light Types (UV/Red/NV) (Priority: High)

**Goal:** All light types should be visible on the Client when held by Host.

#### C.1: Verify LightEmitter Components Exist

After `spawn_remote_gear()`, query the spawned entity for `LightEmitter`:

```rust
let entity = spawn_remote_gear(&mut params, g_sync);
if let Ok(light) = params.query_light.get(entity) {  // Need to add this query
    debug!("Remote gear {} has LightEmitter: power={}, type={:?}", g_sync.id, light.power, light.light_type);
} else {
    debug!("Remote gear {} has NO LightEmitter", g_sync.id);
}
```

#### C.2: Check Light Gathering for Remote Gear

The `gather_flashlights_system` gathers lights from:

1. Deployed gear (placed on ground)
2. Player-held gear (via `PlayerGear` component)

For player-held gear, it checks `gear.left_hand`, `gear.right_hand`, `gear.inventory`. If these are `None` or invalid,
no lights will be gathered.

**Dependency:** This requires Phase A to be complete first.

### Phase D: Fix Door Sync (Priority: Medium)

**Goal:** Door state changes should sync bidirectionally.

#### D.1: Understand Current Map Tile Sync

The snapshot includes `map_tiles: Vec<MapTileState>`. On the Host side:

```rust
// In host_send_snapshots_system
let map_tiles: Vec<MapTileState> = if is_full_sync {
    // Collect all changed tiles
} else {
    Vec::new()  // Only on full sync?
};
```

**Verify:** Is `map_tiles` only populated during full sync? If so, incremental door changes won't sync.

#### D.2: Track Changed Tiles Incrementally

Create a system that detects tile state changes and marks them for sync:

```rust
#[derive(Resource, Default)]
pub struct ChangedTiles(pub Vec<MapTileState>);
```

In `interaction_event_handler`, when a door changes state:

```rust
if authority == Authority::Host {
    changed_tiles.0.push(MapTileState {
        x: bpos.x as i32,
        y: bpos.y as i32,
        z: bpos.z as i32,
        tileset: new_behavior.cfg().tileset.clone(),
        tileuid: new_behavior.cfg().tileuid,
    });
}
```

In `host_send_snapshots_system`, drain `ChangedTiles` into snapshot.

#### D.3: Client → Host Door Interaction

Currently, the Client fires `ExecuteInteractionEvent` locally. This should instead:

1. Send a network message to Host requesting the interaction
2. Host validates and executes
3. Host includes the changed tile in the next snapshot
4. Client applies the snapshot

**New Message:**

```rust
InteractionRequest {
    player_id: NetworkId,
    tile_position: [i32; 3],
    interaction_type: InteractionExecutionType,
}
```

### Phase E: Fix Van Entry (Priority: Medium)

**Goal:** Van entry should only affect the player who triggered it.

#### E.1: Investigate Current Flow

The current implementation in `execute_interaction()`:

```rust
match authority {
    Authority::Host => {
        self.game_next_state.set(GameState::Truck);  // Affects Host!
    }
    Authority::Client => {
        self.net_events.write(NetworkMessage::RequestTruckEntry);  // Sends to Host
    }
}
```

On Host, when receiving `RequestTruckEntry`:

```rust
if near_van {
    game_next_state.set(GameState::Truck);  // Also affects Host!
}
```

**Problem:** `GameState::Truck` is a global state. When Host sets it, Host's UI changes to truck.

**Question:** Is the intent that:

1. Both players should transition to truck UI when any player enters? (Current behavior)
2. Only the entering player transitions? (Separate UI states needed) <- this one is the correct one.

If (2), this requires significant architectural changes — potentially per-player `GameState` or a different mechanism.

**For now:** Investigate and create a full plan for this as docs/multiplayer/14_van_entry_refactor.md

---

## Part 4: Testing Protocol

### Test 1: Gear Existence

1. Start Host and Client
2. On Client, open debug console and check `PlayerGear` component values
3. Verify `left_hand`, `right_hand`, `inventory` contain valid entity IDs
4. Verify those entities exist and have expected components

### Test 2: Light Direction

1. Host turns on flashlight
2. Host rotates to face different directions
3. Client observes light direction changes
4. Client should see light pointing the same direction as Host is facing

### Test 3: Light Types

1. Host picks up UV Torch, turns it on
2. Client should see purple light
3. Repeat for Red Torch, Night Vision

### Test 4: Door Sync

1. Host opens a closed door
2. Client should see door open
3. Client opens a different closed door
4. Host should see door open

### Test 5: Van Entry

1. Client walks to van and enters
2. Observe which screen(s) show truck UI
3. Document actual vs expected behavior

---

## Part 5: Implementation Order

1. **Phase A** (Gear Spawning) — MUST be done first; blocks B and C
2. **Phase B** (Light Direction) — Can proceed after A
3. **Phase C** (Light Types) — Can proceed after A
4. **Phase D** (Door Sync) — Independent of A/B/C
5. **Phase E** (Van Entry) — Independent; may require design decision

---

## Summary of Key Findings

| Issue                        | Root Cause                                                   | Fix Location                                                   | Priority |
| ---------------------------- | ------------------------------------------------------------ | -------------------------------------------------------------- | -------- |
| Client gear doesn't exist    | Orchestrator only spawns for MainPlayer                      | `spawn_remote_gear()` not working OR gear list empty           | Critical |
| Light points wrong direction | Orientation encoding/decoding mismatch                       | `host_send_snapshots_system` + `client_apply_snapshots_system` | High     |
| UV/Red/NV lights invisible   | Depends on gear spawning; may also need `LightEmitter` sync  | Same as gear spawning                                          | High     |
| Doors don't sync             | No incremental tile sync; no client→host interaction request | Add `ChangedTiles` resource + `InteractionRequest` message     | Medium   |
| Van entry affects both       | `GameState::Truck` is global state                           | Design decision needed                                         | Medium   |

---

## Next Steps for Worker AI

1. **First:** Add comprehensive debug logging to `client_apply_snapshots_system` and `host_send_snapshots_system`:
   - Log gear list contents
   - Log when `spawn_remote_gear()` is called
   - Log `PlayerGear` state after sync
   - Log orientation values

2. **Second:** User tests with debug logging enabled to capture actual runtime values

3. **Third:** Based on logs, implement the appropriate fix from Phase A

4. **Fourth:** Proceed to Phases B-E based on results
