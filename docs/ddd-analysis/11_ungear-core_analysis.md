# Crate Analysis: `ungear-core`

## Bounded Context

**Gear Management & Equipment Logic**

This crate manages the representation and foundational lifecycle of tools (gear) in the game. It handles how gear is
data-defined, how it's held by the player, and how it's deployed in the environment.

## Responsibility

- Define base components for gear (e.g., `Battery`, `Electronic`, `EvidenceSensor`).
- Manage the relationship between gear and the world (`DeployedGear`) or the player (`PlayerGear`).
- Provide the `GearSpawnerRegistry` to handle generic item creation.
- Provide `GearStuff`, a system parameter bundle for gear-related operations.

## Dependencies Audit

- `unfoundation-core` / `untypes-core`: Base enums and states.
- `unspatial-core` / `unboard-core`: For physical location of gear.
- `uninteraction-core`: Integration with the interaction system (picking up/placing).
- `bevy`: Deeply integrated ECS functionality.

## Encapsulation Assessment

- **Privacy**: The `GearSpawnerRegistry` is a good example of encapsulation, allowing specific items to define their
  behavior without `ungear-core` needing to know about every item type.
- **Leakage**: `GearStuff` is a "God Parameter" that couples the gear system to almost every other core system (Ghosts,
  Board, Difficulty). This makes it hard to test the gear system in isolation.

## Semantic & Infrastructure Leakage

- **Logic Coupling**: The crate acts as a coordinator for many domains. This suggests that "Gear" is the primary
  interface through which many other systems (Ghost hunting, evidence collection) are expressed.
- **Infrastructure**: Heavily tied to Bevy.

## Future Recommendations

- **Decouple GearStuff**: Break down `GearStuff` into smaller, context-specific bundles to reduce cross-domain coupling.
- **Trait-based Gear**: If the component-based approach becomes too verbose, consider using Bevy Observers or a more
  trait-like system for item-specific behaviors.
