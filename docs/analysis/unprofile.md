# Crate Analysis: `unprofile`

## Core Responsibilities

The `unprofile` crate is responsible for managing persistent player data. It handles:

- **Progression Tracking**: Storing and calculating player XP, level, and bank balance.
- **Persistence**: Saving and loading player profile data using `bevy-persistent`.
- **Data Structures**: Defining the schema for player profiles (`ProgressionData`).

## Architectural Analysis

- **Type**: Data/Utility Crate
- **Role**: Provides the persistent data model for the player's long-term progress.
- **Dependencies**:
  - `bevy-persistent`: For saving/loading data.
  - `serde`, `ron`: For serialization.
  - `uncore-foundation`, `uncore-types`: For shared types.

## 2D/3D Coupling

- **None**: This crate is purely data-driven and contains no rendering or spatial logic. It is completely agnostic to
  the game's dimensionality.

## Migration Risks

- **None**: This crate can be used as-is in the 3D version.

## Key Files

- `src/data.rs`: Defines `ProgressionData` and XP calculation logic.
- `src/plugin.rs`: Sets up the persistence plugin (implied).
