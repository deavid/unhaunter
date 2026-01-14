# Domain-Driven Design (DDD) Refactor: Architectural Evolution

**Date:** January 14, 2026

## 1. Context and Intent

### What is DDD?

**Domain-Driven Design (DDD)** is an architectural approach that centers software development around a deep
understanding of the business domain. Instead of organizing code by technical layers (e.g., "All Components", "All
Systems"), DDD organizes code into **Bounded Contexts**—independent modules that own a specific area of the game's logic
(e.g., "Ghost behavior", "Player movement", "Inventory logic").

### Why Pursue DDD?

The transition from a monolithic "Uncore" structure to the current **Core vs. Plugin** architecture was the first major
step toward DDD. The intent is to:

- **Enforce Bounded Contexts:** Ensure that change in the `ungame-plugin` (game flow) doesn't require a recompile or
  architectural change in the `unghost-plugin` (ghost AI).
- **Prevent Semantic Leakage:** Stop technical details (like sprite indices or asset handles) from polluting the core
  game rules.
- **Scalability:** Make the codebase easier for multiple developers (or AI agents) to work on simultaneously without
  stepping on each other's toes.

---

## 2. Current Status and Analysis

The project has successfully separated **Core Data** (`un*-core`) from **System Logic** (`un*-plugin`). However, our
current analysis identifies remaining "Semantic Leakage" where domain boundaries are still porous:

### Visual Leakage

- **Observation:** Core types (e.g., `GearSpriteID`) are still defined using hardcoded indices that map directly to the
  spritesheet.
- **DDD Analysis:** The "Gear Domain" should only know about logical states (e.g., `EMFMeter::Level4`). The "Rendering
  Domain" should be the only one that knows this translates to "Atlas Index 14".

### Infrastructure Leakage

- **Observation:** `PlayerSprite` (the core player entity) still carries its own `ControlKeys`.
- **DDD Analysis:** This couples the player's physical existence in the game world to the user's keyboard configuration.
  Ideally, an "Input Domain" would translate keys into "Intentions" (e.g., `MoveDirection::North`), which the Player
  Domain then consumes.

---

## 3. Open Ideas and Future Opportunities

While the `ungear` and `unplayer` domains are stable, the **Map Loading Engine** presents a significant opportunity for
a more generic "Engine" approach.

### Generic Map Loading (Registry Pattern)

Currently, `unmapload-plugin` acts as a coordinator that knows about specific game entities (Ghosts, Players, Gear).
This creates a bottleneck where adding a new entity type requires modifying the map loader.

**The Concept:** Transform the map loader into a generic **Entity Factory**:

1. **Registry:** Plugins (`unghost`, `unplayer`, `unnpc`) "register" a spawner function with a specific Tiled `class` or
   `property`.
2. **Lookup:** When `unmapload` parses a Tiled object, it looks up the registered handler for that object's type.
3. **Spawn:** The handler (owned by the specific domain plugin) takes the Tiled properties and spawns the correct bundle
   of components.

**Benefits:**

- **Extensibility:** You can add NPCs, interactive furniture, or new floor hazards just by creating a new plugin and
  registering its spawner.
- **Engine Decoupling:** The map loader becomes a "pure utility" that could theoretically be used for a different game
  entirely.

### Presentation Mapping Layer

Introduce a formal "Mapper" system in `unrender-plugin`:

- Logic updates the `GearKind`.
- A "View Synchronizer" system looks at the `GearKind` and updates the `Sprite::index` based on a configuration file or
  internal mapping.
- This allows the game to support "Skins", "Alternative Graphics", or even a "3D Pivot" in the future without changing a
  single line of game logic.

## 4. Conclusion

The refactor has successfully moved the project from a "distributed monolith" to a "modular workspace." The next
frontier of the DDD journey is **purity**: removing the last traces of visual and infrastructure details from our core
game rules and turning our map loading into a truly generic entity-orchestration engine.
