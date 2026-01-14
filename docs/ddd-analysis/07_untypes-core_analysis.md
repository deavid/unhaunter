# Crate Analysis: `untypes-core`

## Bounded Context

**Global Game Lifecycle & Configuration**

This crate handles the state machine of the application and the top-level orchestration states (Main Menu, Loading,
Gameplay).

## Responsibility

- Define `AppState` and `GameState` enums for the entire project.
- Provide definitions for higher-level game rules like `Difficulty` (tutorial vs. challenge).
- Act as the central hub for coordination states that every plugin needs to know about.

## Dependencies Audit

- `bevy`: For the `States` trait and reflection.
- `serde`: For serialization of states and configs.
- `enum-iterator`: Used to iterate through difficulty levels in the UI.

## Encapsulation Assessment

- **Privacy**: Public enums and data structures.
- **Logic**: Minimal helper logic (e.g., state transition checks).

## Semantic & Infrastructure Leakage

- **Engine Leakage**: Tied to Bevy states.
- **Semantic Leakage**: Very low; it keeps the naming close to its purpose in the lifecycle.

## Future Recommendations

- **Refinement**: Ensure that only "Project Global" states live here. Feature-specific states (e.g.,
  `GhostSpecificState`) should live in their respective core crates.
- **Documentation**: Keep the `Difficulty` logic documented as it serves as the prototype for future
  mission-parameterization features.
