# DDD Analysis: `untiled-core`

## 1. Bounded Context

**Tiled Integration Infrastructure**. This crate provides the low-level data structures needed to bridge external Tiled
Map Editor data (.tmx, .tsx) with the Bevy engine.

## 2. Responsibility

- **Tileset Management**: Defines `MapTileSet` and `MapTileSetDb` for storing and looking up parsed tilesets.
- **Visual Mapping**: Bridges raw Tiled data with Bevy `TextureAtlasLayout` and the game's `CustomMaterial1`.
- **Abstract Layering**: Provides the `AtlasData` abstraction to handle both sprite-sheet-based and
  individual-image-based tilesets.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unrender-std`, `bevy`, `bevy_platform`, `tiled`.
- **Appropriateness**: High as a specialized infrastructure core. It depends on `unrender-std` because visual metadata
  is tightly coupled with tilesets in this engine.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Defines structs for map asset management.
- **Logic**: Mostly data-holding logic; the actual parsing and loading logic resides in the `untmxmap-plugin`.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It is aware of the specific isometric material (`CustomMaterial1`) used by the game.
- **Infrastructure Leakage**: High. It is explicitly designed as a wrapper around the `tiled` library.

---

## Technical Debt & Strategic Notes

- **Generic Map Format**: If the game were to move away from Tiled to a different editor, this crate would be replaced.
- **Dependency on Render**: The dependency on `unrender-std` reflects how maps are treated as a "visual first" entity in
  this project.
