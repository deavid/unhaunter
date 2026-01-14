# Crate Analysis: `uncampaign-plugin`

## Bounded Context

**Campaign & Custom Mission Selection Logic**

The crate manages the transition between the Main Menu and the Game, specifically handling mission discovery, filtering
(Campaign vs. Custom), player progression checks (locked/unlocked missions), and the UI for selecting which map to play.

## Responsibility

- Implement the `UnhaunterCampaignPlugin`.
- Provide a unified UI for mission selection.
- Handle mission filtering based on mode (`CurrentMissionSelectMode`).
- Track player progress badges (via `BadgeUtils`).
- Manage navigation flows for different game entry points.

## Dependencies Audit

- `undifficulty-core`: Used to understand difficulty settings for mission selection.
- `unfoundation-core`: Basic infrastructure.
- `unassets-core`: To access map and asset metadata.
- `unprofile-core`: To check player level and completed missions.
- `untags-core`: Metadata tagging.
- `unui-core`: Base UI components.
- `unwalkie-core` / `unwalkie-types`: Shared types (possibly for UI sounds or badges).
- `unrender-std`: Shared rendering bundles (badges).

## Encapsulation Assessment

- **Privacy**: The crate follows the logic-plugin pattern. Modules like `unified_mission_selection` are private,
  exposing only the `Plugin` implementation.
- **Leakage**:
  - Depends on `unrender-std`, which is expected for a UI-heavy logic plugin.
  - Seems well-contained; it doesn't appear to export data types that other crates depend on, maintaining its status as
    a "leaf node".

## Semantic & Infrastructure Leakage

- **Visuals**: Contains UI implementation logic (Bevy UI). This is appropriate for a plugin, as logic plugins are
  allowed to know about "how things look" in terms of UI layout.
- **Engine Readiness**: Does it assume a specific game world? It relies on `unassets-core` to find maps, which is
  generic. However, the mission selection logic is specific to the "Unhaunter" progression model (player levels,
  badges). To be truly generic, the "Mission Selection" would need to be a Registry of mission-providers.

## Future Recommendations

- If the goal is a generic engine, the "Campaign" logic should be separable from the "Map Loading" logic.
- Ensure that `unprofile-core` (progression) is purely data-driven so this plugin remains the sole owner of the _rules_
  for unlocking.
