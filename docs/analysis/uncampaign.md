# Crate Analysis: `uncampaign`

## Core Responsibilities

`uncampaign` is responsible for the "Mission Selection" phase of the game. Despite its name suggesting a focus only on
"Campaign" mode, it actually implements a **Unified Mission Selection System** that handles both:

- **Campaign Mode:** Progression-based mission unlocking.
- **Custom Mode:** Free selection of available maps.

It provides the user interface for browsing maps, filtering them by mode and player level, displaying progress
(badges/grades), and initiating the level load.

## Public API

The crate exposes:

- **`plugin` Module**:

  - **`UnhaunterCampaignPlugin`**: The plugin that initializes the mission selection system.

- **`unified_mission_selection` Module**:
  - **`app_setup`**: Registers the systems for the mission selection UI.
  - **UI Components**: Various marker components for the UI (`MissionSelectUI`, `MissionSelectCamera`, etc.).
  - **Logic**: Systems for filtering maps, handling user input (clicks, navigation), and transitioning to the game
    state.

## Dependencies

- `uncore-foundation`
- `uncore-assets`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncoremenu`
- `unprofile` (for player progress/save data)
- `unmaphub` (for badge utilities and flow integration)
- `undifficulty`
- `bevy`
- `bevy-persistent`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate is focused on "Mission Selection". It aggregates data from
  `unprofile` (progress) and `uncore-resources` (maps) to present a choice to the user.
- **Open/Closed Principle:** The unified selection system is designed to handle different modes (`MissionSelectMode`)
  without duplicating the UI logic, which is a good improvement over previous separate implementations mentioned in the
  code comments.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** Like `unmaphub`, this is almost entirely UI logic.
  - **No Spatial Logic:** It deals with metadata about maps (names, descriptions, completion status), not the maps
    themselves.
  - **Camera:** Uses `Camera2d` for UI rendering.

### Game Logic vs. Engine Logic

- **Conclusion:** **Game Logic**.
- **Reasoning:** The rules for which missions are unlocked, how they are sorted, and the flow from selection to gameplay
  are specific to _Unhaunter_.

### Other Notes

- **Integration:** It sits between the Main Menu/Map Hub and the actual Game Load.
- **Refactoring History:** The code explicitly mentions it replaces a previous separate implementation, indicating a
  successful refactor towards better code reuse.
