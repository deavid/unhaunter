# Architecture Audit & SOLID Analysis

## Executive Summary

This document evaluates the `unhaunter` codebase against software engineering best practices, specifically SOLID
principles, Domain-Driven Design (DDD), and Clean Architecture.

**Overall Health**: The project demonstrates a strong modular structure with a clear separation between "Core"
(Infrastructure/Data) and "Features" (Gameplay). The use of Bevy's ECS naturally promotes decoupling via Components and
Events.

## 1. Domain-Driven Design (DDD) Alignment

The crate structure closely mirrors the business domains, which is excellent for maintainability.

| Domain           | Crate(s)                             | Responsibility                                                                          |
| :--------------- | :----------------------------------- | :-------------------------------------------------------------------------------------- |
| **Core Kernel**  | `uncore-*`                           | Shared types, board state, resources. Acts as the "Ubiquitous Language" of the project. |
| **Player**       | `unplayer`, `unprofile`              | Player control, state, and persistence.                                                 |
| **Ghost**        | `unghost`                            | AI, behavior, and ghost-specific mechanics.                                             |
| **Inventory**    | `ungear`, `ungearitems`              | Item management and definitions.                                                        |
| **Environment**  | `unlight`, `unfog`, `uncore-board`   | Simulation of the game world (light, visibility, physics).                              |
| **UI/Interface** | `untruck`, `uncoremenu`, `unsummary` | User interaction layers.                                                                |

**Observation**: The separation of `ungear` (logic) and `ungearitems` (content) is a good example of separating
**Rules** from **Data**.

## 2. SOLID Principles Analysis

### S - Single Responsibility Principle (SRP)

- **Good**: Most crates have a single, well-defined purpose (e.g., `unlight` only handles lighting simulation).
- **Critical Issue**: **Code Duplication**. The `board` module in `uncore-components` (`position.rs`,
  `boardposition.rs`, `direction.rs`) is a near-exact duplicate of `uncore-board`.
  - _Impact_: Violates DRY. Creates ambiguity about which type to use. `uncore-components` depends on `uncore-board`, so
    it should likely re-export or remove these types.
- **Concern**: `unstd` and `uncore-foundation` risk becoming "junk drawers" for miscellaneous code.
  - _Recommendation_: Monitor `unstd`. If `unstd::materials` or `unstd::tiledmap` grows, consider promoting them to
    dedicated crates (e.g., `unassets`, `unmaploader`).

### O - Open/Closed Principle (OCP)

- **Good**: Bevy's Plugin architecture allows adding new features without modifying existing `App` setup code.
- **Good**: The `Behavior` component in `uncore-board` allows defining new entity behaviors without changing the core
  board logic.
- **Concern**: `unlight`'s `identify_active_light_sources` iterates over specific metadata. Adding a new type of light
  source might require modifying the lighting engine.
  - _Recommendation_: Ensure light sources are defined by a Component (e.g., `LightSource`) rather than a hardcoded list
    or metadata lookup.

### L - Liskov Substitution Principle (LSP)

- **Context**: In Rust/ECS, this applies to Traits and Component composition.
- **Good**: The `Interactive` component allows any entity to be interactable, regardless of what it is (Door, Switch,
  Item). The interaction system treats them uniformly.

### I - Interface Segregation Principle (ISP)

- **Good**: Systems generally query only the Components they need. For example, `movement.rs` queries `Position` and
  `CollisionHandler`, but doesn't care about `LightFieldData`.
- **Good**: `uncore-events` defines small, specific events (`RoomChangedEvent`, `NpcHelpEvent`) rather than a monolithic
  "GameEvent".

### D - Dependency Inversion Principle (DIP)

- **Good**: High-level gameplay logic (`unplayer`) depends on abstractions (`uncore-board` data structures,
  `uncore-events`) rather than low-level implementation details of other systems.
- **Good**: `untruck` (UI) depends on `CurrentDifficulty` (abstraction) via the `FromTab` trait, preventing direct
  coupling to the difficulty implementation details.

## 3. Clean Architecture & Data Flow

The data flow follows a unidirectional pattern typical of ECS:

1. **Input** (Keyboard/Mouse) -> `unplayer` Systems
2. **State Change** (Components/Resources) -> `Position`, `BoardData`
3. **Simulation** (Systems) -> `unlight`, `unfog` react to changes
4. **Presentation** (Rendering/UI) -> `untruck`, `unmenu` read state to draw

**Coupling Analysis**:

- **Data Coupling**: High. Most systems couple to `Position` and `BoardData`. This is acceptable as these are the "Core
  Entities" of the game.
- **Logical Coupling**: Low. `unghost` doesn't call `unplayer` functions directly. They interact via the Board (spatial)
  or Events.

## 4. Refactoring Recommendations

1. **Standardize 3D Math**:

   - Ensure `Position` (logical) and `Transform` (visual) are synchronized consistently.
   - If moving to a 3D view, ensure `Position`'s `z` (floor) and `global_z` (offset) are sufficient for vertical
     gameplay, or consider fully adopting `Vec3` for logical position if verticality becomes continuous (e.g., ramps,
     jumping).

2. **Decouple Simulation from Rendering**:

   - `unlight` calculates `LightFieldData`. Ensure this data is exposed in a way that can be used by _any_ renderer (2D
     Tilemap or 3D Mesh).
   - Currently, it seems well-separated. The "View" layer just needs to read `LightFieldData`.

3. **Event-Driven Interactions**:

   - Continue using `uncore-events` for cross-crate communication. Avoid `pub` functions in `systems` modules that are
     called directly by other crates (unless they are utility functions).

4. **Crate Consolidation (Optional)**:
   - `unwalkie`, `unwalkie_types`, `unwalkiecore`: Consider if these need to be 3 separate crates. If they are always
     used together and don't cause circular deps, merging them might simplify the workspace.

## Conclusion

The `unhaunter` codebase is architecturally sound. It avoids common pitfalls like monolithic "Game Manager" classes or
spaghetti code. The heavy reliance on `uncore-*` crates is a strength, providing a stable foundation for feature
development. The transition to a 3D presentation layer is primarily a "View" update, as the "Model" (`Position`,
`BoardData`, `LightFieldData`) is already spatially aware (3D grid).
