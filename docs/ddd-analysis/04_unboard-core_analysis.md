# Crate Analysis: `unboard-core`

## Bounded Context

**Physical Board & Environment Simulation**

This crate defines the physical structure of the game world. It is the "source of truth" for the 3D grid, handling
environmental properties like light, temperature, collisions, and the presence of rooms.

## Responsibility

- Define the `BoardData` resource, which manages a 3D grid of tiles.
- Store and manage environmental state (light levels, temperature fields).
- Map raw Tiled map layers to semantic floors and rooms.
- Provide spatial querying for rooms and tile properties.
- Define components for world entities like `Door`, `Switch`, and `Room`.

## Dependencies Audit

- `unfoundation-core` / `unspatial-core`: Basic types and utilities.
- `ndarray`: Used for high-performance 3D grid simulation.
- `bevy`: Standard ECS dependency.
- `tiled`: Used to bridge raw map data into the semantic board.

## Encapsulation Assessment

- **Privacy**: `BoardData` is a public resource, allowing wide access to the world state. Functional areas like `RoomDB`
  encapsulate search logic well.
- **Leakage**: Attempts to decouple from Tiled by mirroring data, but still relies on Tiled's layer-based organizational
  concepts.

## Semantic & Infrastructure Leakage

- **Visual Leakage**: The `Config` modules (e.g., `DoorConfig`) still contain properties related to visual
  representation (`SpriteCVOKey`), though they attempt to separate "definition" from "rendering".
- **Gameplay Leakage**: The crate includes level-design-specific logic, such as counting "ghost-attracting objects" in a
  room. This belongs more in a "Scenario" or "Level Logic" context than a "Physical Board" context.
- **Infrastructure Leakage**: The way floors are mapped to Z-indices is heavily tied to how Tiled organizes layers.

## Future Recommendations

- **Generic Map abstraction**: If the engine is to support other editors (or procedurally generated maps),
  `unboard-core` should define a clean interface for "Populating the Board" that doesn't reference Tiled types directly.
- **Scenario/Logic Split**: Move gameplay-specific room properties (like attraction counts or haunting state) to a
  separate `unscenario-core` to keep `unboard-core` focused on physics and environment.
- **Visual Decoupling**: Stricter separation of "Logical Entity" (Door) from its "Visual Component" (Sprite
  index/offset).
