# High-Level Refactoring Plan: From Monolith to Modular

**Goal:** Transform `unhaunter` from a distributed monolith into a clean, domain-driven architecture where domains are
self-contained, core logic is isolated from presentation, and the engine is generic.

## Phase 1: Foundation & Primitives (The "Common" Layer)

**Objective:** Establish a clean, dependency-free foundation for shared types to resolve the "Position duplication"
architectural smell and provide a stable base.

1.  **Create `uncommon` (or `unspatial`):**
    - Extract `Position`, `Direction`, `BoardPosition` from `uncore-board`.
    - Extract generic math/utility functions.
    - **Constraint:** This crate must have minimal dependencies (ideally no Bevy, or only `bevy_math`).
2.  **Retarget Dependencies:**
    - Update `uncore-board`, `uncore-components`, and domain crates to depend on `uncommon`.
    - Remove the re-exports in `uncore-components` that masked the dependency flow.

## Phase 2: Shattering the "Uncore" Monolith

**Objective:** Dissolve `uncore-components` and `uncore-resources` by redistributing their contents to owning domains
(Bounded Contexts).

1.  **Audit & Categorize:**
    - Map every struct in `uncore-components` and `uncore-resources` to a specific domain: `Ghost`, `Player`, `Truck`,
      `Gear`, `Map`, `UI`, or `Gameplay`.
2.  **Migrate Components (Iterative):**
    - **Ghost Domain:** Move `GhostSprite`, `GhostBreach`, `GhostBehavior` -> `unghost`.
    - **Player Domain:** Move `PlayerSprite`, `Stamina` -> `unplayer`.
    - **Truck/UI Domain:** Move `TruckUI`, `TruckButton` -> `untruck` (or new `unui`).
    - **Rendering:** Move `Light`, `GameSprite` -> `unlight` or `unrender`.
3.  **Migrate Resources:**
    - Move `GhostGuess` -> `unghost` or `ungameplay`.
    - Move `MouseVisibility` -> `uninput`.
    - Move `BoardData` -> `uncore-board` (if it belongs to the board logic) or `unmap`.
4.  **Decommission:**
    - Delete `uncore-components` and `uncore-resources` once empty.

## Phase 3: Purifying the Core (Clean Architecture)

**Objective:** Decouple Game Logic from Rendering and Infrastructure.

1.  **Abstract Assets:**
    - Remove `Handle<Image>` and `TextureAtlasLayout` from `uncore-types` (specifically `ImageAssets`).
    - Replace with logical `AssetKey` or `ResourceID` enums in the core.
2.  **Abstract Presentation:**
    - Remove `GearSpriteID` and other visual indices from core logic types.
    - **Implement Presentation Layer:** Create a system (e.g., in `unassets` or `unrender`) that maps logical IDs (e.g.,
      `EMFReader`) to their visual representation (Sprite Handle + Atlas Index).

## Phase 4: Interface Segregation (SOLID)

**Objective:** Break "God Traits" and decouple logic from view in traits.

1.  **Refactor `GearUsable`:**
    - Split into `GearLogic` (update, trigger, state, power) and `GearPresentation` (display name, description, sprite).
    - `GearLogic` remains in `ungear`.
    - `GearPresentation` moves to a UI/View layer.
2.  **Trait Audit:**
    - Review other traits for similar mixing of concerns (e.g., Ghost behavior traits) and apply ISP.

## Phase 5: Engine Decoupling

**Objective:** Make `unmapload` and other systems generic and reusable.

1.  **Generic Spawners (Registry Pattern):**
    - Refactor `unmapload` to remove hard dependencies on `GhostSprite` or `PlayerSprite`.
    - Implement a "Spawner Registry" where domain crates (`unghost`, `unplayer`) register their own spawning functions
      for specific Tiled object types.
    - `unmapload` becomes a pure parser/coordinator.
2.  **Decouple `unlight`:**
    - Ensure lighting logic depends only on generic `Position` and `LightSource` components.
    - Remove any logic branching based on specific `GhostType` or game state.

## Phase 6: Final Polish & Verification

**Objective:** Ensure the new architecture is robust and strictly layered.

1.  **Dependency Graph Analysis:**
    - Verify a strict DAG (Directed Acyclic Graph).
    - Ensure `uncommon` is at the bottom.
    - Ensure `unhaunter` (main executable) is the only place tying everything together.
2.  **Testability Verification:**
    - Write a unit test for a core game mechanic (e.g., Ghost movement or Gear trigger) that runs in pure Rust without
      spinning up a Bevy App or loading assets.
