# Crate Analysis: `unlight-plugin`

## Bounded Context

**Map Lighting & Visibility**

This crate manages how light and visibility are calculated and applied to the world. It determines what the player can
see and how bright or colored objects appear.

## Responsibility

- Implement the `UnhaunterLightPlugin`.
- Perform Line-of-Sight (LoS) calculations based on obstacles in `unboard-core`.
- Aggregate light sources (flashlights, ambient, ghosts) into a global light field.
- Apply lighting colors and visibility alpha to all sprites in the scene.
- Modulate ambient audio volumes based on the indoor/outdoor visibility ratio.

## Dependencies Audit

- `unboard-core`: To check for lighting obstacles and room types.
- `unrender-std`: For the `GameSprite` and `Material` components that are modified by lighting.
- `ndarray`: Used for the high-performance lighting/visibility grid calculations.
- `fastapprox`: Used for optimized lighting math.
- `bevy`: Deeply integrated.

## Encapsulation Assessment

- **Privacy**: High. Internal algorithms and system modules are private.
- **Behavior**: Acts as a "system-of-systems" for visibility, pulling in data from ghosts, gear, and board.

## Semantic & Infrastructure Leakage

- **Visual Leakage**: Deeply coupled to the specific rendering implementation of sprites and materials. It "knows"
  exactly how to tint a `GameSprite`.
- **Domain Coupling**: Includes audio modulation logic (`ambient_audio.rs`). While visibility _influences_ audio (indoor
  vs outdoor), having the audio logic inside the light plugin is a leakage of concerns.

## Future Recommendations

- **Audio Extraction**: Move ambient audio modulation to an `unambient-plugin` that listens for changes in a "Visibility
  Ratio" resource published by `unlight-plugin`.
- **Abstract Tinting**: Instead of `unlight-plugin` modifying sprite colors directly, it should update a `LightingData`
  component on each entity. The rendering engine can then use that data in a shader or a separate pass to apply the
  tint, decoupling light logic from sprite code.
