# Crate Analysis: `unmaphub`

## Core Responsibilities

`unmaphub` manages the "Map Hub" or "Mission Control" phase of the game, specifically focusing on the user interface and
logic for selecting a mission's difficulty. It acts as the transitional state between the main menu and the actual
gameplay (or map loading).

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterMapHubPlugin`**: The plugin that initializes the `MapHubState` and sets up the difficulty selection
    screen.

- **`difficulty_selection` Module**:
  - **`DifficultySelectionItem`**: A component used to tag UI elements (buttons) with a specific `Difficulty` level.
  - **Systems**: Handles user input (clicks, selection) on the difficulty menu and updates the `CurrentDifficulty`
    resource.

## Dependencies

- `uncore-foundation`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncoremenu`
- `undifficulty`
- `bevy`
- `bevy_platform`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is well-scoped to the "Map Hub" context. It delegates the actual
  UI widget construction to `uncoremenu`, keeping its own logic focused on the _behavior_ of the difficulty selection
  screen.
- **Modularity:** It uses `MapHubState` to manage its internal flow (e.g., `DifficultySelection`), which allows for
  future expansion (e.g., adding a "Mission Selection" screen before or after difficulty).

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** The crate is almost entirely UI logic (menus, buttons, text).
  - **Camera2d:** It uses a `Camera2d` for the menu, which is the standard way to render UI in Bevy, regardless of
    whether the main game is 2D or 3D.
  - **No Spatial Logic:** It does not interact with the game board, physics, or rendering pipeline.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic**.
- **Reasoning:** The specific flow of selecting a difficulty and the rules for what difficulties are available are
  specific to _Unhaunter_.

### Other Notes

- **Integration:** It serves as a consumer of `uncoremenu`, demonstrating how the shared menu components are intended to
  be used.
