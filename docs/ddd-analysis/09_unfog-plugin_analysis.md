# Crate Analysis: `unfog-plugin`

## Bounded Context

**Environmental Dynamics (Miasma/Fog)**

This crate manages the "Miasma" system, which is a physical simulation of a corruption/fog field that flows through the
map.

## Responsibility

- Implement the `UnhaunterFogPlugin`.
- Calculate the diffusion and flow of miasma (fog) across the board.
- Spawn and animate fog particles (`MiasmaSprite`).
- Factor in visibility and lighting to determine fog density and color.
- Interface with `unboard-core` to read collision and room data.

## Dependencies Audit

- `unfog-core`: Data structures and components for fog.
- `unboard-core`: For the spatial grid and rooms.
- `unrender-std`: For sprite and animation components.
- `noise`: Used for procedural fog movement.
- `ndarray`: Used for the physical simulation of the field.

## Encapsulation Assessment

- **Privacy**: The plugin uses `pub(crate)` for internal systems and only exposes the `Plugin` itself.
- **Logic**: Simulation logic is well-contained within this plugin.

## Semantic & Infrastructure Leakage

- **Feature Awareness**: `unboard-core` stores a `MiasmaField` index, which means the "Core Board" is aware of the "Fog"
  feature. This is a form of semantic leakage where a basic infrastructure crate knows about a specific gameplay
  mechanic.
- **Visuals**: The plugin handles sprite spawning directly, which couples it to the specific `GameSprite` and
  `SpriteCVO` definitions.

## Future Recommendations

- **Registry / Modular Fields**: Move toward a system where `unboard-core` provides generic "Field" slots (floating
  point grids), and `unfog-plugin` registers its miasma field into one of those slots. This would remove the hardcoded
  "Miasma" reference from the board core.
- **Abstract Particles**: Consider using a more generic particle system if many plugins start spawning their own
  sprites.
