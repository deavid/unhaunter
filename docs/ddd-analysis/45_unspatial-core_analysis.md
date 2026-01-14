# DDD Analysis: `unspatial-core`

## 1. Bounded Context

**Spatial & Geometric Domain**. This crate owns the coordinate systems and mathematical primitives that define the
physical structure of the game world.

## 2. Responsibility

- **Coordinate Primitives**: Defines `Position` (continuous 3D space) and `BoardPosition` (discrete grid indices).
- **Metric Definition**: Establishes the real-world scale of the game (e.g., 1 tile = 0.6m x 0.6m x 2.2m).
- **Geometric Utilities**: Provides methods for distance calculation, neighbor lookup, and linear interpolation (lerp).
- **Perspective Constants**: Contains the transformation matrices for mapping logical 3D coordinates to isometric screen
  space.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `bevy_math`, `serde`.
- **Appropriateness**: High. It provides the essential math used by almost every other crate in the workspace.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Defines structs for positions and directions.
- **Logic**: Contains pure mathematical functions. It does not contain any Bevy systems.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. The inclusion of `to_screen_coord` directly on the `Position` struct leaks
  visual/perspective knowledge into the spatial domain. The spatial domain should ideally represent "What the world is",
  while the rendering domain should handle "How it looks on a 2D screen".
- **Infrastructure Leakage**: Low. It uses `bevy_math` for vector operations, which is a standard choice in this
  ecosystem.

---

## Technical Debt & Strategic Notes

- **Perspective Decoupling**: To achieve a clean separation for a potential 3D engine pivot, the `to_screen_coord`
  method should be moved from `unspatial-core::position` to a utility in `unrender-std`.
- **Global Z**: The `global_z` field in `Position` is a "render hack" used for visual layering in 2D. This is another
  example of visual logic leaking into the spatial core.
