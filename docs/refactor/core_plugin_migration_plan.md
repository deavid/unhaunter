# Core vs. Plugin Architectural Migration Plan

## 1. Executive Summary
This document outlines the plan to restructure the `unhaunter` workspace to enforce a strict **"Core vs. Plugin"** architecture. This separation will decouple data (Components, Resources, Events) from logic (Systems, Observers), leading to improved compile times, better testability, and architectural clarity.

## 2. The Architectural Policy
We are moving to a strict three-layer separation of concerns:

1.  **`un*-core` Crates (Low-Level Data):**
    *   **Contents:** Base components, resources, events, and enums.
    *   **Constraint:** Minimal dependency chains. Should mostly depend on `unfoundation-core` or other `un*-core` crates.
    *   **Visibility:** Public exports allowed.
    *   **Logic:** No systems, logic, or observers.

2.  **`un*-std` Crates (Standard Data):**
    *   **Contents:** Higher-level data structures, complex bundles, or data that depends on other `un*-std` crates.
    *   **Constraint:** If a crate depends on a `*-std` crate, it must be a `*-std` crate itself (or a plugin).
    *   **Visibility:** Public exports allowed.
    *   **Logic:** No systems, logic, or observers.

3.  **`un*-plugin` Crates (Logic Layer):**
    *   **Contents:** Systems, Observers, Internal Logic.
    *   **Constraint:** The **ONLY** public export allowed is the `struct MyPlugin`. All systems must be private or `pub(crate)` only.
    *   **Leaf Node Rule:** No other crate (except the main App) may depend on a `*-plugin` crate.

4.  **`*-types` / `*-lib` Crates (Shared/External):**
    *   **Context:** Reserved for types shared with external tools.
    *   **Constraint:** Pure data, minimal dependencies.

## 3. Crate Classification Table

| Crate | Plugin? | Shared? | Action | Proposed Name | Notes |
| :--- | :---: | :---: | :--- | :--- | :--- |
| `uncampaign` | Yes | No | **DONE** | `uncampaign-plugin` | Leaf node. |
| `uncore-assets` | No | Yes | **RENAME** | `unassets-core` | Shared data. |
| `uncore-board` | No | Yes | **RENAME** | `unboard-core` | Shared data. |
| `uncore-components` | No | Yes | **DISSOLVE** | - | God crate. Still exists. |
| `uncore-events` | No | Yes | **RENAME** | `unevents-core` | Shared data. |
| `uncore-foundation` | No | Yes | **RENAME** | `unfoundation-core` | Base types. |
| `uncore-resources` | No | Yes | **DISSOLVE** | - | God crate. Still exists. |
| `uncore-types` | No | Yes | **RENAME** | `untypes-core` | Shared data. |
| `uncoremenu` | Yes | No | **RENAME** | `uncoremenu-plugin` | Leaf node. |
| `undifficulty` | No | Yes | **RENAME** | `undifficulty-core` | Shared data. |
| `unfog` | Yes | No | **DONE** | `unfog-core`, `unfog-plugin` | Leaf node. |
| `ungame` | Yes | No | **DONE** | `ungame-plugin` | Leaf node. |
| `ungear` | Yes | Yes | **DONE** | `ungear-core`, `ungear` | Shared data + logic. |
| `ungearitems` | Yes | No | **DONE** | `ungearitems-plugin` | Splitted. |
| `unghost` | Yes | Yes | **DONE** | `unghost-core`, `unghost` | `unghost-core` created. |
| `unghost-core` | No | Yes | **DONE** | `unghost-core` | Shared data. |
| `uninteraction` | No | Yes | **DONE** | `uninteraction-core` | Shared data. |
| `unlight` | Yes | No | **DONE** | `unlight-plugin` | Leaf node. |
| `unmanual` | Yes | No | **DONE** | `unmanual-plugin` | Leaf node. |
| `unmaphub` | Yes | No | **SPLIT** | `unmaphub-core`, `unmaphub-plugin` | Shared state + logic. |
| `unmapload` | Yes | No | **DONE** | `unmapload-plugin` | Leaf node. |
| `unmenu` | Yes | No | **DONE** | `unmenu-plugin` | Leaf node. |
| `unmenusettings` | Yes | No | **DONE** | `unmenusettings-plugin` | Leaf node. |
| `unmetrics` | Yes | Yes | **DONE** | `unmetrics-core`, `unmetrics-plugin` | Shared trait + system. |
| `unnavigation` | No | Yes | **DONE** | `unnavigation-core` | Shared data. |
| `unnoise` | No | Yes | **DONE** | `unnoise-core` | Shared data. |
| `unnpc` | Yes | No | **DONE** | `unnpc-plugin` | Leaf node. |
| `unpicking` | Yes | No | **DONE** | `unpicking-core`, `unpicking-plugin` | Leaf node. |
| `unplayer` | Yes | Yes | **DONE** | `unplayer-core`, `unplayer` | `unplayer-core` exists. |
| `unplayer-core` | No | Yes | **DONE** | `unplayer-core` | Base player data. |
| `unprofile` | Yes | No | **DONE** | `unprofile-core`, `unprofile-plugin` | Split data and logic. |
| `unrender` | No | Yes | **SPLIT** | `unrender-std`, `unrender-plugin` | High dependency chain. |
| `unroot` | Yes | No | **DONE** | `unroot-plugin` | Main app plugin. |
| `unsettings` | Yes | Yes | **DONE** | `unsettings-core`, `unsettings-plugin` | Shared data + logic. |
| `unspatial` | No | Yes | **DONE** | `unspatial-core` | Shared data. |
| `unsummary` | Yes | No | **DONE** | `unsummary-plugin` | Leaf node. |
| `untags` | No | Yes | **DONE** | `untags-core` | Shared data. |
| `untiled` | No | Yes | **DONE** | `untiled-core` | Shared data. |
| `untmxmap` | Yes | No | **DONE** | `untmxmap-plugin` | Leaf node. |
| `untruck` | Yes | No | **DONE** | `untruck-plugin` | Leaf node. |
| `untruck-core` | No | Yes | **DONE** | `untruck-core` | Shared data. |
| `unui` | No | Yes | **DONE** | `unui-core` | Shared data. |
| `unwalkie` | Yes | No | **DONE** | `unwalkie-plugin` | Leaf node. |
| `unwalkie_types` | No | Yes | **KEEP** | `unwalkie_types` | Shared data. |
| `unwalkiecore` | No | Yes | **DONE** | `unwalkie-core` | Shared data. |

## 4. The "Dissolution" Map

### `uncore-components` Dissolution
| Component | Target Crate |
| :--- | :--- |
| `ItemName`, `ItemDescription` | `unfoundation-core` |
| `GearSprite`, `Handheld`, `StatusText`, `Battery`, `Electronic`, `Toggleable`, `Triggered` | `ungear-core` |
| `LightEmitter` | `unlight-core` |
| `EvidenceSensor` | `ungear-core` |

### `uncore-resources` Dissolution
| Resource | Target Crate |
| :--- | :--- |
| `AppState`, `GameState` | `unfoundation-core` |
| `MapHubState`, `MissionSelectMode`, `CurrentMissionSelectMode` | `unmaphub-core` |
| `MouseVisibility` | `unpicking-core` |
| `SummaryData` | `unsummary-std` |

## 5. The "Split" Manifest (Major Crates)

### `unplayer` -> `unplayer-core` & `unplayer`
- **`unplayer-core`**:
    - `src/components/` (Base components)
    - `src/resources/`
- **`unplayer`** (Plugin):
    - `src/systems/`
    - `src/plugin.rs`
    - `src/components/` (Higher level bundles)

### `unghost` -> `unghost-core` & `unghost`
- **`unghost-core`**:
    - `src/components/`
    - `src/resources/`
- **`unghost`** (Plugin):
    - `src/systems/`
    - `src/ghost.rs`
    - `src/ghost_events.rs`

### `unfog` -> `unfog-core` & `unfog-plugin`
- **`unfog-core`**:
    - `src/components.rs`
    - `src/resources.rs`
- **`unfog-plugin`**:
    - `src/systems.rs`
    - `src/plugin.rs`

### `untruck` -> `untruck-core` & `untruck-plugin`
- **`untruck-core`**:
    - `src/components/`
    - `src/types/`
- **`untruck-plugin`**:
    - `src/systems/`
    - `src/plugin.rs`
    - `src/ui.rs`

### `unwalkie` -> `unwalkie-core` & `unwalkie-plugin`
- **`unwalkie-core`** (Currently `unwalkiecore`):
    - `src/events.rs`
    - `src/resources.rs`
- **`unwalkie-plugin`**:
    - `src/plugin.rs`
    - `src/walkie_play.rs`

### `unrender` -> `unrender-std` & `unrender-plugin`
- **`unrender-std`**:
    - `src/components/`
    - `src/materials.rs`
    - `src/resources/`
    - Depends on `unplayer-core`, `unghost-core`, `unboard-core`.
- **`unrender-plugin`**:
    - `src/systems/`
    - `src/board/`
    - `src/plugin.rs`

### `unsettings` -> `unsettings-core` & `unsettings-plugin`
- **`unsettings-core`**:
    - `src/controls.rs`
- **`unsettings-plugin`**:
    - `src/systems.rs`
    - `src/plugin.rs`

### `unmetrics` -> `unmetrics-core` & `unmetrics-plugin`
- **`unmetrics-core`**:
    - `SendMetric` trait, `Data` struct, and channel logic.
- **`unmetrics-plugin`**:
    - `receive_data` system and `app_setup`.

## 6. Risk Analysis: Circular Dependencies

1.  **Cross-Feature Data Dependencies**:
    - **Risk**: `unplayer-core` needing `unghost-core` (e.g., for sanity effects) while `unghost-core` needs `unplayer-core` (e.g., for player detection).
    - **Mitigation**: Move shared high-level types (like `GhostType` or `PlayerId`) to `unfoundation-core` or a dedicated `untypes-core`. Ensure `-core` crates only depend on other `-core` crates or `unfoundation-core`.

2.  **Rendering Components**:
    - **Risk**: `unrender-std` (containing `GameSprite`) being needed by almost every other `-core` crate to define bundles.
    - **Mitigation**: `unrender-std` must be positioned as a higher-level dependency. If a `-core` crate needs it, that crate must become a `-std` crate.

3.  **Settings and Controls**:
    - **Risk**: `unplayer-std` depends on `unsettings-core` for `ControlKeys`.
    - **Mitigation**: This is acceptable as `unplayer-std` is in the standard layer.

4.  **Leaf Node Enforcement**:
    - **Risk**: A `-plugin` crate accidentally depending on another `-plugin` crate.
    - **Mitigation**: Use CI tools or `cargo-deny` to enforce that `-plugin` crates are only depended upon by the main app (`unroot-plugin` or the final binary).
