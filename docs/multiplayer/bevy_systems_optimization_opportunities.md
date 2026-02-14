# Bevy Systems Optimization Status for Dedicated Server

This document tracks the status of system-level optimizations for the dedicated server.

## Summary of Optimizations

We have implemented extensive system gating to ensure the dedicated server uses minimal CPU when not in an active
mission.

### 1. Plugin Removal (Complete)

The following plugins were removed entirely from the dedicated server build:

- `UnhaunterThermalPlugin`
- `UnhaunterFogCorePlugin`
- `UnhaunterWalkieCorePlugin`
- `UnhaunterProfilePlugin`

### 2. Gameplay System Gating (Complete)

All major gameplay systems are now gated using `.run_if(in_state(AppState::InGame))`. This stops the Bevy scheduler from
evaluating system bodies during Lobby or MainMenu states.

**Gated Systems include:**

- **Lighting**: `rebuild_lighting_field` and `player_visibility_system`.
- **Ghost AI**: `decay_evidence_clarity_system`, movement, enrage, and visual sync.
- **Player Simulation**: Input clearing, movement, and hand-held gear synchronization.
- **Interactions**: Door, switch, and breaker interaction handlers.
- **Gear Items**: Battery drain, interference, and specific item logic for all gear types.
- **Environmental**: Breaker overload and synchronization logic.

### 3. Transition System Gating (Complete)

Systems used during the map loading and entity initialization phase have been gated to only run when necessary (i.e.,
not in `MainMenu` or `Lobby`).

**Gated Transition Systems:**

- **Hydration**: All stage-based entity hydration systems (Players, Ghosts, NPCs, Tiles).
- **Map Loading**: Conveyor logic and pre-mesh processing.
- **Lighting Setup**: Grid initialization and light pre-baking.

### 4. Ongoing/Active Systems (Ungated)

The following systems are intentionally ungated as they are required for server operations:

- **Networking**: IO, heartbeats, session management, and lobby synchronization.
- **Server Lifecycle**: FPS limiting (essential for throttling) and performance reporting.
- **Lobby Logic**: Map and difficulty selection handlers.

## Next Steps

1. **Performance Validation**: Measure CPU usage of a dedicated server in a full 4-player lobby.
2. **Memory Analysis**: Check if any large resources (like map textures) can be completely skipped in headless mode
   beyond what is currently stubbed.

## Implementation Recommendations

### High Priority Optimizations (Ungated Systems)

The following systems currently run every frame on the dedicated server, even in the lobby. Gating them with
`run_if(in_state(AppState::InGame))` (or appropriate event-based gating) would provide significant CPU savings.

#### 1. Light Core Systems

- `init_light_grid` (PreUpdate)
- `prebake_lighting_on_level_ready` (Update)
- `rebuild_lighting_field` (PostUpdate) **Location**: `crates/unlight-plugin/src/plugin.rs`

#### 2. Ghost systems

- `decay_evidence_clarity_system` (Update)
- `ghost_movement`, `ghost_enrage`, `ghost_fade_out_system` (Update)
- `update_ghost_warning_field` (Update)
- `ghost_visual_sync`, `ghost_influence_visual_sync` (Update) **Location**: `crates/unghost-plugin/src/systems/`

#### 3. Interaction & Engine Systems

- `boardfield_update` (PostUpdate): Currently only gated by event, but only relevant in-game.
- `interaction_event_handler`, `room_state_sync_system` (Update) **Locations**:
  `crates/unengine-plugin/src/boardfield_update.rs`, `crates/uninteraction-plugin/src/systems/mod.rs`

#### 4. Player Systems (Host-side)

- `player_input_clear_system` (PostUpdate)
- `sync_held_gear_position`, `update_held_object_position` (Update)
- `grab_object`, `drop_object`, etc. (Update) **Location**: `crates/unplayer-plugin/src/systems/`

#### 5. Gear Item Systems

- `system_electronic_interference` (Update)
- `system_battery_drain` (Update)
- `system_apply_gear_intent_from_input` (Update)
- Individual gear systems (EMF, Salt, etc.) **Location**: `crates/ungearitems-plugin/src/plugin.rs`

#### 6. Environmental Mechanics

- `fuse_box_overload_system`, `breaker_sync_system` (Update) **Location**:
  `crates/unclassic-mode-plugin/src/environmental_mechanics.rs`

### Medium Priority Optimizations (Map Load Support)

These systems mostly process entities during map load (via `HydrationStage`). While they don't do much in lobby, gating
them ensures they are completely inactive.

- `hydration_simulation_system` (`unrender-plugin`)
- `hydration_player_logic_system` (`unplayer-plugin`)
- `hydration_ghost_logic_system` (`unghost-plugin`)
- `hydration_npc_system` (`unnpc-plugin`)

### Systems That Are Properly Gated

These systems already have appropriate gating:

1. **UnhaunterSummaryCorePlugin**: `update_time` runs only in `FixedUpdate` during `InGame`.
2. **UnhaunterMissionPlugin**: All systems run only during `InGame`.
3. **UnhaunterTruckCorePlugin**: Systems run on `OnEnter/OnExit` `InGame` or during `InGame`.
4. **UnhaunterClassicModeCorePlugin**: `spawn_joined_player` runs during `InGame`.
5. **Net Plugins**: Snapshots and despawning are already gated by `InGame`.

### Systems That Should NOT Be Gated

These systems need to run even in lobby:

1. **Network systems**: `lobby_broadcast_state_system`, `host_liveness_system`, `client_heartbeat_system`,
   `host_status_updater_system`.
2. **Player connection management**: Systems handling player joins/disconnects.
3. **Lobby state synchronization**: Systems maintaining lobby consistency.
4. **FPS Limiter**: Essential for server performance.
5. **Metrics/Performance Reporting**: Useful for monitoring server health.
