# Refined Upscaling Strategy

This document incorporates final technical decisions and addresses risks identified in previous critiques.

## 1. Discovery Mechanism: Filename Prefixing

To ensure compatibility with `assetidx` and simplify path resolution, upscaled assets will follow a prefixing convention
within a dedicated folder.

- **Base Path**: `assets/img/item.png`
- **Upscaled Path**: `assets/upscaled/zoom0Nx_item.png` (e.g., `zoom03x_item.png`)

### 1.1. Benefits for AssetIdx

By adding `("upscaled", vec!["png"])` to the `assetidx_updater.rs`, all high-res assets will be indexed under a single
`upscaled-png.assetidx` file. This is efficient and works perfectly in WASM environments.

### 1.2. Discovery Logic

The engine will attempt to resolve a path by looking for the `upscaled/zoom0Nx_{filename}` variant first, starting from
the highest allowed factor.

## 2. Implementation Focus: Tiled Map Tiles (TSX)

The primary goal is upscaling the environment. The focus will be on intercepting the image loading process in
`bevy_load_map` ([crates/untmxmap-plugin/src/bevy.rs](crates/untmxmap-plugin/src/bevy.rs)).

### 2.1. Scaling Constants

When an upscaled asset is detected:

1. **TextureAtlasLayout**: The `tile_size` and `spacing` provided by the TSX (which are in 1x units) must be multiplied
    by $N$.
2. **CustomMaterial1Data**: The `sprite_width` and `sprite_height` (used by the shader for anti-aliasing and margins)
    must be multiplied by $N$ to match the physical pixel dimensions of the upscaled image.
3. **Mesh size**: The `sprite_size` used for mesh generation in `sprite_db.rs` **must remain in 1x units**. This
    ensures the tile occupies the same physical space in the game world regardless of the texture resolution.

## 3. Dynamic Scaling Logic Audit

Several systems that modify `Transform.scale` must be updated to account for the `ResolutionFactor(f32)` component.

- **Ghost Glitch System** ([crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs)):
  - Lerping back to "rest" must target `1.0 / resolution_factor` instead of `1.0`.
- **Tile Spawning Flip** ([crates/unmapload-plugin/src/tile_spawning.rs](crates/unmapload-plugin/src/tile_spawning.rs)):
  - Flipping horizontally should set `scale.x = -1.0 / resolution_factor`.
- **Ad-hoc Spawns**:
  - Overlays, particles, and icons spawned via `asset_server.load` must have their `Transform.scale` multiplied by
    `1.0 / resolution_factor`.

## 4. Risks & Clarifications

### 4.1. Asset Consistency

We support heterogeneous scaling (some 3x, some 6x, some 1x). Each entity carries its own `ResolutionFactor`.

### 4.2. Shader UV Space

The shader works with normalized UVs but uses `sprite_width` (physical pixels) to compute sub-pixel anti-aliasing.
Mapping this correctly is critical for maintaining visual quality at non-integer zoom levels.

### 4.3. Script Requirements

The bash script for upscaling must:

1. Identify all PNGs in `assets/img/`.
2. Optionally exclude assets like vignettes or grainy smoke textures.
3. Call `xbrzscale` and output to `assets/upscaled/zoom0Nx_...`.
4. Rely on the existing `assetidx_updater.rs` for indexing (once updated).

## 5. Summary of Key Modifications

| Component              | File                                                                                         | Logic                                                               |
| :--------------------- | :------------------------------------------------------------------------------------------- | :------------------------------------------------------------------ |
| **AssetIdx Updater**   | [unhaunter/src/assetidx_updater.rs](unhaunter/src/assetidx_updater.rs)                       | Include `"upscaled"` folder in scan.                                |
| **TSX Image Resolver** | [crates/untmxmap-plugin/src/bevy.rs](crates/untmxmap-plugin/src/bevy.rs)                     | Priority look-up in `upscaled/` with prefix.                        |
| **Layout Builder**     | [crates/untmxmap-plugin/src/bevy.rs](crates/untmxmap-plugin/src/bevy.rs)                     | Multiply `tile_size` and `spacing` by $N$ for `TextureAtlasLayout`. |
| **Ghost Visuals**      | [crates/unghost-plugin/src/ghost.rs](crates/unghost-plugin/src/ghost.rs)                     | Adapt `ghost_scale_glitch_system` to `ResolutionFactor`.            |
| **Tile Spawn**         | [crates/unmapload-plugin/src/tile_spawning.rs](crates/unmapload-plugin/src/tile_spawning.rs) | Apply `1.0 / N` to flipped tile transforms.                         |
| **Shader Data**        | [crates/unrender-std/src/materials.rs](crates/unrender-std/src/materials.rs)                 | Set `sprite_width` to physical pixels.                              |
| **Settings**           | [crates/unsettings-core/src/graphics.rs](crates/unsettings-core/src/graphics.rs)             | Add `MaxResolutionFactor` setting.                                  |
