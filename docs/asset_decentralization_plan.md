# Asset Decentralization Plan

## 1. Problem Statement

Currently, `unroot-plugin` acts as a monolithic "sysadmin" for assets. It loads every single asset in the game into a giant `GameAssets` resource via `crates/unroot-plugin/src/assets_loading.rs`.

**Issues:**

* **Central Point of Failure:** Missing assets in one module break the entire compilation/runtime.
* **Coupling:** The bootstrap plugin knows implementation details (paths, filenames) of all other plugins.
* **Monolithic Resource:** Systems request `Res<GameAssets>`, taking a dependency on *everything*.
* **Hardcoded Constants:** Anchor points and Atlas layouts are hardcoded in the loading logic.

## 2. The Solution

We will use `bevy_asset_loader` to allow each plugin to define its own asset requirements. Plugins will define `AssetCollection` structs, and the dependency injection system will ensure they are loaded before the game starts.

### key Principles

1. **Decentralized Loading:** Each plugin manages its own assets.
2. **State-Based Loading:** All assets are loaded during `AppState::Loading`.
3. **Single Transition Authority:** `unroot-plugin` manages the transition from `Loading` to `MainMenu`. Other plugins only register their collections.
4. **Embedded Configuration:** Anchors and Atlas layouts move to the asset definition.

---

## 3. Asset Groups & Destination Plugins

We will dissolve `GameAssets` and distribute its contents as follows:

### A. `ghost-plugin` (New: `GhostAssets`)

* **Images:**
  * `img/ghost.png`
  * `img/breach.png`
  * `img/focus_ring_vignette.png` (maybe generic, but used by ghost/breach)
  * `img/miasma-base-01.png`
* **Logic:** `grid1x1x4` anchor (used for breach) should be defined here or in `unmapload`.

### B. `unplayer-plugin` (New: `PlayerAssets`)

* **Images:**
  * `img/characters-model1-demo.png`
* **Atlas:**
  * Configuration for the player sprite sheet.
* **Logic:** `character` anchor.

### C. `ungear-plugin` (New: `GearAssets`)

* **Images:**
  * `img/gear_spritesheetA_48x48.png`
  * `img/items.aseprite` (if used directly, otherwise just the exported pngs)
* **Atlas:**
  * Gear atlas configuration.

### D. `unui-plugin` (New Plugin & `UiAssets`)

**Action:** Create `crates/unui-plugin` to separate high-level UI assets from `unroot`.

* **Fonts:** (All fonts currently in `GameAssets`).
* **Images:**
  * `img/title.png`
  * `img/menu-background.jpg`
  * `img/scroll_*.png`
  * `img/badges.png`
* **Atlas:** Badge atlas.

### E. `unmanual-plugin` (New: `ManualAssets`)

* **Images:** Note: There are ~50 manual images.
* **Strategy:** Map these using `bevy_asset_loader`'s dynamic mapping checks if possible, or verbose struct fields if necessary.
  * Currently, they are individual fields. We will keep them as individual fields in `ManualAssets` for type safety in the manual UI code.

### F. `untmxmap-plugin` / `unmapload-plugin` (New: `MapAssets`)

* **Images:**
  * `img/vignette.png` (Lighting/Map related)
* **Logic:** `grid1x1` and `grid1x1x4` Anchors (often used for map entities/props).

---

## 4. Execution Plan

### Phase 1: Preparation (Workspace)

1. Add `bevy_asset_loader` version `0.21` (or matching Bevy 0.14) to `workspace.dependencies` in `Cargo.toml`.
2. Enable the `2d` feature for `bevy_asset_loader`.

### Phase 2: Create `unui-plugin`

1. Initialize `crates/unui-plugin`.
2. Move Font and Global UI assets loading here.
3. Define `UiAssets` derivation.
4. Remove these fields from `GameAssets`.

### Phase 3: Plugin Migration (Iterative)

For each plugin (`unghost`, `unplayer`, `ungear`, `unmanual`):

1. **Define Struct:** Create `assets.rs` in the plugin, defining the `AssetCollection`.
2. **Configure Atlas:** Use `#[asset(texture_atlas_layout(...))]` for atlases.
3. **Configure Anchors:**
    * If the anchor is constant, define it as a `const` in the `assets.rs` file.
    * If it depends on image size (dynamic), use a `FromWorld` implementation or a separate system to calculate it, but for pixel art, constants are usually safe and preferred.
    * *Decision:* Migrate `Anchors::calc` logic into simple `Vec2::new()` constants for standard sprites types in their respective plugins.
4. **Register:** In `plugin.rs`, add `app.add_loading_state(... .load_collection::<XAssets>())`.
5. **Refactor usage:** Search workspace for usages of the old `GameAssets` fields and update them to use `Res<XAssets>`.

### Phase 4: Clean up `unroot`

1. Once `GameAssets` is empty, delete the struct.
2. Remove `assets_loading.rs` from `unroot-plugin`.
3. Ensure `unroot-plugin` (or `unmainmenu-plugin`) retains the logic to switch state:

    ```rust
    app.add_loading_state(
        LoadingState::new(AppState::Loading)
            .continue_to_state(AppState::MainMenu)
    );
    ```

## 5. Technical Details

### Handling Anchors

Instead of the `Anchors` resource, we will prefer constants or methods on the Asset struct if possible, or Component defaults.
Example:

```rust
// crates/unplayer-plugin/src/assets.rs

pub const PLAYER_ANCHOR: Vec2 = Vec2::new(0.5, 0.1); // Normalized (Bevy default is center 0.5, 0.5)

// OR, if keeping the pixel math:
pub fn player_anchor() -> Vec2 {
    // Logic from Anchors::calc(13, 43, 26, 48)
    Vec2::new(...)
}
```

*Note: Bevy Sprites use normalized anchors (0.0 to 1.0) relative to the sprite size, or custom Vec2. The current `Anchors::calc` produces values centered around 0.0 (center) to +/- 0.5. We will preserve the resulting `Vec2` values.*

### Texture Atlases

Old way:

```rust
character1_atlas: texture_atlases.add(TextureAtlasLayout::from_grid(UVec2::new(64, 64), 16, 4, ...))
```

New way:

```rust
#[derive(AssetCollection, Resource)]
pub struct PlayerAssets {
    #[asset(texture_atlas_layout(tile_size_x = 64., tile_size_y = 64., columns = 16, rows = 4))]
    pub character_layout: Handle<TextureAtlasLayout>,
    #[asset(path = "img/characters-model1-demo.png")]
    pub character_image: Handle<Image>,
}
```
