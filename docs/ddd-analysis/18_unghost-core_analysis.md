# Crate Analysis: `unghost-core`

## Bounded Context

**Ghost Logic & State Context**

This crate manages the identity, behavioral state, and environmental presence of the ghost. It is the "source of truth"
for what the ghost currently is and what it is doing.

## Responsibility

- Define the `GhostSprite` component (logical state: rage, position, target, hunting status).
- Manage the `Haunt` resource (tracking evidences found, hunting progress).
- Implement the "Evidence Processing" logic (how evidence clarity accumulates over time).
- Define the `GhostInfluence` component to allow external entities to affect the ghost.

## Dependencies Audit

- `unfoundation-core`: For ghost types and evidence enums.
- `unspatial-core`: For world-space coordinates and distances.
- `bevy`: Essential ECS traits.

## Encapsulation Assessment

- **Privacy**: High.
- **Terminology**: The name `GhostSprite` is a significant case of semantic leakage into the name itself; it is a
  logical state component, not a sprite.

## Semantic & Infrastructure Leakage

- **Naming Leakage**: `GhostSprite` suggests a visual entity but contains pure logic (rage, repellent hits). This
  reflects a history where logic and visuals were interleaved.
- **Engine Coupling**: Standard Bevy coupling for time and ECS.

## Future Recommendations

- **Rename GhostSprite**: Rename to `GhostState` or `GhostLogic` to clearly state its role as a data-only component.
- **Evidence Decentralization**: Instead of hard-coding evidence accumulation in `unghost-core`, consider an "Evidence
  Provider" trait to allow gear or other systems to contribute to evidence acquisition more modularly.
