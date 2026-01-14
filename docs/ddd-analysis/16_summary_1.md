# DDD Analysis Summary #1: Foundation, Board, and Gear

## Overview

This first batch of analysis covered the foundation of the project, asset management, the physical board representation,
and the equipment (gear) system.

## Key Findings

### 1. Architectural Strengths

- **Core vs. Plugin Isolation**: The separation between data-only `-core` crates and logic-heavy `-plugin` crates is
  consistently applied. This provides a clear "ground truth" for domain objects.
- **Shared Kernel**: `unfoundation-core` and `untypes-core` provide a robust shared vocabulary.
- **Registry Pattern**: `ungear-core` successfully implements a registry for gear spawning, allowing new tools to be
  added without modifying the core crate's internal switch statements. This is a model for engine-level modularity.

### 2. Identified Semantic Leakage

- **Content-Heavy Assets**: `unassets-core` is currently a maintenance bottleneck. It "knows" about every manual page,
  UI icon, and specific asset path. It is more of a "Content Manifest" than a generic "Asset Infrastructure" crate.
- **Field Awareness in Board**: `unboard-core` has "Miasma" (Fog) specific indices and "Ghost Attraction" properties.
  This couples the physical representation of the world to specific gameplay features. Ideally, the board should handle
  generic data grids (Fields) and semantic tags, leaving the interpretation to the plugins.
- **Technical Orchestration**: `ungame-plugin` handles low-level technical rebuilds (collision/light fields). This
  forces the gameplay layer to understand rendering implementation details.

### 3. "Ghost Engine" Portability Score

- **Foundation/Types**: 9/10 (Highly generic).
- **Board/Spatial**: 7/10 (Strong, but needs to shed feature-specific properties).
- **Gear/Interaction**: 8/10 (Registry system is excellent, but needs to decouple from the global "GearStuff"
  god-parameter).

## Strategic Recommendations

- **Asset Decentralization**: Move toward a system where plugins register their own required assets into a central
  manager, rather than having a central `GameAssets` struct.
- **Generic Field Registry**: Modify `unboard-core` to provide a registry for arbitrary floating-point grids (Fields),
  allowing `unfog-plugin` to register its "Miasma" field without hard-coding it into the board.
- **Event-Driven Technical Updates**: Shift technical rebuilds (collision/lighting) to be reactive (triggered by
  `unboard-core` events) rather than imperative (manually called by `ungame-plugin`).
