# Phase 1: Straightforward Architectural Cleanup

## 1. Objective
This plan covers the "low-hanging fruit" of the architectural migration. These are tasks that are straightforward, carry low risk of circular dependencies, and provide immediate clarity to the workspace structure.

## 2. Phase 1: Renaming Leaf Plugins
These crates are already leaf nodes (or nearly so) and primarily contain logic. The action is a simple rename and enforcing internal visibility.

**Tasks:**
- Rename the following crates to `*-plugin`:
    - `uncampaign` -> `uncampaign-plugin`
    - `uncoremenu` -> `uncoremenu-plugin`
    - `unfog` -> `unfog-plugin`
    - `ungame` -> `ungame-plugin`
    - `ungearitems` -> `ungearitems-plugin`
    - `unlight` -> `unlight-plugin`
    - `unmanual` -> `unmanual-plugin`
    - `unmapload` -> `unmapload-plugin`
    - `unmenu` -> `unmenu-plugin`
    - `unmenusettings` -> `unmenusettings-plugin`
    - `unnpc` -> `unnpc-plugin`
    - `unprofile` -> `unprofile-plugin`
    - `unroot` -> `unroot-plugin`
    - `untmxmap` -> `untmxmap-plugin`
    - `untruck` -> `untruck-plugin`
    - `unwalkie` -> `unwalkie-plugin`
- **Visibility Enforcement:** In each of these, ensure all systems and internal logic are `pub(crate)` or private. Only the `Plugin` struct should be `pub`.

## 3. Phase 2: Renaming Core Data Crates
These crates contain shared data and no logic. Renaming them clarifies their role as low-level building blocks.

**Tasks:**
- Rename the following crates to `*-core`:
    - `uncore-assets` -> `unassets-core`
    - `uncore-board` -> `unboard-core`
    - `uncore-events` -> `unevents-core`
    - `uncore-foundation` -> `unfoundation-core`
    - `uncore-types` -> `untypes-core`
    - `undifficulty` -> `undifficulty-core`
    - `uninteraction` -> `uninteraction-core`
    - `unnavigation` -> `unnavigation-core`
    - `unnoise` -> `unnoise-core`
    - `untags` -> `untags-core`
    - `untiled` -> `untiled-core`
    - `unui` -> `unui-core`
    - `unwalkiecore` -> `unwalkie-core`

## 4. Phase 3: Dissolving God Crates
Distribute components and resources from the monolith crates to their specific feature-core crates.

**Tasks:**
- **Dissolve `uncore-components`:**
    - Move `ItemName`, `ItemDescription` to `unfoundation-core`.
    - Move `GearSprite`, `Handheld`, `StatusText`, `Battery`, `Electronic`, `Toggleable`, `Triggered`, `EvidenceSensor` to `ungear-core`.
    - Move `LightEmitter` to `unlight-core` (or `unlight-std` if needed).
- **Dissolve `uncore-resources`:**
    - Move `AppState`, `GameState` to `unfoundation-core`.
    - Move `MapHubState`, `MissionSelectMode`, `CurrentMissionSelectMode` to `unmaphub-core`.
    - Move `MouseVisibility` to `unpicking-core`.
    - Move `SummaryData` to `unsummary-std`.

## 5. Phase 4: Simple Splits
Crates with very clear separation and minimal dependencies.

**Tasks:**
- **Split `unmetrics`**:
    - `unmetrics-core`: `SendMetric` trait and channel logic.
    - `unmetrics-plugin`: `receive_data` system.
- **Split `unsettings`**:
    - `unsettings-core`: `ControlKeys` and settings data.
    - `unsettings-plugin`: Logic for loading/saving settings.

## 6. Intentionally Left Over
The following crates are excluded from this first plan as they require more in-depth analysis of their dependency chains and potential circularities:
- `unplayer` (Complex dependencies on settings/spatial)
- `unghost` (Complex behavior logic)
- `unrender` (High-level rendering data)
- `unmaphub` & `unpicking` (State management)
- `unsummary` (Depends on multiple feature crates)

## 7. Next Steps
After completing these tasks, a second round of in-depth analysis will be performed to map out the `std` vs `core` boundaries for the remaining complex crates.
