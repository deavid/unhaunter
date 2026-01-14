# Crate Analysis: `unnpc-plugin`

## Bounded Context

**NPC Interaction & Help System**

This crate manages the presence and behavior of Non-Player Characters (NPCs) in the map, specifically handling the
"Stranger" dialog system and proximity-based triggers.

## Responsibility

- Implement the `UnhaunterNPCPlugin`.
- Detect when a player is near an NPC to trigger dialog.
- Construct and display the NPC dialog UI.
- Manage transitions to the `NpcDialog` state.

## Dependencies Audit

- `unboard-core`: To identify NPC entities in the world.
- `unplayer-core`: To track the player position for proximity checks.
- `unassets-core`: For UI fonts and assets.
- `unrender-std`: For UI materials and styling.
- `bevy`: Standard UI and ECS logic.

## Encapsulation Assessment

- **Privacy**: Internal systems and modules are private.
- **Data Location**: NPC metadata is stored in `unboard-core`, while logic is here.

## Semantic & Infrastructure Leakage

- **Logic in Core**: Found a violation in `unboard-core`. The `NPCStranger` component (defined in board-core) contains
  mutable gameplay state (`seen`, `trigger`) and parsing logic for Tiled properties. This violates the "Data-Only" rule
  for core crates.
- **Feature Awareness**: `unboard-core` is explicitly aware of the "NPC Stranger" gameplay feature.
- **Hardcoded Heuristics**: Proximity distances and timers are hardcoded constants in the plugin's logic.
- **Centralized Asset Coupling**: The UI logic is tightly coupled to the centralized `GameAssets` in `unassets-core`.

## Future Recommendations

- **Move NPC Logic**: Move the `NPCStranger` parsing and state-tracking logic out of `unboard-core` and into this plugin
  or a dedicated `unnpc-core`.
- **Abstract Proximity**: Instead of the NPC plugin knowing about the `Player` component, use a generic
  `ProximityTrigger` that can be used for NPCs, doors, or items.
- **Relocate UI Content**: If NPC text is to be moddable, it should be moved out of the map properties and into a
  data-driven dialog system.
