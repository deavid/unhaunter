# Phase 4: Remaining Splits Plan

**Date:** December 31, 2025

## 1. Objective
This plan details the steps to split the remaining complex crates: `ungear` and `unrender`. These crates currently mix data (components, resources, types) with logic (systems, plugins), creating dependency bottlenecks.

## 2. Splitting `ungear`

**Goal:** Split `ungear` into `ungear-core` (data) and `ungear-plugin` (logic).

### 2.1. Create `ungear-core`
- **Location:** `crates/ungear-core`
- **Contents:**
    - `src/components/`: All component definitions.
    - `src/resources/`: All resource definitions.
    - `src/types/`: All shared types (enums, structs).
- **Dependencies:**
    - `bevy` (minimal features)
    - `uncore-*` crates (as needed for types)
    - **Crucially:** Should NOT depend on `unrender` or `ungear-plugin`.

### 2.2. Create `ungear-plugin`
- **Location:** `crates/ungear-plugin`
- **Contents:**
    - `src/systems/`: All systems.
    - `src/gear_stuff.rs`: `GearStuff` system param.
    - `src/evidence_systems.rs`: Evidence logic.
    - `src/plugin.rs`: The `UngearPlugin`.
- **Dependencies:**
    - `ungear-core`
    - `unrender-std` (for `GameSprite` and materials)
    - `uncore-*` crates
    - `bevy`

### 2.3. Migration Steps
1.  Create `crates/ungear-core`.
2.  Move `components`, `resources`, `types` from `ungear` to `ungear-core`.
3.  Update `ungear` (renamed to `ungear-plugin`) to depend on `ungear-core`.
4.  Update all other crates that depend on `ungear` to depend on `ungear-core` (for types) or `ungear-plugin` (for the plugin).

## 3. Splitting `unrender`

**Goal:** Split `unrender` into `unrender-std` (shared rendering data/utils) and `unrender-plugin` (rendering systems).

### 3.1. Create `unrender-std`
- **Location:** `crates/unrender-std`
- **Contents:**
    - `src/components/`: `GameSprite`, `SpriteType`, etc.
    - `src/resources/`: `VisibilityData`, etc.
    - `src/materials.rs`: `CustomMaterial1` and `Material2d` impls.
    - `src/board/`: `SpriteDB`, `MapTileComponents`.
    - `src/utils/`: `light.rs` (color utils).
- **Dependencies:**
    - `bevy` (rendering features)
    - `uncore-board` (for `Behavior`, `SpriteCVOKey`)
    - `uncore-types`

### 3.2. Create `unrender-plugin`
- **Location:** `crates/unrender-plugin`
- **Contents:**
    - `src/systems/`: All rendering systems.
    - `src/plugin.rs`: The `UnrenderPlugin`.
- **Dependencies:**
    - `unrender-std`
    - `bevy`
    - `uncore-*` crates

### 3.3. Migration Steps
1.  Create `crates/unrender-std`.
2.  Move `components`, `resources`, `materials.rs`, `board`, `utils` from `unrender` to `unrender-std`.
3.  Update `unrender` (renamed to `unrender-plugin`) to depend on `unrender-std`.
4.  Update all other crates that depend on `unrender` to depend on `unrender-std` (for components/materials) or `unrender-plugin` (for the plugin).

## 4. Execution Order
1.  **Split `unrender` first.** Since `ungear` depends on `unrender`, splitting `unrender` first allows `ungear-plugin` to depend on `unrender-std` cleanly.
2.  **Split `ungear` second.** Once `unrender` is split, `ungear` can be split with clear dependencies.

## 5. Detailed Task List

### Unrender Split
- [x] Create `crates/unrender-std`.
- [x] Move `unrender/src/{components,resources,materials.rs,board,utils}` to `unrender-std`.
- [x] Fix imports in `unrender-std`.
- [x] Rename `unrender` to `unrender-plugin`.
- [x] Update `unrender-plugin` to depend on `unrender-std`.
- [x] Update workspace `Cargo.toml` (if applicable) and other crates to point to `unrender-std` or `unrender-plugin`.

### Ungear Split
- [x] Create `crates/ungear-core`.
- [x] Move `ungear/src/{components,resources,types}` to `ungear-core`.
- [x] Fix imports in `ungear-core`.
- [x] Rename `ungear` to `ungear-plugin`.
- [x] Update `ungear-plugin` to depend on `ungear-core` and `unrender-std`.
- [x] Update workspace `Cargo.toml` and other crates.
