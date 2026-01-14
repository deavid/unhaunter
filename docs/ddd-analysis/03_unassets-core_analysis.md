# Crate Analysis: `unassets-core`

## Bounded Context

**Infrastructure / Data Access**

This crate serves as the bridge between external assets (Tiled maps, images, sounds) and the engine's internal handle
system. It is the technical foundation for providing the rest of the game with the resources they need.

## Responsibility

- Implement `AssetLoader` for custom formats (TMX, TSX, Index files).
- Provide centralized `Resource` types (e.g., `GameAssets`, `MapMetadata`, `Maps`) for accessing Bevy `Handle`s.
- Extract domain-specific metadata from raw asset files (e.g., converting Tiled properties into `MissionData`).
- Manage asset indexing for WASM/Web compatibility.

## Dependencies Audit

- `untypes-core`: Used for common primitives (coordinates, enums).
- `bevy`: Deeply coupled to Bevy's asset system.
- `tiled`: Used for map parsing.
- `thiserror`: Error handling.

## Encapsulation Assessment

- **Privacy**: The internal parsing logic is well-encapsulated within `AssetLoader` implementations. However, the data
  structures (`GameAssets`, `MapMetadata`) expose most fields as `pub`, which is common in Bevy but leads to high data
  coupling.
- **Leakage**: The crate is highly coupled to Bevy. It cannot exist without it.

## Semantic & Infrastructure Leakage

- **Content Leakage**: High. `image_assets.rs` (and similar files) often contain explicit references to specific game
  assets (manual pages, UI icons). Adding a new UI element often requires updating this "core" crate.
- **Logic Leakage**: `MissionData` (rewards, level requirements) is defined here. This couples "how to load a map" with
  "how the game economy/progression works".
- **Infrastructure Leakage**: Includes explicit workarounds for platform-specific limitations (WASM directory listing).

## Future Recommendations

- **Registry Pattern**: Move toward a more generic "Discovery" system where plugins can register their own required
  assets, instead of having a central `GameAssets` struct that knows about every single image in the game.
- **Domain Split**: Consider moving `MissionData` and progression-related types to a `unmission-core` or similar, as
  `unassets-core` should ideally only care about "loading bytes into handles".
- **Dynamic Assets**: Explore using Bevy's dynamic assets or a manifest-based system to reduce the need for hardcoded
  asset paths in Rust code.
