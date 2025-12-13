# Phase 2 Status: Shattering the Monolith - Stability Reached

**Date:** December 13, 2025 **Status:** Stability Reached (Compiles)

## 1. Current Status

We have successfully reached a compilable state for Phase 2. The monolithic structure has been broken down into
domain-specific crates, and the circular dependencies that arose during this process have been resolved.

### Key Achievements

- **Domain Segregation:** Core logic is now distributed across `unghost`, `unplayer`, `ungear`, `untruck`, etc.
- **Cycle Resolution:** We implemented a "Shared Kernel" pattern using `uncore-components` to house data structures
  shared between domains that otherwise caused circular dependencies.
- **Decoupling via Resources:** The `PlayerState` resource in `uncore-resources` now acts as a bridge, allowing the
  Ghost AI to read player data (health, sanity, position, hiding status) without a direct dependency on the full
  `unplayer` crate logic.
- **Tag-Based Queries:** We are utilizing `untags` (e.g., `PlayerTag`, `GhostTag`) to query entities without needing
  access to specific component definitions in all systems.

### The "Shared Kernel" (`uncore-components`)

To break the `unplayer` <-> `ungear` <-> `unghost` cycles, the following components were moved to `uncore-components`:

- **`player_shared`**: `PlayerSprite`, `HeldObject`, `Inventory`.
- **`ghost_shared`**: `GhostSprite`, `GhostBehaviorDynamics`.

This allows `unplayer` to define player logic, `unghost` to define ghost logic, and `ungear` (and items) to interact
with both without creating dependency loops.

## 2. Remaining Tasks for Phase 2 Closure

While the code compiles, a few cleanup tasks remain to ensure the codebase is clean before fully moving to Phase 3.

### A. Warning Cleanup

The refactoring left behind several unused imports and variables.

- **Action:** Run `cargo fix --lib` or manually address warnings in:
  - `uncore-systems` (unused `player_tag`)
  - `ungear` (unused imports, unnecessary parentheses)
  - `unplayer` (unused `HeldObject` import)
  - `unghost` (unused `OrderedFloat`, `PlayerTag` re-import)

### B. Runtime Verification

We have heavily modified imports and some logic paths (specifically in `quartz.rs` and `ghost.rs`).

- **Action:** Launch the game and verify:
  - **Ghost Hunting:** Does the ghost still hunt correctly? (Logic in `ghost.rs` was touched).
  - **Quartz Usage:** Does the Quartz stone still repel the ghost? (Logic in `quartz.rs` was significantly updated to
    fix types).
  - **Player Spawning:** Does the player spawn with the correct sprite and gear?

### C. Dependency Audit (Optional)

- **Action:** Check `Cargo.toml` files in domain crates. Some might still list dependencies they no longer use after the
  moves.

## 3. Next Steps: Phase 3 (Event-Driven Architecture)

With the domains physically separated, we can now focus on logical decoupling using events.

- **Objective:** Replace direct system-to-system calls or tight coupling with Bevy Events.
- **Focus:**
  - Ghost interactions (e.g., `GhostSensedEvent`, `PlayerSpottedEvent`).
  - Gear triggers (e.g., `GearActivatedEvent`).
  - Sound and UI decoupling.

## 4. Manual Intervention Required?

If you are proceeding manually due to AI limits:

1.  **Fix Warnings:** The compiler warnings are your best guide to cleaning up the debris from the refactor.
2.  **Test:** Play the game. If `quartz` crashes or the ghost stands still, check `ungearitems/src/components/quartz.rs`
    and `unghost/src/ghost.rs`.
