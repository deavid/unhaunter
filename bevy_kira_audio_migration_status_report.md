Progress report
-------------------

This section is for YOU, GitHub Copilot.

I want you to report here your plan on what you will be doing overall, the high level view, and the status report on each of the items.

Create a plan for what `bevy_kira_audio_migration.md` describes, in this file and every now and then update the status of each item. I want a high level report.

Write below:

---

# Bevy Kira Audio Migration Plan & Status Report

## ⚠️ CRITICAL MIGRATION FAILURE ⚠️

**Migration Status**: TECHNICALLY COMPLETE BUT FUNCTIONALLY FAILED

The migration has been successfully completed from a code perspective - all systems compile and use the new API. However, **bevy_kira_audio is not a viable solution** for this project due to severe audio issues.

### ❌ CRITICAL ISSUES DISCOVERED
1. **Native Audio Failure**: Cannot open audio device (Focusrite Scarlett on Pipewire)
   - Works perfectly with `bevy_audio`
   - Completely fails with `bevy_kira_audio`

2. **WASM Audio Quality**: Even worse than the original `bevy_audio`
   - Severe audio drops
   - Quality degradation
   - Original goal of fixing WASM audio crackling **NOT ACHIEVED**

### Original Target Benefits - ACTUAL RESULTS
- ❌ Fix WASM audio crackling issues → **MADE WORSE**
- ❌ Better audio compatibility → **BROKE NATIVE AUDIO**
- ✅ Advanced features like channels, tweens → **Not usable due to broken audio**
- ✅ Resource-based API → **Implemented but irrelevant**

### ✅ TECHNICAL MIGRATION COMPLETED
**All primary audio systems have been successfully migrated to bevy_kira_audio API:**
- ✅ 6 audio systems migrated across 5 crates
- ✅ New AudioInstance-based architecture implemented
- ✅ Full workspace compilation verified
- ✅ Plugin integration and dependency management complete
- ❌ **But audio is completely broken on both native and WASM**

## High-Level Migration Plan

### Phase 1: Dependencies & Core Setup ✅ COMPLETED
1. ✅ **Cargo.toml Configuration**: Update dependencies to disable `bevy_audio` and add `bevy_kira_audio`
2. ✅ **Version Compatibility**: Ensure Bevy 0.16 + bevy_kira_audio 0.23.0 compatibility
3. ✅ **Plugin Integration**: Replace `bevy::audio::AudioPlugin` with `bevy_kira_audio::AudioPlugin`

### Phase 2: API Migration ✅ COMPLETED
4. ✅ **Core Systems Migration**: All audio playback systems migrated to bevy_kira_audio APIs
5. ⏳ **Advanced Features**: Implement AudioChannels for better sound organization
6. ⏳ **Spatial Audio**: Update spatial audio implementation
7. ⏳ **Web Build Testing**: Validate WASM compatibility and browser-specific fixes

### Phase 3: Testing & Optimization ⏳ PENDING
8. ⏳ **Audio Quality Testing**: Verify audio improvements, especially on WASM
9. ⏳ **Performance Validation**: Ensure no performance regressions
10. ⏳ **Documentation Update**: Update any audio-related documentation

## Detailed Status Report

### ✅ COMPLETED ITEMS

#### 1. Cargo.toml Dependencies
- **Status**: ✅ COMPLETED
- **Details**:
  - `bevy_audio` feature disabled in bevy dependency
  - `bevy_kira_audio = "0.23.0"` added
  - All required format features available (mp3, wav, ogg, flac)
- **Location**: `/home/deavid/git/rust/unhaunter/Cargo.toml` lines 78, 176

#### 2. Version Compatibility Check
- **Status**: ✅ COMPLETED
- **Details**: Confirmed Bevy 0.16 + bevy_kira_audio 0.23.0 compatibility
- **Result**: No version conflicts detected

### 🔄 IN PROGRESS ITEMS

#### 3. Plugin Integration
- **Status**: ✅ COMPLETED
- **Details**: Added `bevy_kira_audio::AudioPlugin` to main app
- **Location**: `/home/deavid/git/rust/unhaunter/unhaunter/src/app.rs`

#### 4. Core Audio Systems Migration
- **Status**: ✅ COMPLETED
- **Files Requiring Migration**:

  **A. Sound Effect System** (`ungear/src/systems.rs`)
  - **Current**: ✅ COMPLETED - Migrated to `audio.play()` API with spatial panning
  - **Status**: Uses `Audio` resource with volume and panning controls
  - **Lines**: ~70-110

  **B. Walkie Talkie System** (`unwalkie/src/walkie_play.rs`)
  - **Current**: ✅ COMPLETED - Migrated to `audio.play()` with volume control
  - **Status**: Simplified from AudioPlayer+PlaybackSettings to direct audio.play()
  - **Lines**: ~35, 175-185

  **C. Truck UI Sound System** (`untruck/src/systems/truck_ui_systems.rs`)
  - **Current**: ✅ COMPLETED - Migrated sound effects for UI interactions
  - **Status**: Two audio calls migrated to bevy_kira_audio
  - **Lines**: ~170, ~325

  **D. Menu Music System** (`unmenu/src/mainmenu.rs`)
  - **Current**: ✅ COMPLETED - Migrated to use AudioInstance handles with fade controls
  - **Status**: Complex looped music with volume fade in/out now uses AudioTween for smooth transitions
  - **Lines**: ~205-250
  - **Note**: Replaced AudioPlayer+PlaybackSettings+AudioSink with audio.play().handle() and AudioInstance management

  **E. Ambient Sound System** (`unlight/src/maplight.rs`)
  - **Current**: ✅ COMPLETED - Migrated to use AudioInstance handles for volume control
  - **Status**: AudioSink queries replaced with AmbientSoundInstances resource and AudioInstance management
  - **Lines**: ~1014-1120
  - **Note**: System architecture changed to use resource-based AudioInstance tracking instead of entity queries

  **F. Map Loading Ambient Sound System** (`unmapload/src/entity_spawning.rs`)
  - **Current**: ✅ COMPLETED - Migrated from AudioPlayer entities to AudioInstance handles
  - **Status**: spawn_ambient_sounds() now uses Audio resource and AmbientSoundInstances resource
  - **Lines**: ~228-267
  - **Note**: Changed from spawning entities with AudioPlayer components to storing AudioInstance handles in a central resource

#### New Infrastructure Added
- **AmbientSoundInstances Resource** (`uncore/src/resources/audio.rs`)
  - **Status**: ✅ COMPLETED - Central resource to track ambient sound AudioInstance handles
  - **Purpose**: Replaces entity-based AudioSink queries with resource-based AudioInstance management
  - **Usage**: Used by both unmapload (creation) and unlight (volume control) systems

### ⏳ PENDING ITEMS - NOW IRRELEVANT DUE TO AUDIO FAILURE

#### 5. Advanced AudioChannel Implementation
- **Status**: ❌ CANCELLED - Audio is broken
- **Plan**:
  - Create custom channels: `MusicChannel`, `SFXChannel`, `VoiceChannel`, `AmbientChannel`
  - Migrate volume controls to channel-based system
  - Implement in audio settings integration

#### 6. Spatial Audio Enhancement
- **Status**: ❌ CANCELLED - Audio is broken
- **Current**: Basic spatial audio via `PlaybackSettings::spatial`
- **Target**: Enhanced spatial audio with `SpatialAudioPlugin`

#### 7. Web Build & Browser Compatibility
- **Status**: ❌ FAILED - Audio quality worse than before
- **Requirements**:
  - Test Chrome interaction requirements
  - Validate Firefox audio quality
  - Confirm .ogg/.wav format support (avoid .mp3 on web)

#### 8. Audio Quality Validation
- **Status**: ❌ FAILED - WASM audio quality degraded, native audio completely broken
- **Goal**: Confirm WASM audio crackling is resolved → **NOT ACHIEVED**

## Migration Complexity Assessment

### Low Complexity ✅
- ✅ Dependencies configuration
- ✅ Basic plugin replacement

### Medium Complexity 🔄
- 🔄 Simple audio playback migration (`audio.play()`)
- ⏳ Volume control migration
- ⏳ AudioChannel setup

### High Complexity ⏳
- ⏳ Menu music system (uses AudioSink queries for fade in/out)
- ⏳ Ambient sound system (complex volume modulation)
- ⏳ Spatial audio positioning system

## Risk Mitigation - POST-MORTEM

### Identified Risks - ACTUAL OUTCOMES
1. **Audio Type Name Conflicts**: Both `bevy::prelude` and `bevy_kira_audio::prelude` export `Audio`
   - **Mitigation Used**: ✅ Used explicit imports `use bevy_kira_audio::Audio;`
   - **Result**: ✅ No naming conflicts encountered

2. **Complex Volume Control Logic**: Existing fade-in/fade-out systems
   - **Mitigation Used**: ✅ Used `AudioTween` for smooth transitions instead of manual `AudioSink` manipulation
   - **Result**: ✅ Code migrated successfully but irrelevant due to broken audio

3. **Spatial Audio Behavior Changes**: Different spatial audio implementation
   - **Mitigation Planned**: Thorough testing and potentially keeping fallback options
   - **Result**: ❌ Never reached testing phase due to fundamental audio device issues

### NEW RISKS DISCOVERED
4. **Platform Compatibility**: Assumed bevy_kira_audio would work on common Linux audio setups
   - **Impact**: ❌ CRITICAL - Cannot open audio device on Pipewire/Focusrite (common professional audio setup)
   - **Lesson**: Always test on target hardware configurations before migration

5. **WASM Performance Assumption**: Expected bevy_kira_audio to fix WASM audio issues
   - **Impact**: ❌ CRITICAL - Made WASM audio quality worse than original bevy_audio
   - **Lesson**: Benchmark both platforms before migration, not just theoretical benefits

6. **Alternative Availability**: Assumed bevy_kira_audio was the best/only alternative
   - **Impact**: 🔍 INVESTIGATION NEEDED - Need to research other options
   - **Lesson**: Research multiple alternatives before committing to a specific solution

## Next Immediate Actions - MIGRATION ROLLBACK REQUIRED

### 🚨 PRIORITY 1: ROLLBACK TO WORKING STATE
1. **Revert to bevy_audio**: Restore working audio on native platforms
2. **Alternative Research**: Investigate other audio solutions:
   - `bevy_odd_audio` (newer alternative)
   - `bevy_fundsp` (DSP-focused)
   - Custom WASM-specific audio handling
   - Hybrid approach (different audio backends per platform)

### 🔍 INVESTIGATION NEEDED
1. **bevy_kira_audio Issues**:
   - Check if Pipewire/Focusrite compatibility can be fixed
   - Test with different audio drivers (ALSA, PulseAudio)
   - Check for platform-specific configuration options

2. **Alternative Solutions**:
   - Research audio crates that specifically support Pipewire
   - Investigate WASM-specific audio optimization techniques
   - Consider audio format changes (OGG vs MP3 vs WAV)

### 📝 LESSONS LEARNED
- `bevy_kira_audio` has significant platform compatibility issues
- Migration success requires testing on target hardware/software configurations
- Need platform-specific testing before committing to audio library changes
- The resource-based API approach is good and can be applied to other audio solutions

## Timeline Estimate

- **Rollback Phase**: ~2-4 hours (revert changes, test native audio)
- **Research Phase**: ~1-2 days (investigate alternatives)
- **Alternative Implementation**: TBD based on research findings

**Migration Progress**: 100% complete but 0% functional - ROLLBACK REQUIRED

---

*Last Updated: 2025-06-21*
*Status Legend: ✅ Completed | 🔄 In Progress | ⏳ Pending | 🎯 Next Priority | ❌ Failed | 🚨 Critical*

