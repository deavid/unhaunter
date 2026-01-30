# Refactor Plan: Visual Purification of Gear Domain

**Date:** 2026-01-30
**Target:** `ungear-core`, `unrender-std`, and dependent plugins
**Goal:** Move visual assets (`GearAssets`) out of the core data crate (`ungear-core`) into the rendering standard crate (`unrender-std`).

## Context
The DDD Audit identified `ungear-core` as having "Visual Leakage" because it defines `GearAssets`, which contains hardcoded paths to sprite sheets (`gear_spritesheetA_48x48.png`). This violates the separation of concerns where Core crates should be data-only and agnostic to rendering implementation.

## Step-by-Step Instructions for Copilot

### Phase 1: Preparation (Dependencies)
1.  **MODIFY** `crates/unrender-std/Cargo.toml`:
    *   Add `bevy_asset_loader = { workspace = true }` to `[dependencies]`.
2.  **MODIFY** `crates/ungear-core/Cargo.toml`:
    *   Remove `bevy_asset_loader = { workspace = true }` (if no other file uses it).

### Phase 2: Move the Asset Definition
1.  **CREATE** `crates/unrender-std/src/assets.rs` with the following content (moved from `ungear-core`):
    ```rust
    use bevy::prelude::*;
    use bevy_asset_loader::prelude::*;

    #[derive(AssetCollection, Resource, Debug, Clone)]
    pub struct GearAssets {
        #[asset(path = "img/gear_spritesheetA_48x48.png")]
        pub gear: Handle<Image>,
        #[asset(texture_atlas_layout(tile_size_x = 96, tile_size_y = 96, columns = 10, rows = 10))]
        pub gear_layout: Handle<TextureAtlasLayout>,
    }
    ```
2.  **MODIFY** `crates/unrender-std/src/lib.rs`:
    *   Add `pub mod assets;` to the module list.
3.  **DELETE** `crates/ungear-core/src/assets.rs`.
4.  **MODIFY** `crates/ungear-core/src/lib.rs`:
    *   Remove `pub mod assets;`.

### Phase 3: Update Consumers (Search & Replace)
1.  **SEARCH** for `use ungear_core::assets::GearAssets;` and **REPLACE** with `use unrender_std::assets::GearAssets;`.
    *   *Files to check:*
        *   `crates/unclassic-mode-plugin/src/game_ui.rs`
        *   `crates/unclassic-mode-plugin/src/gear_ui.rs`
        *   `crates/ungear-plugin/src/plugin.rs`
        *   `crates/ungear-plugin/src/systems.rs`
        *   `crates/untruck-plugin/src/loadoutui.rs`
        *   `crates/untruck-plugin/src/ui.rs`
    *   *Note:* Ensure `unrender-std` is in the `Cargo.toml` dependencies of these crates (it usually is, but verify).

### Phase 4: Validation
1.  Run `cargo check -p unrender-std` to ensure it compiles with the new asset struct.
2.  Run `cargo check -p ungear-core` to ensure it compiles without the asset struct.
3.  Run `cargo check -p ungear-plugin` (and others) to ensure the import switch worked.
