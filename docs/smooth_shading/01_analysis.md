# Smooth Shading Analysis

## Current Implementation Overview

The current smooth shading logic in `unhaunter` attempts to blend lighting between discrete cells of the board by
calculating lighting values at the corners of each tile and interpolating them in the fragment shader.

### 1. Sampling Logic and the "Rounding Bias"

In [crates/unlight-plugin/src/maplight.rs](crates/unlight-plugin/src/maplight.rs), the function `apply_lighting` samples
the lighting at five points per tile (Center, TR, TL, BR, BL) with 0.5-unit offsets.

The sampling uses `to_board_position()`, which relies on `f32::round()`. Because `round()` in Rust breaks ties by
rounding away from zero, we get a consistent but heavily biased sampling pattern:

- **Bias Example:** For a tile at `(10, 10)`, the BL corner at `(9.5, 9.5)` rounds to `(10, 10)`. The Center at
  `(10.0, 10.0)` also rounds to `(10, 10)`.
- **Result:** The bottom-left quadrant of _every_ tile is effectively flat-shaded because the corner value is identical
  to the center value.
- **Continuity vs. Accuracy:** While all 4 tiles meeting at an intersection use the same neighbor for their shared
  corner (ensuring no seams), they are all using the lux of _one_ specific neighbor instead of a blend. This shifts the
  visual center of light sources.
- **Upsampling Impact:** On low-res pixel art, this half-pixel bias is filtered by the browser/renderer's own scaling.
  On high-res upsampled art, the artificial "flat" regions and shifted highlights become very distracting.

**Proposed Fix:** Instead of snapping to the nearest cell, a corner sampling function should average the 4 neighbors:

```rust
// Logical pseudo-code for a smooth corner sample at (x.5, y.5)
let x_low = (pos.x - 0.01).floor();
let x_high = (pos.x + 0.01).ceil();
// Average contributing cells...
```

### 2. Hardcoded Shader Geometry and Upscaling

The shader [assets/shaders/custom_material1.wgsl](assets/shaders/custom_material1.wgsl) currently hardcodes values based
on a 128x128 pixel template:

```wgsl
var cnt: vec2<f32> = vec2(63.0/128.0, 95.0/128.0);
var dz: f32 = 2.0 * 35.0/128.0;
```

#### Why upscaling breaks this:

1. **Changing Dimensions:** Upsampling (e.g., to 512x512) still uses the same mesh UVs ($0..1$). If the upscaled texture
   maintains the exact same proportions, the $35/128$ ratio is technically still correct.
2. **Anchor Inconsistency:** The Y-center `95.0/128.0` ($\approx 0.742$) corresponds to the sprite anchor. If upscaled
   tiles or newer assets have different anchors (e.g., `-0.20` instead of `-0.25`), this hardcoded center will be offset
   from the actual floor diamond.
3. **Upscale Smoothing:** When `upscale_factor >= 2.0`, the shader uses bilinear sampling. The contrast between this
   smooth texture and the "shifted" or "biased" lighting logic (from point 1) makes the artifacts more apparent.

**Proposed Fix:** Replace hardcoded `128.0` with `material.sprite_width/height` and use `material.y_anchor` to compute
the center:

```wgsl
let cnt = vec2<f32>(0.5, 0.5 - material.y_anchor);
```

### 3. Crude UV Warp & Floor Focus

The user specifies that we only care about floor tiles. The current shader uses a crude isometric warp:

```wgsl
var uv1_y: f32 = (mesh.uv[1] - cnt[1]) * 2.0 + cnt[1];
```

This is a 1D scaling to approximate the isometric squash. While acceptable for floors, its inaccuracy contributes to the
"wobble" of the gradient as it moves across the tile.

### 4. Status of "Stopped Working"

The system likely degraded because:

- **Art Evolution:** Newer or upscaled assets no longer align with the magic numbers `63`, `95`, `35`, and `18`.
- **Rounding Bias Sensitivity:** As lighting became more dynamic (more light sources/flashlights), the "BL corner =
  Center" bias began creating visible "staircase" patterns in the smooth-upsampled gradients.

## Conclusion

The current system relies on perfect alignment between world-space floating point sampling, integer grid snapping
(`round()`), and local-space UV interpolation in the shader. This chain is easily broken by variations in sprite size,
anchor points, or map geometry.
