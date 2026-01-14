# Crate Analysis: `unmanual-plugin`

## Bounded Context

**In-Game Documentation & Tutorial**

This crate manages the "Manual" – the player's in-game reference guide for ghosts, evidence, and equipment.

## Responsibility

- Implement the `UnhaunterManualPlugin`.
- Define the content of manual pages (text, layout, images).
- Provide the UI for navigating the manual (chapter list, page flipping).
- Manage the "Journal" and "Summary" views within the manual context.

## Dependencies Audit

- `unassets-core`: To load manual-specific images and fonts.
- `unfoundation-core`: For UI design tokens.
- `bevy`: For UI layout and state management.

## Encapsulation Assessment

- **Privacy**: Internal layout functions are private.
- **Content Coupling**: The manual content is hardcoded as Rust code. The "data" (text/images) and the "layout" (UI
  nodes) are one and the same.

## Semantic & Infrastructure Leakage

- **High Infrastructure Leakage**: Manual pages are defined as closures that directly spawn Bevy UI nodes. This means
  the tutorial content is completely locked to the engine's UI system.
- **Hardcoding**: To change a typo in the manual, one must recompile the entire project.

## Future Recommendations

- **Data-Driven Manual**: Move manual content to a separate `unmanual-core` (for schemas) and use external files (e.g.,
  Markdown or RON) to define the content.
- **Generic UI Renderer**: Implement a simple Markdown-to-Bevy-UI renderer to separate the "writer's content" from the
  "programmer's UI code".
