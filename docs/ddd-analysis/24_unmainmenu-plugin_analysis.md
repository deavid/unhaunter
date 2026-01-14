# Crate Analysis: `unmainmenu-plugin`

## Bounded Context

**User Interface / Top-Level Navigation**

This crate manages the entry point of the game: the Main Menu.

## Responsibility

- Implement the `UnhaunterMenuPlugin`.
- Construct the main menu UI layout.
- Handle high-level navigation transitions (Start Game, Settings, Quit).
- Manage the intro music (playback and volume fading).
- Trigger player profile level updates on menu load.

## Dependencies Audit

- `untypes-core`: For global `AppState`.
- `unmenu-core`: For standard button and menu UI templates.
- `unsounds-core`: For music assets.
- `unprofile-core`: To check and update player levels.
- `bevy`: Standard UI implementation.

## Encapsulation Assessment

- **Privacy**: Internal systems are encapsulated.
- **Complexity**: Low. Mostly glue logic between the UI and game states.

## Semantic & Infrastructure Leakage

- **Storage Leakage**: Directly triggers player profile persistence. This couples the UI to the I/O/Storage system.
- **Audio Hardcoding**: The logic for fading the menu music is specific to this plugin.

## Future Recommendations

- **Decouple Persistence**: Use an event or a dedicated "Profile Manager" system to handle persistence, so the Menu only
  needs to signal a "Ready" state.
- **Shared Audio Services**: Move song-fading logic to a more generic audio plugin to allow other menus (or transitions)
  to reuse it.
