# Unhaunter Binary Refactoring Research

**Date:** 2026-01-16 **Author:** GitHub Copilot

## 1. Introduction

This document expands on the initial investigation report, providing specific analysis and refactoring steps to clean up
the `unhaunter` binary crate. The goal is to move towards a "plugin aggregator" architecture.

## 2. Detailed Findings

### 2.1. `UnhaunterDifficultyPlugin` Refactoring

- **Issue:** `CurrentDifficulty` is initialized in `app.rs`.
- **Target:** `crates/undifficulty-core`.
- **Finding:** `undifficulty-core` does not have a `Plugin` struct. It exposes `difficulty_state.rs`,
  `difficulty_impl.rs` etc.
- **Action:**
  1. Create a new struct `UnhaunterDifficultyPlugin` in `crates/undifficulty-core/src/lib.rs` (or a new `plugin.rs`).
  2. Implement `Plugin` for it.
  3. Move `app.init_resource::<CurrentDifficulty>()` into its `build` method.
  4. Update `unhaunter/src/app.rs` to use `app.add_plugins(UnhaunterDifficultyPlugin)`.

### 2.2. `UnhaunterGhostPlugin` Refactoring

- **Issue:** `ObjectInteractionConfig` and `HauntState` are initialized in `app.rs`.
- **Target:** `crates/unghost-plugin` or `crates/unghost-core`.
- **Finding:**
  - `unghost-plugin` exists and has `UnhaunterGhostPlugin`.
  - `unghost-core` contains the resources definition.
- **Action:**
  1. Modify `UnhaunterGhostPlugin::build` in `crates/unghost-plugin/src/plugin.rs`.
  2. Add `app.init_resource::<ObjectInteractionConfig>()` and `app.init_resource::<HauntState>()`.
  3. Remove these lines from `unhaunter/src/app.rs`.

### 2.3. Rendering & Materials

- **Issue:** `Material2dPlugin` and `UiMaterialPlugin` are explicitly added in `app.rs`.
- **Target:** `crates/unrender-plugin`.
- **Finding:** `unrender-plugin` has `UnhaunterBoardPlugin` but not a general "Render Plugin" that encapsulates all
  rendering dependencies.
- **Action:**

  1. Rename `UnhaunterBoardPlugin` to `UnhaunterRenderPlugin` OR create a new `UnhaunterRenderPlugin` in
     `crates/unrender-plugin` that includes `UnhaunterBoardPlugin` and the material plugins.
  2. Ideally, `UnhaunterRenderPlugin` should be the entry point.
  3. Move the material registration there:

     ```rust
     app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
        .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());
     ```

  4. Ensure `unrender-std` (where materials are defined) is accessible to `unrender-plugin` (it appears to be, judging
     by `Cargo.toml`).

### 2.4. Performance Reporting

- **Issue:** `report_performance` logic is in `unhaunter/src/report_timer.rs`.
- **Target:** `crates/unmetrics-plugin`.
- **Finding:** `unmetrics-plugin` is very thin, just `receive_data`.
- **Action:**
  1. Move `report_timer.rs` content to `crates/unmetrics-plugin/src/performance_report.rs`.
  2. Register the system `report_performance` in `UnmetricsPlugin::build`.
  3. Fix dependencies in `unmetrics-plugin/Cargo.toml` (needs `bevy` deps, `untypes-core`).

### 2.5. Window Icon & Asset Utils

- **Issue:** `set_window_icon` and `find_assets_directory` are in the binary.
- **Target:** New `UnhaunterWindowPlugin` or `unfoundation-core`.
- **Finding:** `find_assets_directory` is used for `assetidx_updater` (dev tool) and icon loading (native window).
- **Action:**
  1. Create `UnhaunterWindowPlugin` in `crates/unmainmenu-plugin` (maybe?) or better, a new small plugin crate
     `crates/unwindow-plugin` or just inside `unrender-plugin` since window is display-related.
  2. Actually, `unrender-plugin` is a good place for window setup as it deals with "presenting" the game.
  3. Move `set_window_icon` logic to `unrender-plugin`.
  4. Move `find_assets_directory` to `unfoundation-core::platform` or `utils`, making it public.

### 2.6. `assetidx_updater`

- **Issue:** It generates an index of assets for WASM/etc.
- **Finding:** It's a build/dev tool. It runs on startup in native non-wasm builds.
- **Action:**
  - Since it updates files on disk, it's best kept as a separate "tool" or dev-only plugin.
  - Leave it in `unhaunter` for now but mark it as a "Dev Tool" responsibility, or move it to
    `crates/tools/asset_indexer` and run it via `cargo run --bin asset_indexer`.
  - For `unhaunter` binary convenience, we can keep calling it, but maybe move the code to `crates/unassets-core` as a
    helper function `update_asset_index()`.

## 3. Execution Plan

The migration should happen in stages to avoid breaking the build.

1. **Stage 1: Core Systems**: Move Resource initialization to their respective plugins.
2. **Stage 2: Render & Metrics**: Consolidate rendering setup and move metrics logic.
3. **Stage 3: Platform Utils**: Factor out asset finding and window management.

This research confirms that all problematic code has a logical "home" in the existing crate structure.
