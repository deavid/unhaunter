# DDD Analysis Summary #2: Ghosts, UI, and Navigation

## Overview

This second batch of analysis covered the core gameplay entities (Ghosts), the UI infrastructure (Menus), and the basic
orchestration of level loading and navigation.

## Key Findings

### 1. Architectural Successes

- **Modular UI**: The `unmenu-core` and `unmenu-plugin` split effectively separates UI static structure from interaction
  logic.
- **Hub-Centric Flow**: The use of specialized plugins for `unmaphub`, `unmainmenu`, and `unmapload` keeps the
  transition logic contained and manageable.
- **Unified Telemetry**: `unmetrics-core` provides a very clean, thread-safe way for any part of the engine to report
  performance without fighting Bevy's ECS borrow checker.

### 2. Significant Semantic Leakage

- **Hardcoded Content**: Both `unmanual-plugin` and `unmainmenu-plugin` have their content (text, layout, logic)
  hardcoded in Rust. This creates high friction for non-programmers to update game content.
- **Logic in Core Crates**: This batch revealed several `-core` crates that contain significant logic:
  - `uninteraction-core`: Handles sound emitters and material manipulation.
  - `unnavigation-core`: Contains the collision avoidance math.
  - `unmetrics-core`: Contains the drain system logic. _This suggests the "Core vs Plugin" rule needs a refinement: Core
    should be for Data/Types/Interfaces, while Logic should strictly be in Plugins._
- **Cross-Domain Coupling**: `unnavigation-core` is aware of "Interactions," and `unlight-plugin` is aware of "Ambient
  Audio."

### 3. "Ghost Engine" Portability Score

- **Ghost System**: 6/10 (Smart behavior, but high visual/asset coupling).
- **UI System**: 7/10 (Clean architecture but engine-locked).
- **Navigation**: 5/10 (Heavily tied to the board's specific grid-based collision map).

## Strategic Recommendations

- **Purge Logic from Core**: Move `CollisionHandler` and `InteractiveStuff` logic into dedicated plugins to restore the
  "Data-Only" purity of core crates.
- **Abstract Navigation**: The navigation system should work on a generic "Obstacle Map" rather than the specific
  `BoardData` collision field to allow for more diverse world representations.
- **Factory Extraction**: Take the "Entity Recipes" out of `unmapload-plugin` and move them into domain-specific
  factories (e.g., `PlayerFactory`, `GhostFactory`) to allow for better encapsulation of what constitutes a "Ghost".
