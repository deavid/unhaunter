# DDD Analysis: `untmxmap-plugin`

## 1. Bounded Context

**Tiled Asset Infrastructure**. This crate implements the technical bridge that allows Bevy to load and understand
`.tmx` and `.tsx` files as native assets.

## 2. Responsibility

- **Asset Loading**: Implements `TmxMapLoader` and `TsxSheetLoader` to integrate Tiled files into the Bevy
  `AssetServer`.
- **Tileset Indexing**: Populates the `MapTileSetDb` with parsed tileset data, including handles to images and texture
  atlas layouts.
- **Property Extraction**: Parses custom properties from Tiled objects, converting them into game-specific domain types.
- **Material Binding**: Assigns the correct `CustomMaterial1` to loaded tiles based on tileset metadata.

## 3. Dependencies and Appropriateness

- **Dependencies**: `unboard-core`, `unassets-core`, `unevents-core`, `untypes-core`, `untiled-core`, `unrender-std`,
  `bevy`, `tiled`.
- **Appropriateness**: High as a technical infrastructure plugin. It depends on `unrender-std` because visual metadata
  is an intrinsic part of the map loading process.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Mostly works with raw types from the `tiled` crate and the `untiled-core` resource.
- **Logic**: Contains the imperative parsing logic and Bevy asset loader glue.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. The `properties.rs` module contains logic for interpreting specific Tiled property
  names as domain concepts.
- **Infrastructure Leakage**: High. It is explicitly an infrastructure layer for file format support.

---

## Technical Debt & Strategic Notes

- **Physical vs. Logical**: This crate is the "Physical Loader" (files to memory). It should remain isolated from
  "Logical Spawning" (data to world entities), which is the responsibility of `unmapload-plugin`.
- **Engine Portability**: If the game were to move to a different map format, this entire plugin would be swapped out
  while the rest of the game logic (depending on `unboard-core`) would remain intact.
