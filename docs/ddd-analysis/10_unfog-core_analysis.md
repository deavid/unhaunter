# Crate Analysis: `unfog-core`

## Bounded Context

**Miasma/Atmospheric Fog (Data)**

This crate defines the data structures and components for the Miasma system.

## Responsibility

- Define the `MiasmaConfig` resource (simulation parameters).
- Define the `MiasmaSprite` component (per-particle simulation state).
- Expose the "schema" of the fog system for use by other crates (like `unboard-core` or `unfog-plugin`).

## Dependencies Audit

- `unspatial-core`: For world positioning.
- `bevy`: Standard ECS traits.

## Encapsulation Assessment

- **Privacy**: Fields are mostly public, acting as POD (Plain Old Data) structs.
- **Logic**: No logic; follows the core crate "data only" principle.

## Semantic & Infrastructure Leakage

- **Naming**: The crate is named "unfog" but the domain uses "Miasma". This is consistent but worth noting.
- **Visual Coupling**: The name `MiasmaSprite` implies a visual entity, but the struct contains physics data like
  `velocity` and `pressure`. This blends the "Simulation Entity" with the "Visual Entity".

## Future Recommendations

- **Rename to Simulation**: If the system becomes more physics-heavy and less about "fog pixels", consider renaming
  `MiasmaSprite` to something like `MiasmaParticle` to decouple from the "Sprite" (rendering) concept.
