# Unhaunter Binary Investigation Report

**Date:** 2026-01-16 **Author:** GitHub Copilot

## 1. Executive Summary

The `unhaunter` binary crate (`unhaunter/src`) is already quite lean, serving primarily as an entry point that registers
a large collection of plugins. However, it is not yet a pure "plugin aggregator". It still contains:

- Local systems for performance reporting and window icon management.
- Explicit resource initialization for some core game states.
- Explicit material plugin registration.
- Platform-specific logic (WASM vs Native) mixed in `app.rs` and `wasm.rs`.
- Helper modules (`assetidx_updater`, `utils`, `report_timer`).

To reach the ideal state of "just load plugins", these responsibilities need to be migrated into existing or new
plugins.

## 2. Analysis of `unhaunter/src/app.rs`

The `app_run` function in `app.rs` is the main bootstrapping logic. It largely follows the pattern of
`app.add_plugins(...)`, but includes several deviations:

### 2.1. Explicit Resource Initialization

The following resources are initialized directly in the app builder, rather than in their respective core plugins:

```rust
app.init_resource::<CurrentDifficulty>()
    .init_resource::<ObjectInteractionConfig>()
    .init_resource::<HauntState>();
```

- `CurrentDifficulty`: Should likely be initialized in `UnhaunterDifficultyPlugin` (or `undifficulty-core` if it had a
  plugin).
- `ObjectInteractionConfig`, `HauntState`: Should likely be initialized in `UnhaunterGhostPlugin` (or `unghost-core`).

### 2.2. Material Plugins

The application explicitly registers Render plugins for custom materials:

```rust
app.add_plugins(Material2dPlugin::<CustomMaterial1>::default())
    .add_plugins(UiMaterialPlugin::<UIPanelMaterial>::default());
```

These are dependencies of the rendering system. They should be encapsulated within `UnhaunterRenderPlugin` (from
`unrender-plugin`) or `UnhaunterBoardPlugin` to ensure that importing the plugin brings all necessary render
dependencies.

### 2.3. Picking Plugin

```rust
app.add_plugins(CustomSpritePickingPlugin);
```

This is explicitly added. It comes from `unpicking-plugin`, so it is already a plugin, but it is added separately from
the main group of plugins.

### 2.4. Local Systems

`app.rs` registers local systems:

```rust
app.add_systems(Update, crate::report_timer::report_performance);
app.add_systems(Startup, set_window_icon); // Native only
```

- `report_performance`: Defined in `src/report_timer.rs`.
- `set_window_icon`: Defined locally in `app.rs`.

### 2.5. Window Configuration

The `DefaultPlugins` configuration includes window setup logic:

```rust
DefaultPlugins.set(WindowPlugin {
    primary_window: Some(Window {
        title: format!("Unhaunter {}", plt::VERSION),
        resolution: default_resolution(),
        present_mode: bevy::window::PresentMode::AutoVsync,
        ..default()
    }),
    ..default()
})
```

This logic tightly couples the binary to the windowing configuration. A `UnhaunterWindowPlugin` could encapsulate this
configuration.

## 3. Analysis of Local Modules

The `unhaunter` crate contains the following local modules:

### 3.1. `src/report_timer.rs`

Contains `report_performance` system.

- **Function**: Logs performance metrics of systems and checks for `AppState` / `GameState` consistency.
- **Dependency**: Depends on `untypes-core`.
- **Recommendation**: Move to a `UnhaunterDebugPlugin` or `UnhaunterDiagnosticsPlugin`. The state consistency check is
  valuable but might belong in a state management plugin.

### 3.2. `src/assetidx_updater.rs`

- **Function**: Updates asset index files (likely for WASM compat or asset loading optimization).
- **Usage**: Called in `main.rs` before the app starts.
- **Recommendation**: This looks like a build-tool or pre-launch step. If it must run at runtime, it could be a startup
  system in `UnhaunterAssetsPlugin`.

### 3.3. `src/utils.rs`

- **Function**: `find_assets_directory()` for locating assets in Dev vs Release modes.
- **Usage**: Used by `set_window_icon` in `app.rs`.
- **Recommendation**: This is a platform helper. It could be part of `unfoundation-core` or a platform-specific plugin.

## 4. Recommendations

To achieve the goal of a "thin shell" binary:

1. **Create `UnhaunterCorePlugin` (or similar aggregation plugin)**:

   - This plugin could group the essential "core" plugins (Difficulty, Ghost, Render, etc.) so `app.rs` doesn't need to
     list 30+ plugins.
   - Alternatively, keep the list explicit but clean up the initialization logic.

2. **Move Resource Initialization**:

   - Move `init_resource::<CurrentDifficulty>()` to `undifficulty-core` (expose a plugin) or `uncampaign-plugin`.
   - Move `init_resource::<HauntState>()` and `ObjectInteractionConfig` to `unghost-plugin`.

3. **Encapsulate Rendering Setup**:

   - Move `Material2dPlugin` and `UiMaterialPlugin` registration into `UnhaunterRenderPlugin`.

4. **Extract Local Systems**:

   - Move `report_performance` to `unmetrics-plugin` (seems like a good fit) or a new `undiagnostics-plugin`.
   - Move `set_window_icon` and window configuration to a `UnhaunterWindowPlugin` (maybe in `unmainmenu-plugin` or
     `unrender-plugin`? or a new `unwindow-plugin`).

5. **Refactor `assetidx_updater`**:
   - If this is purely for development/build, consider moving it to a `tool` crate or a `build.rs` script. If it's
     needed at runtime for dev builds, wrap it in a `DevToolsPlugin`.

## 5. Conclusion

The `unhaunter` binary is close to the goal but still "leaks" some implementation details that belong in the crates.
Refactoring `app.rs` to delegate resource initialization and material setup to the respective plugins will significantly
clean up the entry point. The remaining local logic (diagnostics, window icon) can be easily modularized.
