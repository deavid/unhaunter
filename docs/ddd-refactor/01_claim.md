This is a much healthier constraint. By removing the 3D pivot requirement, we can look at the architecture for what it
_is_, rather than what it _might become_.

Analyzing `unhaunter` purely through the lens of rigorous software engineering principles reveals several deep
structural "smells" that, while functional now, will eventually calcify the project, making it hard to maintain, test,
or extend.

Here is the deep-dive analysis:

### 1. The "Uncore" Monolith (Anti-DDD)

**The Problem:** You have effectively built a distributed monolith disguised as microservices (crates).

In Domain-Driven Design (DDD), you aim for **Bounded Contexts**. `unplayer`, `unghost`, and `untruck` look like Bounded
Contexts, which is good. However, the `uncore-*` ecosystem completely undermines this.

- **Shared Kernel Abuse:** In DDD, a "Shared Kernel" (`uncore-foundation`) should be tiny—only the absolute minimum
  shared language (e.g., `Difficulty`, `Time`).
- **The Violation:** `uncore-components` and `uncore-resources` are not a Kernel; they are a **Global State Dump**.
  - Because `GhostSprite` (Ghost Domain) and `TruckUI` (UI Domain) live in the same crate (`uncore-components`), you
    cannot compile the Ghost AI without also compiling the Truck UI.
  - **Consequence:** You have tight coupling between unrelated domains. If you change a field in the `TruckUI`
    component, the Ghost AI system has to recompile. This violates the **Common Closure Principle** (components that
    change for different reasons should be separated).

**Recommendation:** Shatter `uncore-components`. The `GhostSprite` component belongs in `unghost`. The `PlayerSprite`
belongs in `unplayer`. Only truly generic primitives (like `Position`) belong in a core crate.

### 2. Clean Architecture Violations (Infrastructure Leakage)

**The Problem:** The "Entities" (your data models) know too much about the "Details" (rendering and assets).

In Clean Architecture, the inner layers (Game Rules/Entities) should not know about the outer layers (UI, Database,
Rendering Engine).

- **Leakage 1: Visuals in Domain Data.**
  - `uncore-types` defines `GearSpriteID`. This is an Enum mapping items to _spritesheet indices_. This is a purely
    visual presentation detail leaking into the core type definition of an item.
  - `ImageAssets` in `uncore-types` holds `Handle<Image>`. Your core types are tightly coupled to the asset management
    implementation.
- **Leakage 2: Input in Domain Entities.**
  - `PlayerSprite` in `uncore-components` depends on `ControlKeys` from `unsettings`. Your player entity definition is
    coupled to the configuration infrastructure.
- **Consequence:** You cannot test your game logic without mocking up the asset loader or the settings system. Your
  "Game Rules" are polluted with "View" concerns.

**Recommendation:** Use "IDs" or "Keys" in your core types (e.g., `GearID`), and have a separate Presentation Layer
(system) map those logical IDs to `SpriteID`s or `Handle<Image>`.

### 3. SOLID Analysis: The "S" and "D" are in trouble

#### S - Single Responsibility Principle (SRP)

- **The Offender:** `uncore-components` and `uncore-resources`.
- **The Analysis:** These crates have _no_ defined responsibility other than "holding things." `uncore-resources` holds
  `BoardData` (Physics/World), `MouseVisibility` (UI Input), and `GhostGuess` (Gameplay State).
- **The Risk:** These files become "magnet classes" (or magnet crates). Developers will dump anything new into them
  because it's convenient, leading to low cohesion. `unstd` is also developing this trait as a "junk drawer" for
  systems.

#### D - Dependency Inversion Principle (DIP)

- **The Offender:** `uncore-board` vs. `uncore-components`.
- **The Analysis:** You have a circular logic dependency resolved by code duplication (the `Position` struct).
  - High-level gameplay (`unplayer`) depends on `uncore-components`.
  - `uncore-components` depends on `uncore-board` (for `BoardPosition`).
  - But `uncore-board` contains the logic for the board, which arguably should be the lowest level.
- **The Violation:** The duplication of `Position` and `Direction` in both crates is a desperate attempt to satisfy the
  borrow checker or dependency graph without fixing the architectural layering.
- **The Fix:** Define `Position` in a tiny, zero-dependency crate (e.g., `unmath` or `unspatial`) that both
  `uncore-board` and `uncore-components` depend on.

#### I - Interface Segregation Principle (ISP)

- **The Offender:** `GearUsable` trait (`ungear`).
- **The Analysis:** The trait includes methods for _everything_: `get_display_name` (UI), `set_trigger` (Input),
  `update` (Logic), and `get_sprite_idx` (Rendering).
- **The Risk:** If you create an item that is invisible (has no sprite) or has no display name (internal tool), you are
  forced to implement dummy methods for UI/Rendering.
- **The Fix:** Split the trait. `GearLogic` (update/trigger) vs. `GearPresentation` (name/sprite).

### 4. Semantic Coupling (The "Ghost Engine" Fallacy)

You mentioned wanting to build a "Ghost Game Engine." Currently, your "Engine" code (`unlight`, `unfog`, `unmapload`) is
semantically coupled to your "Game" code (`unhaunter`-specific logic).

- **Example:** `unlight` is supposed to be a lighting engine. However, if it differentiates logic based on `GhostType`
  or specific game components (which `unlight` imports), it ceases to be an engine and becomes hardcoded game logic.
- **Observation:** `unmapload` parses Tiled properties and immediately spawns `GhostSprite` or `PlayerSprite`. A true
  engine approach would spawn a generic `Actor` with a `Config` component, and a separate Game System would attach the
  `GhostSprite`.

### Summary of Hidden Debt

1.  **False Modularity:** You have many crates, but they are so tightly interwoven via the `uncore-*` dependencies that
    you effectively have a Monolith. You pay the compile-time cost of microservices without gaining the decoupling
    benefits.
2.  **Presentation/Logic Mixing:** Your game state cannot exist independently of your rendering strategy
    (Sprites/Tiled).
3.  **Ambiguous Truth:** The duplication of `Position` means you have two sources of truth for where things are. This is
    a bug waiting to happen if one system updates `uncore-board::Position` and another reads
    `uncore-components::Position`.

**The most impactful refactor (disregarding 3D) would be:** Dissolve `uncore-components` and `uncore-resources`. Move
components to the crates that _own_ the logic (Context-driven package structure). Create a `uncommon` crate only for
primitives used by 90% of the app (like `Position`).
