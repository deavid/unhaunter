# Unhaunter AI Coding Instructions

This project is a 2D isometric paranormal investigation game built with Bevy.

Note that Bevy releases quite often and this game is likely using a Bevy version much newer than the one you might be
familiar with.

See the following migration summaries for up-to-date Bevy patterns and project-specific choices:

- docs/bevy_0.16_migration_filtered.md
- docs/bevy_0.17_migration_filtered.md
- docs/bevy_0.18_migration_filtered.md

Do not guess. If something feels off ask questions to the user.

Unless the user asks specifically and directly, DO NOT CODE, DO NOT FIX. If the user asks to review, DO NOT CODE,
generate a review report instead.

If the ask is to do X, do not overstep the ask. Don't do more than asked.

You have been warned:

- If you modify code when you were not asked to fix or code anything, your code will be reverted no questions asked.
- If you do what the user asked but you also modified something else without being asked, everything you did will be
  reverted no questions asked, including what was correctly done.

We have zero BS tolerance.

## Architecture & Module Structure

The codebase follows a strict modular structure to minimize compile times and separate concerns. See
[PROJECT_FILE_DESCRIPTIONS.md](PROJECT_FILE_DESCRIPTIONS.md) for a detailed map.

- **`un*-core`**: Low-level data types, components, and resources. **Must contain zero game logic, systems, or
  observers. It must NOT contain any Bevy `Plugin`.**
- **`un*-plugin`**: High-level game flow and mechanics. Logic is contained in Bevy `Plugin` implementations. **The only
  thing that can be exported out of these crates is the `Plugin` itself.**
- **`unhaunter`**: The main entry point that assembles all plugins in [unhaunter/src/app.rs](unhaunter/src/app.rs).

### State Management

- `AppState`: Controls high-level contexts (e.g., `MainMenu`, `Loading`, `InGame`).
- `GameState`: Controls in-mission phases (e.g., `None`, `Normal`, `Hunting`). Systems should be gated by these states
  using `.run_if(in_state(AppState::InGame))`.

## Key Patterns

- **Plugin Decomposition**: Bevy plugins must be declared in a `plugin.rs` file. This file must contain the `Plugin`
  implementation and nothing else. They typically follow this structure:
  - `plugin.rs`: `impl Plugin` that calls `systems::app_setup(app)`.
  - `systems/mod.rs`: `app_setup` function that groups systems and adds them to the `App` with `run_if` and `.chain()`.
- **Event-Driven Communication**: Use the custom event system in [crates/unevents-core](crates/unevents-core).
  - Register events with `app.add_message::<T>()`.
  - Events often use `#[derive(Message)]`.
- **Asset Loading**: We use `bevy_asset_loader`. Asset collections are defined in `core` crates (e.g.,
  [crates/unplayer-core/src/assets.rs](crates/unplayer-core/src/assets.rs)) and loaded in plugins.
- **Tiled Integration**: Maps are `.tmx`. Custom components are often spawned based on Tiled object properties (see
  [crates/untmxmap-plugin](crates/untmxmap-plugin)).

## Developer Workflow & Commands

- **Searching:** `git grep` (usage of `grep -r` is forbidden).
- **Lints**: `cargo clippy`, `cargo machete` (usage of `-p $cratename` is forbidden, use `cargo clippy` globally
  instead).
- **Tests**: Do not run tests on this codebase. They are not a reliable way to test anything. Use `cargo clippy`
  instead.
- **Running**: AI Agents must not run the game. Ask the user to test the game instead.

Avoid running commands where possible. Usage of commands to read, write or edit files is forbidden; for example `cat`,
`sed`, `awk` are not allowed.

## Project Conventions

- **Naming**: Crates use `un` prefix (e.g., `unplayer-plugin`).
- **Module Files**: `lib.rs` and `mod.rs` must **never** contain actual code. Only `mod` statements are allowed.
- **No Re-exports**: `pub use` is forbidden globally. Every symbol must have exactly one canonical path.
- **Imports**: Prefer explicit imports over wildcards, except for `bevy::prelude::*`.
- **Error Handling**: Use `anyhow` for top-level tools; use `thiserror` for library-level error definitions.
- **Coordinate Systems**: We use a custom isometric projection. Logic often happens in "board" coordinates (see
  [crates/unspatial-core](crates/unspatial-core)).
