# Crate Analysis: `unmenu`

## Core Responsibilities

`unmenu` implements the **Main Menu** of the game. It is the entry point for the user after the application launches.
Its responsibilities include:

- **UI Construction:** Building the main menu screen using components from `uncoremenu`.
- **Navigation:** Handling transitions to other states:
  - Campaign Mode (`AppState::MapHub` with `MissionSelectMode::Campaign`)
  - Custom Mission (`AppState::MapHub` with `MissionSelectMode::Custom`)
  - Manual (`AppState::Manual`)
  - Settings (`AppState::Settings`)
  - Quit (`AppExit`)

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterMenuPlugin`**: Registers the main menu systems.

- **`mainmenu` Module**:
  - **`app_setup`**: Registers the systems for entering/exiting the `AppState::MainMenu` state.
  - **`MenuID` Enum**: Defines the available options in the main menu.

## Dependencies

- `uncore-foundation`
- `uncore-types`
- `uncore-resources`
- `unsettings`
- `uncoremenu`
- `unprofile`
- `bevy`
- `bevy-persistent`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is strictly focused on the "Main Menu". It delegates the actual
  UI styling to `uncoremenu` and the settings logic to `unsettings`.
- **Dependency Inversion:** It depends on `uncoremenu` for UI primitives, which is a good layering.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** Like other menu crates, it deals with 2D UI nodes.
  - **State Transitions:** Its primary output is changing the `AppState`, which is abstract.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic**.
- **Reasoning:** The specific options available in the main menu (Campaign vs. Custom, etc.) are specific to
  _Unhaunter_.

### Other Notes

- **Simplicity:** This is a very thin crate, which is good. It orchestrates the main menu flow without getting bogged
  down in implementation details.
