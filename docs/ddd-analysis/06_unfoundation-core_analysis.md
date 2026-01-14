# Crate Analysis: `unfoundation-core`

## Bounded Context

**Shared Kernel / Foundation**

This crate is the base layer of the entire project. It defines the fundamental terminology (enunms, types) and shared
utilities used by every other crate. It is the "lingua franca" of the Unhaunter workspace.

## Responsibility

- Define domain enums that are used across multiple contexts (e.g., `GhostType`, `Evidence`).
- Provide platform-agnostic utilities (WASM/Native wrappers).
- Centralize shared constants (UI color tokens, game scales).
- Implement serialization support and reflection for base types.

## Dependencies Audit

- `bevy`: For `Component` and reflection (`Reflect`) support.
- `serde`: For serialization.
- `rand`: Fundamental for any game needing random generation.
- `bevy_platform`: Path/WASm utilities.

## Encapsulation Assessment

- **Privacy**: High visibility (`pub`) as this is a library of shared primitives.
- **Logic**: Successfully maintains a "data-only" approach with minimal helper logic.

## Semantic & Infrastructure Leakage

- **Engine Leakage**: Deeply integrated with Bevy's reflection and component system.
- **UI Leakage**: Includes design tokens like colors and UI scales. While this centralizes the "Theme", it does couple
  the base foundation to a specific visual style.
- **Platform Leakage**: Centralized platform workarounds (WASM paths). This is a "controlled leak" to keep other crates
  clean.

## Future Recommendations

- **Theme Separation**: As the game matures, move UI design tokens into a `untheme-core` or similar to keep
  `unfoundation-core` purely about game-logic primitives.
- **Minimize Engine Surface**: Continue to favor standard library types or small utility crates where possible, using
  Bevy only where necessary for the ECS integration.
