# Dedicated Server Plugin Cleanup Summary

## Changes Made

### 1. Plugin Renaming (Naming Convention Compliance)

**Renamed plugins to follow `Unhaunter` prefix convention:**

- `ThermalPlugin` → `UnhaunterThermalPlugin`
- `SoundPlugin` → `UnhaunterSoundPlugin`

**Files updated:**

- `crates/unthermal-plugin/src/plugin.rs`
- `crates/unsound-plugin/src/plugin.rs`
- `unhaunter/src/app.rs`
- Documentation files updated to reflect new names

### 2. Plugin Removal (Dedicated Server Optimization)

**Removed plugins that should NOT run on dedicated server:**

1. **UnhaunterThermalPlugin** - Thermal simulation is client-side only (temperatures are local to clients)
2. **UnhaunterFogCorePlugin** - Miasma/fog simulation is client-side only
3. **UnhaunterWalkieCorePlugin** - Walkie-talkie functionality is client-side only
4. **UnhaunterProfilePlugin** - Server should not have player profiles

**Files updated:**

- `unhaunter/src/app.rs` - Removed plugins from dedicated server build
- Documentation files updated to reflect current plugin list

### 3. Documentation Updates

**Updated files:**

- `docs/multiplayer/bevy_systems_dedicated_server_inventory.md` - Removed removed plugins, updated names
- `docs/multiplayer/bevy_systems_optimization_opportunities.md` - Added section about removed plugins, updated CPU
  savings estimates
- `docs/multiplayer/26_thin_server_analysis.md` - Updated plugin names

## Current Dedicated Server Plugin List

### Common Plugins (Logic, Core, Hardware-agnostic)

1. **UnhaunterFpsPlugin** - FPS limiting and monitoring
2. **UnhaunterSettingsPlugin** - Game settings persistence
3. **UnhaunterDifficultyPlugin** - Difficulty management
4. **UnhaunterEngineCorePlugin** - Core engine systems
5. **UnhaunterSummaryCorePlugin** - Mission summary core functionality
6. **UnhaunterMetricsPlugin** - Performance metrics
7. **UnhaunterRenderCorePlugin** - Core rendering systems (headless mode)
8. **UnhaunterGearCorePlugin** - Core gear functionality
9. **UnhaunterInteractionCorePlugin** - Core interaction systems
10. **UnhaunterMissionPlugin** - Mission management
11. **UnhaunterGearItemsPlugin** - Gear item components

### Additional Common Plugins

12. **UnhaunterTruckCorePlugin** - Truck core functionality
13. **UnhaunterPlayerCorePlugin** - Core player systems
14. **UnhaunterGhostCorePlugin** - Core ghost AI and behavior
15. **UnhaunterLightCorePlugin** - Core lighting simulation
16. **UnhaunterNPCCorePlugin** - Core NPC systems
17. **UnhaunterNetPlugin** - Networking
18. **UnhaunterTmxMapPlugin** - TMX map loading
19. **UnhaunterMapLoadPlugin** - Map loading systems
20. **UnhaunterClassicModeCorePlugin** - Classic game mode core

## CPU Savings from Plugin Removal

**Estimated CPU reduction in lobby scenarios:**

- **Thermal Simulation**: ~15-25% (removed)
- **Fog Simulation**: ~8-12% (removed)
- **Walkie-Talkie Systems**: ~5-10% (removed)
- **Profile Management**: ~1-2% (removed)

**Total savings from plugin removal: ~29-49% CPU reduction**

## Next Steps for Further Optimization

### Additional System Gating Opportunities

1. **Light Core Systems** - Gate lighting systems with `AppState::InGame`
2. **Ghost Systems** - Gate evidence decay and hydration systems with `AppState::InGame`
3. **Gear Systems** - Gate mission-specific gear systems with `AppState::InGame`

**Potential additional savings: ~21-32% CPU reduction**

### Total Expected Savings with All Optimizations

**~50-81% CPU reduction in lobby scenarios**

## Verification Needed

1. **Test dedicated server functionality** - Ensure all core game logic still works
2. **Test mission start/end transitions** - Verify systems activate/deactivate correctly
3. **Test player connection handling** - Ensure lobby and networking still work properly
4. **Measure CPU usage** - Validate actual CPU savings in lobby scenarios

## Files Modified

### Code Files

- `unhaunter/src/app.rs` - Plugin imports and additions
- `crates/unthermal-plugin/src/plugin.rs` - Plugin rename
- `crates/unsound-plugin/src/plugin.rs` - Plugin rename

### Documentation Files

- `docs/multiplayer/bevy_systems_dedicated_server_inventory.md` - Plugin list and details
- `docs/multiplayer/bevy_systems_optimization_opportunities.md` - Optimization analysis
- `docs/multiplayer/26_thin_server_analysis.md` - Plugin references
- `docs/multiplayer/plugin_cleanup_summary.md` - This summary file
