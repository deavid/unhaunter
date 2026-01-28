# Early Refactors for Multiplayer Readiness - Round 9

This document identifies specific code patterns in the current codebase that should be refactored **today**. These
changes are considered "Technical Debt" that will actively block multiplayer implementation but are also beneficial for
the current single-player structure (e.g., supporting multiple characters or better testability).

## 1. The `.single()` Consolidation

**Problem**: Numerous systems in `unplayer-plugin`, `unwalkie-plugin`, and `unghost-plugin` use `query.single()` to
access the player. This will cause an immediate crash if more than one player entity exists.

### Priority Refactors

- **Advisor Triggers**: All systems in [crates/unwalkie-plugin/src/triggers/](crates/unwalkie-plugin/src/triggers/)
  should be refactored to use `query.iter()`.
- **Ghost Tracking**: The Ghost's target selection in
  [crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs) must handle a list of players instead of a
  single entity.
- **Sanity/Health**: Systems like `hunger_system` and `sanity_drain_system` should iterate over all entities with
  `PlayerSprite`.

**Benefit**: Prevents crashes and allows for "Follower" NPCs or multiple player characters to exist in the world
simultaneously.

## 2. Component-Based Input

**Problem**: `PlayerInput` is currently a global `Resource`. This makes it impossible to distinguish between inputs for
Player 1 and Player 2 (or a remote player).

### Refactor Plan

1. **New Component**: Create `unplayer_core::components::PlayerInput`.
2. **Source Systems**: Update the keyboard/mouse/waypoint systems in
    [crates/unplayer-plugin/src/systems/input/](crates/unplayer-plugin/src/systems/input/) to write to the `PlayerInput`
    component on an entity marked with `MainPlayer`.
3. **Consumer Systems**: Update `player_movement_system` in
    [crates/unplayer-plugin/src/systems/movement.rs](crates/unplayer-plugin/src/systems/movement.rs) to read the
    component from the entity it is currently processing.

**Benefit**: Enables local split-screen (if ever desired) and is the foundation for networking (where remote inputs
populate the component).

## 3. Decoupling Logic from the Advisor

**Problem**: Many gameplay triggers are tightly coupled to the Advisor's `WalkiePlay` resource.

### Refactor Plan

- **Events**: Systems should emit generic gameplay events (e.g., `PlayerLowSanityEvent`) instead of calling
  `walkie_play.set()`.
- **Listeners**: The `unwalkie-plugin` should listen for these events and decide whether to play a voice line based on
  the local player's context.

**Benefit**: Separates "Simulation" (Player is crazy) from "Presentation" (Advisor talks about it). In multiplayer, the
Host can broadcast the event, and each client decides if their Advisor should speak.

## 4. Multi-Viewer Visibility

**Problem**: `VisibilityData` is calculated for a single hardcoded ID.

### Refactor Plan

- Refactor [crates/unlight-plugin/src/maplight.rs](crates/unlight-plugin/src/maplight.rs) to maintain a pool of
  `VisibilityBuffers`, one for each entity with a `Viewer` component.
- The rendering system should blend these or select the `Viewer` associated with the `MainPlayer` for the local camera.

**Benefit**: Allows the ghost to "know" which players can see it, and enables spectator cameras or security camera
features.

## 5. Localized RNG for Ghost AI

**Problem**: The Ghost uses `random_seed::rng()` which is global and non-deterministic.

### Refactor Plan

- Add a `GhostRng(StdRng)` component to the Ghost entity.
- Seed this RNG once from the `GameConfig` seed at spawn time.
- All AI decisions (where to walk, when to hunt) use this component's RNG.

**Benefit**: Makes the ghost's behavior deterministic relative to its initial seed, which significantly reduces
"teleporting" corrections in multiplayer snapshots.

## 6. Shared Resource Refactor: `HauntState`

**Problem**: `HauntState` is a global resource that tracks "The Ghost" even if multiple ghosts were added.

### Refactor Plan

- Move fields like `ghost_warning_intensity` and `ghost_warning_position` into components on the Ghost entity itself.
- Re-implement `HauntState` (or a similar resource) as a view-model that aggregates data from all ghosts for the UI.

**Benefit**: Robustness against future updates and easier replication (snapshotting an entity is easier than
snapshotting a global resource).
