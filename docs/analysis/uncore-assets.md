# Crate Analysis: `uncore-assets`

## Core Responsibilities

This crate is responsible for defining and loading custom Bevy `Asset` types. Its primary role is to get raw asset data
(especially for Tiled maps and tilesets) into the Bevy asset system, preparing it for further processing by other
systems. It defines custom `AssetLoader`s for `.tmx` (Tiled Map), `.tsx` (Tiled Tileset), and a custom `.assetidx`
format.

## Public API

The crate exposes two main modules:

### `assets` Module

This module contains the `AssetLoader` implementations for the custom asset types.

- **`AssetIdx` & `AssetIdxLoader`**:

  - Loads `.assetidx` files, which are simple text files listing asset paths.
  - The comments explicitly state this is a workaround for the inability to list directories in WASM environments,
    allowing the game to discover assets at runtime.
  - The loaded `AssetIdx` contains a `Vec<String>` of asset paths.

- **`TmxMap` & `TmxMapLoader`**:

  - Loads `.tmx` files (Tiled maps).
  - The `TmxMap` asset itself stores the raw `bytes` of the `.tmx` file for deferred full parsing.
  - **Performance Optimization**: It uses a `naive_tmx_loader` function to perform a fast, line-by-line parse to extract
    top-level map properties (`display_name`, `is_campaign_mission`, etc.) without incurring the cost of a full XML DOM
    parse. This is a crucial optimization for inspecting many map files at startup.

- **`TsxSheet` & `TsxSheetLoader`**:
  - Loads `.tsx` files (Tiled tileset sheets).
  - Similar to `TmxMap`, the `TsxSheet` asset simply stores the raw `bytes` of the file, presumably for later parsing
    when the tileset is actually used.

### `types` Module

This module defines higher-level structs that wrap the raw asset handles.

- **`Map`**: A struct containing the `name`, `path`, Bevy `Handle<TmxMap>`, and the associated `MissionData` for a map.
- **`Sheet`**: A struct containing the `path` and Bevy `Handle<TsxSheet>` for a tileset.

## Dependencies

- `uncore-types`
- `bevy`
- `bevy_platform`
- `thiserror`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** This crate adheres well to SRP. Its responsibility is clearly defined: load
  custom asset formats into the Bevy asset system. It does not contain game logic or rendering logic; it only handles
  the initial loading phase.
- **Open/Closed Principle (OCP):** The pattern is extensible. New custom asset types and their loaders could be added
  without modifying the existing ones.

### 2D/3D Coupling

- **Conclusion:** **Very High Coupling.**
- **Reasoning:** This crate is fundamentally tied to the 2D isometric pipeline through its deep integration with the
  Tiled map editor's file formats (`.tmx` and `.tsx`).
  - Tiled is primarily a 2D map editor, and while it has isometric and some 3D features, its workflow is intrinsically
    2D-centric. This entire crate is built to support that workflow.
  - The data structures and parsers are designed specifically for the XML-based `.tmx` and `.tsx` formats.
- A pivot to 3D would require a complete replacement or significant augmentation of this crate. A new system for loading
  3D models (like `.gltf` or `.obj`), materials, and potentially a different map/scene format (like Bevy's own scene
  format or a 3D level editor's format) would be necessary. The `TmxMapLoader` and `TsxSheetLoader` would likely be
  deprecated or relegated to a legacy 2D mode.

### Game Logic vs. Engine Logic

- **Conclusion:** This crate is almost entirely **Engine-level Logic**.
- **Reasoning:** An asset loading system is a core component of any game engine. While the specific formats (`.tmx`,
  `.tsx`) are a choice, the infrastructure for defining custom `Asset` types and `AssetLoader`s is a generic, reusable
  engine pattern. The `AssetIdx` loader is also a generic engine-level solution for WASM asset discovery. The logic here
  is not specific to _Unhaunter_'s gameplay mechanics.

### Other Notes

- **WASM Compatibility:** The existence of `AssetIdxLoader` is a deliberate and important architectural choice to ensure
  the game can run in a web browser. This shows forethought in platform compatibility.
- **Deferred Parsing:** The pattern of loading raw bytes (`Vec<u8>`) into an asset struct and parsing them later is a
  smart performance optimization. It separates the fast, initial I/O operation from the slower, CPU-intensive parsing,
  which only needs to happen when an asset is actually used. The naive TMX parser is another example of this
  performance-conscious design.
- **Dependency on `uncore-types`:** The `Map` struct in this crate contains `MissionData` from `uncore-types`, showing a
  clear link where asset metadata is combined with game-specific data structures.
