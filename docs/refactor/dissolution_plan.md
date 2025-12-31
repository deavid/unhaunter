# Plan: Dissolution of `uncore-components` and `uncore-resources`

**Goal**: Eliminate "God Crates" by distributing their contents to domain-specific crates, enforcing the Core vs. Plugin architecture.

## 1. `uncore-components` Dissolution

This crate contains a mix of unrelated components. They will be migrated as follows:

| Component | Target Crate | Rationale |
| :--- | :--- | :--- |
| `GearSprite` | `ungear-core` | Strictly related to gear visualization. |
| `ItemName` | `ungear-core` | Primarily used for identifying gear/items. |
| `ItemDescription` | `ungear-core` | Primarily used for describing gear/items. |
| `Electronic` | `ungear-core` | Gear behavior (EMI sensitivity). |
| `Battery` | `ungear-core` | Gear behavior (Power management). |
| `EvidenceSensor` | `ungear-core` | Gear behavior (Evidence gathering). |
| `Handheld` | `ungear-core` | Gear behavior (Inventory/Hands). |
| `StatusText` | `ungear-core` | Gear behavior (UI/Display). |
| `LightEmitter` | `unrender-std` | Core lighting component, belongs with lighting logic/types. |
| `Toggleable` | `uninteraction-core` | Represents interactive state (on/off), used by gear and map objects. |
| `Triggered` | `uninteraction-core` | Interaction event marker. |

**Action Items:**
1.  Move code to target crates.
2.  Update `Cargo.toml` in target crates to add necessary dependencies (e.g., `bevy`, `reflect`).
3.  Update all consumers to import from the new locations.
4.  Delete `uncore-components`.

## 2. `uncore-resources` Dissolution

This crate contains global state and resources. They will be migrated as follows:

| Resource/Type | Target Crate | Rationale |
| :--- | :--- | :--- |
| `AppState` | `untypes-core` | Fundamental application states, needed everywhere. |
| `GameState` | `untypes-core` | Fundamental game states. |
| `MapHubState` | `untypes-core` | Fundamental map hub states. |
| `MouseVisibility` | `unui-core` | UI-related resource. |
| `SummaryData` | `unsummary-core` (New) | Data contract for the summary screen. |
| `MissionSelectMode` | `unmenu-core` | Related to menu navigation/selection. |
| `CurrentMissionSelectMode` | `unmenu-core` | Related to menu navigation/selection. |
| `CliOptions` | `unsettings-core` | Application configuration/settings. |
| `GameConfig` | `unsettings-core` | Game configuration/settings. |

**Action Items:**
1.  Create `unsummary-core` crate.
2.  Move `SummaryData` to `unsummary-core`.
3.  Move States (`AppState`, etc.) to `untypes-core`.
4.  Move `MouseVisibility` to `unui-core`.
5.  Move Mission Select types to `unmenu-core`.
6.  Move Config/Options to `unsettings-core`.
7.  Update all consumers.
8.  Delete `uncore-resources`.

## 3. Execution Order

1.  **Phase 1: Components**: **COMPLETED**. `uncore-components` has been dissolved.
2.  **Phase 2: Resources**: **COMPLETED**. `uncore-resources` has been dissolved.

## 4. Summary of Changes

- **`uncore-components`**: All components moved to `ungear-core`, `unrender-std`, or `uninteraction-core`.
- **`uncore-resources`**:
    - `SummaryData` -> `unsummary-core` (New crate).
    - `AppState`, `GameState`, `MapHubState` -> `uncore-types`.
    - `MissionSelectMode`, `CurrentMissionSelectMode` -> `unmenu-core`.
    - `MouseVisibility` -> `unui-core`.
- **Project-wide**: Updated ~50 files to fix imports and `Cargo.toml` dependencies.
- **Cleanup**: `uncore-components` and `uncore-resources` removed from workspace `Cargo.toml` and all crate dependencies.

## 5. Risks & Mitigation

*   **Circular Dependencies**: Moving `AppState` to `untypes-core` is safe as `untypes-core` is at the bottom of the dependency graph.
*   **Massive Refactor**: `AppState` is used everywhere. We should use `sed` or global search-and-replace carefully.
*   **New Crates**: Creating `unsummary-core` adds overhead but improves separation.

