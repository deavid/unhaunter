# Deep Dive: Mouse Clicking Waypoints Not Working on Client

## 1. Problem Description

In multiplayer matches, when playing as a client, clicking on the ground or interactive objects to create waypoints for
movement/interaction does not function as expected. The player character fails to follow the clicked path or interact
with the clicked objects.

**Observed behavior**: The client player's character does not move at all in response to mouse clicks. Waypoint creation
log messages DO appear, but the character remains stationary.

## 2. Confirmed Facts (Verified in Code)

### 2.1 System Architecture

- **`waypoint_creation_system`** and **`waypoint_following_system`** are in
  [crates/unplayer-plugin/src/systems/waypoint.rs](crates/unplayer-plugin/src/systems/waypoint.rs) and run on both Host
  and Client. They are gated by `With<MainPlayer>`.
- **`player_movement_system`** in
  [crates/unplayer-plugin/src/systems/movement.rs](crates/unplayer-plugin/src/systems/movement.rs) is gated by `is_host`
  — it only runs on the Host. The client NEVER locally simulates movement.
- **`client_send_input_system`** in [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs) reads
  `PlayerInput` from the `MainPlayer` entity and sends it to the Host. The Host applies it via
  `host_apply_input_system`.
- **`client_apply_snapshots_system`** receives position updates from the Host and applies them to entities.

### 2.2 The Movement Pipeline

The intended pipeline for client mouse-click movement:

1. Client clicks → `waypoint_creation_system` creates waypoint entities on the `MainPlayer` entity's `WaypointQueue`.
2. `waypoint_following_system` reads `WaypointQueue`, computes direction to next waypoint, writes to
   `PlayerInput.movement`.
3. `client_send_input_system` reads `PlayerInput.movement` from `MainPlayer` and sends `NetworkMessage::PlayerInput` to
   the Host.
4. Host's `host_apply_input_system` (in `PreUpdate`) applies the received movement to the remote player's `PlayerInput`.
5. Host's `player_movement_system` (in `Update`) reads `PlayerInput.movement` and actually moves the `Position`.
6. Host's `host_send_snapshots_system` includes the updated `Position` in the next `Snapshot`.
7. Client's `client_apply_snapshots_system` applies the new `Position` to the entity.

### 2.3 System Scheduling

**unplayer-plugin** registers this `.chain()` group in `Update`:

```
keyboard_input_system → gear_usage → walk_target_indicator → mouse_scroll → mouse_over →
mouse_out → waypoint_creation → remote_player_waypoint → waypoint_following →
waypoint_queue_cleanup → player_interaction → player_movement (host only) → stairs (host only)
```

**unnet-plugin** registers this SEPARATE `.chain()` group in `Update`:

```
host_send_snapshots → client_request_grab → client_send_input → client_apply_snapshots →
client_connection_monitor
```

**Critical**: These two `.chain()` groups are in the same `Update` schedule but are **NOT ordered relative to each
other**. Bevy may run them in any order or even interleave them.

Additionally, `player_input_clear_system` runs in `PostUpdate` and clears transient fields on `PlayerInput`.

### 2.4 Entity Spawning (Two Independent Sources)

**Source A — `classic_mode_orchestrator`** in
[crates/unclassic-mode-plugin/src/systems/orchestrator.rs](crates/unclassic-mode-plugin/src/systems/orchestrator.rs):

- Triggered by `MapEntitiesReadyEvent` (runs in `Update`).
- On `NetMode::Join`, spawns TWO player entities: `NetworkId(1)` (host) and `NetworkId(2)` (client).
- The client's own entity (`id == 2`) gets the `MainPlayer` component.
- Spawns gear only for non-`Join` mode (`if !matches!(net_mode, Join { .. })`).
- Also spawns the ghost, haunted objects, etc.

**Source B — `spawn_remote_player`** in [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs):

- Called by `client_apply_snapshots_system` when a snapshot references a `NetworkId` not yet in `net_to_entity`.
- Spawns a player entity with `PlayerSprite`, `NetworkId`, `PlayerInput`, `WaypointQueue`, `PlayerGear`, etc.
- Does **NOT** insert `MainPlayer`, `PlayerInputMapping`, `Viewer`, or `SpatialListener`.
- Does **NOT** check whether an entity with the same `NetworkId` already exists from the orchestrator.

**`net_to_entity` rebuilding**: Each frame, `client_apply_snapshots_system` rebuilds the map via:

```rust
let mut net_to_entity: HashMap<NetworkId, Entity> = params
    .query_net_entities.iter().map(|(e, id)| (*id, e)).collect();
```

If two entities have the same `NetworkId`, the HashMap keeps whichever iteration order produces last. The "losing"
entity becomes invisible to the networking system.

### 2.5 InTruck Sync Gap

`client_apply_snapshots_system` has an explicit guard:

```rust
if main_player.is_none() {
    // ... update InTruck ...
}
```

This means the `InTruck` component is **NEVER** updated by the network for the `MainPlayer` entity. If it's incorrectly
set, `waypoint_creation_system` would return early silently.

### 2.6 Log Evidence

Client logs show during gameplay:

```
DEBUG unplayer_plugin::systems::waypoint: Created 1 waypoints for pathfinding
DEBUG unplayer_plugin::systems::waypoint: Created 1 waypoints for pathfinding
DEBUG unplayer_plugin::systems::waypoint: Created 1 waypoints for pathfinding
```

These appear every ~300ms. **Important caveat**: This log message comes from `create_pathfinding_waypoints()`, which is
called by both `waypoint_creation_system` (main player clicks) AND `remote_player_waypoint_system` (visualizing remote
player paths). The `remote_player_waypoint_system` fires on `Changed<PlayerInput>` for entities `Without<MainPlayer>`.
These logs may well be from the HOST's movement being visualized on the client, **not** the client's own clicks. We
cannot distinguish from these logs alone.

## 3. Fixes Attempted and Results

### 3.1 PlayerInput::clear() Race Condition Fix — FAILED

**Hypothesis**: `player_input_clear_system` runs in `PostUpdate` and was clearing ALL of `PlayerInput` (including
`movement`, `run`, `aim_direction`, `target_position`). Because the two `.chain()` groups in `Update` have no ordering
guarantee, `client_send_input_system` could read zeroed-out movement before `waypoint_following_system` wrote to it.

**Change**: Modified `PlayerInput::clear()` to only clear transient one-shot events (`interact`, `grab`, `drop`,
`use_right_hand`, `use_left_hand`, `inventory_cycle`, `inventory_swap`). Persistent state (`movement`, `run`,
`aim_direction`, `target_position`) is no longer reset. Also changed `keyboard_input_system` to always write
`player_input.movement = movement` (including zero when no keys pressed).

**Files modified**:

- [crates/unplayer-core/src/components.rs](crates/unplayer-core/src/components.rs) — `PlayerInput::clear()`
- [crates/unplayer-plugin/src/systems/input/keyboard.rs](crates/unplayer-plugin/src/systems/input/keyboard.rs) —
  `keyboard_input_system`

**Result**: No change. Character still does not move on client. **This disproves the theory that the system ordering
race condition is the sole cause.** The fix may still be correct for correctness reasons (avoiding stale data), but it
is not sufficient.

### 3.2 target_position Sync in PlayerState — NO EFFECT ON MOVEMENT

**Change**: Added `target_position: Option<[f32; 2]>` to `PlayerState` network message. Host reads
`PlayerInput.target_position` and includes it in snapshots. Client applies received `target_position` to remote players'
`PlayerInput`.

**Files modified**:

- [crates/unnet-core/src/messages.rs](crates/unnet-core/src/messages.rs) — `PlayerState` struct
- [crates/unnet-plugin/src/systems.rs](crates/unnet-plugin/src/systems.rs) — Host and Client snapshot handling

**Result**: No effect on the movement bug. This was for visual waypoint display on remote players, not related to the
core movement issue.

### 3.3 remote_player_waypoint_system — NO EFFECT ON MOVEMENT

**Change**: Added a new system that watches for `Changed<PlayerInput>` on remote players (`Without<MainPlayer>`) and
creates visual waypoints using the existing pathfinding logic.

**Files modified**:

- [crates/unplayer-plugin/src/systems/waypoint.rs](crates/unplayer-plugin/src/systems/waypoint.rs) — added
  `remote_player_waypoint_system`
- [crates/unplayer-plugin/src/systems/setup.rs](crates/unplayer-plugin/src/systems/setup.rs) — registered it

**Result**: No effect on the core bug. This was a feature addition, not a fix.

## 4. Theories (Unverified)

### Theory A: Double Entity Problem (HIGH CONFIDENCE — NOT YET CONFIRMED)

The most likely cause. On the Client, there are potentially **two entities** with `NetworkId(2)`:

- **Entity A**: Spawned by `client_apply_snapshots_system` → `spawn_remote_player`. Has `NetworkId(2)` but NO
  `MainPlayer`.
- **Entity B**: Spawned by `classic_mode_orchestrator`. Has `NetworkId(2)` AND `MainPlayer`.

The `net_to_entity` HashMap can only hold one. If Entity A wins:

- Snapshot position updates go to Entity A (no `MainPlayer`, no camera, no input systems).
- Entity B (has `MainPlayer`, camera follows it) stays frozen at spawn position.
- `waypoint_following_system` sets `PlayerInput.movement` on Entity B.
- `client_send_input_system` reads from Entity B and sends to Host.
- Host applies movement and moves NetworkId(2), sends new position back.
- Client applies position to Entity A (the one net_to_entity points to).
- **Result**: Entity B (visible to player) never moves. Entity A (invisible) moves.

**Key question**: Does `client_apply_snapshots_system` actually call `spawn_remote_player` for the client's own ID? This
depends on timing. If the first snapshot arrives before `classic_mode_orchestrator` runs (before the map loads), then
`net_to_entity` won't find NetworkId(2) and WILL spawn a duplicate. If the orchestrator runs first, `net_to_entity`
would find the existing entity and skip spawning.

**How to verify**: Add a log in `spawn_remote_player` that prints the ID being spawned, and check client logs for
`"Spawning remote player NetworkId(2)"`. If it appears, the double entity problem is confirmed.

### Theory B: Stale InTruck Component (MEDIUM CONFIDENCE — NOT YET CONFIRMED)

If the `MainPlayer` entity has `InTruck` stuck on it, both `waypoint_creation_system` and `keyboard_input_system` would
return early silently (they both check `q_in_truck.is_empty()`). The `client_apply_snapshots_system` intentionally skips
InTruck sync for `MainPlayer` entities, so a stale value would persist forever.

**How to verify**: Add a log at the early return in `waypoint_creation_system` where `q_in_truck` is checked.

### Theory C: Movement is Sent but Position Not Applied to MainPlayer (MEDIUM CONFIDENCE)

Even without a double entity, there could be a simpler problem: `client_apply_snapshots_system` might successfully apply
position updates to the correct entity, but the camera or rendering might not follow. Or the `MainPlayer` entity might
not be the one receiving position updates.

The code applies position like this:

```rust
if let Ok((..., main_player, ...)) = params.query_players.get_mut(p_entity) {
    pos.x = p_state.position[0];
    ...
}
```

This DOES update position for the MainPlayer entity (no guard). So if `p_entity` is the MainPlayer entity, position
should be applied. The question is whether `net_to_entity` points to the right entity.

### Theory D: Keyboard Movement Also Broken (UNKNOWN — NOT TESTED)

We have not definitively confirmed whether KEYBOARD movement (WASD) works on the client. If it also doesn't work, the
bug is in the fundamental input→network→movement pipeline, not in the waypoint system specifically. If keyboard DOES
work, the bug is specifically in how waypoints set `PlayerInput.movement`.

**How to verify**: Test WASD movement on the client.

## 5. Key Code References

| System                          | File                                                | Schedule                | Runs On       |
| ------------------------------- | --------------------------------------------------- | ----------------------- | ------------- |
| `classic_mode_orchestrator`     | `unclassic-mode-plugin/src/systems/orchestrator.rs` | `Update` (on event)     | Both          |
| `keyboard_input_system`         | `unplayer-plugin/src/systems/input/keyboard.rs`     | `Update` (chained)      | Both          |
| `waypoint_creation_system`      | `unplayer-plugin/src/systems/waypoint.rs`           | `Update` (chained)      | Both          |
| `waypoint_following_system`     | `unplayer-plugin/src/systems/waypoint.rs`           | `Update` (chained)      | Both          |
| `player_movement_system`        | `unplayer-plugin/src/systems/movement.rs`           | `Update` (chained)      | **Host only** |
| `player_input_clear_system`     | `unplayer-plugin/src/systems/input/keyboard.rs`     | **PostUpdate**          | Both          |
| `host_apply_input_system`       | `unnet-plugin/src/systems.rs`                       | **PreUpdate** (chained) | Host only     |
| `client_send_input_system`      | `unnet-plugin/src/systems.rs`                       | `Update` (chained)      | Client only   |
| `client_apply_snapshots_system` | `unnet-plugin/src/systems.rs`                       | `Update` (chained)      | Client only   |
| `host_send_snapshots_system`    | `unnet-plugin/src/systems.rs`                       | `Update` (chained)      | Host only     |

## 6. Recommended Next Steps (Priority Order)

1. **Verify Theory D**: Test WASD keyboard movement on the client. If it also fails, the problem is not
   waypoint-specific.
2. **Verify Theory A**: Add debug logging to `spawn_remote_player` and `classic_mode_orchestrator` to check for
   duplicate entities. Log the Entity ID and NetworkId. On the client, check if `"Spawning remote player NetworkId(2)"`
   appears in the logs.
3. **Verify Theory B**: Add a log at the `q_in_truck.is_empty()` check in `waypoint_creation_system` to see if it's
   silently returning early.
4. **Add pipeline tracing**: Add `debug!` logs at each step of the pipeline:
   - `waypoint_following_system`: log when it sets `player_input.movement` to a non-zero value.
   - `client_send_input_system`: log the movement values being sent.
   - `host_apply_input_system`: log the movement values received and applied.
   - `player_movement_system`: log when it moves a non-MainPlayer entity.
5. **Fix the Double Entity problem** (if confirmed): Either make `classic_mode_orchestrator` skip player spawning on
   `Join` mode, or make `spawn_remote_player` check for and adopt existing `MainPlayer` entities with matching IDs.

## 7. Changes Currently in Place

The following changes from session work are still in the codebase (some may be correct fixes that are just not
sufficient alone):

- `PlayerInput::clear()` only clears one-shot events, not persistent movement/run/aim fields.
- `keyboard_input_system` always writes `player_input.movement`, even when zero.
- `PlayerState` network message includes `target_position`.
- `remote_player_waypoint_system` exists and is registered (visual feature, not a fix).
- `PlayerState` network message includes `health` and `sanity` fields (unrelated fix, working).
- `GearDetails::SpiritBox` variant added and synced (unrelated fix, working).
- `GhostState` includes `hunt_target` (unrelated fix, working).
- Unused file `crates/unplayer-plugin/src/systems/waypoint_append.rs` exists (should be deleted).
