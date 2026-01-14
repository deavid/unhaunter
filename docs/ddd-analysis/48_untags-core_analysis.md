# DDD Analysis: `untags-core`

## 1. Bounded Context

**Cross-Cutting Identity Domain**. This crate defines the foundational markers (tags) that allow different subsystems to
identify and interact with entities without knowing their internal domain details.

## 2. Responsibility

- **Entity Taxonomy**: Defines the core "roles" an entity can play: `PlayerTag`, `GhostTag`, `InteractableTag`,
  `GearTag`, `NpcTag`, and `TruckTag`.
- **System Filtering**: Provides the components used by Bevy queries to filter for specific classes of objects.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`.
- **Appropriateness**: High. By moving these tags into a dedicated "bottom-level" crate, we avoid circular dependencies
  between plugins (e.g., the Ghost AI can find a "Player" without the Ghost domain needing to depend on the Player
  domain).

## 4. Encapsulation (Data/Logic Split)

- **Data**: Pure marker structs.
- **Logic**: Zero logic.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Low. These tags represent the high-level semantic vocabulary of the game.
- **Infrastructure Leakage**: Low. Only depends on Bevy for the `Component` derive.

---

## Technical Debt & Strategic Notes

- **Ubiquitous Language**: This crate is the technical manifestation of the game's "Ubiquitous Language".
- **Decoupling Power**: This is currently one of the most effective decoupling tools in the codebase. Whenever a plugin
  needs to know "is this a player?", it should use `PlayerTag` rather than checking for domain-specific components like
  `PlayerSprite`.
