# DDD Analysis: `untruck-plugin`

## 1. Bounded Context

**Investigation Platform & UI Orchestration**. This crate implements the "Command Center" of the game, providing the
player with tools to monitor the haunting, manage evidence, and prepare for the final encounter.

## 2. Responsibility

- **Truck UI Management**: Orchestrates the complex UI systems for the truck's monitors, loadout selection, and
  equipment status.
- **Journal System**: Implements the logic for the investigation journal, where players track evidence and guess the
  ghost type.
- **Remote Sensing**: Provides the logic for "sensors" that monitor the haunted house from the safety of the truck
  (e.g., sanity monitors, activity sensors).
- **Repellent Crafting**: Implements the final gameplay loop where players combine evidence to craft the repellent
  needed to finish the mission.
- **Loadout Management**: Handles the logic for taking items from the truck into the house.

## 3. Dependencies and Appropriateness

- **Dependencies**: Depends on nearly every `-core` crate in the workspace (`unfoundation`, `unboard`, `unghost`,
  `unplayer`, `ungear`, `unprofile`, etc.).
- **Appropriateness**: Moderate. While a "Command Center" naturally needs to know about many things, the current
  implementation is very broad. It acts as a massive orchestration layer.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Uses types from `untruck-core` and many other domain cores.
- **Logic**: Contains a vast array of systems for UI interaction, sensor updates, and mission progression.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: High. It is deeply coupled with every major domain (Ghost, Player, Gear, Evidence). It is the
  primary consumer of the "Unified Domain" data.
- **Infrastructure Leakage**: High. Intensive use of Bevy UI, cameras, and input events.

---

## Technical Debt & Strategic Notes

- **The "God Plugin" Problem**: `untruck-plugin` is one of the most coupled crates in the workspace. Its dependency list
  is a "who's who" of the project.
- **Refactoring Target**: Ideally, the sub-modules (Journal, Sensors, Loadout) should be extracted into their own
  plugins to reduce the cognitive load and compilation surface of this crate.
- **Communication Hub**: This crate relies heavily on `unevents-core` to communicate with other plugins (like
  `unmapload-plugin` or `ungame-plugin`), which is a good architectural pattern, but the direct dependency on many cores
  still remains.
