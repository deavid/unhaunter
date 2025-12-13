# Crate Analysis: `unmenusettings`

## Core Responsibilities

`unmenusettings` implements the **Settings Menu** UI. It allows the user to configure various game options (Audio,
Gameplay, etc.). It acts as the view/controller layer for the configuration data defined in `unsettings`.

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterMenuSettingsPlugin`**: Registers the settings menu state (`SettingsState`) and systems.

- **`menus` Module**:

  - **Enums**: Defines the structure of the settings menu (`MenuSettingsLevel1`, `AudioSettingsMenu`, etc.) using
    `strum` for iteration and display.

- **`menu_ui` Module**:

  - **UI Construction**: Builds the settings screens using `uncoremenu` templates.

- **`systems` Module**:
  - **Event Handling**: Processes user input (changing volume, toggling controls) and updates the persistent settings.

## Dependencies

- `uncore-foundation`
- `uncore-types`
- `uncore-resources`
- `unsettings`
- `uncoremenu`
- `bevy`
- `bevy_platform`
- `bevy-persistent`
- `strum`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is focused on the "Settings UI". It doesn't define _what_ the
  settings are (that's `unsettings`), only how to display and edit them.
- **Interface Segregation:** It breaks down the settings into sub-menus (Audio, Gameplay), keeping the logic manageable.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** Like `unmenu`, it's all 2D UI nodes.
  - **Configuration:** It manipulates abstract configuration values (volume floats, boolean toggles) that are
    independent of the rendering dimension.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic / UI**.
- **Reasoning:** While settings menus are common in engines, the specific options (e.g., "Camera Controls", "Movement
  Style") are specific to _Unhaunter_.

### Other Notes

- **Persistence:** It interacts with `bevy-persistent` (via `unsettings`) to ensure changes are saved to disk.
- **Strum:** The use of `strum` to iterate over enum variants to generate menu items is a clean and maintainable
  pattern.
