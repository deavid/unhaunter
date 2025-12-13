# Crate Analysis: `unwalkiecore`

## Core Responsibilities

The `unwalkiecore` crate provides the core definitions and resources for the Walkie Talkie system. It handles:

- **Event Definitions**: Defining `WalkieTalkingEvent` and `WalkieEvent` for communication between systems.
- **Resource Management**: Managing `WalkiePlay` and `WalkieSoundState` to track the state of audio playback.
- **Traits**: Defining `ConceptTrait` for generated voice lines.

## Architectural Analysis

- **Type**: Core/Utility Crate
- **Role**: Provides the backbone for the walkie-talkie functionality, separating data and events from the
  implementation in `unwalkie`.
- **Dependencies**:
  - `unwalkie_types`: For shared types.
  - `uncore-foundation`, `uncore-types`: For base types.
  - `bevy`: For events and resources.

## 2D/3D Coupling

- **None**: This crate is purely logic and data-driven. It defines events and resources that are agnostic to the
  rendering dimension.

## Migration Risks

- **None**: This crate can be used as-is in the 3D version.

## Key Files

- `src/events.rs`: Defines the events used to trigger walkie-talkie lines.
- `src/resources.rs`: Defines the resources for tracking playback state.
