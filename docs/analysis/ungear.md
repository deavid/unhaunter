# Crate Analysis: `ungear`

## Core Responsibilities

`ungear` defines the fundamental architecture for the equipment system in _Unhaunter_. It provides the traits and
structures that allow different items (EMF readers, flashlights, thermometers) to be treated uniformly by the game
engine. It does _not_ appear to contain the specific implementations of these items (which are likely in `ungearitems`),
but rather the "shape" of what gear is.

## Public API

The crate exposes:

- **`gear_usable` Module**:

  - **`GearUsable` Trait**: The central interface that all equipment must implement. It defines methods for:
    - Display info (`get_display_name`, `get_description`, `get_status`).
    - Interaction (`set_trigger`).
    - Update loop (`update`).
    - Visuals (`get_sprite_idx`, `power`, `color`).

- **`types` Module**:

  - **`Gear` Struct**: A container that holds a `GearKind` (enum) and a `Box<dyn GearUsable>` (the implementation). This
    allows the ECS to store "any gear" in a single component.

- **`components` Module**:
  - **`PlayerGear`**: A component attached to the player entity that holds their inventory (left hand, right hand,
    inventory slots).

## Dependencies

- `undifficulty`
- `uncore-foundation`
- `uncore-board`
- `uncore-events`
- `uncore-types`
- `uncore-resources`
- `uncore-components`
- `unsettings`
- `unprofile`
- `bevy`
- `bevy-persistent`

## Architectural Analysis & Notes

### SOLID Principles

- **Open/Closed Principle:** The `GearUsable` trait is a textbook example of OCP. New gear types can be added by
  creating a new struct that implements `GearUsable` without modifying the core player or inventory systems that consume
  the trait.
- **Dependency Inversion:** The high-level game logic (in `ungame`) depends on the `GearUsable` abstraction defined
  here, not on concrete gear implementations.

### 2D/3D Coupling

- **Conclusion:** **Low Coupling.**
- **Reasoning:**
  - **Trait-Based:** The `GearUsable` trait is mostly logical.
  - **Visuals:** `get_sprite_idx` returns a `GearSpriteID` (likely an enum or index), which is an abstraction over the
    specific visual asset. While currently used for 2D sprites, this ID could map to a 3D model in a different renderer.
  - **Position:** The `update` method takes a `Position` (from `uncore-board`), which is the main point of coupling to
    the spatial system.

### Game Logic vs. Engine Logic

- **Conclusion:** **Engine Feature (Inventory/Item System)**.
- **Reasoning:** This crate defines the _system_ for items, not the items themselves. It's a reusable framework for
  handling "usable objects" in a game.

### Other Notes

- **Polymorphism:** The use of `Box<dyn GearUsable>` allows for dynamic dispatch, enabling a flexible inventory system
  where any slot can hold any item.
