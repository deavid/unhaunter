# Crate Analysis: `uncoremenu`

## Core Responsibilities

`uncoremenu` acts as a **UI Component Library** for the _Unhaunter_ game. It provides reusable Bevy components, systems,
and templates for building consistent user interfaces across different game states (Main Menu, Map Hub, Settings, etc.).
It standardizes the look and feel of menus and handles common interaction patterns like mouse hovering, clicking, and
keyboard navigation.

## Public API

The crate exposes:

- **`components` Module**:

  - **`MenuItemInteractive`**: Marks an entity as a clickable/selectable menu item.
  - **`MenuRoot`**: Manages the state of a menu (e.g., which item is selected).
  - **`MenuMouseTracker`**: Prevents accidental hover selection when a menu first opens.
  - **Visual Markers**: `MenuBackground`, `MenuContentArea`, `PrincipalMenuText`, etc., for styling.

- **`templates` Module**:

  - **`create_background`**: Spawns the standard menu background image.
  - **`create_logo`**: Spawns the game logo.
  - **`create_menu_strip`**: Helper to create a vertical list of menu items.
  - **`create_window`**: Helper to create a window-like container.

- **`systems` Module**:

  - **`menu_interaction_system`**: Handles mouse hover and click events, emitting `MenuItemClicked` and
    `MenuItemSelected`.
  - **`keyboard_navigation_system`**: Handles arrow key navigation.
  - **`update_menu_item_visuals`**: Updates text color and size based on selection state.

- **`events` Module**:

  - **`MenuItemClicked`**, **`MenuItemSelected`**, **`MenuEscapeEvent`**: Standard events for menu interactions.

- **`scrollbar` Module**:
  - **`ScrollableListContainer`**: A reusable component for creating scrollable lists (used in `uncampaign`).

## Dependencies

- `uncore-foundation`
- `uncore-types`
- `uncore-resources`
- `unprofile`
- `bevy`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** The crate focuses purely on "Menu UI Primitives". It doesn't know _what_
  the menus do (e.g., starting the game), only _how_ they look and behave.
- **DRY (Don't Repeat Yourself):** This crate is a direct answer to the DRY principle. Instead of re-implementing button
  logic and styling in every crate (`unmenu`, `unmaphub`, `unsettings`), they all consume these shared components.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **UI-Centric:** It deals exclusively with Bevy UI nodes (`Node`, `ImageNode`, `Text`).
  - **Agnostic:** The UI overlay is independent of the underlying 2D or 3D game world.

### Game Logic vs. Engine Logic

- **Conclusion:** **Engine/UI Framework**.
- **Reasoning:** This is effectively a mini "UI Framework" built on top of Bevy UI specifically for _Unhaunter_. It
  defines the visual language of the game but not the game rules.

### Other Notes

- **Standardization:** The use of `templates` ensures that all menus in the game have a consistent visual style (colors,
  fonts, layout).
- **Input Handling:** Centralizing input handling (mouse vs. keyboard) here ensures consistent navigation behavior
  across the entire application.
