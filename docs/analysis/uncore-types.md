# Crate Analysis: `uncore-types`

## Core Responsibilities

This crate acts as a central repository for a wide variety of "data-in-motion" types. While `uncore-foundation` defines
static, foundational concepts, `uncore-types` defines the data structures that are used by Bevy systems and components
to manage game state, represent assets, and configure UI. The crate's description as a "bridge" is accurate.

## Public API

This crate exposes a large number of types, which can be grouped by theme:

- **Game State & Logic:**

  - `Difficulty`: A simple enum representing difficulty levels (the detailed struct is in `uncore-foundation`).
  - `SoundType`: Enum for identifying background sounds.
  - `MissionData`: A large struct containing all metadata for a mission (name, rewards, difficulty, etc.), parsed from
    TMX files.
  - `MiasmaGrid`: A struct holding `Array3<f32>` and `Array3<Vec2>` for pressure and velocity fields, strongly
    suggesting a 3D grid simulation for the fog effect.

- **Gear & Player Inventory:**

  - `GearKind`: An enum listing all available gear items. Maps gear to the evidence it finds.
  - `PlayerGearKind`: Struct holding the gear in the player's hands and inventory.
  - `EquipmentPosition`: Enum to track gear location (`Hand`, `Stowed`, `Deployed`).
  - `GearSpriteID`: **(2D Coupling)** Enum mapping gear states to specific indices in a 2D spritesheet.

- **UI & Assets:**
  - `GameAssets` (Bevy Resource): A top-level resource that holds handles to all other assets (images, fonts).
  - `ImageAssets` & `FontAssets`: Large structs containing Bevy `Handle<Image>` and `Handle<Font>` for every visual
    asset in the game.
  - `Anchors`: **(2D Coupling)** A struct and functions for calculating `Vec2` anchor points for sprites.
  - `EvidenceStatus`: A struct for representing the UI state of an evidence item.
  - `TruckButtonType` & `TruckButtonState`: Enums defining the type and state of buttons in the truck UI.
  - `ManualPageData` & `ManualChapter`: Structs defining the content and structure of the in-game manual.

## Dependencies

- `uncore-foundation`
- `bevy`
- `bevy_platform`
- `serde`
- `ndarray`
- `enum-iterator`

## Architectural Analysis & Notes

### SOLID Principles

- **Single Responsibility Principle (SRP):** This crate is a borderline case. It acts as a large "grab bag" of
  miscellaneous types. While the individual files (`gear_kind.rs`, `mission_data.rs`) are cohesive, the crate as a whole
  has low cohesion. It might benefit from being split into more domain-specific crates, for example `uncore-ui-types` or
  `uncore-asset-types`.

### 2D/3D Coupling

- **Conclusion:** **High Coupling.**
- **Reasoning:** This crate has several strong and direct ties to a 2D rendering pipeline:
  - `GearSpriteID` directly maps to a 2D spritesheet by index.
  - `Anchors` provides utility functions specifically for calculating `Vec2` anchor points for sprites.
  - The pervasive use of `Handle<Image>` and `Handle<TextureAtlasLayout>` in `ImageAssets` points to a 2D sprite-based
    approach.
- This crate would require significant refactoring for a 3D pivot. The data structures holding asset handles would need
  to be changed to support `Handle<Mesh>` and `Handle<StandardMaterial>`, and `GearSpriteID` would need to be replaced
  with a system for managing 3D models and animations.

### Game Logic vs. Engine Logic

- This crate is heavily weighted towards **Game-specific Logic**.
  - `GearKind`, `MissionData`, `ManualChapter`, `TruckButtonType`, and the specific asset handles in `ImageAssets` are
    all intimately tied to the design and UI of _Unhaunter_.
  - The `MiasmaGrid` is an interesting exception. A 3D fluid simulation could be considered **Engine-level**, though its
    specific application here is for the game's fog effect.

### Other Notes

- **Circular Dependency Avoidance:** Comments in the source code (e.g., in `types/root/map.rs` and `types/manual.rs`)
  explicitly state that types were moved to other crates (`uncore-assets`, `uncore-resources`) to avoid circular
  dependencies. This confirms that the `uncore-*` family of crates has a complex, tangled dependency graph that the
  original author was actively managing. This is a critical area to map out.
- The presence of `ndarray` for the `MiasmaGrid` is notable. It's a powerful numerical computation library, and its use
  here suggests a more complex simulation than might be expected.
