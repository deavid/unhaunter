# Bevy Systems Optimization Opportunities for Dedicated Server

This document identifies systems that could be additionally gated with `AppState::InGame` to reduce CPU usage in the
lobby.

## Current Analysis

### Systems That Should NOT Run on Dedicated Server (REMOVED)

The following plugins have been removed from dedicated server builds:

1. **UnhaunterThermalPlugin** - Thermal simulation is client-side only
2. **UnhaunterFogCorePlugin** - Miasma/fog simulation is client-side only
3. **UnhaunterWalkieCorePlugin** - Walkie-talkie functionality is client-side only
4. **UnhaunterProfilePlugin** - Server should not have player profiles

### Systems Running Every Frame Without Proper Gating

#### 1. Light Core Systems

**File**: `crates/unlight-plugin/src/plugin.rs`

- `init_light_grid` (PreUpdate) - **Should be gated**: Only needed during missions
- `prebake_lighting_on_level_ready` (Update) - **Should be gated**: Only needed during level load
- `rebuild_lighting_field` (PostUpdate) - **Should be gated**: Only needed during missions
- `player_visibility_system` (PostUpdate) - **Should be gated**: Only needed during missions

**Recommendation**: Add `.run_if(in_state(AppState::InGame))` to lighting systems

#### 2. Ghost Evidence Decay System

**File**: `crates/unghost-plugin/src/systems/evidence_decay.rs`

- `decay_evidence_clarity_system` (Update) - **Should be gated**: Only needed when ghost is active

**Recommendation**: Add `.run_if(in_state(AppState::InGame))`

#### 3. Ghost Hydration System

**File**: `crates/unghost-plugin/src/systems/hydration.rs`

- `hydration_ghost_logic_system` (Update) - **Should be gated**: Only needed when ghost exists

**Recommendation**: Add `.run_if(in_state(AppState::InGame))`

#### 4. Ghost AI Visual Sync Systems

**File**: `crates/unghost-plugin/src/systems/ghost_ai/mod.rs`

- `ghost_visual_sync` (Update) - **Should be gated**: Visual sync not needed in headless mode
- `ghost_influence_visual_sync` (Update) - **Should be gated**: Visual sync not needed in headless mode

**Recommendation**: These should either be gated with `InGame` or removed entirely from dedicated server builds

#### 5. Network Systems (Partial)

**File**: `crates/unnet-plugin/src/systems/setup.rs`

- `headless_summary_reset_system` (Update) - **Should be gated**: Only needed during/after missions
- `lobby_broadcast_state_system` (Update) - **Keep as-is**: Needed in lobby
- `host_liveness_system` (Update) - **Keep as-is**: Needed in lobby

**Recommendation**: Gate `headless_summary_reset_system` with `InGame` or `Summary` state

#### 5. Gear Item Systems

**File**: `crates/ungearitems-plugin/src/components/repellentflask.rs`

- `update_repellentflask` (Update) - **Should be gated**: Only needed during missions

**Recommendation**: Add `.run_if(in_state(AppState::InGame))`

### Systems That Are Properly Gated

These systems already have appropriate gating:

1. **UnhaunterSummaryCorePlugin**: `update_time` runs only in `FixedUpdate` during `InGame`
2. **UnhaunterMissionPlugin**: `handle_mission_events` and `evaluate_mission_end` run only during `InGame`
3. **UnhaunterTruckCorePlugin**: Systems run on `OnEnter/OnExit` `InGame`
4. **UnhaunterClassicModeCorePlugin**: Systems run during `InGame` or on specific events
5. **Some gear item systems**: Like `repellent_update` already gated with `InGame`

### Systems That Should NOT Be Gated

These systems need to run even in lobby:

1. **Network systems**: `lobby_broadcast_state_system`, `host_liveness_system`, `client_heartbeat_system`,
   `host_status_updater_system`
2. **Player connection management**: Systems handling player joins/disconnects
3. **Lobby state synchronization**: Systems maintaining lobby consistency

## Implementation Recommendations

### High Priority Optimizations

1. **Thermal Systems** - Add `run_if(in_state(AppState::InGame))` to all thermal update systems
2. **Ghost Systems** - Add `run_if(in_state(AppState::InGame))` to evidence decay and hydration systems
3. **Walkie-Talkie Systems** - Gate all walkie systems with `InGame` state
4. **Gear Item Systems** - Gate repellent flask and other mission-specific gear systems

### Medium Priority Optimizations

1. **Ghost Visual Sync** - Either gate with `InGame` or remove from dedicated server entirely
2. **Network Summary System** - Gate `headless_summary_reset_system` appropriately

### Code Changes Required

#### UnhaunterThermalPlugin

```rust
// Before
app.add_systems(Update, temperature_update)
    .add_systems(Update, (init_thermal_grid_allocation, init_thermal_grid_content));

// After
app.add_systems(Update, temperature_update.run_if(in_state(AppState::InGame)))
    .add_systems(Update, (
        init_thermal_grid_allocation.run_if(in_state(AppState::InGame)),
        init_thermal_grid_content.after(init_thermal_grid_allocation).run_if(in_state(AppState::InGame))
    ));
```

#### Ghost Systems

```rust
// Evidence Decay
app.add_systems(Update, decay_evidence_clarity_system.run_if(in_state(AppState::InGame)));

// Ghost Hydration
app.add_systems(Update, hydration_ghost_logic_system.run_if(in_state(AppState::InGame)));

// Ghost AI Visual Sync (consider removing entirely for dedicated server)
app.add_systems(Update, (ghost_visual_sync, ghost_influence_visual_sync).run_if(in_state(AppState::InGame)));
```

#### Walkie-Talkie Systems

```rust
// Example for walkie systems
app.add_systems(Update, (
    focus_ring_showcase_system,
    update_focus_rings,
    update_walkie_stats,
    // ... other walkie systems
).run_if(in_state(AppState::InGame)));
```

## Expected CPU Savings

By implementing these changes, we should see significant CPU reductions in lobby:

### From Plugin Removal (Already Done):

1. **Thermal Simulation**: ~15-25% CPU reduction (thermal grid updates are expensive)
2. **Fog Simulation**: ~8-12% CPU reduction (miasma grid updates)
3. **Walkie-Talkie Systems**: ~5-10% CPU reduction (stat tracking and event processing)
4. **Profile Management**: ~1-2% CPU reduction (profile persistence overhead)

**Savings from Plugin Removal**: ~29-49% CPU reduction in lobby scenarios

### From Additional System Gating (Still To Do):

1. **Lighting Systems**: ~10-15% CPU reduction (light grid and visibility calculations)
2. **Ghost Systems**: ~8-12% CPU reduction (evidence decay and hydration logic)
3. **Gear Systems**: ~3-5% CPU reduction (mission-specific gear logic)

**Additional Potential Savings**: ~21-32% CPU reduction in lobby scenarios

**Total Expected Savings**: ~50-81% CPU reduction in lobby scenarios

## Validation Approach

1. **Before Changes**: Measure CPU usage in lobby with current code
2. **After Changes**: Measure CPU usage in lobby with gated systems
3. **Verify Functionality**: Ensure all systems still work correctly during missions
4. **Stress Test**: Test with multiple players joining/leaving lobby

## Risk Assessment

**Low Risk**:

- Thermal system gating (thermal only matters during missions)
- Gear system gating (gear only used during missions)
- Most walkie-talkie gating (walkie-talkie mostly mission-specific)

**Medium Risk**:

- Ghost system gating (need to ensure ghost state is preserved correctly)
- Network system changes (need to ensure lobby functionality remains intact)

**Testing Focus**:

- Mission start/end transitions
- Player join/leave during lobby
- Ghost behavior at mission start
- Gear functionality at mission start
