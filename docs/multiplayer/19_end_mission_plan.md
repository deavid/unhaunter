# End mission and spectate plan

## Goals

- End mission is global and only available when all active players are in the truck.
- Death is not end mission. Death moves a player into spectate.
- Spectators can free-fly through walls, cannot interact, and are invisible to alive players.
- Everyone receives mission summary and score when the global end condition is met.
- Host disconnect forces client into a blocking pause screen that requires quit.

## Desired rules (clear behavioral contract)

- **Truck Authority:** The "End Mission" button is only interactable when all active (non-spectating) players are
  physically inside the truck.
- **Spectate vs Death:** Death inserts a `PlayerSpectating` component. This player is removed from the "active players"
  count for truck gating.
- **Automatic End:** If every single player is in the `PlayerSpectating` state (all dead), the mission ends immediately
  and globally.
- **Financial Authority:** All persistent profile changes (bank increases, insurance losses) must be calculated by the
  host and synced via `MissionSummary` to avoid client-side cheating or desyncs.

## Architectural Deep Dive

### 1. Data Layer Changes

- **`unplayer-core`**: Add marker component `PlayerSpectating`.
- **`unnet-core`**:
  - Add `is_spectating` to `PlayerState`.
  - Add `can_end_mission` (bool) to `SnapshotMsg` so the host can drive the truck button's enabled state.
  - Add `RequestEndMission` to `NetworkMessage` for clients to poke the host.

### 2. State & System Transitions

#### Death Flow (`unplayer-plugin/sanityhealth.rs`)

- Modify `handle_player_death`:
  - **Remove:** `next_app_state.set(AppState::Summary)`.
  - **Add:** `commands.entity(player).insert(PlayerSpectating)`.
  - Result: The player remains in the map but enters the Spectator state.

#### Spectator Capabilities (`unplayer-plugin/movement.rs`)

- **Movements:** If `PlayerSpectating`, skip collision detection (`colhand.delta`).
- **Interactions:** If `PlayerSpectating`, skip all `E` key and gear usage systems.
- **Visibility:** Use `styling.rs` to set alpha to `0.0` for remote players (invisible) and `0.5` for the local player
  (ghostly).

#### Global End Condition (`unmission-plugin/lib.rs`)

- New Host-authoritative system `evaluate_mission_end`:
  - Calculate `active_players` (Connected + !Spectating).
  - If `active_players.is_empty()`, trigger `MissionEvent::End` (Auto-fail).
  - If `RequestEndMission` received AND `all(active_players) in InTruck`, trigger `MissionEvent::End`.

### 3. Host Disconnect Logic

- **`unnet-plugin/systems.rs`**: Detect TCP stream closure. If in-game, transition to `GameState::Pause` and set a
  `HostDisconnected` flag.
- **`unengine-plugin/pause_ui.rs`**:
  - If `HostDisconnected` is set, change the UI to a "Critical Error" style.
  - Disable `ESC` to resume.
  - Only allow `Q` (MissionSelect).

---

## Risks and Pitfalls (Detailed)

1. **The "Disconnected Ghost" Problem:** If a player crashes while outside the truck, the host must not wait for them.
   The `active_players` count must strictly ignore `PlayerDisconnected` entities.
2. **Double-Death Stats:** If a client dies and then the host ends the mission 1 second later, we must ensure the
   `total_deaths` statistic is only incremented once. Death logic should be moved to a single "Event-to-Persist" flow on
   the host.
3. **Ghost Interaction:** Spectators shouldn't be targeted by the ghost or trigger sanity drains. `sanityhealth.rs`
   systems must be gated to `Without<PlayerSpectating>`.
4. **Summary Data Lag:** Clients might switch to `AppState::Summary` before the `MissionSummary` packet arrives. We must
   add a "Waiting for Host Summary..." loading state or gate the state transition on the data's arrival.

## Testing Checklist

- [ ] Client dies: remains in game, walks through walls, truck button still works for others.
- [ ] Host dies: remains in game, client still sees ghost, mission continues.
- [ ] No one in truck: "End Mission" button is greyed out.
- [ ] All in truck: "End Mission" lights up; pressing it ends for everyone.
- [ ] Host hits Alt+F4: Client UI immediately changes to "Host Disconnected" pause menu.
