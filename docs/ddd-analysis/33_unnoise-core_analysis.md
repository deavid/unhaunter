# Crate Analysis: `unnoise-core`

## Bounded Context

**Infrastructure / Procedural Generation**

This crate provides precomputed noise tables for use by systems requiring deterministic randomness (e.g., ghost
movement, miasma flow).

## Responsibility

- Initialize and store a large precomputed 2D noise lookup table.
- Provide high-performance access to noise values via world-space coordinates.

## Dependencies Audit

- `bevy`: For resource management.
- `noise`: For the underlying Perlin noise implementation.

## Encapsulation Assessment

- **Privacy**: The internal storage is private; the coordinate mapping logic is hidden within the `Noise` resource
  methods.
- **Role**: This is a utility/infrastructure library.

## Semantic & Infrastructure Leakage

- **Logic in Core**: **Significant Violation.** This crate contains heavy initialization logic and coordinate mapping
  math within the `-core` crate. Per the "Data-Only" goal, the logic for generating the table should reside in a plugin,
  while the core should only define the data structure.
- **Gameplay Constants**: The crate defines frequencies like `SANITY_REGAIN_FREQ`. This couples a generic "Noise
  Utility" to specific gameplay mechanics from other systems (Ghost/Sanity).
- **Resource Footprint**: The resource hardcodes a 16-million-element table allocation. This infrastructure side-effect
  is fixed in a "core" crate, preventing modular adjustment based on hardware or engine needs.

## Future Recommendations

- **Plugin Migration**: Move the noise generation logic to an `unnoise-plugin`. The `unnoise-core` should only contain
  the storage buffer and basic accessor types.
- **Parameterization**: Instead of hardcoding constants (frequencies, table size), allow these to be configured via a
  resource or an external config file.
- **Domain Decoupling**: Move gameplay-specific frequencies (`SANITY_REGAIN_FREQ`) to their respective domain crates
  (`unghost-core` or `unplayer-core`).
