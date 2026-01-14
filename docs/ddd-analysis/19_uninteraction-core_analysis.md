# Crate Analysis: `uninteraction-core`

## Bounded Context

**Entity Interaction & State Synchronization**

This crate defines how the player and the world interact. It handles the "trigger" and "reaction" logic for doors,
switches, and special objects like the Van.

## Responsibility

- Define interaction markers (`Target`, `Toggleable`, `Interactive`).
- Implement `InteractiveStuff` for executing state changes (e.g., flipping a switch, opening a door).
- Sync interactive object states with the `BoardData`.
- Handle high-level gameplay transitions triggered by interactions (e.g., entering the Van).

## Dependencies Audit

- `unboard-core`: To sync state with the world grid.
- `unspatial-core` / `untypes-core`: Base spatial and state data.
- `unrender-std`: For visual feedback (material changes).
- `bevy`: Standard logic orchestration.

## Encapsulation Assessment

- **Privacy**: The `InteractiveStuff` struct encapsulates the "how" of interaction effectively.
- **Architectural Violation**: This "-core" crate contains significant **Logic**. It handles sound emitters and material
  manipulation, which should ideally live in an `uninteraction-plugin` or be handled via events.

## Semantic & Infrastructure Leakage

- **Visual Leakage**: Directly manipulates `Material` and `Sprite` properties for visual feedback.
- **Sound Leakage**: Embedded sound event emission logic.
- **Story Leakage**: Contains hardcoded knowledge of the "Van," which is a specific gameplay artifact, making the
  interaction system less generic.

## Future Recommendations

- **Split into Plugin**: Move the logic inside `InteractiveStuff` to a new `uninteraction-plugin`. Keep only the markers
  and components in `uninteraction-core`.
- **Event-Driven Reactions**: Instead of `uninteraction-core` changing materials directly, it should emit an
  `InteractionEvent`. A separate rendering plugin can then react to that event to update the visuals.
- **Registry of Interactions**: Replace hardcoded "Van" logic with a generic interaction trigger that plugins can
  register for.
