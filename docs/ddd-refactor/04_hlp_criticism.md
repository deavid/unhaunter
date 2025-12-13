This is a **strong, architecturally sound roadmap**. It explicitly addresses the pain points we identified (False
Modularity, Infrastructure Leakage, Monolithism) and maps perfectly to SOLID and DDD principles.

However, as you move from theory to practice in Rust and Bevy, you are going to hit one massive wall that this plan
underestimates: **The Circular Dependency Trap in Phase 2.**

Here is my review, broken down by strengths, specific risks, and refinements.

---

### The Verdict

**Grade: A-** This plan will successfully decouple your architecture. If executed, `unhaunter` will become a robust,
extensible engine. The "minus" is solely because **Phase 2** is significantly harder than it looks due to how Bevy
queries work.

---

### Strengths of the Plan

1.  **Phase 1 (The Single Source of Truth):** Creating `unspatial` (better name than `uncommon`) is the "Key to the
    City." By making Position/Direction purely data types with _zero_ dependencies, you solve the DRY violation and
    create a neutral ground for all domains to communicate.
2.  **Phase 3 (The Abstraction):** Replacing `Handle<Image>` with `AssetKey` is pure **Clean Architecture**. It means
    your game rules can run in a headless environment (server, tests) without initializing a rendering context.
3.  **Phase 5 (The Registry):** This is the **Open/Closed Principle** in action. By allowing plugins to _register_
    themselves with the map loader, you never have to touch `unmapload` code to add a new entity type. This is how you
    get your "Ghost Game Engine."

---

### The Hidden Monster: Phase 2 & The Cycle Trap

**The Scenario:**

- You move `PlayerSprite` to `unplayer`.
- You move `GhostSprite` to `unghost`.
- **The Problem:** The Ghost AI system (in `unghost`) needs to know where the Player is to hunt them. It runs a query:
  `Query<&Position, With<PlayerSprite>>`.
- **The Error:** `unghost` now must import `PlayerSprite` from `unplayer`.
- Later, the Player needs to know if a Ghost is nearby to react (sanity drain). `unplayer` must import `GhostSprite`
  from `unghost`.
- **Result:** `unghost` depends on `unplayer`, and `unplayer` depends on `unghost`. **Rust compiler panic.**

**The Solution (Add to Phase 2):** You need an **Interaction/Interface Layer** or use **Tag Components** in a shared
space.

**Refined Strategy for Phase 2:**

1.  **Option A (The Shared Tags):** Keep specific interaction tags in a shared crate (e.g., `un-tags` or
    `uncore-types`). `PlayerTag` and `GhostTag` live there. The rich data components (`PlayerState`, `GhostAI`) move to
    their domain crates. Systems query on `With<PlayerTag>` (shared) but process `GhostAI` (private).
2.  **Option B (The Spatial Query):** The Ghost AI shouldn't care about "Players." It should care about "Targets."
    - Create a `Target` component in `unspatial` or `uninteraction`.
    - `unplayer` attaches `Target` to the player entity.
    - `unghost` queries for `Position` and `Target`.
    - **Result:** `unghost` never needs to depend on `unplayer`.

### Specific Refinements per Phase

#### Phase 1: Naming Matters

Don't call it `uncommon`. "Common" is a magnet for "stuff I don't know where to put," and it will become the new
`uncore-components`.

- **Recommendation:** Call it `unspatial` if it's just math/position. Call it `unprimitives` if it includes basic ID
  types. Keep it strict.

#### Phase 3: The View System

When you abstract `GearSpriteID` out of the core, you need a way to reconnect it.

- **Recommendation:** Implement a **View Synchronizer System**.
  - Logic Layer: Player picks up `GearID::EMF`.
  - View Layer System: Detects `Added<GearID>`, looks up `EMF` in a `AssetDictionary` resource, and attaches a
    `SpriteBundle`.
  - This keeps logic and visuals completely decoupled (Reactive UI pattern).

#### Phase 5: The Registry Implementation

Implementing a registry in Bevy can be tricky because generic traits with Systems are hard.

- **Recommendation:** Use Bevy's `OneShotSystems` or a simpler **Factory Pattern**.
  - Define a trait `MapEntitySpawner`.
  - Register `Box<dyn MapEntitySpawner>` in a Resource.
  - `unmapload` iterates through the resource to find a spawner that claims the Tiled Class string.

### Summary Checklist for Success

1.  [ ] **Strict Rule:** `unspatial` must depend on nothing (except `bevy_math` perhaps).
2.  [ ] **Strict Rule:** Domain crates (`unghost`, `unplayer`) **must not** depend on each other. They interact via
        `unspatial`, `uncore-events`, or a lightweight `un-tags` crate.
3.  [ ] **Acceptance Test:** Can I compile `unghost` without `unplayer` existing? (Ideally, yes).

This plan is solid. Proceed with caution on the dependency graphs in Phase 2, and you will achieve your goal.
