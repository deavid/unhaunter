# Crate Analysis: `unmaphub-plugin`

## Bounded Context

**Pre-Game Hub & Mission Selection**

This crate manages the transition between the Main Menu and the Game, specifically the "Map Hub" where players choose
their difficulty and finalize their mission parameters.

## Responsibility

- Implement the `UnhaunterMapHubPlugin`.
- Manage the Hub UI (difficulty selection list, map previews).
- Coordinate the transition between the `MainMenu` and `Loading` states.
- Filter valid difficulties for the current map selection.

## Dependencies Audit

- `untypes-core`: For application states and difficulty types.
- `undifficulty-core`: To read difficulty details.
- `unui-core` / `unmenu-core`: For standard UI templates and button interactions.
- `bevy`: Standard UI implementation.

## Encapsulation Assessment

- **Privacy**: Internal UI modules are private.
- **Glue Logic**: Acts as a high-level orchestrator of states and UI components.

## Semantic & Infrastructure Leakage

- **Flow Control**: Hardcodes the intended game flow (e.g., "From here, go to MissionSelection").
- **UI Scaling**: Direct dependency on `unfoundation-core` design tokens and `bevy_platform` timing for debouncing
  inputs.

## Future Recommendations

- **Generic State Flow**: Instead of hardcoding the next state, use an event-driven flow where the "Map Hub" simply
  publishes a "Mission Configured" event, allowing a global orchestrator to decide where to navigate next.
- **Decouple Timing**: Use Bevy's built-in `Time` or an input-action system instead of `bevy_platform` specific timing
  directly in the UI.
