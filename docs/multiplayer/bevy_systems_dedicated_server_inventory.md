# Bevy Systems Inventory for Dedicated Server

This document inventories all Bevy systems that run on the dedicated server in Unhaunter.

## Plugins Loaded on Dedicated Server

Based on `unhaunter/src/app.rs`, these plugins are loaded when `cli_options.dedicated` is true:

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

13. **UnhaunterTruckCorePlugin** - Truck core functionality
14. **UnhaunterPlayerCorePlugin** - Core player systems
15. **UnhaunterGhostCorePlugin** - Core ghost AI and behavior
16. **UnhaunterLightCorePlugin** - Core lighting simulation
17. **UnhaunterNPCCorePlugin** - Core NPC systems
18. **UnhaunterNetPlugin** - Networking
19. **UnhaunterTmxMapPlugin** - TMX map loading
20. **UnhaunterMapLoadPlugin** - Map loading systems
21. **UnhaunterClassicModeCorePlugin** - Classic game mode core

## Detailed Systems Inventory

### 1. UnhaunterFpsPlugin

- **Systems**: `fps_limiter` (in `Last` schedule)
- **Purpose**: Limits FPS to target rate, tracks usage metrics
- **File**: `crates/unfps-plugin/src/systems/limit.rs`
- **Gating**: None (Essential for dedicated server performance)

### 2. UnhaunterSettingsPlugin

- **Systems**: None (Initializes persistent resources)
- **Resources**: `GameplaySettings`, `VideoSettings`, `AudioSettings`, `ProfileSettings`, `ControlKeys`
- **File**: `crates/unsettings-plugin/src/plugin.rs`

### 3. UnhaunterDifficultyPlugin

- **Systems**: None (Initializes `CurrentDifficulty` resource)
- **File**: `crates/undifficulty-plugin/src/systems/mod.rs`

### 4. UnhaunterEngineCorePlugin

- **Systems**:
  - `boardfield_update` (`PostUpdate`, in `BoardUpdateSet::Collision`): Updates board topology and rebuilds collision
    data.
- **Resources**: `Maps`, `AppState`, `GameState`, `PerlinNoise`, `PlayerInput`
- **Events**: `BoardTopologyToRebuild`
- **File**: `crates/unengine-plugin/src/boardfield_update.rs`
- **Gating**: `on_message::<BoardTopologyToRebuild>` (Opportunity: Add `AppState::InGame`)

### 5. UnhaunterSummaryCorePlugin

- **Systems**: `update_time` (`FixedUpdate`)
- **Resources**: `SummaryData`
- **File**: `crates/unsummary-plugin/src/plugin.rs`
- **Gating**: `run_if(in_state(AppState::InGame))`

### 6. UnhaunterMetricsPlugin

- **Systems**:
  - `receive_data` (`PostUpdate`): Reads metrics from internal channels.
  - `report_performance` (`Update`): Logs performance averages and checks state consistency.
- **File**: `crates/unmetrics-plugin/src/plugin.rs`
- **Gating**: None (Keep for monitoring)

### 7. UnhaunterRenderCorePlugin

- **Systems**:
  - `sync_map_entity_field` (`Update`): Synchronizes `BoardEntityField` with entity positions.
  - `hydration_simulation_system` (`Update`): Initializes board components during map load.
- **Resources**: `BoardTopology`, `BoardEntityField`, `BoardCollisionField`, `SpriteDB`, `RoomDB`
- **File**: `crates/unrender-plugin/src/systems/board_sync.rs`, `hydration.rs`
- **Gating**: None (Opportunity: Add `AppState::InGame`)

### 8. UnhaunterGearCorePlugin

- **Systems**: None
- **Resources**: `GameConfig`, `GearSpawnerRegistry`
- **Events**: `SoundEvent`
- **File**: `crates/ungear-plugin/src/plugin.rs`

### 9. UnhaunterInteractionCorePlugin

- **Systems**:
  - `interaction_event_handler` (`Update`): Processes `ExecuteInteractionEvent`.
  - `room_state_sync_system` (`Update`): Synchronizes room states.
- **Events**: `ExecuteInteractionEvent`, `RoomStateSyncEvent`
- **File**: `crates/uninteraction-plugin/src/systems/mod.rs`
- **Gating**: None (Opportunity: Add `AppState::InGame`)

### 10. UnhaunterMissionPlugin

- **Systems**:
  - `handle_mission_events` (`Update`)
  - `evaluate_mission_end` (`Update`)
- **Resources**: `MissionEndRequested`
- **Events**: `MissionEvent`
- **File**: `crates/unmission-plugin/src/systems/setup.rs`
- **Gating**: `run_if(in_state(AppState::InGame))`

### 11. UnhaunterTruckCorePlugin

- **Systems**:
  - `init_repellent_tracker` (`OnEnter(AppState::InGame)`)
  - `reset_repellent_tracker` (`OnExit(AppState::InGame)`)
  - `force_discard_evidence_system` (`Update`)
  - `host_handle_journal_messages_system` (`Update`)
  - `on_enter_truck` (`OnEnter(GameState::Truck)`)
  - `on_exit_truck` (`OnExit(GameState::Truck)`)
- **Resources**: `GhostGuess`, `RepellentCraftTracker`
- **Events**: `TruckUIEvent`, `EventButtonClicked`, `ForceDiscardEvidenceEvent`
- **File**: `crates/untruck-plugin/src/plugin.rs`, `journal.rs`, `systems/in_truck_manager.rs`
- **Gating**: Already properly gated for mission states.

### 12. UnhaunterPlayerCorePlugin

- **Systems**:
  - `hydration_player_logic_system` (`Update`): Map load initialization for player spawns.
  - `player_interaction_system` (`Update`)
  - `player_movement_system` (`Update`)
  - `player_input_clear_system` (`PostUpdate`)
  - `player_gear_usage_system` (`Update`)
  - `sync_held_gear_position` (`Update`)
  - `update_held_object_position` (`Update`)
  - `grab_object`, `drop_object`, etc. (`Update`)
  - `lose_sanity`, `health_regen`, `detect_and_apply_death` (`Update`)
- **Resources**: `GameConfig`
- **File**: `crates/unplayer-plugin/src/systems/setup.rs`, `hydration.rs`, `grabdrop.rs`, `sanityhealth.rs`
- **Gating**: Mixed. (Opportunity: Many core systems could be additionally gated with `AppState::InGame`).

### 13. UnhaunterGhostCorePlugin

- **Systems**:
  - `hydration_ghost_logic_system` (`Update`): Map load initialization for ghosts.
  - `decay_evidence_clarity_system` (`Update`)
  - `ghost_movement`, `ghost_enrage`, `ghost_fade_out_system` (`Update`)
  - `update_ghost_warning_field` (`Update`)
  - `ghost_visual_sync`, `ghost_influence_visual_sync` (`Update`)
- **Resources**: `ObjectInteractionConfig`, `HauntState`, `CurrentEvidenceReadings`
- **Events**: `GhostInteractionEvent`
- **File**: `crates/unghost-plugin/src/plugin.rs`, `systems/`
- **Gating**: Mixed. (Opportunity: Add `AppState::InGame` to AI and decay systems).

### 14. UnhaunterLightCorePlugin

- **Systems**:
  - `init_light_grid` (`PreUpdate`)
  - `prebake_lighting_on_level_ready` (`Update`)
  - `rebuild_lighting_field` (`PostUpdate`)
  - `player_visibility_system` (`PostUpdate`)
- **Resources**: `LightGrid`
- **File**: `crates/unlight-plugin/src/plugin.rs`
- **Gating**: Only `player_visibility_system` is gated. (Opportunity: Gate other lighting systems).

### 15. UnhaunterNPCCorePlugin

- **Systems**:
  - `hydration_npc_system` (`Update`): Map load initialization for NPCs.
- **File**: `crates/unnpc-plugin/src/hydration.rs`
- **Gating**: None (Opportunity: Add `AppState::InGame`).

### 16. UnhaunterNetPlugin

- **Systems**:
  - `headless_summary_reset_system` (`Update`)
  - `lobby_broadcast_state_system`, `host_liveness_system` (`Update`)
  - `host_send_snapshots_system` (`Update`)
  - `network_io_system`, etc. (`PreUpdate`)
- **Resources**: `NetworkConn`, `PlayerRegistry`, `LocalPlayer`, `ChangedTiles`, `MissionEndRequested`, `HostGone`,
  `LobbyData`, `PendingMapLoad`
- **Events**: `NetworkDataEvent`, `NetworkDisconnectEvent`, `PlayerJoinedEvent`, `PlayerDiedEvent`,
  `SendNetworkMessage`, `TransientEvent`
- **File**: `crates/unnet-plugin/src/plugin.rs`, `systems/setup.rs`
- **Gating**: Mixed. (Opportunity: Gate `headless_summary_reset_system`).

### 17. UnhaunterTmxMapPlugin

- **Systems**:
  - `map_index_preload`, `tmxmap_preload` (`Update`)
  - `load_level_handler` (`Update`)
- **Resources**: `MapTileSetDb`, `MapAssetIndexHandle`, `UpscaleIndex`
- **File**: `crates/untmxmap-plugin/src/init_maps.rs`, `load_level.rs`
- **Gating**: Event-based or state-based preloading.

### 18. UnhaunterMapLoadPlugin

- **Systems**:
  - `hydration_conveyor_system` (`PostUpdate`)
  - `load_level_handler` (`PostUpdate`)
- **Events**: `LoadLevelEvent`, `LevelLoadedEvent`, `LevelReadyEvent`, `MapEntitiesReadyEvent`,
  `MapGeometryInitializedEvent`
- **File**: `crates/unmapload-plugin/src/module.rs`, `conveyor.rs`, `level_setup.rs`
- **Gating**: Event-based.

### 19. UnhaunterClassicModeCorePlugin

- **Systems**:
  - `classic_mode_orchestrator` (`Update`)
  - `spawn_joined_player` (`Update`)
  - `fuse_box_overload_system`, `breaker_sync_system` (`Update`)
- **Resources**: `ActiveMissionEvaluator` (`ClassicEvaluator`)
- **Events**: `RoomChangedEvent`
- **File**: `crates/unclassic-mode-plugin/src/plugin.rs`, `environmental_mechanics.rs`
- **Gating**: Mixed. (Opportunity: Add `AppState::InGame` where missing).

### 20. UnhaunterGearItemsPlugin

- **Systems**:
  - `system_electronic_interference` (`Update`)
  - `system_battery_drain` (`Update`)
  - `system_apply_gear_intent_from_input` (`Update`)
  - Individual gear systems (Quartz, Salt, etc.)
- **File**: `crates/ungearitems-plugin/src/plugin.rs`
- **Gating**: None (Opportunity: Add `AppState::InGame`).

## Key Observations

1. **Headless Mode Adaptations**: Several plugins detect headless mode and adjust behavior:
   - `UnhaunterEngineCorePlugin`: Uses low-memory Perlin noise
   - `UnhaunterRenderCorePlugin`: Registers stub assets
   - `UnhaunterMapLoadPlugin`: Skips asset loading

2. **Core Game Logic**: All core game systems run on dedicated server:
   - Ghost AI and behavior
   - Mission management
   - Player state tracking
   - Gear functionality
   - Environmental systems (lighting, fog, thermal)

3. **Network-Centric**: The dedicated server includes full networking support but excludes:
   - Client-side rendering
   - UI systems
   - Audio systems
   - Input handling

4. **State Management**: Systems are properly gated by AppState and GameState.

## Performance Considerations

The dedicated server runs a significant number of simulation systems:

- Board topology and collision updates
- Lighting simulation
- Thermal simulation
- Fog/miasma simulation
- Ghost AI and behavior
- Gear item logic
- Mission progression

These systems are optimized to run without rendering overhead in headless mode.
