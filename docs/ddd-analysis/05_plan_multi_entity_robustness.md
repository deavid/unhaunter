# Refactor Plan: Multi-Entity Robustness (.single() elimination)

**Date:** 2026-01-30
**Target:** `unghost-plugin`, `unplayer-plugin`, `unwalkie-plugin`
**Goal:** Replace brittle `.single()` calls with robust iterators to support multiple players and ghosts, preventing crashes in multiplayer or complex hauntings.

## Context
The codebase contains many systems that assume a single Player or Ghost. While functionally correct for the current single-player tutorial, these assumptions are "technical debt" that will cause crashes as soon as a second player (multiplayer) or second ghost (complex haunting) is added.

## Step-by-Step Instructions for Copilot

### Phase 1: Ghost Interaction System (GIS)
1.  **MODIFY** `crates/unghost-plugin/src/systems/gis/selection.rs`:
    *   Change `ghost_interaction_selection_system` to iterate over `q_ghost.iter()`.
    *   The personality and rage logic should be calculated per-ghost inside the loop.
    *   If an interaction is emitted, use `return` to stop processing further ghosts in the same frame (maintains anti-spam behavior).

### Phase 2: Player Waypoint & Movement Visuals
1.  **MODIFY** `crates/unplayer-plugin/src/systems/waypoint.rs`:
    *   Refactor `waypoint_creation_system` and `waypoint_following_system` to use `iter()` and `iter_mut()` instead of `single()` and `single_mut()`.
    *   Ensure that waypoints created by a click are associated with the `MainPlayer` entity that processed the click (associative iteration).
2.  **MODIFY** `crates/unplayer-plugin/src/systems/walk_target_indicator.rs`:
    *   Refactor `manage_walk_target_indicator` to support multiple indicators (one per player).
    *   *Approach:* The indicator should probably be a child of the player or have a `Owner(Entity)` component to track which player it belongs to.
    *   *Simplified Fix:* If multiple `MainPlayer`s have targets, ensure the system iterates and doesn't crash, even if it only shows one indicator for now (using `.iter().next()`).

### Phase 3: Advisor Triggers (Walkie)
1.  **BULK MODIFY** `crates/unwalkie-plugin/src/triggers/*.rs`:
    *   The goal is to eliminate `qp.single()` and `q_ghost.single()`.
    *   **Rule:** For systems checking if "The Player" did something (e.g., forgot equipment), iterate over `qp.iter()` with `With<MainPlayer>`.
    *   **Rule:** For systems checking "The Ghost" state (e.g., ghost is hunting), iterate over `q_ghost.iter()`.
    *   *Preferred Pattern:* Replace `let Ok(...) = q.single()` with `for (...) in q.iter() { ... }`.

### Phase 4: Validation
1.  Run `cargo check` across the workspace.
2.  Run the game and verify that keyboard and mouse movement still work as expected.
3.  Verify that advisor voice lines still trigger during gameplay.
