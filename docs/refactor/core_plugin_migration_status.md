# Core vs. Plugin Architectural Compliance Report

**Date:** December 30, 2025
**Reference:** `core_plugin_migration_plan.md`

## 1. Executive Summary
*   **Logic Separation:** Excellent. No logic (systems, observers) was found in `*-core` or `*-std` crates.
*   **Plugin Encapsulation:** Mostly good, but some plugins export more than just the `Plugin` struct.
*   **Leaf Node Rule:** **IMPROVED**. Major violations involving `unrender-plugin` and `unlight-plugin` have been resolved by moving shared logic to `unrender-std`.
*   **Legacy Crates:** A significant number of `uncore-*` crates still exist and need to be dissolved or renamed.

---

## 2. Violations Report

### 🔴 Dependency Violations (Leaf Node Rule)
The policy states: *"No other crate (except the main App) may depend on a `*-plugin` crate."*

| Crate | Depends On | Violation Details |
| :--- | :--- | :--- |
| **`unfog-plugin`** | `unrender-plugin` | **FIXED**. `rebuild_collision_data` moved to `unrender-std`. |
| **`ungame-plugin`** | `unrender-plugin`, `unlight-plugin` | **FIXED**. Shared logic moved to `unrender-std`. Dependencies removed. |
| **`unmapload-plugin`** | `unlight-plugin`, `unrender-plugin` | **FIXED**. Shared logic moved to `unrender-std`. Dependencies removed. |
| **`unroot-plugin`** | `unmetrics-plugin` | **FIXED**. Dependency moved to `unhaunter/src/app.rs`. |

### 🟡 Export Violations (Plugin Encapsulation)
The policy states: *"The **ONLY** public export allowed is the `struct MyPlugin`."*

| Crate | Exported Items | Notes |
| :--- | :--- | :--- |
| **`unmanual-plugin`** | `create_manual`, `draw_manual_page`, `Manual`, `ManualChapter` | **FIXED**. Made private/internal. Unused fields removed. |
| **`unmetrics-plugin`** | `app_setup` | **FIXED**. Removed. |
| **`unlight-plugin`** | `rebuild_lighting_field`, `prebake_lighting_field` | **FIXED**. Moved to `unrender-std`. Exports removed. |

### 🟠 Legacy Crates (Pending Migration)
These crates are marked for renaming or dissolution in the plan but still exist in their old form.

*   `uncore-assets` (Target: `unassets-core`)
*   `uncore-board` (Target: `unboard-core`)
*   `uncore-components` (Target: **DISSOLVE**)
*   `uncore-events` (Target: `unevents-core`)
*   `uncore-foundation` (Target: `unfoundation-core`)
*   `uncore-resources` (Target: **DISSOLVE**)
*   `uncore-types` (Target: `untypes-core`)
*   `unwalkie-types` (Target: `unwalkie-types` - Keep)

---

## 3. Compliant Crates
The following crates appear to strictly follow the policy (Clean dependencies, correct exports, no misplaced logic).

### Core Crates (Data Only)
*   `undifficulty-core`
*   `unfog-core`
*   `ungear-core`
*   `ungearitems-core`
*   `unghost-core`
*   `uninteraction-core`
*   `unmenu-core`
*   `unmetrics-core`
*   `unnavigation-core`
*   `unnoise-core`
*   `unpicking-core`
*   `unplayer-core`
*   `unprofile-core`
*   `unsettings-core`
*   `unspatial-core`
*   `untags-core`
*   `untiled-core`
*   `untruck-core`
*   `unui-core`
*   `unwalkie-core`

### Std Crates (Standard Data)
*   `unrender-std`

### Plugin Crates (Logic Only, Correctly Isolated)
*   `uncampaign-plugin`
*   `unfog-plugin`
*   `ungame-plugin`
*   `ungear-plugin`
*   `ungearitems-plugin`
*   `unghost-plugin`
*   `unlight-plugin`
*   `unmainmenu-plugin`
*   `unmanual-plugin`
*   `unmaphub-plugin`
*   `unmapload-plugin`
*   `unmenu-plugin`
*   `unmenusettings-plugin`
*   `unmetrics-plugin`
*   `unnpc-plugin`
*   `unpicking-plugin`
*   `unplayer-plugin`
*   `unprofile-plugin`
*   `unrender-plugin`
*   `unroot-plugin`
*   `unsettings-plugin`
*   `unsummary-plugin`
*   `untmxmap-plugin`
*   `untruck-plugin`
*   `unwalkie-plugin`
