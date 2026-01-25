# Proposed Upscaling Workflow

This document outlines the high-level design for implementing and using upscaled assets in Unhaunter.

## 1. Batch Upscaler Tool

A bash script will be provided to automate the upscaling process using `xbrzscale`.

- **Dependencies**: Requires `xbrzscale` installed on the system (`sudo apt install xbrzscale`).
- **Input**: Operates on `assets/img/*.png` files (ignoring `.aseprite` files).
- **Operation**:
  1. The script takes a target scale factor (2x to 9x).
  2. It creates the target directory `assets/upscaled/{Nx}/img/`.
  3. It runs `xbrzscale` on all PNGs.
- **Uniformity**: Everything is upscaled uniformly; the tool does not need to distinguish between tilesets and sprites.

## 2. Asset Discovery & AssetIdx

The game will use a "Highest Available" loading strategy.

1. **Settings**: A User Setting `MaxUpscaleFactor` (1x-9x) will limit the search.
2. **Resolution**: For a requested asset `img/item.png`, the loader checks:
    - `assets/upscaled/{MaxNx}/img/item.png`
    - ... descending factors ...
    - `assets/img/item.png` (fallback)
3. **AssetIdx**: We will use `assetidx` files to efficiently check for the existence of upscaled assets, especially in
    WASM environments where directory listing is not possible.

## 3. Tiled Integration (Zero TSX changes)

Tiled maps will keep their original `.tsx` file references.

1. **Loader Interception**: The `UnhaunterMapLoader` (and its helper `resolve_tiled_image_path`) will be modified to
    redirect image requests to the `upscaled/` folders.
2. **Layout Compensation**: In `bevy_load_map` (where `TextureAtlasLayout` is built), the `tile_size` and `spacing`
    extracted from the TSX will be multiplied by the $N$ factor of the successfully loaded image.

## 4. Sprite Collections (Rust)

For assets loaded via `bevy_asset_loader`:

1. **Dynamic Metadata**: We need to move away from purely hardcoded `tile_size` in macros. A possible approach is to
    store the "Base Resolution" metadata and compute the `TextureAtlasLayout` at runtime after the image resolution is
    known.
2. **Scale Adjustment**: Entities spawned with upscaled textures will be assigned a `ResolutionFactor(f32)`. Their
    `Transform.scale` will be adjusted during initialization to compensate for the higher pixel density.

## 5. UI & Settings

- **Settings Menu**: Add a slider or dropdown to select `MaxUpscaleFactor`.
- **Consistency**: Changing the upscale factor may require a game restart or a full asset reload to apply changes
  globally.
