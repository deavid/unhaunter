# Multiplayer: Comprehensive Diagnostic & Roadmap

**Date:** February 1, 2026 **Status:** Deep Investigation Complete **Branch:** `dev-deavid`

---

## Executive Summary

After thorough investigation, the multiplayer implementation has **foundational transport working** (TCP, handshake,
basic snapshot sync), but **critical architectural gaps** prevent game logic from working correctly across the network.
The issues reported by the user are **symptoms of a deeper problem**: most game systems were designed for single-player
and are **not authority-aware**.

### Key Problems Identified

1. **Authority Confusion**: Many systems run on both Host and Client without checking authority, causing duplicate or
   conflicting actions.
2. **State Transition Leaks**: `InteractiveStuff::game_next_state` writes to `NextState<GameState>` on **both**
   instances, but only the Host's state should propagate.
3. **Component Mutation Without Authority**: Gear toggling, door interactions, and other mutations happen locally on
   both sides, then get out of sync.
4. **Missing Sync Data**: Flashlight visibility, ghost appearance, and other visual states are computed locally and
   never sent over the network.
5. **Type System Duplication**: `NetworkId` is defined **twice** with incompatible types (`u32` vs `u64`).

---

## Detailed Symptom Analysis

### Symptom 1: Client Opens Truck → Host UI Opens

**What the User Sees:**

- Client presses `E` near the van.
- Host's screen transitions to `GameState::Truck` and shows the truck UI.
- Client stays in `GameState::None`.

**Root Cause:**

```rust
// uninteraction_plugin/src/systems/interactivestuff.rs:100
self.game_next_state.set(GameState::Truck);
```

This is called from `execute_interaction()`, which is invoked by **movement systems that run on both Host and Client**:

- `unplayer-plugin/src/systems/movement.rs:215` (runs on both)
- `unplayer-plugin/src/systems/waypoint.rs:247` (runs on both)

The problem:

1. Client detects player near van and calls `execute_interaction()`.
2. This sets `NextState<GameState>` on **the Client's local Bevy state machine**.
3. However, the snapshot system syncs `GameState` **from Host to Client**, so:
   - Client sees: `NextState` → `Truck` (for 1 frame) → `None` (overwritten by snapshot).
   - Host sees: `None` (no local interaction happened).

**But wait, why does the Host open the truck?**

Looking closer at the snapshot sync:

```rust
// unnet-plugin/src/systems.rs:387-397
if server_state_str != current_state_str {
    let new_state = match server_state_str {
        "None" => GameState::None,
        "Truck" => GameState::Truck,
        ...
    };
    if new_state != *current_game_state.get() {
        interactive_stuff.game_next_state.set(new_state);
    }
}
```

This means the **Client's snapshot receiver** is setting `NextState` on the Client. But the reported bug says "Host
opens truck when Client presses E". This suggests either:

- **Hypothesis A**: The snapshot is being applied to the wrong instance (unlikely based on code structure).
- **Hypothesis B**: There's a race condition where both instances interact with the van simultaneously.
- **Hypothesis C**: The movement system runs on the Client, which sends its `PlayerInput` to the Host, the Host
  simulates the Client player walking to the van, and **the Host-side Client player entity** triggers
  `execute_interaction()` on the Host.

**Hypothesis C is correct.** Looking at the system setup:

```rust
// unplayer-plugin/src/systems/setup.rs:56-65
movement::player_movement_system,
    .run_if(in_state(GameState::None).and(in_state(AppState::InGame)).and(is_host)),
```

The movement system runs **only on the Host**, but it processes **all** `PlayerSprite` entities, including the remote
player (the Client's character as simulated on the Host). When the Host's simulation of the Client player walks near the
van, it calls `execute_interaction()`, which sets the Host's `GameState` to `Truck`.

**Conclusion**: The van interaction system is not checking **which player** triggered the interaction. It just says "a
player is near the van → open truck UI". In multiplayer, there are two players, so **either one** can trigger the state
change **on the Host**.

**Fix Required**: Van interactions (and similar global state transitions) must check if the interacting player is the
**local MainPlayer** before changing state.

---

### Symptom 2: Flashlight Doesn't Work on Client

**What the User Sees:**

- Client presses the flashlight button (or Tab to toggle modes).
- Nothing happens. No light appears.

**Root Cause:** The flashlight toggle system:

```rust
// unplayer-plugin/src/systems/input/mouse_interaction.rs:13-27
pub(crate) fn player_gear_usage_system(
    q_players: Query<(&PlayerGear, &PlayerInput), With<PlayerSprite>>,
    mut q_toggleable: Query<(&mut Toggleable, Option<&Position>)>,
    ...
) {
    for (player_gear, player_input) in q_players.iter() {
        if player_input.use_right_hand && let Some(entity) = player_gear.right_hand {
            if let Ok((mut toggle, pos)) = q_toggleable.get_mut(entity) {
                toggle.is_on = !toggle.is_on;
                ...
            }
        }
    }
}
```

This system is gated by `is_host` (see `setup.rs:36`), so it **only runs on the Host**. The Client never mutates
`Toggleable.is_on` locally. Instead:

1. Client presses the button → `PlayerInput.use_right_hand = true`.
2. This input is sent to the Host via `NetworkMessage::PlayerInput`.
3. The Host's `host_apply_input_system` writes this to the **Client player's `PlayerInput` component** on the Host.
4. The Host's `player_gear_usage_system` reads that input and toggles the flashlight **on the Host**.
5. The snapshot should send the new `Toggleable.is_on` state back to the Client via `GearSyncState`.

**Let's check if `Toggleable` is synced:**

```rust
// unnet-plugin/src/systems.rs:252
query_gear: Query<(&NetworkId, &Position, &Toggleable)>,
```

Yes! Gear sync includes `Toggleable`. So why doesn't the Client see the light?

### The Missing Link: Light Visibility

The flashlight's **visual effect** is computed by:

```rust
// unlight-plugin/src/maplight/systems/gathering.rs:95-151
pub(crate) fn gather_flashlights_system(
    qp: Query<(&Position, &Direction, &PlayerGear)>,
    q_flashlight: Query<(&LightEmitter, &Toggleable)>,
    ...
) {
    for (pos, direction, gear) in qp.iter() {
        // Check left/right hand for flashlights
        if let Some(e) = gear.right_hand {
            if let Ok((fl, toggle)) = q_flashlight.get(e) && toggle.is_on {
                // Add this flashlight to the active list
            }
        }
    }
}
```

This system queries **`PlayerGear`**, which is **only populated for the local `MainPlayer`** on each instance. Let's
verify:

```rust
// unclassic-mode-plugin/src/systems/orchestrator.rs:223-231
if is_main_player {
    ec.insert(MainPlayer)
      .insert(Viewer { id, ..default() })
      .insert(player_gear.clone());
} else {
    // Remote player needs some dummy gear or none for now
    ec.insert(PlayerGear::default());
}
```

**There it is.** Remote players get `PlayerGear::default()`, which has empty hands and inventory. Even if the Host syncs
`Toggleable.is_on` for the gear entity, the **light gathering system can't find the gear** because `PlayerGear` on the
remote player entity is empty.

**Fix Required**: Either:

- **Option A**: Sync `PlayerGear` contents (which gear entities are in which slots) across the network.
- **Option B**: Change the light gathering system to iterate over **all** gear entities with `Toggleable.is_on = true`
  and a `Position` matching a player, regardless of the `PlayerGear` query.

---

### Symptom 3: Host's Flashlight Not Visible on Client

**What the User Sees:**

- Host turns on flashlight.
- Host sees light.
- Client does not see the Host's light.

**Root Cause:** Same as Symptom 2, but in reverse. The Client's `gather_flashlights_system` queries:

```rust
qp: Query<(&Position, &Direction, &PlayerGear)>,
```

On the Client, the **Host player entity** (the remote player) has `PlayerGear::default()` (empty). So the Client never
finds the Host's flashlight to include in the light calculations.

Additionally, looking at the Client's player query:

```rust
// unlight-plugin/src/maplight/systems/gathering.rs:48
qp: Query<(&Position, &Direction, &PlayerGear)>,
```

This will **only match the local `MainPlayer`** on the Client (Player ID 2), not the remote Host player (Player ID 1),
because the remote player has an empty `PlayerGear`.

**Fix Required**: Same as Symptom 2. The light gathering must work independently of `PlayerGear` ownership, or
`PlayerGear` must be synced.

---

### Symptom 4: Door State Not Synced Properly

**What the User Sees:**

- Client clicks on a door.
- Door opens on Host.
- Door stays closed on Client.

**Root Cause:**

Doors are `Behavior` entities with `Interactive` and `Toggleable` components. When a player interacts with a door:

```rust
// unplayer-plugin/src/systems/movement.rs:213-223
if interactive_stuff.execute_interaction(
    entity, &item_pos, Some(interactive), &beh, room_state,
    InteractionExecutionType::ChangeState,
) {
    ev_room.write(RoomChangedEvent::default());
}
```

This happens on **both Host and Client** because the movement system runs everywhere (input gathering happens
everywhere, though physics only runs on Host).

Wait, let's re-check:

```rust
// unplayer-plugin/src/systems/setup.rs:65
.and(is_host),
```

The movement system (including physics and interaction execution) **only runs on the Host**. So when the Client clicks
on a door:

1. Client sends `PlayerInput.interact = true` to Host.
2. Host's `host_apply_input_system` writes this to the Client player's `PlayerInput` on the Host.
3. Host's `player_movement_system` processes the Client player entity, detects it's near an interactive door, and calls
   `execute_interaction()`.
4. This changes the door's `Behavior` (visual state) and emits a `RoomChangedEvent`.
5. The snapshot system sends back the **door's new tile state** via `MapTileState`.

Let's check if doors are synced:

```rust
// unnet-plugin/src/systems.rs:288-300
let map_tiles = query_map_tiles
    .iter()
    .map(|(pos, beh): (&Position, &Behavior)| MapTileState {
        x: pos.x as i32,
        y: pos.y as i32,
        z: pos.z as i32,
        tileset: beh.cfg().tileset.clone(),
        tileuid: beh.cfg().tileuid,
    })
    .collect();
```

Yes! Door states are synced via `map_tiles`. The Client receives this and applies it:

```rust
// unnet-plugin/src/systems.rs:462-493
for t_sync in map_tiles {
    let bpos = BoardPosition { x: t_sync.x as i64, y: t_sync.y as i64, z: t_sync.z as i64 };
    // Find entities at this position
    let entities = &board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
    for &entity in entities {
        if let Ok((pos, beh)) = query_tiles.get(entity)
            && (beh.cfg().tileset != t_sync.tileset || beh.cfg().tileuid != t_sync.tileuid)
        {
            interactive_stuff.execute_interaction(
                entity, pos, None, beh, None,
                InteractionExecutionType::ChangeState,
            );
        }
    }
}
```

**Wait, there's a problem here.** The Client is calling `execute_interaction()` with
`InteractionExecutionType::ChangeState`, which will:

1. Update the visual sprite ✓
2. Update the `Behavior` component ✓
3. Update the `RoomDB` state ✓
4. **Play a sound effect** ✗ (duplicate sound on Client)
5. **Emit a `RoomChangedEvent`** ✗ (duplicate event on Client)

But more critically, the query used on the Client side is:

```rust
query_tiles: Query<(&Position, &Behavior), (Without<PlayerSprite>, Without<GhostTag>, Without<NetworkId>)>,
```

This query **excludes entities with `NetworkId`**. But looking at the orchestrator:

```rust
// unclassic-mode-plugin/src/systems/orchestrator.rs:116-136
// Gear spawning adds NetworkId
commands.entity(gear_entity).insert(untags_core::tags::NetworkId(gear_id_counter));
```

So gear has `NetworkId`, but **doors don't**. Doors are map entities spawned by the Tiled loader, not by the
orchestrator. They likely don't have `NetworkId` components.

**However**, the issue might be different. Let's trace the actual door interaction flow more carefully:

**On Host:**

1. Client player entity (ID 2) has `PlayerInput.interact = true`.
2. Movement system detects player near door, calls `execute_interaction()`.
3. Door's `Behavior` changes (e.g., from "closed" to "open").
4. Snapshot sends `MapTileState { tileuid: <open_door_tileuid> }`.

**On Client:**

1. Snapshot arrives with new `MapTileState`.
2. Client looks up the entity at that board position.
3. Client checks if `beh.cfg().tileuid != t_sync.tileuid`.
4. If different, calls `execute_interaction()` to change the door.

**Potential Issue**: The Client's `execute_interaction()` call uses `interactive_stuff`, which is a `SystemParam` that
includes:

```rust
pub game_next_state: ResMut<'w, NextState<GameState>>,
```

If the door happens to be a van entry, the Client would **again** try to change the game state. But more importantly,
the Client is using `InteractionExecutionType::ChangeState`, which will:

- Change the room state in `RoomDB` ✓ (needed)
- Play sounds ✗ (not needed, Host already played it)
- Update visuals ✓ (needed)

But the **real problem** might be that the snapshot doesn't send **all** map tiles, only ones with `Interactive`:

```rust
query_map_tiles: Query<(&Position, &Behavior), With<Interactive>>,
```

And on the Client side, the query is:

```rust
query_tiles: Query<(&Position, &Behavior), (Without<PlayerSprite>, Without<GhostTag>, Without<NetworkId>)>,
```

These queries don't match! The Host sends tiles with `Interactive`, but the Client searches for tiles **without**
`NetworkId`. If some interactive tiles have `NetworkId` and others don't, there will be a mismatch.

**But wait, why would door states not sync?** Let me look at the board field access:

```rust
// unnet-plugin/src/systems.rs:470-476
let rel_x = bpos.x - board_topo.origin.0 as i64;
let rel_y = bpos.y - board_topo.origin.1 as i64;
let rel_z = bpos.z - board_topo.origin.2 as i64;

if rel_x >= 0 && rel_y >= 0 && rel_z >= 0
    && rel_x < board_field.0.shape()[0] as i64
    && rel_y < board_field.0.shape()[1] as i64
    && rel_z < board_field.0.shape()[2] as i64
{
    let entities = &board_field.0[[rel_x as usize, rel_y as usize, rel_z as usize]];
```

This lookup is **correct** if the Client and Host have the same map loaded with the same `board_topo.origin`. But if the
Client joins mid-session and hasn't loaded the map yet, or if there's a mismatch in the origin, the lookup will fail.

**More likely issue**: The Client's `execute_interaction()` expects to find the **same door entity** at that position,
but the query might not match it. Or the Client hasn't received a `RoomChangedEvent` to update the door sprites.

Actually, looking back at the code:

```rust
// unnet-plugin/src/systems.rs:485
interactive_stuff.execute_interaction(
    entity, pos, None, beh, None,
    InteractionExecutionType::ChangeState,
);
```

The Client passes `None` for `interactive` and `room_state`, which means:

- No sound will be played (good!).
- Room state won't be updated (bad if this door is room-connected).

But then separately:

```rust
// unnet-plugin/src/systems.rs:455-458
let mut room_changed = false;
for r_sync in rooms {
    // Update room states from snapshot
}
if room_changed {
    ev_room.write(RoomChangedEvent::default());
}
```

So room states **are** synced separately. The door visual should update.

**Hypothesis**: The Client **does** update the door visual, but the user doesn't see it because:

- The Client's camera is in a different position.
- The door update happens but the room lighting doesn't update (rooms are turned off/on by doors).
- There's a timing issue where the snapshot arrives before the map finishes loading on the Client.

**More Evidence Needed**: The user says "door opens on Host but Client does not see it". This suggests the Client's
visual update is **not** happening. Possible reasons:

1. The `board_field` lookup fails (Client's board isn't populated).
2. The `query_tiles` doesn't match the door entity (wrong query filters).
3. The `execute_interaction()` call doesn't change the visual because of a missing component.

**Most Likely Culprit**: The Client's `query_tiles` excludes entities `Without<NetworkId>`, but map tiles (doors,
lights, etc.) spawned by the Tiled loader **might have** `NetworkId` added to them later by some other system, causing
the query to fail.

**Fix Required**: Debug logging to confirm whether:

- The Client receives the `MapTileState` for the door.
- The Client finds the entity at that position.
- The Client's `execute_interaction()` is called.
- The door's `Behavior` component updates on the Client.

---

## Architectural Root Causes

### 1. Authority Ambiguity

**Problem**: Most systems don't explicitly check if they're the authority before mutating state.

**Examples**:

- `InteractiveStuff::execute_interaction()` sets `NextState<GameState>` regardless of authority.
- Gear usage systems are gated by `is_host`, but this is inconsistent.
- Room state changes can happen on both sides if `RoomChangedEvent` is emitted by non-authoritative systems.

**Impact**: State desynchronization, race conditions, duplicate events.

**Solution**: Introduce a consistent **authority model**:

- **Authoritative Systems**: Only run on Host. Write to Components/Resources.
- **Predictive Systems**: Run on both. Read Components. Write to local transient state only.
- **Replication Systems**: Run on Client. Read network messages. Write to Components (overwriting predictions).

### 2. Component Replication Gaps

**Problem**: Not all gameplay state is included in snapshots.

**Missing from Snapshots**:

- `PlayerGear` (which items are in which slots).
- `LightEmitter` (light power, color, enabled state - partially covered by `Toggleable`).
- `Flashlight` (mode: Off/Low/High, heat level, battery).
- `Electronic::glitch_intensity` (EMF reader display state).
- `Stamina` (affects run speed and animations).
- `GhostSprite` (warp, hunt warning, calm time, hit/miss deltas) - **partially synced**.
- `Hiding` (player hiding in closet).
- Transient events: sounds, particle spawns, one-shot animations.

**Impact**: Visual and functional differences between Host and Client.

**Solution**: Expand `NetworkMessage::Snapshot` to include:

- `PlayerGear` contents.
- Gear-specific state (flashlight mode, battery level, thermometer reading).
- Visual modifiers (ghost warp, player hiding).
- One-shot events (sounds, particles) in a separate event buffer.

### 3. Query Filter Mismatches

**Problem**: The Host queries entities with certain components to build snapshots, but the Client queries with
**different filters** to apply them.

**Example**:

- Host: `Query<(&Position, &Behavior), With<Interactive>>`
- Client: `Query<(&Position, &Behavior), (Without<PlayerSprite>, Without<GhostTag>, Without<NetworkId>)>`

If an entity has both `Interactive` and `NetworkId`, the Host includes it in the snapshot, but the Client's query
**excludes** it, so it never gets updated.

**Impact**: Some entities sync, others don't, with no clear pattern.

**Solution**: Use **matching queries** or ensure the Client's query is a superset of the Host's query.

### 4. Type System Conflicts

**Problem**: `NetworkId` is defined in **two places** with different types:

- `untags-core::tags::NetworkId(u32)`
- `unnet-core::NetworkId(u64)`

**Impact**:

- Gear entities use `u32` `NetworkId`.
- Players use `u64` IDs in network messages.
- Type casts and potential overflow issues.

**Solution**: Unify to a single `NetworkId(u64)` in `unnet-core`. Refactor all usages to this canonical type.

### 5. Event Duplication

**Problem**: Events like `RoomChangedEvent`, `SoundEvent`, and entity spawns can be triggered on both Host and Client.

**Example**: Door opens → Host plays sound → Snapshot triggers `execute_interaction()` on Client → Client also plays
sound.

**Impact**: Duplicate sounds, duplicate particle effects, wasted CPU.

**Solution**:

- **Host**: Emit events normally.
- **Snapshot**: Include a `TransientEvents` list (sounds, particles) that the Client should replay.
- **Client**: Suppress local event emission from replicated actions.

---

## Revised Roadmap

### Phase 0: Foundation Cleanup (1-2 days)

**Goal**: Resolve technical debt and type conflicts before building on top.

#### 0.1: Unify `NetworkId`

- **Action**: Remove `NetworkId` from `untags-core/src/tags.rs`.
- **Action**: Refactor all imports to use `unnet_core::NetworkId(u64)`.
- **Action**: Update `GearSyncState` and `PlayerState` in `messages.rs` to use the `NetworkId` type.
- **Validation**: `cargo clippy` passes with no type errors.

#### 0.2: Core Crate Cleanup

- **Action**: Move `evidence_decay` system from `unghost-core` to `unghost-plugin`.
- **Action**: Remove all `pub use` re-exports from core crates.
- **Validation**: Grep for `pub use` in `crates/un*-core` returns zero matches.

#### 0.3: Add Debug Logging

- **Action**: Add `debug!()` calls in:
  - `client_apply_snapshots_system` (when receiving map tiles).
  - `execute_interaction` (when changing state).
  - `player_gear_usage_system` (when toggling gear).
- **Goal**: Enable runtime diagnostics to trace bugs.

---

### Phase 1: Authority Enforcement (2-3 days)

**Goal**: Ensure only the Host mutates authoritative state.

#### 1.1: Guard `InteractiveStuff`

- **Problem**: `execute_interaction()` is called on both Host and Client.
- **Action**: Add an `authority: Authority` parameter to `execute_interaction()`.

  ```rust
  pub enum Authority {
      Host,      // Can mutate state
      Client,    // Read-only, visual updates only
  }
  ```

- **Action**: When `authority == Authority::Client`:
  - Update visuals (sprites, materials).
  - **Do NOT** emit sounds (Host will send sound events).
  - **Do NOT** change `NextState<GameState>`.
  - **Do NOT** emit `RoomChangedEvent`.
- **Validation**: Client can receive door state changes without playing duplicate sounds.

#### 1.2: Separate Van Entry Logic

- **Problem**: Van entry triggers `GameState::Truck` on the wrong instance.
- **Action**: Create a dedicated `request_truck_entry()` function that:
  - Host: Calls `game_next_state.set(GameState::Truck)`.
  - Client: Sends `NetworkMessage::RequestTruckEntry` to Host.
- **Action**: Host receives request, validates (player is near van), then changes state.
- **Validation**: Client pressing E near van → Host transitions to Truck → Client receives snapshot → Client transitions
  to Truck.

#### 1.3: Audit All `is_host` Gates

- **Action**: Grep for `run_if(is_host)` and verify:
  - Movement systems: ✓ Correct (Host-only physics).
  - Input systems: ✗ Should run on **both** (to gather input).
  - Gear usage systems: ✓ Correct (Host-only mutation).
  - UI systems: ✓ Correct (Host-only truck UI).
- **Action**: Fix any systems that gather input but are incorrectly gated.
- **Validation**: Client can press keys → input is sent → Host processes it.

---

### Phase 2: Component Replication Expansion (3-4 days)

**Goal**: Sync all missing gameplay state.

#### 2.1: Sync `PlayerGear`

- **Action**: Add `PlayerGearState` to `NetworkMessage::Snapshot`:

  ```rust
  pub struct PlayerGearState {
      pub player_id: u64,
      pub left_hand: Option<u64>,  // NetworkId of gear
      pub right_hand: Option<u64>,
      pub inventory: Vec<u64>,
      pub held_item: Option<u64>,
  }
  ```

- **Action**: Host populates this from `Query<(&PlayerSprite, &PlayerGear)>`.
- **Action**: Client applies this by:
  1. Looking up gear entities by `NetworkId`.
  2. Updating the remote player's `PlayerGear` component.
- **Validation**: Host toggles flashlight → Client sees Host's flashlight light up.

#### 2.2: Sync Flashlight Mode

- **Action**: Expand `GearSyncState`:

  ```rust
  pub struct GearSyncState {
      pub id: u32,
      pub position: [f32; 3],
      pub is_on: bool,
      pub mode: Option<String>,  // "Off", "Low", "High", etc.
      pub battery: f32,
  }
  ```

- **Action**: Host encodes `Flashlight::status` as a string.
- **Action**: Client decodes and updates `Flashlight` component.
- **Validation**: Host cycles flashlight modes → Client sees correct brightness.

#### 2.3: Sync Ghost Visual State

- **Action**: Expand `GhostState`:

  ```rust
  pub struct GhostState {
      pub position: [f32; 3],
      pub warp: f32,
      pub hunt_warning_active: bool,
      pub hunt_warning_intensity: f32,
      pub calm_time_secs: f32,
      pub repellent_hits_delta: f32,
      pub repellent_misses_delta: f32,
  }
  ```

- **Action**: Host copies from `GhostSprite` component.
- **Action**: Client writes to `GhostSprite` component (instead of computing locally).
- **Validation**: Host sees ghost warp → Client sees identical warp effect.

#### 2.4: Sync Player Hiding

- **Action**: Add `is_hiding: bool` to `PlayerState`.
- **Action**: Host checks for `Hiding` component.
- **Action**: Client inserts/removes `Hiding` component based on snapshot.
- **Validation**: Host hides in closet → Client sees Host player sprite hidden.

---

### Phase 3: Event Replication (2-3 days)

**Goal**: Sync one-shot events (sounds, particles) that don't have persistent state.

#### 3.1: Add Transient Event Buffer

- **Action**: Extend `NetworkMessage::Snapshot`:

  ```rust
  pub struct Snapshot {
      // ...existing fields
      pub events: Vec<TransientEvent>,
  }

  pub enum TransientEvent {
      PlaySound { file: String, position: Option<[f32; 3]>, volume: f32 },
      SpawnParticle { particle_type: String, position: [f32; 3] },
  }
  ```

- **Action**: Host accumulates events during the frame.
- **Action**: Client replays events from snapshot.

#### 3.2: Suppress Client-Side Event Emission

- **Action**: Modify `execute_interaction()` to **not** emit sounds when `authority == Authority::Client`.
- **Action**: Host emits sound via `SoundEvent`, which gets captured and sent in `Snapshot.events`.
- **Validation**: Door opens → only one sound plays on Client (from snapshot, not local).

#### 3.3: Sync Item Pickup/Drop Sounds

- **Action**: `grab_object()` and `drop_object()` systems emit sounds.
- **Action**: These systems are gated by `is_host`, so sounds naturally only play on Host.
- **Action**: Capture these sounds in the snapshot event buffer.
- **Validation**: Host drops item → Client hears the "clunk" sound.

---

### Phase 4: Light Visibility Fix (1-2 days)

**Goal**: Make player-held lights visible across the network.

#### 4.1: Option A - Sync `PlayerGear` (Preferred)

- Already covered in Phase 2.1.
- Once `PlayerGear` is synced, the `gather_flashlights_system` will naturally find the remote player's flashlight.

#### 4.2: Option B - Query All Lights

- **Action**: Change `gather_flashlights_system` to:

  ```rust
  q_all_lights: Query<(&LightEmitter, &Toggleable, &Position)>,
  ```

- **Action**: Iterate over **all** lights, not just those in `PlayerGear`.
- **Issue**: This might include deployed lights and other light sources, mixing them with handheld ones.
- **Decision**: Use Option A (sync `PlayerGear`).

---

### Phase 5: Door & Interaction Debugging (1 day)

**Goal**: Resolve the specific "door not syncing" issue.

#### 5.1: Add Diagnostic Logging

- **Action**: Add `info!()` in `client_apply_snapshots_system` when processing `MapTileState`.

  ```rust
  info!("Client: Applying map tile at ({}, {}, {}): {} -> {}",
        t_sync.x, t_sync.y, t_sync.z, beh.cfg().tileuid, t_sync.tileuid);
  ```

- **Action**: Add `info!()` in `execute_interaction` when changing door state.
- **Goal**: Determine if the Client is receiving and processing door updates.

#### 5.2: Fix Query Mismatch

- **Action**: Ensure `query_tiles` on the Client uses the **same filters** as `query_map_tiles` on the Host:

  ```rust
  query_tiles: Query<(&Position, &Behavior), With<Interactive>>,
  ```

- **Validation**: Door updates are received and applied on Client.

#### 5.3: Test Room-Connected Doors

- **Action**: Test a door that's connected to a room (e.g., turns lights on/off).
- **Validation**: Client sees both door state change **and** room lighting change.

---

### Phase 6: Stamina & Animation Sync (1 day)

**Goal**: Make running animations consistent.

#### 6.1: Sync Stamina Level

- **Action**: Add `stamina: f32` to `PlayerState`.
- **Action**: Host includes `Stamina::level` in snapshot.
- **Action**: Client writes to `Stamina` component (or just uses it for animation, not simulation).
- **Validation**: Host player runs → Client sees correct running animation speed.

---

### Phase 7: UI & Investigation State (2-3 days)

**Goal**: Sync investigation progress and mission end conditions.

#### 7.1: Sync Evidence Found

- **Action**: Add `evidence: Vec<Evidence>` to snapshot.
- **Action**: Host includes `CurrentEvidenceReadings` state.
- **Action**: Client updates local `CurrentEvidenceReadings`.
- **Validation**: Host finds evidence → Client sees evidence on UI.

#### 7.2: Sync Mission End

- **Action**: Add `mission_result: Option<MissionResult>` to snapshot.
- **Action**: Host determines when mission ends (all evidence found, time limit, death).
- **Action**: Client transitions to summary screen when `mission_result.is_some()`.
- **Validation**: Host completes mission → Client sees results screen.

---

### Phase 8: Reconnection & Session Persistence (2-3 days)

**Goal**: Allow clients to reconnect after a disconnect and continue their session exactly where they left off.

#### 8.1: Ghosting & Entity Retention

- **Problem**: When a client disconnects, the Host currently treats it as a permanent leave, which can lead to item loss
  if the player was holding gear.
- **Action**: On Host: Detect disconnects and mark the `PlayerSprite` as "Inactive" instead of despawning immediately.
- **Action**: Keep the player's gear and position in the world so they are exactly where they were upon return.
- **Validation**: Killing the client process leaves the representative "ghost" player in the world on the Host.

#### 8.2: Connection Re-association

- **Problem**: The Host currently hardcodes `NetworkId(2)`. This prevents a client from re-joining their own slot if the
  Host still thinks the old connection is alive (or just assigns a new ID).
- **Action**: Update the handshake so the Client can include its previous `NetworkId` in the `Hello` message.
- **Action**: Host validates the ID and re-links the new TCP stream to the existing "Inactive" player entity.

#### 8.3: Full State Reconciliation (The "Join-in-Progress" Fix)

- **Problem**: A re-joining (or late-joining) client starts with a blank world and might miss the current ghost
  evidence, toggled light states, or open doors.
- **Action**: Implement a "Full Sync" flag in the snapshot or a dedicated message sent immediately after `Welcome`.
- **Action**: Ensure the client overwrites all local state (including `GhostGuess`, `SummaryData`, and all `Interactive`
  states) to match the Host immediately.
- **Validation**: Disconnect -> Reconnect -> Client is holding the same flashlight in the same room with the same
  evidence discovered.

---

## Testing Strategy

### Unit Tests

- [ ] `NetworkId` serialization/deserialization.
- [ ] `Authority` parameter in `execute_interaction()`.
- [ ] `PlayerGearState` encoding/decoding.

### Integration Tests

- [ ] Host-Client handshake completes.
- [ ] Player movement syncs (Host moves → Client sees movement).
- [ ] Ghost position syncs (Host spawns ghost → Client sees ghost).
- [ ] Door state syncs (Host opens door → Client sees open door).
- [ ] Flashlight syncs (Host toggles → Client sees light).
- [ ] Item pickup syncs (Host picks up item → Client sees item disappear).

### Manual Tests

- [ ] Two instances run simultaneously (Host + Client).
- [ ] Both players walk around together.
- [ ] Both players can use flashlights independently.
- [ ] Doors open/close correctly for both.
- [ ] Van entry works (only one player needs to enter to open truck UI? Or both?).
- [ ] Ghost chases both players.
- [ ] Mission completes when either player finishes objectives.

---

## Risk Assessment

| Risk                                       | Likelihood | Impact   | Mitigation                                       |
| ------------------------------------------ | ---------- | -------- | ------------------------------------------------ |
| Authority conflicts cause state divergence | **High**   | **High** | Phase 1 (Authority Enforcement) addresses this.  |
| Snapshot bandwidth too high                | Low        | Medium   | Phase 8 (Tuning) includes compression.           |
| Query mismatches cause missing sync        | **Medium** | **High** | Phase 5 (Debugging) will identify mismatches.    |
| Late join causes desyncs                   | Medium     | Medium   | Phase 8 (Late Join) sends full state snapshot.   |
| Transient event buffer fills up            | Low        | Low      | Limit buffer size, drop old events.              |
| `PlayerGear` sync breaks inventory UI      | Medium     | High     | Test inventory cycling with synced `PlayerGear`. |

---

## Next Steps

1. **Confirm User Priorities**: Which symptoms are most critical?
   - Truck entry bug?
   - Flashlight visibility?
   - Door sync?
2. **Start with Phase 0**: Clean up technical debt (NetworkId unification).
3. **Implement Phase 1.1**: Authority-aware `execute_interaction()`.
4. **Test Door Sync**: Validate that Phase 1.1 fixes door state sync.
5. **Move to Phase 2.1**: Sync `PlayerGear` to fix flashlight visibility.

**Recommendation**: Start with **Phase 0.1 (NetworkId unification)** since it's a foundational issue that will cause
type errors later. Then move to **Phase 1.1 (Authority Guard)** to fix the truck entry bug, which is the most disruptive
user-facing issue.
