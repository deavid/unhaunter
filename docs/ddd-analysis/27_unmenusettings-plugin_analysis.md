# Crate Analysis: `unmenusettings-plugin`

## Bounded Context

**Settings UI**

This crate implements the visual interface for configuring the game's audio, gameplay, and video settings.

## Responsibility

- Implement the `UnhaunterMenuSettingsPlugin`.
- Construct the settings menu categories and value selectors.
- Translate UI changes (e.g., sliding a volume bar) into persistent setting updates.
- Manage internal navigation within the settings categories.

## Dependencies Audit

- `unsettings-core`: For the actual setting data types and persistence logic.
- `unmenu-core`: For UI templates and interaction markers.
- `untypes-core`: For global application states.
- `bevy`: Standard UI implementation.

## Encapsulation Assessment

- **Privacy**: High. Navigation state and layout generators are private.
- **Modularity**: Keeps the settings UI separate from the settings _storage_ (`unsettings-core`).

## Semantic & Infrastructure Leakage

- **Persistence Leakage**: Directly coordinates with `bevy-persistent` via `unsettings-core` to save changes.
- **Template Coupling**: Strictly tied to the `unmenu-core` template layout.

## Future Recommendations

- **Settings Registry**: If more plugins need to add their own settings, `unmenusettings-plugin` should provide a way
  for other plugins to register a "Setting Category" and "Setting Items" rather than hardcoding all categories (Audio,
  Gameplay) in one place.
- **Delayed Persistence**: Use an "Apply Changes" button pattern to avoid saving to disk on every slider movement,
  reducing I/O and potential UI stutter.
