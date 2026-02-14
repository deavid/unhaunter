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

### 2. UnhaunterSettingsPlugin

- **Systems**: None (only initializes persistent resources)
- **Resources**: GameplaySettings, VideoSettings, AudioSettings, ProfileSettings, ControlKeys
- **File**: `crates/unsettings-plugin/src/plugin.rs`

### 3. UnhaunterDifficultyPlugin

- **Systems**: None (only initializes CurrentDifficulty resource)
- **File**: `crates/undifficulty-plugin/src/systems/mod.rs`

### 4. UnhaunterEngineCorePlugin

- **Systems**: Various core engine systems
- **Resources**: Maps, AppState, GameState, PerlinNoise, PlayerInput
- **Events**: BoardTopologyToRebuild, OnScreenHintEvent
- **File**: `crates/unengine-plugin/src/plugin.rs`

### 5. UnhaunterSummaryCorePlugin

- **Systems**: `update_time` (in FixedUpdate schedule, runs during InGame state)
- **Resources**: SummaryData
- **File**: `crates/unsummary-plugin/src/plugin.rs`

### 6. UnhaunterMetricsPlugin

- **Systems**:
  - `receive_data` (PostUpdate)
  - `report_performance` (Update)
- **File**: `crates/unmetrics-plugin/src/plugin.rs`

### 7. UnhaunterThermalPlugin

- **Systems**:
  - `temperature_update` (Update)
  - `init_thermal_grid_allocation` (Update)
  - `init_thermal_grid_content` (Update, after init_thermal_grid_allocation)
- **Resources**: ThermalGrid
- **File**: `crates/unthermal-plugin/src/plugin.rs`

### 8. UnhaunterRenderCorePlugin

- **Systems**: Various board and rendering systems (headless mode)
- **Resources**: BoardTopology, BoardEntityField, BoardCollisionField, SpriteDB, RoomDB
- **File**: `crates/unrender-plugin/src/plugin.rs`
- **Note**: In headless mode, registers stub assets for required resource types

### 9. UnhaunterGearCorePlugin

- **Systems**: None (only initializes resources)
- **Resources**: GameConfig, GearSpawnerRegistry
- **Events**: SoundEvent
- **File**: `crates/ungear-plugin/src/plugin.rs`

### 10. UnhaunterInteractionCorePlugin

- **Systems**: Various interaction systems
- **Events**: ExecuteInteractionEvent, RoomStateSyncEvent
- **File**: `crates/uninteraction-plugin/src/plugin.rs`

### 11. UnhaunterMissionPlugin

- **Systems**:
  - `handle_mission_events` (Update, during InGame)
  - `evaluate_mission_end` (Update, during InGame)
- **Resources**: MissionEndRequested
- **Events**: MissionEvent
- **File**: `crates/unmission-plugin/src/lib.rs`

### 12. UnhaunterGearItemsPlugin

- **Systems**: Component setup for various gear items:
  - Quartz, Salt, Sage, Thermometer, EMF Meter, Recorder, Flashlight
  - Geiger Counter, UV Torch, Video Camera, Red Torch, Photo Camera
  - Spirit Box, Ion Meter, Thermal Imager, EStatic Meter, Compass
  - Motion Sensor, Repellent Flask
- **Resources**: GameConfig
- **File**: `crates/ungearitems-plugin/src/plugin.rs`

### 13. UnhaunterTruckCorePlugin

- **Systems**:
  - `init_repellent_tracker` (OnEnter InGame)
  - `reset_repellent_tracker` (OnExit InGame)
- **Resources**: GhostGuess, RepellentCraftTracker
- **Events**: TruckUIEvent, EventButtonClicked
- **File**: `crates/untruck-plugin/src/plugin.rs`

### 14. UnhaunterPlayerCorePlugin

- **Systems**: Core player setup systems
- **Resources**: GameConfig
- **File**: `crates/unplayer-plugin/src/plugin.rs`

### 15. UnhaunterGhostCorePlugin

- **Systems**:
  - Hydration systems
  - Evidence decay systems
  - Ghost AI systems
- **Resources**: ObjectInteractionConfig, HauntState, CurrentEvidenceReadings
- **Events**: GhostInteractionEvent
- **File**: `crates/unghost-plugin/src/plugin.rs`

### 16. UnhaunterLightCorePlugin

- **Systems**:
  - `init_light_grid` (PreUpdate)
  - `prebake_lighting_on_level_ready` (Update)
  - `rebuild_lighting_field` (PostUpdate, in BoardUpdateSet::Lighting)
  - `player_visibility_system` (PostUpdate, after lighting, during InGame)
- **Resources**: LightGrid
- **File**: `crates/unlight-plugin/src/plugin.rs`

### 17. UnhaunterNPCCorePlugin

- **Systems**: Hydration systems for NPCs
- **File**: `crates/unnpc-plugin/src/plugin.rs`

### 18. UnhaunterNetPlugin

- **Systems**: Network setup systems
- **Resources**: NetworkConn, PlayerRegistry, LocalPlayer, ChangedTiles, MissionEndRequested, HostGone, LobbyData,
  PendingMapLoad
- **Events**: NetworkDataEvent, NetworkDisconnectEvent, PlayerJoinedEvent, PlayerDiedEvent, SendNetworkMessage,
  TransientEvent
- **File**: `crates/unnet-plugin/src/plugin.rs`

### 19. UnhaunterTmxMapPlugin

- **Systems**: Map initialization and loading systems
- **Resources**: MapTileSetDb, MapAssetIndexHandle, UpscaleIndex
- **File**: `crates/untmxmap-plugin/src/plugin.rs`

### 20. UnhaunterFogCorePlugin

- **Systems**:
  - `init_miasma_grid` (Update)
  - `initialize_miasma` (Update, on LevelReadyEvent, after init_miasma_grid)
  - `update_miasma` (Update, during InGame)
- **Resources**: MiasmaConfig, MiasmaGrid
- **File**: `crates/unfog-plugin/src/plugin.rs`

### 21. UnhaunterWalkieCorePlugin

- **Systems**: None (only initializes resources)
- **Resources**: WalkiePlay, PotentialIDTimer
- **Events**: WalkieTalkingEvent
- **File**: `crates/unwalkie-plugin/src/plugin.rs`

### 22. UnhaunterMapLoadPlugin

- **Systems**: Level loading and entity spawning systems
- **Events**: LoadLevelEvent, LevelLoadedEvent, LevelReadyEvent, MapEntitiesReadyEvent, MapGeometryInitializedEvent
- **File**: `crates/unmapload-plugin/src/plugin.rs`
- **Note**: Skips asset loading in headless mode

### 23. UnhaunterClassicModeCorePlugin

- **Systems**:
  - `classic_mode_orchestrator` (Update, on MapEntitiesReadyEvent)
  - `spawn_joined_player` (Update, during InGame)
- **Resources**: ActiveMissionEvaluator (ClassicEvaluator)
- **Events**: RoomChangedEvent
- **File**: `crates/unclassic-mode-plugin/src/plugin.rs`

### 24. UnhaunterProfilePlugin

- **Systems**: None (only initializes persistent player profile)
- **Resources**: PlayerProfileData, RuntimeInstallationId
- **File**: `crates/unprofile-plugin/src/plugin.rs`

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
