# Crate Analysis: `unsummary`

## Core Responsibilities

The `unsummary` crate handles the end-of-mission summary screen. It manages:

- **UI Display**: Rendering the summary screen using Bevy UI (2D).
- **Data Presentation**: Displaying mission results, time taken, and rewards.
- **State Management**: Handling the transition to and from the summary state.

## Architectural Analysis

- **Type**: UI/Feature Crate
- **Role**: Presents the results of the gameplay session.
- **Dependencies**:
  - `uncore_components`: For UI components (`SummaryUI`, `SCamera`) and `PlayerSprite`.
  - `uncore_resources`: For `SummaryData` and game states.
  - `undifficulty`: For difficulty context.
  - `unprofile`: For updating player progress.

## 2D/3D Coupling

- **UI-Based**: The crate primarily uses Bevy's UI system, which is 2D. This is expected and compatible with a 3D game
  (as an overlay).
- **Asset Usage**: It references `PlayerSprite`, likely to display the player character on the summary screen. This
  might need adjustment if the 3D model should be shown instead, or if the 2D sprite is sufficient for the UI.

## Migration Risks

- **Low**: The UI can largely remain as-is. The main consideration is whether to replace 2D character sprites with 3D
  models in the summary view.

## Key Files

- `src/summary.rs`: Contains the setup, cleanup, and update logic for the summary screen.
