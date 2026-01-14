# Crate Analysis: `ungearitems-core`

## Bounded Context

**Gear Equipment State (Specific Items)**

This crate defines the specific data structures and internal states for the various tools used in the game (Flashlight,
EMF Reader, Thermometer, etc.).

## Responsibility

- Define the `Component` structs for specific items (e.g., `EMFReader`, `Thermometer`).
- Store item-specific configuration and status (e.g., current reading, battery level).
- Provide helper methods for internal state transitions (e.g., switching modes).

## Dependencies Audit

- `unfoundation-core`: For domain constants (e.g., temperature).
- `bevy`: Standard ECS component traits.
- `enum-iterator`: To handle item modes.

## Encapsulation Assessment

- **Privacy**: High. Each item is modularized.
- **Data over Logic**: Follows the core pattern of being a data repository for specific tools.

## Semantic & Infrastructure Leakage

- **Cleanliness**: Very high. It is almost pure domain data.
- **Engine Coupling**: Standard Bevy component coupling.

## Future Recommendations

- **Template System**: If the number of items grows significantly, consider a more generic "Item Property" pattern, but
  for now, the explicit per-item structs provide excellent type safety.
