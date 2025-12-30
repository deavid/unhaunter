# Core vs. Plugin Architectural Compliance Report

**Date:** December 30, 2025
**Reference:** `core_plugin_migration_plan.md`

## 1. Executive Summary
*   **Logic Separation:** Excellent. No logic (systems, observers) was found in `*-core` or `*-std` crates.
*   **Plugin Encapsulation:** Mostly good, but some plugins export more than just the `Plugin` struct.
*   **Leaf Node Rule:** **VIOLATED**. Several plugins depend on other plugins (`unrender-plugin`, `unlight-plugin`), creating tight coupling between logic layers.
*   **Legacy Crates:** A significant number of `uncore-*` crates still exist and need to be dissolved or renamed.

---

## 2. Violations Report

### 🔴 Dependency Violations (Leaf Node Rule)
The policy states: *"No other crate (except the main App) may depend on a `*-plugin` crate."*

| Crate | Depends On | Violation Details |
| :--- | :--- | :--- |
| **`unfog-plugin`** | `unrender-plugin` | Imports `rebuild_collision_data`. |
| **`ungame-plugin`** | `unrender-plugin`, `unlight-plugin` | Direct dependency in `Cargo.toml`. |
| **`unmapload-plugin`** | `unlight-plugin`, `unrender-plugin` | Direct dependency in `Cargo.toml`. |
| **`unroot-plugin`** | `unmetrics-plugin` | **FIXED**. Dependency moved to `unhaunter/src/app.rs`. |

> **Recommendation:** Move shared logic (like `rebuild_collision_data`) to a `*-std` crate or use Events/Resources to decouple these plugins.

### 🟡 Export Violations (Plugin Encapsulation)
The policy states: *"The **ONLY** public export allowed is the `struct MyPlugin`."*

| Crate | Exported Items | Notes |
| :--- | :--- | :--- |
| **`unmanual-plugin`** | `create_manual`, `draw_manual_page`, `Manual`, `ManualChapter` | **FIXED**. Made private/internal. Unused fields removed. |
| **`unmetrics-plugin`** | `app_setup` | **FIXED**. Removed. |

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
*   `ungear-plugin`
*   `ungearitems-plugin`
*   `unghost-plugin`
*   `unlight-plugin` (Leaf node, but depended upon by others)
*   `unmainmenu-plugin`
*   `unmanual-plugin`
*   `unmaphub-plugin`
*   `unmenu-plugin`
*   `unmenusettings-plugin`
*   `unmetrics-plugin`
*   `unnpc-plugin`
*   `unpicking-plugin`
*   `unplayer-plugin`
*   `unprofile-plugin`
*   `unrender-plugin` (Leaf node, but depended upon by others)
*   `unroot-plugin`
*   `unsettings-plugin`
*   `unsummary-plugin`
*   `untmxmap-plugin`
*   `untruck-plugin`
*   `unwalkie-plugin`
