# Crate Analysis: `unmenu-core`

## Bounded Context

**Menu Infrastructure & Templates**

This crate provides the building blocks for all menus in the game. It defines the shared vocabulary of the UI system.

## Responsibility

- Define base UI components (`MenuRoot`, `MenuItemInteractive`).
- Provide shared UI events (`MenuItemClicked`, `MenuEscapeEvent`).
- Implement reusable UI templates (standard buttons, column layouts).
- Manage complex generic widgets like `ScrollContainer`.

## Dependencies Audit

- `unassets-core`: For UI textures and fonts.
- `unfoundation-core`: For design tokens (colors, scales).
- `untypes-core`: For shared states used in UI logic.
- `bevy`: Deeply integrated with the UI node system.

## Encapsulation Assessment

- **Privacy**: High. Templates and scaffolding functions are public for reuse.
- **Pattern**: Uses a "Scaffolding" pattern where core crates provide helper functions to build UI trees.

## Semantic & Infrastructure Leakage

- **Engine Coupling**: Irreplaceable link to Bevy's `Node` and `UI` systems.
- **Domain Coupling**: Includes templates that directly reference `unprofile-core` (player badges/levels), which makes
  the generic menu system aware of specific gameplay progression concepts.

## Future Recommendations

- **Purge Domain data from Templates**: Refactor templates to take generic "Text" and "Icons" rather than taking a whole
  `PlayerProfile`. This would keep `unmenu-core` a pure UI library.
- **Component-Based Styling**: Move hardcoded color and style constants to a separate "Theme" resource to allow for
  easier skinning of the UI.
