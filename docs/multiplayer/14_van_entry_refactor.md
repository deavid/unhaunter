# Phase E: Van Entry Refactor Plan (Document 14)

This document provides a detailed implementation plan for fixing the van/truck entry system for multiplayer.

---

## Problem Summary

**Current Behavior (Broken):**

1. Client walks to van, presses E
2. Client's code runs with `Authority::Client`, so it does NOT set `GameState::Truck` locally
3. Instead, Client sends `NetworkMessage::RequestTruckEntry` to Host
4. Host receives the message and sets `GameState::Truck` on the **Host's** instance
5. Result: Host sees truck UI, Client sees nothing

**Additional Problem:** When either player enters `GameState::Truck`, many game systems stop running (animation, input,
movement) because they have `run_if(in_state(GameState::None))`. This causes the other player to freeze.

**Critical Discovery: State Sync Override**

The `client_apply_snapshots_system` currently syncs **both** `AppState` and `GameState` from Host to Client every frame:

```rust
// Sync AppState
if *server_app_state != *params.states.current_app_state.get() {
    params.states.app_next_state.set(*server_app_state);
}
// Sync GameState
if *server_game_state != *params.states.current_game_state.get() {
    params.states.game_next_state.set(*server_game_state);
}
```

This means if Client enters `GameState::Truck` locally, the next snapshot from Host will force them back to
`GameState::None`. **This sync must be modified.**

---

## Design Goals

1. **Each player independently enters/exits the truck UI** — One player can be in the truck while another explores
2. **Game systems continue running** — The game doesn't "pause" when someone is in the truck
3. **Visual feedback for "in truck" players** — Semi-transparent, frozen animation (like hiding)
4. **Per-player sanity recovery** — Each player recovers sanity independently while in truck
5. **Per-player mission end** — A player can leave the mission while the other continues
6. **`GameState` is NOT synced for Truck** — Each instance manages its own `GameState::Truck` locally
7. **`AppState` sync is conditional** — Only sync for certain transitions (e.g., mission loading), not for Summary

---

## Architecture Overview

### New Component: `InTruck`

**Location:** `crates/untruck-core/src/components/in_truck.rs`

```rust
/// Marker component for a player entity that is currently in the truck/van.
/// This is synced over the network so other players see the visual effect.
#[derive(Component, Debug, Clone, Copy, Default)]
pub struct InTruck;
```

### State Flow

**Local Player Enters Truck:**

1. Player interacts with van → `execute_interaction()` detects `is_van_entry()`
2. Instead of checking `Authority`, always:
   - Insert `InTruck` component on local player entity
   - Insert `Hiding` component (for visual effect)
   - Set `GameState::Truck` locally
3. `InTruck` state is synced to other instances via snapshot

**Remote Player Enters Truck (as seen by local):**

1. Snapshot arrives with player's `is_in_truck: true`
2. Local instance inserts `InTruck` and `Hiding` components on remote player entity
3. Remote player renders as semi-transparent with frozen animation
4. **No** hiding overlay icon spawns (icon is only for local player)

**Player Exits Truck:**

1. Player clicks "Exit Truck" or presses Escape
2. Remove `InTruck` and `Hiding` components
3. Set `GameState::None` locally
4. Synced via snapshot to other instances

---

## Important Clarifications for Implementation

### Data Structure Corrections

The plan uses some simplified names. Here are the actual structures in the codebase:

| Plan Reference                   | Actual Code                                    | Location                          |
| -------------------------------- | ---------------------------------------------- | --------------------------------- |
| `Sanity` component               | `PlayerSprite.sanity: f32`                     | `unplayer-core/src/components.rs` |
| `PlayerState.position: [f32; 4]` | `position: [f32; 3]` + `orientation: [f32; 2]` | `unnet-core/src/messages.rs`      |
| `SANITY_RECOVERY_RATE`           | Does not exist yet                             | Needs to be defined               |

### Shared Resources (Remain Global)

These resources are intentionally shared between players and remain Host-authoritative:

- **`GhostGuess`** — Evidence tracking and ghost type guess (shared journal)
- **`RepellentCraftTracker`** — Global limit on repellent crafts per mission
- **`RoomDB`** — Room states (lights on/off)

### Per-Player Resources/Components

These are per-player and should NOT be globally synced:

- **`GameState`** — `Truck` vs `None` is local to each instance
- **`InTruck` component** — Per-player marker, synced as data (not as state override)
- **Sanity recovery** — Each player recovers independently when they have `InTruck`

---

## Implementation Phases

### Phase E.0: Fix State Synchronization (CRITICAL - DO THIS FIRST)

**File:** `crates/unnet-plugin/src/systems.rs`

The current `client_apply_snapshots_system` blindly syncs `GameState` from Host. This must be changed.

**Current Code (around line 880-890):**

```rust
// Sync GameState
if *server_game_state != *params.states.current_game_state.get() {
    params.states.game_next_state.set(*server_game_state);
}
```

**New Code:**

```rust
// Sync GameState - but NOT Truck state (that's per-player)
// Only sync if Host is in a "global" state that affects everyone
let dominated_by_server = matches!(
    server_game_state,
    GameState::Pause | GameState::NpcHelp
);
let local_in_truck = *params.states.current_game_state.get() == GameState::Truck;

if dominated_by_server {
    // Server-dominated states override local state
    params.states.game_next_state.set(*server_game_state);
} else if !local_in_truck && *server_game_state != *params.states.current_game_state.get() {
    // Only sync if we're not locally in the truck
    // This allows Client to stay in Truck while Host is in None
    params.states.game_next_state.set(*server_game_state);
}
// If local_in_truck is true, we keep our local Truck state
```

**Similarly for AppState (around line 875):**

```rust
// Sync AppState - but allow independent Summary transition
let dominated_by_server_app = matches!(
    server_app_state,
    AppState::Loading | AppState::MainMenu
);
let local_in_summary = *params.states.current_app_state.get() == AppState::Summary;

if dominated_by_server_app {
    params.states.app_next_state.set(*server_app_state);
} else if !local_in_summary && *server_app_state != *params.states.current_app_state.get() {
    params.states.app_next_state.set(*server_app_state);
}
```

---

### Phase E.1: Create `InTruck` Component

**File:** `crates/untruck-core/src/components/in_truck.rs` (new file)

```rust
use bevy::prelude::*;

/// Marker component for a player entity that is currently in the truck/van.
#[derive(Component, Debug, Clone, Copy, Default, Reflect)]
#[reflect(Component)]
pub struct InTruck;
```

**File:** `crates/untruck-core/src/components/mod.rs`

Add `pub mod in_truck;`

**File:** `crates/untruck-core/src/lib.rs`

Ensure `components` module is exported.

---

### Phase E.2: Fix Van Entry Logic

**File:** `crates/uninteraction-plugin/src/systems/interactivestuff.rs`

**Important Design Note:** The `RequestTruckEntry` message is **KEPT** but repurposed as a notification.

Since the Host is the source of truth for snapshots, the Host needs to know when a Client enters the truck so it can add
the `InTruck` component to that Client's entity. The flow is:

1. Client interacts with van → Client sets local `GameState::Truck`
2. Client sends `RequestTruckEntry` to Host (as notification, not a request)
3. Host receives it → Host adds `InTruck` to Client's entity
4. Host's next snapshot includes Client's `is_in_truck: true`
5. Other clients receive snapshot → apply `InTruck` to that player

**Current Code (around line 186-207):**

```rust
if behavior.is_van_entry() {
    if ietype != InteractionExecutionType::ChangeState {
        return false;
    }
    match authority {
        Authority::Host => {
            if let Some(interactive) = interactive {
                let sound_file = interactive.sound_for_moving_into_state(behavior);
                self.sound_events.write(SoundEvent {
                    sound_file,
                    volume: 1.0,
                    position: Some(*item_pos),
                });
            }
            self.game_next_state.set(GameState::Truck);
        }
        Authority::Client => {
            self.net_events.write(NetworkDataEvent {
                message: NetworkMessage::RequestTruckEntry,
            });
        }
    }
    return false;
}
```

**New Code:**

```rust
if behavior.is_van_entry() {
    if ietype != InteractionExecutionType::ChangeState {
        return false;
    }
    // Play sound regardless of authority
    if let Some(interactive) = interactive {
        let sound_file = interactive.sound_for_moving_into_state(behavior);
        self.sound_events.write(SoundEvent {
            sound_file,
            volume: 1.0,
            position: Some(*item_pos),
        });
    }
    // Each instance handles their own truck entry locally
    self.game_next_state.set(GameState::Truck);
    // Note: InTruck component is added by a separate system that watches GameState changes

    // Client must notify Host so Host can update the entity
    if matches!(authority, Authority::Client) {
        self.net_events.write(NetworkDataEvent {
            message: NetworkMessage::RequestTruckEntry,
        });
    }
    return false;
}
```

**Keep:** The `RequestTruckEntry` message is kept but now functions as a notification.

---

### Phase E.3: Add System to Manage `InTruck` Component

**File:** `crates/untruck-plugin/src/systems/in_truck_manager.rs` (new file)

```rust
use bevy::prelude::*;
use unplayer_core::components::{MainPlayer, PlayerSprite, Hiding};
use untruck_core::components::in_truck::InTruck;
use untypes_core::states::GameState;
use unboard_core::components::mapcolor::MapColor;
use bevy::color::palettes::css;

/// System that adds InTruck component when local player enters GameState::Truck
pub fn on_enter_truck(
    mut commands: Commands,
    query: Query<Entity, (With<MainPlayer>, Without<InTruck>)>,
) {
    for entity in query.iter() {
        commands.entity(entity)
            .insert(InTruck)
            .insert(Hiding { hiding_spot: None })
            .insert(MapColor {
                color: css::DARK_GRAY.with_alpha(0.5).into(),
            });
    }
}

/// System that removes InTruck component when local player exits GameState::Truck
pub fn on_exit_truck(
    mut commands: Commands,
    query: Query<Entity, (With<MainPlayer>, With<InTruck>)>,
) {
    for entity in query.iter() {
        commands.entity(entity)
            .remove::<InTruck>()
            .remove::<Hiding>()
            .insert(MapColor {
                color: Color::WHITE.with_alpha(1.0),
            });
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(GameState::Truck), on_enter_truck);
    app.add_systems(OnExit(GameState::Truck), on_exit_truck);
}
```

---

### Phase E.4: Sync `InTruck` State Over Network

**File:** `crates/unnet-core/src/messages.rs`

Add `is_in_truck` to the existing `PlayerState` struct:

```rust
pub struct PlayerState {
    pub id: NetworkId,
    pub position: [f32; 3],      // Existing: x, y, z
    pub orientation: [f32; 2],   // Existing: dx, dy
    pub is_hiding: bool,
    pub is_in_truck: bool,       // NEW
    pub stamina: f32,
    pub is_running: bool,
    pub frame: u16,
}
```

**File:** `crates/unnet-plugin/src/systems.rs`

**In `host_send_snapshots_system`:** Add `is_in_truck` field when building `PlayerState`.

First, update the query to include `Option<&InTruck>`:

```rust
// Find the existing query for players and add InTruck
Query<(..., Option<&InTruck>), With<PlayerSprite>>
```

Then when building PlayerState:

```rust
PlayerState {
    id: *p,
    position: [pos.x, pos.y, pos.z],
    orientation: [dir.dx, dir.dy],
    is_hiding: hiding.is_some(),
    is_in_truck: in_truck.is_some(),  // NEW
    stamina: stamina.current,
    is_running: stamina.running,
    frame,
}
```

**In `client_apply_snapshots_system`:** Handle `is_in_truck` for remote players:

```rust
// After handling is_hiding...
// NOTE: Only apply InTruck visuals to REMOTE players (not MainPlayer)
if main_player.is_none() {
    match (p_state.is_in_truck, in_truck) {
        (true, None) => {
            params.commands.entity(p_entity)
                .insert(InTruck)
                .insert(Hiding { hiding_spot: None })
                .insert(MapColor { color: css::DARK_GRAY.with_alpha(0.5).into() });
        }
        (false, Some(_)) => {
            params.commands.entity(p_entity)
                .remove::<InTruck>()
            .remove::<Hiding>()
            .insert(MapColor { color: Color::WHITE.with_alpha(1.0) });
    }
    _ => {}
}
```

---

### Phase E.5: Fix Hiding Overlay Icon (Only Show for Local Player)

**Problem:** The hiding overlay icon (eye struck through) spawns as a child of the hiding spot entity. We don't want to
show this for remote players who are hiding or in truck.

**File:** `crates/unplayer-plugin/src/systems/hide.rs`

The current `hide_player` system spawns the overlay inside a `With<MainPlayer>` query, so it only runs for the local
player. This is already correct!

However, we need to ensure that when we insert `Hiding` for a remote player (via snapshot sync), we do NOT spawn the
overlay. The current sync code in `client_apply_snapshots_system` just inserts the `Hiding` component without spawning
the overlay child, which is correct.

**Verification:** The overlay is only spawned in `hide_player` which has `With<MainPlayer>` filter. ✓

---

### Phase E.6: Remove `GameState::None` Requirement from Global Systems

These systems currently require `GameState::None` but should run regardless of truck state:

#### 6.1: Animation System

**File:** `crates/unrender-plugin/src/systems/animation.rs`

**Current:**

```rust
app.add_systems(Update, animate_sprite.run_if(in_state(GameState::None)));
```

**New:**

```rust
app.add_systems(Update, animate_sprite.run_if(in_state(AppState::InGame)));
```

**Additionally:** Modify `animate_sprite` to skip entities with `InTruck` component (freeze their animation):

```rust
fn animate_sprite(
    time: Res<Time>,
    mut query: Query<
        (&mut AnimationTimer, Option<&mut Sprite>, Option<&MeshMaterial2d<CustomMaterial1>>),
        Without<InTruck>,  // Skip in-truck players
    >,
    // ...
) {
    // ... existing logic
}
```

#### 6.2: Mouse Aim System

**File:** `crates/unplayer-plugin/src/systems/mouse.rs`

**Current:**

```rust
app.add_systems(Update, mouse_aim_system.run_if(in_state(GameState::None)));
```

**New:** The mouse aim system should check if the MainPlayer has `InTruck` instead:

```rust
fn mouse_aim_system(
    // ... existing params
    q_in_truck: Query<(), (With<MainPlayer>, With<InTruck>)>,
) {
    // Skip if local player is in truck
    if !q_in_truck.is_empty() {
        return;
    }
    // ... existing logic
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, mouse_aim_system.run_if(in_state(AppState::InGame)));
}
```

#### 6.3: Player Input/Movement Chain

**File:** `crates/unplayer-plugin/src/systems/setup.rs`

**Current:**

```rust
.chain()
.run_if(in_state(GameState::None).and(in_state(AppState::InGame))),
```

**New:** Keep the chain but change the condition:

```rust
.chain()
.run_if(in_state(AppState::InGame)),
```

Then add per-system checks for `InTruck` on `MainPlayer` in systems that should stop for local player:

- `keyboard_input_system` — skip if MainPlayer has InTruck
- `player_gear_usage_system` — skip if MainPlayer has InTruck
- `waypoint_creation_system` — skip if MainPlayer has InTruck
- `waypoint_following_system` — skip if MainPlayer has InTruck
- `player_movement_system` — already runs on Host only; needs to skip players with InTruck

#### 6.4: Gear Keyboard

**File:** `crates/ungear-plugin/src/systems.rs`

**Current:**

```rust
.add_systems(Update, keyboard_gear.run_if(in_state(GameState::None)));
```

**New:** Check for InTruck on MainPlayer inside the system.

#### 6.5: Evidence UI Systems

**File:** `crates/untruck-plugin/src/evidence.rs`

These should still run in truck mode since they're part of the gameplay. Change:

```rust
run_if(in_state(GameState::None).and(in_state(AppState::InGame)))
```

To:

```rust
run_if(in_state(AppState::InGame))
```

---

### Phase E.7: Fix Sanity Recovery Per-Player

**File:** `crates/untruck-plugin/src/sanity.rs`

**Current Understanding:**

- Sanity is stored as `PlayerSprite.sanity: f32` (not a separate `Sanity` component)
- The current `update_sanity` function only updates the UI text, it doesn't actually modify sanity
- Sanity modification happens elsewhere (likely in `unplayer-plugin/src/systems/sanityhealth.rs`)

**Action Required:**

1. Find where sanity actually decreases (sanityhealth.rs or similar)
2. Add a new system that INCREASES sanity for players with `InTruck` component
3. This system should run in `AppState::InGame` (not gated by GameState)

**New System (add to untruck-plugin or unplayer-plugin):**

```rust
/// Sanity recovery rate per second while in truck
const SANITY_RECOVERY_RATE: f32 = 2.0;  // Recover 2% per second

fn truck_sanity_recovery(
    time: Res<Time>,
    mut query: Query<&mut PlayerSprite, With<InTruck>>,
) {
    for mut player in query.iter_mut() {
        // Recover sanity while in truck (cap at 100)
        player.sanity = (player.sanity + time.delta_secs() * SANITY_RECOVERY_RATE).min(100.0);
    }
}

// In app_setup:
app.add_systems(Update, truck_sanity_recovery.run_if(in_state(AppState::InGame)));
```

**Note:** The existing `update_sanity` in `sanity.rs` only updates the UI text display. It reads from
`PlayerSprite.sanity` and formats it. This can remain as-is but should run in `AppState::InGame` instead of only
`GameState::Truck` so the UI updates regardless of truck state.

---

### Phase E.8: Fix Truck UI Visibility

**File:** `crates/untruck-plugin/src/systems/truck_ui_systems.rs`

The `show_ui` and `hide_ui` systems are already triggered by `OnEnter(GameState::Truck)` and `OnExit(GameState::Truck)`.
Since each instance has its own `GameState`, this should work correctly.

**Current:**

```rust
app.add_systems(OnEnter(GameState::Truck), show_ui.run_if(is_host));
app.add_systems(OnExit(GameState::Truck), hide_ui.run_if(is_host));
```

**Remove the `is_host` condition** since both Host and Client should see their own truck UI:

```rust
app.add_systems(OnEnter(GameState::Truck), show_ui);
app.add_systems(OnExit(GameState::Truck), hide_ui);
```

Similarly for other truck UI systems that have `.run_if(is_host)` — evaluate each one:

- Tab interactions: Both players should be able to interact with tabs
- Button interactions: Both players should be able to click buttons
- Craft repellent: Both can craft (with global limit)

---

### Phase E.9: Per-Player Mission End

**CRITICAL LIMITATION:** For the initial implementation, the **Host cannot end the mission while other players are still
playing**. This is because:

- Movement systems (`player_movement_system`) are gated by `AppState::InGame`
- Ghost AI systems are gated by `AppState::InGame`
- If Host transitions to `Summary`, these systems stop running on the Host
- Since the Host runs these systems authoritatively for all players, everyone would freeze

**Phase 1 (Simpler) Implementation:**

1. Client clicks "End Mission" → Client despawns, goes to Summary, Host continues
2. Host clicks "End Mission" → **Entire mission ends for everyone** (existing behavior)

**Future Enhancement (Phase 2):** Allow Host to go to Summary while simulation continues:

- Decouple authoritative systems from Host's `AppState`
- Run simulation systems in `AppState::InGame OR AppState::Summary` with `has_active_players` condition
- This is complex and can be done later

**File:** `crates/untruck-plugin/src/systems/truck_ui_systems.rs`

**In `truckui_event_handle` for End Mission button:**

```rust
TruckUIEvent::EndMission => {
    if matches!(authority, Authority::Host) {
        // Host ending = everyone ends (for now)
        ev_mission.write(MissionEvent::End);
    } else {
        // Client ending = just this client leaves
        // Despawn local player
        for (entity, _, _) in q_player.iter() {
            commands.entity(entity).despawn();
        }
        // Notify Host that this player left
        net_events.write(NetworkDataEvent {
            message: NetworkMessage::PlayerLeft { player_id: local_player_id },
        });
        // Transition to Summary locally
        app_next_state.set(AppState::Summary);
    }
}
```

**UI Feedback for Host:** Consider disabling or showing a tooltip on the "End Mission" button when other players are
still active: "Other players still in mission. Ending will end for everyone."

**Network sync:** Add `PlayerLeft` message:

```rust
NetworkMessage::PlayerLeft { player_id: NetworkId }
```

When Host receives `PlayerLeft`, it despawns that player's entity and removes from tracking.

---

### Phase E.10: Repellent Craft Collision Prevention

**Design:** If two players hold the craft button simultaneously, cancel both.

**File:** `crates/untruck-plugin/src/systems/truck_ui_systems.rs`

This is local-only UI, so each player has their own button state. The collision would only occur if:

- Player A holds craft button → progress bar fills
- During that time, Player B also crafts
- Both try to consume from the global `RepellentCraftTracker`

**Solution:** Make `RepellentCraftTracker` authoritative on Host:

1. Client sends `NetworkMessage::CraftRepellent` request
2. Host validates remaining crafts and either:
   - Accepts: Updates tracker, spawns repellent, sends confirmation
   - Rejects: Sends rejection (sound effect + visual)

**For testing:** Add debug key combo (Shift+R) to simulate 5-second button hold.

---

## Summary of Changes

| File                                                   | Change                                                           |
| ------------------------------------------------------ | ---------------------------------------------------------------- |
| `untruck-core/src/components/in_truck.rs`              | NEW: `InTruck` component                                         |
| `uninteraction-plugin/src/systems/interactivestuff.rs` | Remove Authority check, always set GameState locally             |
| `untruck-plugin/src/systems/in_truck_manager.rs`       | NEW: Systems to manage InTruck on state change                   |
| `unnet-core/src/messages.rs`                           | Add `is_in_truck` to `PlayerState`                               |
| `unnet-plugin/src/systems.rs`                          | Sync `InTruck` component via snapshot, fix state sync            |
| `unrender-plugin/src/systems/animation.rs`             | Remove GameState::None requirement, skip InTruck entities        |
| `unplayer-plugin/src/systems/mouse.rs`                 | Check InTruck instead of GameState                               |
| `unplayer-plugin/src/systems/setup.rs`                 | Remove GameState::None from chain, add per-system InTruck checks |
| `ungear-plugin/src/systems.rs`                         | Check InTruck in keyboard_gear                                   |
| `untruck-plugin/src/sanity.rs`                         | Add truck_sanity_recovery system                                 |
| `untruck-plugin/src/systems/truck_ui_systems.rs`       | Remove `is_host` guards, add per-player mission end              |
| Various walkie triggers                                | Update GameState::None checks                                    |

---

## Testing Checklist

- [ ] Host enters truck → Host sees truck UI, Client keeps playing normally
- [ ] Client enters truck → Client sees truck UI, Host keeps playing normally
- [ ] Both in truck → Both see truck UI, can interact independently
- [ ] Host sees Client in truck → Client appears semi-transparent, frozen
- [ ] Client sees Host in truck → Host appears semi-transparent, frozen
- [ ] No hiding icon on remote players in truck
- [ ] Sanity recovers independently for each player while in truck
- [ ] Repellent crafting respects global limit
- [ ] Client ends mission → Client sees summary, Host continues playing
- [ ] Host ends mission → Both go to summary (limitation for now)

---

## Open Questions / Future Work

1. **Player death while partner is in truck** — How should this be handled?
2. ~~**Ghost hunting while player is in truck**~~ — **Resolved:** Truck IS a safe zone (Hiding component makes ghost
   search less precisely). See Q9.
3. **Reconnection** — If a player disconnects while in truck, how to handle?
4. **Spectator mode** — Should a player who ended mission be able to spectate?

---

## Answers to Implementation Questions

This section addresses specific questions raised during implementation review.

### Q1: GameState Syncing Conflict

**Question:** If a Client enters the truck but the Host remains in the house, won't the next snapshot from the Host
force the Client back into `GameState::None`?

**Answer:** Yes, this is exactly the problem. **Phase E.0** addresses this by modifying the state sync logic in
`client_apply_snapshots_system`. The Client should NOT blindly accept `GameState` from the Host when the Client is
locally in `GameState::Truck`. The fix allows the Client to maintain their local truck state.

### Q2: AppState Syncing & Mission End

**Question:** If the Host clicks "End Mission" and transitions to `AppState::Summary`, will the Client be forced too?

**Answer:** This is a critical limitation. See **Phase E.9** for the updated design:

- **Client ends mission:** Client despawns and goes to Summary. Host and other players continue.
- **Host ends mission:** For the initial implementation, this ends the mission for everyone. This is because
  authoritative systems (movement, ghost AI) are gated by `AppState::InGame` and run on the Host. If Host leaves
  `InGame`, the simulation stops for everyone.

**Future Enhancement:** Decouple simulation from Host's `AppState` so Host can go to Summary while simulation continues.
This requires running authoritative systems with a `has_active_players` condition instead of strictly `InGame`.

### Q3: Component & Data Structure Mismatches

**Question:** The plan refers to a `Sanity` component but sanity is actually `PlayerSprite.sanity: f32`.

**Answer:** Correct. The plan has been updated to use `PlayerSprite.sanity` directly. See the corrected **Phase E.7**.

**Question:** `PlayerState` has different fields than what the plan showed.

**Answer:** Correct. The actual structure is:

```rust
pub position: [f32; 3],      // x, y, z
pub orientation: [f32; 2],   // dx, dy
```

The plan has been updated to match. Just add `is_in_truck: bool` to the existing struct.

**Question:** `SANITY_RECOVERY_RATE` doesn't exist.

**Answer:** Define it as a constant in the new `truck_sanity_recovery` system: `const SANITY_RECOVERY_RATE: f32 = 2.0;`
(adjust value based on design intent — 2 sanity/sec means full recovery from 0 in 50 seconds).

### Q4: Evidence/GhostGuess Sharing

**Question:** Should ghost evidence guesses remain shared?

**Answer:** Yes. `GhostGuess` is a shared resource (one journal for the team). This is intentional and should remain
Host-authoritative. It's already synced via the snapshot's `evidences_found`, `evidences_missing`, and
`ghost_type_guess` fields.

### Q5: Network ID & PlayerLeft Message

**Question:** Where should `PlayerLeft` message be defined?

**Answer:** In `crates/unnet-core/src/messages.rs`, add to the `NetworkMessage` enum:

```rust
/// A player has left the mission (ended their game or disconnected)
PlayerLeft { player_id: NetworkId },
```

The Host handles this by despawning that player's entity and removing them from tracking. The Client receives this via
snapshot (the player entity won't be in the players list anymore).

### Q6: Client-to-Host Communication of Truck Entry (Round 2)

**Question:** How does the Host know when a Client enters the truck if the interaction is handled locally?

**Answer:** The `RequestTruckEntry` message is **kept** but repurposed. See updated **Phase E.2**:

1. Client interacts with van → sets local `GameState::Truck`
2. Client sends `RequestTruckEntry` to Host (as notification)
3. Host receives → adds `InTruck` component to Client's entity
4. Snapshot propagates `is_in_truck: true` to all clients

The message is no longer a "request" (Host doesn't approve/deny), it's a notification that the Client has entered.

### Q7: AppState Sync and MapHub

**Question:** If Host goes to `MapHub` while Client is in `InGame` or `Summary`, won't the Client be stranded?

**Answer:** When the Host leaves the mission entirely (going to MapHub), the mission effectively ends for all players.
This is a session-ending event.

The sync logic in Phase E.0 is designed for the scenario where:

- Host is in `Summary` **waiting** for other players to finish
- Clients continue playing in `InGame`

When Host clicks "Return to Map Hub" or "Main Menu" from Summary, that should trigger a `MissionEnd` or `SessionEnd`
message that kicks all clients back to an appropriate state. This can be handled by:

- Adding `MapHub` to the `dominated_by_server_app` list, OR
- Sending an explicit `SessionEnded` message that forces clients to MainMenu

For the initial implementation, when Host leaves Summary, the session ends for everyone.

### Q8: GameState Variants - Hunting

**Question:** What about `Hunting` or other global states? Should `Truck` take priority over a Hunt?

**Answer:** There is **no `Hunting` GameState**. The actual `GameState` enum is:

```rust
pub enum GameState { None, Truck, Pause, NpcHelp }
```

Ghost hunting is tracked via components/resources on the ghost entity (e.g., `ghost.hunting`, `ghost.hunt_target`), not
via `GameState`. The plan's Phase E.0 logic is correct: only `Pause` and `NpcHelp` are "dominating" server states.

If a player is in the truck during a ghost hunt, they're safe (see Q9).

### Q9: InTruck and Ghost AI (Hiding Interaction)

**Question:** Is it intentional that a player in the truck is treated by ghost logic as "Hiding"?

**Answer:** **Yes, this is intentional and beneficial.** The ghost AI in `movement.rs` uses `Hiding` to:

- Increase search radius from 1.0 to 2.0 (ghost searches less precisely)
- Use a randomized target instead of the player's actual position when `hiding || ghost.calm_time_secs > 5.0`

This makes the truck a **safe zone** — players in the truck are effectively hidden from the ghost. This is the desired
behavior for a paranormal investigation game (the van is the safe retreat).

The ghost AI code:

```rust
let search_radius = if hiding { 2.0 } else { 1.0 };
let ppos = if hiding || ghost.calm_time_secs > 5.0 {
    old_target  // Use randomized position, not actual player position
} else {
    *ppos       // Use actual player position
};
```
