# Crate Analysis: `untruck`

## Core Responsibilities

The `untruck` crate manages the "Truck" interface, which serves as the player's base of operations during a mission. It
handles:

- **UI Management**: Rendering the Truck UI, including tabs for Journal, Loadout, Sanity, and Sensors.
- **Gameplay Mechanics**:
  - **Journal**: Recording evidence and guessing the ghost type.
  - **Loadout**: Selecting and equipping gear.
  - **Status Monitoring**: Displaying player sanity and sensor readings.
  - **Mission Control**: Ending the mission or crafting repellents.

## Architectural Analysis

- **Type**: UI/Feature Crate
- **Role**: Provides the main interface for non-exploration gameplay.
- **Dependencies**:
  - `uncore_components`: For UI components (`TruckUI`, `TruckTab`).
  - `uncore_resources`: For game state and assets.
  - `undifficulty`: For difficulty-based UI defaults.
  - `ungear`, `ungearitems`: For inventory management.
  - `unwalkiecore`: For communication features (implied).

## 2D/3D Coupling

- **UI-Based**: The crate is almost entirely focused on 2D UI using Bevy's UI system.
- **No 3D Logic**: There is no evidence of 3D rendering or spatial logic within the truck interface itself. It acts as
  an overlay or a separate screen.

## Migration Risks

- **None**: The UI is independent of the 3D world and can be reused as-is.

## Key Files

- `src/ui.rs`: Main UI setup and layout.
- `src/journal.rs`: Logic for the journal and ghost guessing.
- `src/loadoutui.rs`: UI for gear selection.
