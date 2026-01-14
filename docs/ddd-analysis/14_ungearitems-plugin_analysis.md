# Crate Analysis: `ungearitems-plugin`

## Bounded Context

**Gameplay Equipment Logic (Specific Items)**

This crate implements the active behavior of every tool in the game. It is the logic engine for thermometers, EMF
readers, motion sensors, etc.

## Responsibility

- Implement the `UnhaunterGearItemsPlugin`.
- Provide the Bevy systems that update item states (e.g., reading temperatures from the board, detecting ghost proximity
  for EMF).
- Implement global gear mechanics like battery drainage and Electronic Magnetic Interference (EMI).
- Register all items with the `GearSpawnerRegistry`.

## Dependencies Audit

- `ungearitems-core`: For item data structures.
- `unboard-core`: To read environmental data (temperature, light).
- `unghost-core`: To detect ghost location/state.
- `unspatial-core`: For distance calculations.
- `bevy`: Deeply integrated ECS logic.

## Encapsulation Assessment

- **Privacy**: High. Implementation modules are private.
- **Complexity**: This is a "Glue" crate. It depends on almost all core systems to make the gear interactive.

## Semantic & Infrastructure Leakage

- **Logic Leakage**: Some gameplay math (e.g., how EMI affects specific meters) is mixed with Bevy system boilerplate.
- **Engine Dependency**: Irreplaceable Bevy coupling.

## Future Recommendations

- **Pure Logic Extraction**: Move complex math (EMI calculations, temperature smoothing) into pure functions or methods
  on the core data structs to make them testable without an `App` instance.
- **Modular Gear**: As the number of items grows, consider if some items (e.g., "Motion Sensor") can be generalized into
  a "Spatial Trigger" category.
