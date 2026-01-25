# Upscale Art Analysis

This document explores the concept of using high-resolution art assets (upscaled) while maintaining the original game
world scale and logic.

## 1. Goal

The primary objective is to allow the game to load higher-resolution versions of its art assets (upscaled via
`xbrzscale`) while maintaining original game world scale and logic.

The goal is to move away from "jagged" pixel art edges and high-frequency noise towards a smoother "liminal", "oil", or
"watercolor" visual style. By upscaling the source assets, we can keep the art production pipeline low-resolution (e.g.,
in Aseprite) while achieving a higher-fidelity look in-game.

## 2. Current Asset Workflow

The game currently uses:

- **Aseprite Source**: `.aseprite` files in `assets/img/`. Authoring happens here at low resolution.
- **PNG Exports**: Matching `.png` files in `assets/img/`. These are the actual assets loaded by the game.

## 3. High-Res Asset Discovery

Instead of a postfix, we will use a dedicated directory structure for upscaled assets:

- **Originals**: `assets/img/item.png`
- **Upscaled**: `assets/upscaled/{Nx}/img/item.png` (where `{Nx}` is `2x`, `3x`, ..., `9x`).

### Discovery Mechanism

The game will attempt to load the highest resolution available for each asset, up to a maximum defined in the user
settings.

1. Consult `assetidx` to check for the existence of files in `assets/upscaled/{Nx}/`.
2. Select the highest `{Nx}` that is $\le$ `MaxUpscaleFactor` (user setting).
3. Fall back to `assets/img/` (1x) if no upscaled version is found.

## 4. Technical Challenges

### 4.1. Texture Atlas Layouts

Most sprites use `TextureAtlasLayout`. If we load a 3x resolution image for a 64x64 base tile:

- The actual texture will be $192 \times 192$.
- The `TextureAtlasLayout` (which defines the grid) must have its `tile_size` and `spacing` multiplied by $N$.
- Failure to do this will result in the engine only rendering the top-left corner of the larger texture.

### 4.2. Entity Scaling (Downscaling)

To keep the sprite the same size in the game world, the `Transform` scale of the entity must be divided by $N$. If an
entity has a world scale $S$, and we use an $N x$ texture, the entity's `Transform.scale` must be set to $S / N$.

This adjustment should happen at **spawn/load time**, not continuously every frame.

### 4.3. Tiled Maps (`.tmx`)

Tiled maps reference `.tsx` (tileset) files. We will **not** modify the `.tsx` files. Instead, the `UnhaunterMapLoader`
will:

1. Intercept the image path resolution.
2. Switch the source image to the upscaled version.
3. Internally compensate for the $N x$ resolution when building the `TextureAtlasLayout` for the tileset.

### 4.4. Filtering

The game's custom shader handles filtering (linear when small, nearest when upscaling). This logic is expected to work
without modification with upscaled assets.

## 5. Proposed Architecture: "Resolution Factor"

We can introduce a `ResolutionFactor(f32)` component.

- **Storage**: Attached to entities with a `Sprite` or `TextureAtlas`.
- **Logic**: When spawning or initializing a sprite, the `ResolutionFactor` is determined based on the loaded asset.
- **Transform**: The entity's `Transform.scale` is adjusted once: `transform.scale /= resolution_factor`.
- **Dynamic Logic**: Components that modify scale (e.g., ghost repellent effects) must account for the
  `ResolutionFactor` to avoid overwriting the compensation.

## 6. Open Questions for Strategy Refinement

- **AssetIdx Generation**: Since `assetidx` is used for WASM compatibility, the batch script must generate these for
  each upscale directory.
- **Shader Coordinates**: Does the custom shader use absolute pixel counts or normalized UVs? This determines if
  constants like `sprite_width` in `CustomMaterial1` need to be scaled or remain at the board-logical dimensions.
- **UI Interaction**: Will UI assets use the same `Transform`-based downscaling, or will they leverage Bevy's `UiScale`?
- **Ghost Scaling**: Specific interactions that modify transform scale (like repellent effects) must be audited to
  multiply their logic by the `ResolutionFactor`.
