# Crate Analysis: `unsettings`

## Core Responsibilities

The `unsettings` crate manages the global configuration of the game. It handles:

- **Configuration Storage**: Defining and persisting settings for Audio, Controls, Game, Profile, and Video.
- **Persistence**: Saving and loading settings using `bevy-persistent`.
- **Data Structures**: Providing enums and structs for various configuration options (e.g., `WindowSize`,
  `AspectRatio`).

## Architectural Analysis

- **Type**: Configuration Crate
- **Role**: Central repository for user preferences.
- **Dependencies**:
  - `bevy-persistent`: For saving/loading settings.
  - `serde`: For serialization.
  - `enum-iterator`, `strum`: For iterating over configuration options.

## 2D/3D Coupling

- **None**: The settings defined (window size, UI scale, volume, etc.) are generic and applicable to both 2D and 3D
  contexts.

## Migration Risks

- **None**: This crate can be used as-is in the 3D version.

## Key Files

- `src/video.rs`: Defines video settings (resolution, aspect ratio, UI scale).
- `src/audio.rs`: Defines audio settings (volume).
- `src/controls.rs`: Defines control bindings (implied).
