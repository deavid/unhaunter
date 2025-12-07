This document condenses the official Bevy 0.16 -> 0.17 migration guide, filtering out 3D features (PBR, glTF, etc.) and
focusing strictly on the APIs used in **Unhaunter** (`uncore`, `unlight`, `unstd`, `untruck`, etc.).

---

# Unhaunter Migration Guide: 0.16 to 0.17

## 1. Project Configuration (WASM Blocker)

**Affected:** `.cargo/config.toml`, `Justfile`, CI/CD workflows.

Bevy 0.17 updated `rand` and `getrandom`. WASM builds will **fail** to compile without explicit backend configuration.

- **Action:** Add `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` to your build environment variables for WASM builds.

## 2. Core Architecture: The Event Split

**Affected:** Every system using `EventWriter` or `EventReader` (approx. 90% of your systems).

Bevy has split events into **Messages** (buffered queues, what you use now) and **Events** (immediate observers).

- **Rename Types:**
  - `Event` trait (derive) $\rightarrow$ `Message`
  - `EventWriter<T>` $\rightarrow$ `MessageWriter<T>`
  - `EventReader<T>` $\rightarrow$ `MessageReader<T>`
  - `app.add_event::<T>()` $\rightarrow$ `app.add_message::<T>()`
- **Rename Methods:**
  - `writer.send(val)` $\rightarrow$ `writer.write(val)`

_Note: Bevy also introduced `Observer`s. You generally don't need to migrate to them yet, just rename your existing
logic to Messages to keep behavior identical._

## 3. The Great Import Reorganization

**Affected:** `uncore`, `unlight`, `unstd`.

Rendering types have been moved out of `bevy_render`. You must update imports in `uncore/src/components` and `types`.

| Type                 | Old Import                      | **New Import**                 |
| :------------------- | :------------------------------ | :----------------------------- |
| `Camera`, `Camera2d` | `bevy::render::camera`          | **`bevy::camera`**             |
| `Visibility`         | `bevy::render::view`            | **`bevy::camera::visibility`** |
| `Mesh`               | `bevy::render::mesh`            | **`bevy::mesh`**               |
| `Image`              | `bevy::render::texture`         | **`bevy::image`**              |
| `Shader`             | `bevy::render::render_resource` | **`bevy::shader`**             |
| `Material2d`         | `bevy::sprite`                  | **`bevy::sprite_render`**      |

## 4. Sprites & Rendering

**Affected:** `unmapload` (tile spawning), `unstd` (materials), `unghost`.

### A. Anchors are now Components

The `anchor` field has been **removed** from the `Sprite` struct. It is now a required component.

- **Before:**
  ```rust
  commands.spawn(Sprite {
      image: handle,
      anchor: Anchor::Center, // Compilation Error
      ..default()
  });
  ```
- **After:**
  ```rust
  commands.spawn(Sprite {
      image: handle,
      ..default()
  }).insert(Anchor::CENTER); // Note: Enum variants now CONSTS
  ```

### B. Material2d Location

Your custom material `CustomMaterial1` (in `unstd/src/materials.rs`) relies on `Material2d`. This trait has moved to
`bevy::sprite_render`.

## 5. User Interface (High Breakage Risk)

**Affected:** `untruck`, `unmenu`, `unsummary`, `uncoremenu`.

### A. Transform Split

`Node` bundles no longer use standard `Transform`. Bevy has introduced `UiTransform`.

- Any code manually manipulating `transform.translation` on a UI node will fail.
- You must query for `&mut UiTransform`.
- Note: `UiTransform` uses `Val` (pixels/percent) for positioning, not `f32`.

### B. Z-Ordering

The `z_order` is now an `f32` (was `stack_index`).

- **Check:** `uncore/src/colors.rs` or `untruck` logic. If you were doing math on z-indices, verify types.
- Bevy now enforces strict draw order: `Shadows -> Background -> Border -> Image -> Text`. Ensure your UI layering logic
  relies on hierarchy, not just z-index hacks.

## 6. Logic & Audio Changes

### A. Timer API (`uncore/src/utils/time.rs`)

- `.paused()` $\rightarrow$ `.is_paused()`
- `.finished()` $\rightarrow$ `.is_finished()`

### B. Audio Volume (`unlight/src/audio.rs`)

`Volume` no longer implements `Add` or `Sub` (because adding percentages is ambiguous).

- **Fix:** Use `.increase_by_percentage(10.0)` or explicit construction if you were calculating volume fades manually.

### C. Picking (`unplayer`, `unstd/src/picking`)

- Events renamed: `Pointer<Pressed>` $\rightarrow$ `Pointer<Press>`, `Pointer<Released>` $\rightarrow$
  `Pointer<Release>`.
- `RelativeCursorPosition` is now object-centered `(0,0)` (formerly top-left). This might offset your custom cursor
  logic in `unplayer` if you use relative coordinates.

### D. Asset Handles (`untmxmap`)

`Handle::Weak` is removed.

- **Fix:** Use `Handle::clone()` (strong handle) or `AssetId` (if you just need to track ID without holding the asset).

---

## Recommended Migration Order

1.  **Skeleton:** Update `Cargo.toml` and `.cargo/config.toml` (WASM flags).
2.  **Imports:** Run `cargo check` and fix all `use bevy::...` errors in `uncore` first.
3.  **Messages:** Mass-rename `Event` -> `Message` logic to stop the compiler screaming about trait bounds.
4.  **Sprites:** Fix `unmapload` sprite spawning (move Anchor to `.insert()`).
5.  **UI:** Fix `untruck` and `unmenu` structs to use `UiTransform`.
6.  **Logic:** Fix Timers and Audio math.

---

# --------

---

This is a significant architectural migration. Bevy 0.17 splits "Events" into two distinct concepts ("Messages" for
buffered queues, "Events" for observers), explodes the `bevy_render` crate, and fundamentally changes how UI positioning
works.

Here is the migration plan, ordered by dependency tiers.

### Phase 1: The Build Configuration (The Skeleton)

Before touching code, you must ensure the project dependency graph resolves.

1.  **Cargo.toml Updates (All Crates):**
    - Update `bevy` to `0.17` in the workspace and all sub-crates.
    - Update `bevy_persistent`, `bevy_picking`, `bevy_framepace` to their 0.17 compatible versions.
    - **Feature Flag Change:** In `Cargo.toml`, if you used `features = ["zstd"]`, rename it to
      `features = ["zstd_rust"]` (safer) or `features = ["zstd_c"]` (faster).
    - **WASM Support:** In `.cargo/config.toml` (or your build scripts), you **must** add
      `RUSTFLAGS='--cfg getrandom_backend="wasm_js"'` for the `wasm32-unknown-unknown` target. Bevy 0.17 updates `rand`
      and `getrandom`, which no longer defaults to a backend.

### Phase 2: Core Data & Imports (The Foundation)

This affects `uncore`, `unwalkie_types`, and `unsettings`. Bevy 0.17 moved many types out of `bevy_render` and
`bevy_core_pipeline`.

1.  **Fix Import Paths (The Great Reorganization):**

    - `Camera`, `ClearColor`, `Projection`: Move imports from `bevy::render::camera` to `bevy::camera`.
    - `Visibility`, `ComputedVisibility`: Move from `bevy::render::view` to `bevy::camera::visibility`.
    - `Mesh`: Move from `bevy::render::mesh` to `bevy::mesh`.
    - `Image`: Move from `bevy::render::texture` to `bevy::image`.
    - `Shader`: Move from `bevy::render::render_resource` to `bevy::shader`.
    - _Action:_ Scan `uncore/src/components/` and `uncore/src/resources/` to fix these immediately.

2.  **The Event vs. Message Split (Critical Architecture Change):**

    - Bevy 0.17 renames the old buffered events (what you use now) to **Messages**.
    - **Action:** In `uncore/src/events/*.rs`:
      - Derive `Message` instead of `Event` for your event structs.
    - **Action:** In all systems using these events (e.g., `ungame`, `unghost`):
      - Rename `EventWriter<T>` to `MessageWriter<T>`.
      - Rename `EventReader<T>` to `MessageReader<T>`.
      - Rename `app.add_event::<T>()` to `app.add_message::<T>()` (or the equivalent new registration method).

3.  **Timer API Update:**
    - In `uncore/src/utils/time.rs` and `uncore/src/components/animation.rs`:
      - Rename `.paused()` to `.is_paused()`.
      - Rename `.finished()` to `.is_finished()`.

### Phase 3: Shared Utilities & Rendering (The Bridge)

This affects `unstd`, `unlight`, and `unfog`. These crates interact directly with the render pipeline, which has changed
significantly.

1.  **Material2d Migration (`unstd/src/materials.rs`):**

    - `Material2d`, `Material2dPlugin`, and `MeshMaterial2d` have moved to a new crate/module `bevy::sprite_render` (or
      `bevy_sprite_render`).
    - Update imports in `unstd`.

2.  **Custom Shader Migration:**

    - In `assets/shaders/*.wgsl`:
      - Check for imports of `bevy_pbr::view_transformations`. These are deprecated. Replace with `bevy_render::view`
        functions as per the guide.

3.  **Sprite Anchors (`unstd/src/board/tiledata.rs`):**

    - `Anchor` is now a **Required Component** on `Sprite`. It is no longer a field on the `Sprite` struct.
    - **Action:** In your sprite spawning logic (e.g., `unmapload/src/tile_spawning.rs`), stop setting `sprite.anchor`.
      Instead, insert the `Anchor` component: `.insert(Anchor::Custom(...))`.
    - **Action:** Update `Anchor` variant usage (e.g., `Anchor::Center` becomes `Anchor::CENTER`).

4.  **Render Resource Initialization (`unlight`):**
    - If `unlight` initializes custom render resources (pipelines, bind groups) in `Plugin::finish`, this is likely
      broken.
    - **Action:** Move initialization logic into a system scheduled in `RenderStartup`.

### Phase 4: User Interface (The Visuals)

This affects `untruck`, `unmenu`, `unsummary`, `uncoremenu`. This is the highest risk area for visual regression.

1.  **UI Transform Split:**

    - Bevy 0.17 splits `Transform` for UI into `UiTransform`.
    - **Action:** In all your UI spawning code (e.g., `untruck/src/ui.rs`, `unmenu/src/mainmenu.rs`):
      - Replace `Node` bundles that use `Transform` with `UiTransform`.
      - Note that `UiTransform` uses `Val` for translation (responsive positioning), not `Vec3`.
    - _Warning:_ Systems that manipulated `Transform` on UI nodes to animate them will break. You must query for
      `UiTransform` instead.

2.  **Z-Ordering:**
    - `ExtractedUiNode` stack index is now `f32` `z_order`.
    - The guide notes a fixed draw order (Shadows -> BG -> Border -> Image -> Text).
    - **Action:** Review `uncore/src/colors.rs` or layout files. If you relied on hacky Z-manipulation for UI layering,
      verify it still works against the new fixed order.

### Phase 5: Gameplay Logic (The Systems)

This affects `unplayer`, `unghost`, `ungear`, `unwalkie`.

1.  **System::run Returns Result:**

    - If you manually run systems (e.g., in `uncore` utilities or tests), `System::run` now returns a `Result`. You must
      handle it (or `unwrap`).

2.  **Picking/Input (`unstd/src/picking` & `unplayer`):**

    - `Pointer<Pressed>` is renamed to `Pointer<Press>`.
    - `Pointer<Released>` is renamed to `Pointer<Release>`.
    - `PickingPlugin` settings moved to `PickingSettings` resource.
    - If you use `RelativeCursorPosition`, note the coordinate system change to object-centered `(0,0)` at center.

3.  **Handle::Weak Removal:**
    - If `untmxmap` or `unmapload` uses weak handles for asset tracking, replace them.
    - Use `Handle::clone()` (strong) or use `AssetId` if you just need the ID without keeping the asset alive.

### Phase 6: WASM & Audio

1.  **WASM Randomness:**

    - Ensure the `RUSTFLAGS` mentioned in Phase 1 are actually applied in your CI/CD pipelines (`.github/workflows`) and
      local `Justfile`.

2.  **Audio Volume:**
    - `Volume` no longer implements `Add`/`Sub`. Use `.increase_by_percentage()` in `unlight/src/audio.rs` if you were
      doing math on volumes.

### Recommended Execution Order

1.  **Dependencies & Imports:** Fix `Cargo.toml` and run `cargo check`. Fix all "type not found" errors by updating
    imports (Phase 2 & 3).
2.  **Message Renaming:** Bulk rename Event -> Message to satisfy the compiler.
3.  **Sprite Anchors:** Fix sprite spawning syntax.
4.  **UI Transforms:** Fix UI spawning syntax.
5.  **Logic Fixes:** Fix Timers, Picking events, and Volume math.
6.  **Run & Verify:** Fix runtime crashes related to `RenderStartup` or Shader bindings.
