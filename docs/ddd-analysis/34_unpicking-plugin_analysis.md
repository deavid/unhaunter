# Crate Analysis: `unpicking-plugin`

## Bounded Context

**Infrastructure / Interaction Input**

This crate bridges the gap between the picking system (mouse/pointer) and the custom materials used in the game world.
It is a technical plugin for world interaction detection.

## Responsibility

- Implement the `UnhaunterPickingPlugin`.
- Provide a custom `PickingBackend` for Bevy's picking system.
- Calculate pixel-perfect hit detection for entities using the `GameSpriteMaterial`.

## Dependencies Audit

- `unpicking-core`: For picking configuration markers.
- `unrender-std`: To read sprite data and materials for hit detection.
- `bevy_picking`: The underlying framework.
- `bevy`: Standard infrastructure.

## Encapsulation Assessment

- **Privacy**: The hit-test algorithm is hidden.
- **Role**: This is strictly an infrastructure crate.

## Semantic & Infrastructure Leakage

- **Infrastructure Leakage**: High. The picking logic needs to "know" the internal field layout of the custom
  `GameSpriteMaterial` (sprite dimensions, indices) to perform its math.
- **Abstractions**: Currently, the system only supports picking for `GameSpriteMaterial`. If the game engine adopts
  other rendering methods (e.g., standard Bevy meshes), a different picking backend would be required.
- **Compliance**: `unpicking-core` appropriately contains only data, keeping all math and logic in this plugin.

## Future Recommendations

- **Registry Pattern**: Instead of hardcoding specialized logic for one material, allow materials to register their own
  "Hit-Test Logic" with the picking system.
- **Generic Interface**: Move toward a generic "World-Picking" interface that doesn't depend on the specific
  implementation details of the rendering materials.
