# Crate Analysis: `unpicking-core`

## Bounded Context

**Picking / Interaction Infrastructure (Data)**

This crate defines the configuration parameters and identification markers for the world-picking system.

## Responsibility

- Define the `PickingSettings` resource (alpha thresholds, search distances).
- Provide the `CustomPickingCamera` marker component.
- Store default picking constants (e.g., standard tile sizes).

## Dependencies Audit

- `bevy`: For ECS and math types.

## Encapsulation Assessment

- **Privacy**: High. Publicly exports only data structures.
- **Compliance**: **Strict "Data-Only" Rule Followed.** This crate contains no logic or systems; all behavior is in the
  plugin crate.

## Semantic & Infrastructure Leakage

- **Visual Leakage**: The crate stores parameters deeply tied to pixel-based rendering (e.g., `alpha_threshold`). This
  leaks rendering-specific concepts ("transparency") into the core data layer.
- **Hardcoding**: Contains hardcoded defaults for `tile_size` (16x16) and `alpha_threshold` (0.8). While suitable for
  the current artistic direction, these are "magic numbers" baked into the core.

## Future Recommendations

- **Theme/Config Separation**: Move pixel-specific thresholds and tile sizes to an external configuration or a theme
  file so that the picking core can remain generic to any "Selectable Surface".
- **Naming Alignment**: Ensure that "Picking" is understood as an engine-level capability that is agnostic of the "Game"
  logic.
