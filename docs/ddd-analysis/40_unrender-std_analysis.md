# DDD Analysis: `unrender-std`

## 1. Bounded Context

**Rendering Standard & Visual Infrastructure**. This crate provides shared rendering data structures, materials, and
low-level visual logic that are used across multiple plugins.

## 2. Responsibility

- **Visual Primitives**: Defines core visual types like `SpriteType`, `Animation`, and `FocusRing`.
- **Material Management**: Implement `CustomMaterial1`, a multi-layered shader material used for most game sprites
  (handling lighting, gamma, and sprite sheets).
- **Sprite Database**: Manages `SpriteDB`, a resource for caching and indexing pre-built sprite components for efficient
  map loading.
- **Lighting Logic**: Contains the heavy lifting for the lighting propagation system (`rebuild_lighting_field`).

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `unfoundation-core`, `unspatial-core`, `unboard-core`, `bevy_platform`, `ndarray`.
- **Appropriateness**: High as a shared utility crate. It allows domain-specific plugins to use standardized visual
  representations without depending on each other.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Defines most of the visual components (`GameSprite`, `MapTileSprite`, `GameSound`).
- **Logic**: Contains standalone logic for lighting rebuilds and material specialization. It does NOT define Bevy
  systems directly (no `Plugin` implementation); instead, it provides functions and types that are used by `-plugin`
  crates.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. The `SpriteType` enum contains domain-specific variants like `Ghost`, `Player`, and
  `Miasma`. While useful for the renderer to know how to treat these visually, it technically leaks domain concepts into
  the rendering layer.
- **Infrastructure Leakage**: High (by design). It is deeply coupled with Bevy's rendering API (`Material2d`,
  `Handle<Image>`, `Mesh2d`). This is appropriate for a rendering-focused infrastructure crate.

---

## Technical Debt & Strategic Notes

- **Generic Registry**: The `SpriteType` enum could eventually be replaced by a more generic tagging system if the game
  needs to support moddable entity types without recompiling `unrender-std`.
- **Lighting Orchestration**: The fact that `ungame-plugin` calls `rebuild_lighting_field` directly from a system
  illustrates how `unrender-std` acts as a math/logic library for the plugins.
