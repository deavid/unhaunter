# Crate Analysis: `unmenu-plugin`

## Bounded Context

**Menu Interaction Logic**

This crate implements the active behavior for the project's UI menus, such as hover effects, scrolling, and keyboard
navigation.

## Responsibility

- Implement the `UnhaunterCoreMenuPlugin`.
- Handle mouse and keyboard interactions for `unmenu-core` components.
- Manage scrolling logic for `ScrollContainer` widgets.
- Trigger visual state changes (colors, borders) on hovered or selected menu items.

## Dependencies Audit

- `unmenu-core`: The definition layer for components and events.
- `unfoundation-core`: For UI theme colors.
- `untypes-core`: To contextualize navigation events within the game's state machine.
- `bevy`: For UI layout and input handling.

## Encapsulation Assessment

- **Privacy**: High. Interaction systems are private.
- **Separation**: Successfully separates "What a menu is" (`unmenu-core`) from "How it behaves" (`unmenu-plugin`).

## Semantic & Infrastructure Leakage

- **Engine Dependency**: Completely tied to Bevy's UI and input systems.
- **Style Injection**: Implementation systems directly apply colors from `unfoundation-core`, blending "interaction
  logic" with "visual styling".

## Future Recommendations

- **Abstract Styles**: Use a "Menu Style" resource to drive visual updates, allowing the interaction plugin to remain
  purely about _state changes_ (e.g., "Set this item to Focused"), while a separate styling system handles the _visual
  application_ (e.g., "Set background to Blue").
