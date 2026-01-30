# Unhaunter Bevy 0.18 Migration Guide

This document contains a curated subset of the Bevy 0.18 migration guide specifically for the Unhaunter project.

---

## 1. `BorderRadius` added to `Node`

### Original Guide

### `BorderRadius` has been added to `Node` and is no longer a component

**PRs:** [#21781](https://github.com/bevyengine/bevy/pull/21781)

`BorderRadius` is no longer a component, instead a `border_radius: BorderRadius` field has been added to `Node`.

### Unhaunter Context

Found in:
[crates/unclassic-mode-plugin/src/systems/hint_ui_system.rs](crates/unclassic-mode-plugin/src/systems/hint_ui_system.rs#L67)

**Example Migration:**

```rust
// Before:
commands.spawn((
    HintBoxUIRoot,
    Node { ..default() },
    BorderRadius::all(Val::Px(HINT_BOX_BORDER_RADIUS_VAL)),
));

// After:
commands.spawn((
    HintBoxUIRoot,
    Node {
        border_radius: BorderRadius::all(Val::Px(HINT_BOX_BORDER_RADIUS_VAL)),
        ..default()
    },
));
```

---

## 2. `Mesh::try_*` mesh functions

### Original Guide

### Use `Mesh::try_*` mesh functions for `Assets<Mesh>` entries when there can be `RenderAssetUsages::RENDER_WORLD`-only meshes

**PRs:** [#21732](https://github.com/bevyengine/bevy/pull/21732)

Previously, the `Assets<Mesh>` resource would not retain `RenderAssetUsages::RENDER_WORLD`-only meshes once their data
was extracted.

In 0.18, `Assets<Mesh>` retains `RenderAssetUsages::RENDER_WORLD`-only meshes, even after their data is extracted. To
handle such meshes, `Mesh` now contains `Mesh::try_*` functions which return a `Result<..., MeshAccessError>`. These
functions return an `Err(MeshAccessError,ExtractedToRenderWorld)` when the mesh has already been extracted.

If `Assets<Mesh>` can contain `RenderAssetUsages::RENDER_WORLD`-only meshes, the following `Mesh` functions should be
changed to their `try_*` equivalent and handled appropriately:

```rust
// assets: Res<'w, Assets<Mesh>>
let mesh = assets.get(some_mesh_handle).unwrap() // or assets.get_mut(some_mesh_handle).unwrap()

// 0.17
mesh.insert_attribute(...)
mesh.with_inserted_attribute(...)
mesh.remove_attribute(...)
mesh.with_removed_attribute(...)
mesh.contains_attribute(...)
mesh.attribute(...)
mesh.attribute_mut(...)
mesh.attributes(...)
mesh.attributes_mut(...)
mesh.insert_indices(...)
mesh.with_inserted_indices(...)
mesh.indices(...)
mesh.indices_mut(...)
mesh.remove_indices(...)
mesh.with_removed_indices(...)
mesh.duplicate_vertices(...)
mesh.with_duplicated_vertices(...)
mesh.compute_normals(...)
mesh.compute_flat_normals(...)
mesh.compute_smooth_normals(...)
mesh.compute_area_weighted_normals(...)
mesh.compute_custom_smooth_normals(...)
mesh.with_computed_normals(...)
mesh.with_computed_flat_normals(...)
mesh.with_computed_smooth_normals(...)
mesh.with_computed_area_weighted_normals(...)
mesh.with_custom_smooth_normals(...)
mesh.transformed_by(...)
mesh.transform_by(...)
mesh.translated_by(...)
mesh.translate_by(...)
mesh.rotated_by(...)
mesh.rotate_by(...)
mesh.scaled_by(...)
mesh.scale_by(...)
mesh.normalize_joint_weights(...)
// when feature = morph enabled
mesh.has_morph_targets(...)
mesh.set_morph_targets(...)
mesh.morph_targets(...)
mesh.with_morph_targets(...)
mesh.set_morph_target_names(...)
mesh.with_morph_target_names(...)
mesh.morph_target_names(...)

// 0.18
mesh.try_insert_attribute(...)
mesh.try_with_inserted_attribute(...)
mesh.try_remove_attribute(...)
mesh.try_contains_attribute(...)
mesh.try_with_removed_attribute(...)
mesh.try_attribute_option(...) // or mesh.try_attribute(...)
mesh.try_attribute_mut_option(...) // or mesh.try_attribute_mut(...)
mesh.try_attributes(...)
mesh.try_attributes_mut(...)
mesh.try_insert_indices(...)
mesh.try_with_inserted_indices(...)
mesh.try_indices_option(...) // or mesh.try_indices(...)
mesh.try_indices_mut_option(...) // or mesh.try_indices_mut(...)
mesh.try_remove_indices(...)
mesh.try_with_removed_indices(...)
mesh.try_duplicate_vertices(...)
mesh.try_with_duplicated_vertices(...)
mesh.try_compute_normals(...)
mesh.try_compute_flat_normals(...)
mesh.try_compute_smooth_normals(...)
mesh.try_compute_area_weighted_normals(...)
mesh.try_compute_custom_smooth_normals(...)
mesh.try_with_computed_normals(...)
mesh.try_with_computed_flat_normals(...)
mesh.try_with_computed_smooth_normals(...)
```

### Unhaunter Context

Found in: [crates/unrender-std/src/utils/quadcc.rs](crates/unrender-std/src/utils/quadcc.rs#L47-L54)

Since Unhaunter often manually constructs meshes for custom rendering:

```rust
// Proposed change in quadcc.rs
let mut mesh = Mesh::new(...);
mesh.try_insert_indices(indices).unwrap();
mesh.try_insert_attribute(Mesh::ATTRIBUTE_POSITION, positions).unwrap();
```

---

## 3. `RenderTarget` is now a component

### Original Guide

**PRs:** [#20917](https://github.com/bevyengine/bevy/pull/20917)

`RenderTarget` has been moved from a field on `Camera` to a separate required component.

When spawning a camera, specify `RenderTarget` as a component instead of setting `camera.target`:

```rust
// 0.17
commands.spawn((
    Camera3d::default(),
    Camera {
        target: RenderTarget::Image(image_handle.into()),
        ..default()
    },
));

// 0.18
commands.spawn((
    Camera3d::default(),
    RenderTarget::Image(image_handle.into()),
));
```

### Unhaunter Context

Relevant for custom camera spawning and cleanup in:

- [unhaunter/src/app.rs](unhaunter/src/app.rs)
- [crates/unengine-plugin/src/systems.rs](crates/unengine-plugin/src/systems.rs#L7)

---

## 4. `LoadContext::path` now returns `AssetPath`

### Original Guide

### `LoadContext::path` now returns `AssetPath`

**PRs:** [#21713](https://github.com/bevyengine/bevy/pull/21713)

`LoadContext::asset_path` has been removed, and `LoadContext::path` now returns `AssetPath`. So the migrations are:

- `load_context.asset_path()` -> `load_context.path()`
- `load_context.path()` -> `load_context.path().path()`
  - While this migration will keep your code running, seriously consider whether you need to use the `Path` itself. The
    `Path` does not support custom asset sources, so care needs to be taken when using it directly. Consider instead
    using the `AssetPath` instead, along with `AssetPath::resolve_embed`, to properly support custom asset sources.

### Unhaunter Context

Unhaunter uses custom `AssetLoader` implementations in:

- [crates/unassets-core/src/assets/tmxmap.rs](crates/unassets-core/src/assets/tmxmap.rs)
- [crates/unassets-core/src/assets/tsxsheet.rs](crates/unassets-core/src/assets/tsxsheet.rs)
- [crates/unassets-core/src/assets/index.rs](crates/unassets-core/src/assets/index.rs)

Any logic relying on the raw `&Path` from `_load_context.path()` will need to call `.path()` on the returned
`AssetPath`.

---

## 5. Feature cleanup (Picking)

### Original Guide

`animation` has been renamed to `gltf_animation`. `bevy_sprite_picking_backend` has been renamed to `sprite_picking`.
`bevy_ui_picking_backend` has been renamed to `ui_picking`. `bevy_mesh_picking_backend` has been renamed to
`mesh_picking`.

### Unhaunter Context

Found in: [crates/unpicking-plugin/Cargo.toml](crates/unpicking-plugin/Cargo.toml)

Update features for `bevy_picking` as required.

---

## 6. Input Features

### Original Guide

**PRs:** [#21447](https://github.com/bevyengine/bevy/pull/21447)

`bevy_input` provides primitives for all kinds of input. But on consoles you usually don't have things like touch. On
more obscure platforms, like GBA, only gamepad input is needed.

If you use `bevy_window` or `bevy_gilrs`, they will automatically enable the necessary features on `bevy_input`. If you
don't depend on them (for example, if you are developing for a platform that isn't supported by these crates), you need
to enable the required input sources on the `bevy_input` / `bevy` crate manually:

```toml
# 0.17
bevy = { version = "0.17", default-features = false }

# 0.18 (enable sources that you actually use):
bevy = { version = "0.18", default-features = false, features = [
  "mouse",
  "keyboard",
  "gamepad",
  "touch",
  "gestures",
] }
```

### Unhaunter Context

Check [Cargo.toml](Cargo.toml) dependencies. Unhaunter currently uses
`bevy = { version = "0.17", features = ["jpeg", "serialize"] }`.

---

## 7. `#[reflect(...)]` support only parentheses

### Original Guide

### `#[reflect(...)]` now supports only parentheses

**PRs:** [#21400](https://github.com/bevyengine/bevy/pull/21400)

Previously, the `#[reflect(...)]` attribute of the `Reflect` derive macro supported parentheses, braces, or brackets, to
standardize the syntax going forward, it now supports only parentheses.

```rust
/// 0.17
#[derive(Clone, Reflect)]
#[reflect[Clone]]

/// 0.18
#[derive(Clone, Reflect)]
#[reflect(Clone)]
```

```rust
/// 0.17
#[derive(Clone, Reflect)]
#[reflect{Clone}]

/// 0.18
#[derive(Clone, Reflect)]
#[reflect(Clone)]
```

### Unhaunter Context

Used throughout the project for component reflection. Example:
[crates/unfoundation-core/src/types/gear.rs](crates/unfoundation-core/src/types/gear.rs#L6)

Unhaunter is mostly using parentheses already, so this is a preventative note.

---

## 8. Tick-related refactors

### Original Guide

[#21562](https://github.com/bevyengine/bevy/pull/21562), [#21613](https://github.com/bevyengine/bevy/pull/21613)

`TickCells` is now `ComponentTickCells`.

`ComponentSparseSet::get_with_ticks` now returns `Option<(Ptr, ComponentTickCells)>` instead of
`Option<(Ptr, TickCells, MaybeLocation)>`.

The following types have been moved from the `component` module to the `change_detection` module:

- `Tick`
- `ComponentTicks`
- `ComponentTickCells`
- `CheckChangeTicks`

### Unhaunter Context

Unhaunter uses change detection in various game logic systems. Any explicit imports of these types (e.g., from
`bevy::ecs::component`) will need to be updated to `bevy::ecs::change_detection`.

---

## 5. Feature Renames & Input Features

### Original Guide

- `bevy_sprite_picking_backend` -> `sprite_picking`
- `bevy_ui_picking_backend` -> `ui_picking`
- `bevy_input` features (`mouse`, `keyboard`, etc.) must be enabled manually if not using `DefaultPlugins` fully.

### Unhaunter Context

Check [Cargo.toml](Cargo.toml) and [crates/unpicking-plugin/Cargo.toml](crates/unpicking-plugin/Cargo.toml). Unhaunter
uses `bevy_picking`, which will likely need feature rename updates.

---

## 6. `#[reflect(...)]` Parentheses

### Original Guide

Previously, the `#[reflect(...)]` attribute supported parentheses, braces, or brackets. Now it supports only
parentheses.

### Unhaunter Context

Unhaunter is already compliant with this (e.g.,
[crates/unfoundation-core/src/types/gear.rs](crates/unfoundation-core/src/types/gear.rs#L6)), but keep this in mind when
adding new components.

---

## 7. Tick-related Refactors

### Original Guide

`TickCells` is now `ComponentTickCells`. The following types have been moved to the `change_detection` module: `Tick`,
`ComponentTicks`, `ComponentTickCells`, `CheckChangeTicks`.

### Unhaunter Context

Unhaunter relies on change detection for some systems. If any low-level change detection types were imported explicitly,
updating the module path to `bevy::ecs::change_detection` will be necessary.
