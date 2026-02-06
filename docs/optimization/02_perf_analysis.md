# Perf Analysis 02

## Context

- Source: perf session output provided by the user.
- Run mode: Low quality settings.
- Observation: these hotspots are still not acceptable and need investigation.

## Perf Trace (reformatted)

Entry 1 (System::run_unsafe) 9.01%

- core::ops::function::FnMut::call_mut 9.01%
  - unlight_plugin::maplight::systems::tiles::apply_lighting_to_tiles_system 8.73%
    - unlight_plugin::maplight::sampler::LightingSampler::fpos_sampling_corner 3.20%
    - unlight_plugin::maplight::sampler::calc_rgba 1.84%
    - unlight_plugin::maplight::systems::tiles::apply_lighting...::{closure} 0.399%
    - unlight_plugin::maplight::sampler::LightingSampler::fpos_gamma_color 0.19%
    - unlight_plugin::maplight::systems::tiles::apply_lighting...::{closure} 0.132%
    - core::ops::function::impls...::call_mut 0.0784%
    - unlight_plugin::maplight::systems::tiles::apply_lighting...::{closure} 0.0367%
    - bevy_color::Color...::Luminance::luminance 0.0359%
    - core::option::Option...::map 0.0146%
    - unlight_plugin::maplight::visuals::apply_ethereal_visuals 0.0132%
  - unmenu_plugin::systems::menu_interaction_system 0.0119%

Entry 2 (System::run_unsafe) 3.68%

- core::ops::function::FnMut::call_mut 3.68%
  - unlight_plugin::maplight::systems::gathering::player_visibility_system 3.66%
    - unlight_plugin::maplight::visibility::compute_visibility 2.67%
  - unmenu_plugin::systems::menu_interaction_system 0.0198%

Entry 3 (System::run_unsafe) 3.21%

- ungearitems_plugin::components::repellentflask::repellent_update 3.21%
  - core::iter::traits::iterator::Iterator::for_each 2.71%
    - core::iter::traits::iterator::Iterator::fold 2.71%
      - core::iter...::for_each::call::{closure} 2.51%
        - ungearitems_plugin...::repellent_update::{closure} 2.51%
  - core::iter::traits::iterator::Iterator::for_each 0.33%

## Problem Statement

Even in low quality mode, the above hotspots are still too heavy. This indicates that the current performance
bottlenecks are not alleviated by the low quality settings and require deeper investigation.

## Repellent Update: Current Understanding

- Map size upper bound (typical): 64x64x3 (12,288 cells).
- The repellent update runs even when there are zero repellent particles.
- The base field reset is unconditional and per-frame:
  - The loop in repellent update iterates all cells in the map and writes 20.0 for player-free cells or 0.0 otherwise.
  - This happens regardless of whether any repellent particles exist.
- Design intent (as stated):
  - Gas should remain inside walkable space and not pass through walls.
  - Gas should be pushed away from non-walkable cells and avoid walls.
  - Artistic movement for gas behavior is desired.

## Collision Rebuild: Trigger Path

- Collision data is rebuilt only when a BoardTopologyToRebuild event is emitted.
- The collision rebuild path is in PostUpdate, and only runs on message:
  - boardfield_update reads BoardTopologyToRebuild and calls rebuild_collision_data when collision is true.
- This implies collision data is not changing every frame; most frames are reusing the same data.

### BoardTopologyToRebuild Event Sources

- Player interactions or room sync: uninteraction-plugin systems emit BoardTopologyToRebuild with lighting and
  collision.
- Ghost light flicker end: unghost-plugin emits BoardTopologyToRebuild with lighting and collision.
- Room change events: unclassic-mode-plugin emits BoardTopologyToRebuild with lighting and collision.
- Level-ready flows also rebuild collision directly:
  - unmapload-plugin: rebuild_collision_on_level_ready.
  - unlight-plugin: prebake_lighting_on_level_ready.
  - unfog-plugin: initialize_miasma.

## Collision Field Meaning

- player_free means the player can walk through the cell (solid collision for player).
- This is the chosen collision gate for gas: gas movement should avoid non-player-free cells.
- BoardCollisionField changes only when collision rebuild events occur (doors opening/closing, room state sync, etc.).

## Repellent Update: Hot Loop Context

- The perf cost attributed to repellent_update remains even at zero particles due to the per-frame full-array sweep.
- This sweep is keyed only to map size and collision field values, not to particle count.

## Investigation Task List

1. Confirm the exact scenario and duration used for this perf capture.
2. Map each hotspot to specific systems and code paths in the project.
3. Quantify per-frame cost for each hotspot with additional profiling.
4. Identify data sizes involved (entity counts, tile counts, loops) for each hotspot.
5. Verify whether low quality settings are expected to affect each hotspot and if they do.
6. Capture a second perf trace after controlling for map, player count, and effects.
7. Summarize findings and update this document with validated observations.
