# Component Inventory: Mapping Crates to Engine/Mod Concepts

## Purpose

This document maps the existing crate structure to our abstract understanding of "engine" vs "game mode" components.
This is **not** a refactoring plan. It's an inventory to support abstract thinking about where boundaries should be.

---

## Legend

| Category       | Meaning                                                  |
| -------------- | -------------------------------------------------------- |
| **Engine**     | Should exist regardless of game mode                     |
| **Game Mode**  | Specific to Unhaunter's current gameplay                 |
| **Mixed**      | Contains both; needs conceptual split                    |
| **Shared Mod** | Could be shared across game modes but isn't engine-level |
| **?**          | Unclear, needs discussion                                |

---

## The Crate Inventory

### Foundation Layer

| Crate               | Current Role                                            | Category   | Notes                       |
| ------------------- | ------------------------------------------------------- | ---------- | --------------------------- |
| `uncore-foundation` | Primitive types, constants, math                        | **Engine** | Pure foundation             |
| `unspatial`         | Spatial primitives (Position, Direction, BoardPosition) | **Engine** | Created in Phase 1 refactor |
| `untags`            | Marker components (PlayerTag, GhostTag)                 | **Engine** | Created in Phase 2 refactor |
| `uninteraction`     | Generic interaction primitives                          | **Engine** | Created in Phase 2 refactor |

### Board & Space

| Crate          | Current Role                       | Category  | Notes                                                                                                           |
| -------------- | ---------------------------------- | --------- | --------------------------------------------------------------------------------------------------------------- |
| `uncore-board` | Grid, behaviors, Tiled integration | **Mixed** | Grid = engine. `Behavior` system = engine. Specific behaviors (Door, Stairs) = could be game mode or shared mod |

### Simulation Layer

| Crate     | Current Role                  | Category   | Notes                                                         |
| --------- | ----------------------------- | ---------- | ------------------------------------------------------------- |
| `unlight` | Light propagation, visibility | **Engine** | Core simulation, tied to rendering                            |
| `unfog`   | Miasma fluid simulation       | **Engine** | Generic fluid sim, though "miasma" concept might be game mode |

### Type Definitions

| Crate               | Current Role                                    | Category  | Notes                                                                                                     |
| ------------------- | ----------------------------------------------- | --------- | --------------------------------------------------------------------------------------------------------- |
| `uncore-types`      | Grab bag of types (GearKind, MissionData, etc.) | **Mixed** | Some types are engine (field types), some are game mode (GhostType, Evidence, specific gear kinds)        |
| `uncore-events`     | Event definitions                               | **Mixed** | Generic events (LoadLevel, Sound) = engine. Specific events (GhostInteraction types, TruckUI) = game mode |
| `uncore-components` | ECS components dump                             | **Mixed** | Player/Ghost components mixed with spatial primitives. Analysis says this is a "God Component" crate      |
| `uncore-resources`  | Global state dump                               | **Mixed** | BoardData = engine. GhostGuess, WalkiePlay = game mode. Also called a "God Resource" crate                |

### Actor Layer

| Crate      | Current Role                               | Category      | Notes                                                                                   |
| ---------- | ------------------------------------------ | ------------- | --------------------------------------------------------------------------------------- |
| `unplayer` | Player controller, movement, sanity/health | **Mixed**     | Basic player (move, hold items) = engine. Sanity behavior, hiding mechanics = game mode |
| `unghost`  | Ghost AI, behaviors                        | **Game Mode** | This IS the Unhaunter-specific antagonist                                               |
| `unnpc`    | NPC interactions                           | **Game Mode** | Tutorial/helper NPCs are game content                                                   |

### Equipment Layer

| Crate         | Current Role                  | Category      | Notes                                          |
| ------------- | ----------------------------- | ------------- | ---------------------------------------------- |
| `ungear`      | Gear trait, inventory system  | **Engine**    | The _framework_ for equipment is engine-level  |
| `ungearitems` | Specific gear implementations | **Game Mode** | EMF reader, thermometer, etc. are game content |

### Campaign & Missions

| Crate          | Current Role                    | Category      | Notes                                                                   |
| -------------- | ------------------------------- | ------------- | ----------------------------------------------------------------------- |
| `uncampaign`   | Campaign/mission progression    | **Game Mode** | Mission structure is game-specific                                      |
| `undifficulty` | Difficulty settings and scaling | **Mixed**     | Difficulty _framework_ = engine. Specific difficulty params = game mode |

### Map Loading

| Crate       | Current Role              | Category      | Notes                                  |
| ----------- | ------------------------- | ------------- | -------------------------------------- |
| `unmapload` | Map loading orchestration | **Engine**    | Loading maps is engine-level           |
| `untmxmap`  | Tiled TMX parsing         | **Engine**    | TMX format is engine choice            |
| `unmaphub`  | Map selection UI          | **Game Mode** | How maps are selected is game-specific |

### UI Layer

| Crate            | Current Role                       | Category      | Notes                                                         |
| ---------------- | ---------------------------------- | ------------- | ------------------------------------------------------------- |
| `untruck`        | Truck interface (journal, loadout) | **Game Mode** | This is Unhaunter's specific base-of-operations UI            |
| `unmenu`         | In-game menus                      | **Mixed**     | Pause menu = engine. Game-specific options = game mode        |
| `uncoremenu`     | Main menu                          | **Mixed**     | Basic menu system = engine. Specific menu content = game mode |
| `unmenusettings` | Settings UI                        | **Engine**    | Settings management is generic                                |
| `unsummary`      | End-of-mission summary             | **Game Mode** | Scoring/summary is game-specific                              |

### Persistence

| Crate        | Current Role              | Category   | Notes                                                       |
| ------------ | ------------------------- | ---------- | ----------------------------------------------------------- |
| `unprofile`  | Player profile, save/load | **Mixed**  | Save/load _mechanism_ = engine. What gets saved = game mode |
| `unsettings` | Settings management       | **Engine** | Generic settings handling                                   |

### Communication

| Crate            | Current Role            | Category      | Notes                                       |
| ---------------- | ----------------------- | ------------- | ------------------------------------------- |
| `unwalkie`       | Walkie-talkie mechanics | **Game Mode** | Specific communication system for Unhaunter |
| `unwalkie_types` | Walkie type definitions | **Game Mode** | Part of walkie system                       |
| `unwalkiecore`   | Walkie core logic       | **Game Mode** | Part of walkie system                       |

### Utilities

| Crate            | Current Role             | Category   | Notes                                         |
| ---------------- | ------------------------ | ---------- | --------------------------------------------- |
| `unstd`          | Misc utilities           | **Mixed**  | Some utils are engine, some are game-specific |
| `uncore-systems` | Shared ECS systems       | **Mixed**  | Depends on what systems are in here           |
| `uncore-assets`  | Asset loading/management | **Engine** | Asset infrastructure is engine-level          |

### Top Level

| Crate       | Current Role                           | Category      | Notes                                |
| ----------- | -------------------------------------- | ------------- | ------------------------------------ |
| `ungame`    | Game assembly, main loop orchestration | **Mixed**     | The "glue" that connects everything  |
| `unhaunter` | Binary entry point                     | **Game Mode** | This IS the game built on the engine |

---

## Observations

### The "God Crates" Problem

The analysis docs explicitly call out:

- `uncore-components` — "God Component" crate with low cohesion
- `uncore-resources` — "God Resource" crate bundling unrelated global state
- `uncore-types` — "grab bag" of miscellaneous types

These are the main sources of mixing. They contain both engine-level and game-mode-level concepts dumped together.

### What's Clearly Engine

- Spatial primitives (Board, Position, Direction)
- Field framework and propagation (light, sound, fog)
- Actor existence and movement primitives
- Gear/inventory _framework_ (not specific items)
- Map loading infrastructure
- Settings and save/load _mechanisms_
- Asset loading

### What's Clearly Game Mode

- Ghost (AI, types, behaviors)
- Specific gear items (EMF, thermometer, etc.)
- Evidence and identification system
- Walkie-talkie system
- Truck UI
- Mission/campaign structure
- NPCs

### The Interesting Middle Ground

These need conceptual splitting:

1. **Player** — Basic player (move, hold) vs sanity/hiding behavior
2. **Difficulty** — Framework vs specific parameters
3. **Events** — Generic events vs game-specific events
4. **Behaviors** — The Behavior system itself vs specific behavior types (Door, Stairs, etc.)
5. **Health/Sanity** — Container (engine) vs rules (game mode)

---

## Questions This Raises

### 1. Should "Door" be engine or game mode?

**ANSWERED: Engine.**

Doors are common enough that the engine should provide how they work. The engine already has utilities to allow new door
types, but the base door behavior (interactive, toggleable state, blocks collision when closed) is engine-level.

### 2. Where does "threat" come from?

**ANSWERED: It doesn't exist, and shouldn't.**

This question was based on a misunderstanding. I was reaching for an "AI Director" concept like in Left 4 Dead or Alien:
Isolation—a meta-system that manages pacing and tension explicitly.

This goes against Unhaunter's design philosophy: "Anti-Arcade: We simulate hostile atmospheres, not script scares." The
game creates tension through honest simulation, not through a director manipulating difficulty. The player feels tension
from _understanding the system_, not from invisible rubber-banding.

No "threat system" needed. Tension emerges from simulation.

### 3. How do game modes register their systems?

**PARTIALLY ANSWERED:**

- `ungame` could become `ungameclassic` and serve as the entry point for that game mode
- A new crate would be needed to add the concept of a "game mode" and handle spinning systems up/down
- **But this is lower priority than separating components properly first**

The registration mechanism can be designed after the component boundaries are clear.

### 4. What about the simulation fields?

**ANSWERED: Shared-mod level.**

- Light and Sound are **engine-level** (tied to rendering/audio)
- Temperature, EMF, entity presence, etc. are **shared-mod level**
- Both Classic and Escape will share these fields
- They're not engine-core, but they're common enough to be shared infrastructure above the engine

This creates a three-tier model:

1. **Engine** — Board, Light, Sound, Player basics, Gear framework
2. **Shared Mod** — Temperature, EMF, and other game fields that both modes use
3. **Game Mode** — Ghost behavior, specific gear, loop, objectives

---

## Mapping to Our Abstract Layers

From our earlier discussion:

| Layer                       | Crates That Belong Here                             |
| --------------------------- | --------------------------------------------------- |
| Board (spatial container)   | `unspatial`, `uncore-board` (grid part)             |
| Field infrastructure        | Part of `uncore-board`, `unlight`, `unfog`          |
| Core fields (Light, Sound)  | `unlight`, parts of `unfog`                         |
| Actor framework             | Doesn't exist cleanly; mixed in `uncore-components` |
| Player & interaction        | `unplayer` (partially), `uninteraction`             |
| Tool/sensor framework       | `ungear`                                            |
| Common objects (Door, etc.) | Part of `uncore-board` behaviors                    |
| Game state hooks            | Part of `uncore-resources` (AppState, GameState)    |
| **--- ENGINE BOUNDARY ---** |                                                     |
| Shared fields (Temp, EMF)   | Shared mod layer (to be organized)                  |
| Specific fields             | Created by game mode                                |
| Entity behaviors            | `unghost`, `unnpc`                                  |
| Equipment behaviors         | `ungearitems`                                       |
| The loop                    | `uncampaign`, `ungame` → `ungameclassic`            |
| UI content                  | `untruck`, `unsummary`, etc.                        |

---

## What This Inventory Suggests

The codebase has a reasonable intuition about separation (simulation in one place, ghost in another), but the "core"
crates became dumping grounds that blur the boundary.

The refactoring already started (Phase 1+2 created `unspatial`, `untags`, `uninteraction`) was moving in the right
direction.

The next conceptual step might be:

1. Identify what specific types/components in the "God crates" are engine vs game mode
2. Think about whether those should be split out

But that's getting close to concrete action. The abstract question is: **Does this inventory match your mental model of
what should be engine vs game mode?**
