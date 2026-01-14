# Crate Analysis: `unnavigation-core`

## Bounded Context

**Navigation & Path-Following**

This crate manages the "intent to move" and the basic collision avoidance logic for entities (players and ghosts).

## Responsibility

- Define navigation targets (`MoveToTarget`) and waypoint sequences (`WaypointQueue`).
- Provide the `CollisionHandler` to calculate Repulsive Vectors from solid tiles.
- Handle multi-floor collision logic.
- Manage waypoint advancement and queue manipulation.

## Dependencies Audit

- `unspatial-core`: For world-space coordinates.
- `unboard-core`: To access the collision grid.
- `bevy`: For ECS components and math.

## Encapsulation Assessment

- **Privacy**: High. Waypoint queue manipulation is encapsulated in methods.
- **Architectural Violation**: Contains heavy collision math and state-advancement logic. This is more of an "engine
  system" than a "data core".

## Semantic & Infrastructure Leakage

- **Feature Leakage**: `WaypointType` includes an `Interact` variant, which couples the "Where to go" system to the
  "What to do" system.
- **Infrastructure Leakage**: Deeply tied to the board's grid-based `collision_field`.

## Future Recommendations

- **Split out Logic**: Extract the `CollisionHandler` math and `WaypointQueue` advancement systems into an
  `unnavigation-plugin`. Keep only the components and waypoint data in `-core`.
- **Pure Coordinates**: Consider decoupling `WaypointQueue` from `Entity` interactions. Use a higher-level state machine
  to handle "Go to entity and interact" rather than baking the interaction target into the navigation system.
