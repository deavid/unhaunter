# Crate Analysis: `unghost-plugin`

## Bounded Context

**Ghost Behavior & AI Logic**

This is the brain of the ghost. It implements how the ghost moves, interacts with the world, and hunts players.

## Responsibility

- Implement the `UnhaunterGhostPlugin`.
- Manage ghost AI (pathfinding, hunting triggers, rage state).
- Orchestrate the Ghost Interaction System (GIS) for world interactions.
- Update ghost-related telemetry (e.g., haunting efficiency).
- Visualize specific ghost evidence (Ghost Orbs, light flickering).

## Dependencies Audit

- `unghost-core`: For the ghost's base data and identity.
- `unboard-core` / `unnavigation-core`: For world navigation and room concepts.
- `unplayer-core`: To find and target players during hunts.
- `unrender-std`: For sprite and visual effects.
- `bevy`: Deeply integrated.

## Encapsulation Assessment

- **Privacy**: High internal encapsulation; logic is separated into specialized system modules.
- **Modularity**: Good separation within the `GIS` (Ghost Interaction System), allowing for diverse interaction types.

## Semantic & Infrastructure Leakage

- **Visual Leakage**: Directly handles `Sprite` and `Particle` spawning. Ghost behavior logic is mixed with rendering
  commands.
- **Heuristic Leakage**: AI weights and movement constants are hardcoded in the logic, making it difficult to tune via
  data files without recompiling.
- **Domain Coupling**: Tight coupling to specific gear items (e.g., knowing exactly how "Sage" or "Salt" works).

## Future Recommendations

- **Behavior Registry**: Move ghost interactions to a registry-based system (similar to gear) so new interaction types
  can be added without modifying the core GIS logic.
- **Navigation Abstraction**: Decouple the ghost from specific room/board concepts by using a more generic navigation
  interface.
- **Data-Driven AI**: Move AI weights and movement speeds to `unghost-core` or external YAML/RON files so they can be
  tuned without code changes.
