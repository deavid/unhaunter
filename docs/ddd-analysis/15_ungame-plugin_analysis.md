# Crate Analysis: `ungame-plugin`

## Bounded Context

**In-Game Gameplay Orchestration**

This crate is the primary "Glue Plugin" for the active gameplay phase. It coordinates the interaction between players,
ghosts, environment, and gear.

## Responsibility

- Implement the `UnhaunterGamePlugin`.
- Orchestrate the gameplay loop (starting a mission, ending a mission).
- Implement high-level environmental rules (Fuse box/Power system).
- Manage the in-game HUD and pause menu.
- Coordinate "Technical Rebuilds" (e.g., updating collision fields when a door opens).

## Dependencies Audit

- Almost all `un*-core` crates.
- `unrender-std`: For updating simulation data structures.
- `bevy`: Deeply integrated.

## Encapsulation Assessment

- **Privacy**: High internal encapsulation (most systems are `pub(crate)`).
- **Complexity**: This is the most complex plugin as it acts as the integrator for the entire "Investigation"
  experience.

## Semantic & Infrastructure Leakage

- **Technical Knowledge**: The plugin knows about how the rendering engine stores collision data, as it explicitly
  triggers rebuilds of those fields.
- **Mixed Concerns**: It handles both abstract high-level rules (Ghost Evidence logic) and low-level technical updates
  (Field rebuilding).

## Future Recommendations

- **Domain Decoupling**: Move the "Evidence Logic" to a more specialized `unghost-plugin` or `unsummary-plugin` to keep
  `ungame-plugin` focused on the world-orchestration.
- **Reactive Rebuilds**: Instead of explicit rebuild calls, use an event-driven system or Bevy Observers where
  `unboard-core` changes trigger a technical rebuild in `unrender-plugin` automatically without `ungame-plugin` needing
  to know about it.
