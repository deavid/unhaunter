# Crate Analysis: `unevents-core`

## Bounded Context

**Cross-Cutting / Shared Kernel**

This crate defines the shared language used for communication between different parts of the game. It allows decoupled
plugins to interact by emitting and reacting to domain-specific events.

## Responsibility

- Define all shared `Event` types used across the workspace.
- Provide data structures for signalling state changes (e.g., `RoomChanged`, `GameStateUpdate`).
- Coordinate interaction between domains (e.g., a "Player" action triggering a "Ghost" reaction via an event).

## Dependencies Audit

- `unfoundation-core` / `unspatial-core` / `unboard-core`: Base types and world-state identifiers.
- `bevy`: For the `Event` trait and `Entity` type.

## Encapsulation Assessment

- **Privacy**: Almost non-existent (mostly `pub`), which is correct for a "Data Transfer Object" (DTO) style crate.
- **Leakage**: Highly decoupled from logic, but coupled to Bevy as the event infrastructure.

## Semantic & Infrastructure Leakage

- **Semantic**: Very clean. The crate uses domain terminology effectively.
- **Infrastructure**: Tied to Bevy's event system. If the game were to move to a different engine, the event definitions
  would remain semantically valid but would need a new transport mechanism.

## Future Recommendations

- **Avoid Logic**: Maintain the "data-only" rule for this crate. Logic should live in plugins that consume these events.
- **Modularity**: As the game grows, consider splitting events into sub-modules (e.g., `events::player`,
  `events::ghost`) to keep the crate organized, but keep them in one crate to avoid dependency hell when a system needs
  to listen to multiple types.
