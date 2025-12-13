# Architectural Review & Verification

**Date:** December 13, 2025 **Reviewer:** GitHub Copilot (Gemini 3 Pro) **Subject:** Codebase Structural Analysis &
Verification of Architectural Critique

## Executive Summary

I have performed a deep-dive investigation of the `unhaunter` codebase to verify the architectural claims made regarding
DDD violations, Clean Architecture breaches, and SOLID principle adherence.

**Verdict:** The critique is **largely accurate**. The codebase exhibits significant structural debt that mimics a
distributed monolith. The "Uncore" crates act as global state dumps, coupling unrelated domains. However, the specific
claim about "Position duplication" was found to be partially incorrect due to re-exports, though the underlying
dependency layering remains problematic.

---

## Detailed Findings

### 1. The "Uncore" Monolith (Anti-DDD)

**Status:** 🔴 **CONFIRMED**

- **Observation:** `crates/uncore-components/src/components/` acts as a "Global State Dump" rather than a Shared Kernel.
- **Evidence:** The crate contains tightly coupled components from completely unrelated domains:
  - `ghost_sprite.rs` (Ghost Domain)
  - `truck_ui.rs` (UI Domain)
  - `player_sprite.rs` (Player Domain)
  - `light.rs` (Rendering Domain)
  - `evidence_provider.rs` (Gameplay Domain)
- **Impact:** This structure forces tight coupling. A change in a UI component within `uncore-components` could
  theoretically trigger recompilation of Ghost AI logic, violating the Common Closure Principle.

### 2. Clean Architecture Violations (Infrastructure Leakage)

**Status:** 🔴 **CONFIRMED**

- **Observation:** Core entities and types are polluted with presentation and infrastructure details.
- **Evidence:**
  - `crates/uncore-types/src/types/gear/spriteid.rs`: Defines `GearSpriteID`, leaking visual implementation details
    (spritesheet indices) into the core type system.
  - `crates/uncore-types/src/types/root/image_assets.rs`: Defines `ImageAssets` holding `Handle<Image>` (Bevy-specific
    asset handles). This couples the core type definitions directly to the asset loading infrastructure.
- **Impact:** Core business logic cannot be tested or reused in isolation without mocking the entire rendering and asset
  infrastructure.

### 3. SOLID Analysis

#### S - Single Responsibility Principle (SRP)

**Status:** 🔴 **CONFIRMED**

- **Observation:** `uncore-components` and `uncore-resources` lack a defined responsibility.
- **Evidence:** These crates serve as "magnet crates" where any new state or component is dumped regardless of its
  domain context.

#### I - Interface Segregation Principle (ISP)

**Status:** 🔴 **CONFIRMED**

- **Observation:** The `GearUsable` trait is a "God Trait".
- **Evidence:** `crates/ungear/src/gear_usable.rs` forces implementors to define methods for disparate concerns:
  - `get_display_name` (UI)
  - `set_trigger` (Input/Logic)
  - `get_sprite_idx` (Rendering)
  - `power` (Flashlight specific logic)

#### D - Dependency Inversion Principle (DIP) & Position Duplication

**Status:** 🟡 **PARTIALLY INCORRECT / MITIGATED**

- **Claim:** "The duplication of Position and Direction in both crates is a desperate attempt to satisfy the borrow
  checker..."
- **Investigation:** `crates/uncore-components/src/components/board/mod.rs` explicitly **re-exports** `Position` from
  `uncore-board`:
  ```rust
  // Re-export spatial types from uncore-board (single source of truth)
  pub use uncore_board::components::position::Position;
  ```
- **Correction:** There are _not_ two different `Position` structs acting as separate sources of truth. They are the
  same type.
- **Nuance:** While the "duplication" claim is technically false, the architectural critique that high-level
  `uncore-components` depends on low-level `uncore-board` (which contains logic) is structurally true.

### 4. Semantic Coupling (The "Ghost Engine" Fallacy)

**Status:** 🔴 **CONFIRMED**

- **Observation:** The map loader and "engine" systems are hardcoded to specific game entities.
- **Evidence:** `crates/unmapload/src/entity_spawning.rs` explicitly imports and spawns `GhostSprite`, `PlayerSprite`,
  and `PlayerGear`.
- **Impact:** `unmapload` is not a generic map loader; it is the `unhaunter` game loader. It cannot be reused for a
  different game without significant refactoring.

---

## Recommendations

1.  **Shatter `uncore-components`:**

    - Move `GhostSprite` -> `unghost`
    - Move `PlayerSprite` -> `unplayer`
    - Move `TruckUI` -> `untruck` (or `unui`)
    - Create a truly minimal `uncommon` crate for shared primitives like `Position` if needed, or keep them in
      `unboard`.

2.  **Refactor `GearUsable`:**

    - Split the trait into `GearLogic` (update, trigger) and `GearPresentation` (name, sprite).
    - This allows items to exist logically without needing visual representation code.

3.  **Purify `uncore-types`:**
    - Remove `Handle<Image>` and `SpriteID` from core types.
    - Use logical IDs (`GearID`, `AssetKey`) in the core, and map them to assets in a separate Presentation Layer.
