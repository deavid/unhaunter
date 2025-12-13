# Crate Analysis: `unwalkie`

## Core Responsibilities

The `unwalkie` crate implements the gameplay logic for the Walkie Talkie system. It handles:

- **Event Processing**: Listening for `WalkieTalkingEvent` to trigger game effects.
- **Visual Feedback**: Managing "Focus Rings" on ghosts and breaches to highlight them during specific events.
- **Stats**: Tracking walkie-talkie usage statistics (implied by module names).

## Architectural Analysis

- **Type**: Feature Crate
- **Role**: Implements specific gameplay interactions related to the walkie-talkie.
- **Dependencies**:
  - `unwalkiecore`: For events and core types.
  - `uncore_components`: For `FocusRing`, `GhostSprite`, `GhostBreach`.
  - `bevy`: For systems and components.

## 2D/3D Coupling

- **Visuals**: The `focus_ring_system` explicitly updates `Sprite` components, which are 2D. This visual effect will
  need to be adapted for 3D (e.g., using a 3D mesh or particle effect instead of a sprite).
- **Spatial Logic**: The focus ring is attached to `GhostSprite` and `GhostBreach` entities, which are currently 2D
  entities.

## Migration Risks

- **Visual Effects**: The focus ring effect is 2D-specific and will need to be reimplemented for the 3D view.

## Key Files

- `src/focus_ring_system.rs`: Manages the visual focus ring effect.
- `src/walkie_play.rs`: Likely handles the audio playback logic.
