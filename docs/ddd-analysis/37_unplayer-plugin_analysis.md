# DDD Analysis: `unplayer-plugin`

## 1. Bounded Context

The **Player Gameplay Mechanics** context. This is the "Orchestrator" for everything the player _does_ within the game
world.

## 2. Responsibility

- **System Orchestration**: Connects input systems (Keyboard/Mouse) to movement and interaction logic.
- **Gameplay Simulation**: Simulates sanity drain based on environment (light, sound, temperature), handles health
  recovery, and manages the lifecycle of world-space indicators (Walk targets).
- **Interaction Logic**: Implements the mechanics of hiding, stair navigation, and gear manipulation (grab/drop).

## 3. Dependencies and Appropriateness

- **High Sensitivity**: Depends on 15+ internal crates. This is appropriate for a high-level gameplay plugin but makes
  it a "God Plugin" that is difficult to test in isolation.
- **Missing Boundaries**: It reaches directly into `unui-core` components (e.g., `FearIndicator`) to trigger visual
  effects, rather than using an event-driven or decoupled approach.

## 4. Encapsulation (Data/Logic Split)

- **Logic Placement**: While systems like `sanity_drain_system` properly calculate complex environmental effects, they
  fall back on calling logic methods inside the core crate (like `StaminaStat::tick`).
- **Recommendation**: The logic currently residing in `StaminaStat` and `Health/Sanity` (in `-core`) should be pulled
  into systems within `unplayer-plugin`.

## 5. Semantic & Infrastructure Leakage

- **Visual Leakage**: `player_movement` and `player_animation` are tightly coupled to `unrender-std` (animations) and
  `unui-core` (UI colors/elements). A change in how animations are handled (infrastructure) would break the movement
  logic.
- **Domain Coupling**: `unplayer-plugin` correctly consumes `unboard-core` to check light and sound fields, which is
  appropriate as the player exists within the board's domain.

---

## Technical Debt & Strategic Notes

- **Direct UI Coupling**: Use events (e.g., `DamageEvent`) instead of modifying `FearIndicator` directly to decouple
  gameplay from UI.
- **Rigid Animation**: Decouple animation state changes from movement logic using a `MovementState` component.
- **Glue Status**: This is the primary "glue" plugin that connects environmental data (Board) to player physiology. It
  should be the only place where these concepts meet.
