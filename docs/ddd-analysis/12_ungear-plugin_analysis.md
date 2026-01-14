# Crate Analysis: `ungear-plugin`

## Bounded Context

**Gear Implementation & Systems**

This plugin crate implements the logic for how gear behaves in the game world, including visuals, sounds, and UI
integration.

## Responsibility

- Implement the `UnhaunterGearPlugin`.
- Coordinate gear visuals (updating sprites based on state).
- Handle sound effects for gear and other world events.
- Manage the gear inventory UI.
- Cleanup gear assets when a level ends.

## Dependencies Audit

- `ungear-core`: Data and registry for gear.
- `unrender-std`: Common rendering components.
- `unsounds-core`: Sound assets and triggers.
- `unui-core`: Base UI components.
- `bevy`: Standard rendering and ECS systems.

## Encapsulation Assessment

- **Privacy**: Internal systems are private.
- **De-duplication**: Found evidence of stale code and potential duplication (e.g., `gear_stuff.rs` in both core and
  plugin).

## Semantic & Infrastructure Leakage

- **Audio Overreach**: The `SoundSystems` handles general `SoundEvent` messages, which feels like it should be in a
  dedicated audio plugin rather than one focused on "gear".
- **Visuals**: Directly tied to Bevy UI and Sprite types.

## Future Recommendations

- **Audio Extraction**: Move general sound system logic to a `unsound-plugin`.
- **Cleanup**: Remove stale files (`gear_stuff.rs` in plugin if it's a duplicate of core) and empty files
  (`evidence_systems.rs`).
- **Separation**: Ensure that the "Inventory UI" is separable from the "Physical Gear Logic".
