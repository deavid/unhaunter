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

## File Reading Guidelines

When using the `read_file` tool, always read in chunks of **1000 lines or more**. Never read less than 1000 lines in a
single call. This minimizes repetitive tool calls and gets you full context faster. If a file is smaller than 1000
lines, read the entire file. For files larger than 1000 lines, read multiple 1000+ line chunks in parallel where
possible.

## Work Planning & Tool Execution

**BEFORE STARTING ANY SIGNIFICANT WORK PHASE:**

1. **Plan file I/O upfront**: List ALL files you need to read and write for the work phase. Do not discover files as you
   go.
2. **Batch reads in parallel**: Execute all file reads at once using parallel tool calls in a single `<function_calls>`
   block. Do not read files sequentially.
3. **Think about dependencies**: Before executing ANY tool call, ask yourself:
   - Do I really need this tool right now?
   - What other tool calls would I make in the next step?
   - Can I batch them together to minimize interruptions?
4. **Always parallelize first**: Combine independent reads/writes/searches into one batch before triggering execution.

**AFTER COMPLETING WORK:**

- **ALWAYS check for errors and warnings**: Use `get_errors` to verify the workspace compiles cleanly. Do NOT claim work
  is complete if the workspace has errors, lint failures, or warnings. This is critical.
- **Do not ask the user for progress updates**: Continue working autonomously until the requested task is fully
  complete. Only report completion when the workspace is clean.

## Project Conventions

- **Naming**: Crates use `un` prefix (e.g., `unplayer-plugin`).
- **Module Files**: `lib.rs` and `mod.rs` must **never** contain actual code. Only `mod` statements are allowed.
- **No Re-exports**: `pub use` is forbidden globally. Every symbol must have exactly one canonical path.
- **Imports**: Prefer explicit imports over wildcards, except for `bevy::prelude::*`.
- **Error Handling**: Use `anyhow` for top-level tools; use `thiserror` for library-level error definitions.
- **Coordinate Systems**: We use a custom isometric projection. Logic often happens in "board" coordinates (see
  [crates/unspatial-core](crates/unspatial-core)).

## Domain-Driven Architecture (DDD / Hexagonal)

This codebase is organized following DDD and Hexagonal Architecture principles. Every crate belongs to a **tier** and a
**group**. The canonical placement of every crate is the comment block in `[workspace.members]` inside `Cargo.toml`.
**Read that block before touching any crate boundary.**

### The Tier Model

```
T0  Domain Kernel      — pure types, math, markers. No internal game deps.
T1  Domain Core        — the game's world and rules. Headless-safe.
      1a  The Stage    — board, physics fields (thermal, sound, fog, light, nav)
      1b  The Actors   — ghost, player, NPC, locomotion
      1c  The Mechanics— gear, interaction, inventory, vitals, difficulty, truck, walkie
T2  Application Layer  — mission lifecycle, game-mode orchestration, input, networking
T3  Infrastructure     — external I/O and persistence (Tiled ACL, settings, profiles, hub)
      3a  Map Pipeline — Tiled ACL (untiled, untmxmap, unmapload, unbehavior)
      3b  Persistence  — settings, profile
      3c  Net Clients  — unhub
T4  Presentation       — rendering, audio, UI. Client-side only.
      4a  Rendering    — unrender, unpicking
      4b  Audio        — unaudiobg, unaudiospatial
      4c  UI           — unui, menus, screens, HUD
```

### Binding Rules — Always Enforced

These rules apply to **every task**, not only architectural ones. Code must never be left in a worse state than it was
found. Existing drift is tolerated; new drift is not.

1. **Dependency direction is downward only.** A crate may depend on crates in its own tier or lower tiers. It must
   **never** depend on a crate in a higher tier. A T1 crate importing a T4 type is a hard violation.

2. **Cross-tier dependency violations are refused.** If a requested change would introduce an upward dependency (e.g., a
   T1 domain crate importing a T4 rendering type), **refuse, explain the violation, and propose a compliant
   alternative** before doing anything else. This refusal can only be overridden by an explicit direct instruction from
   the user.

3. **A `-core` / `-plugin` pair is one bounded context.** They must always live in the same tier. If you see a pair
   split across tiers, flag it. Never create a split intentionally.

4. **New crate = mandatory tier declaration.** When asked to create a new crate, ask (or state) which tier and group it
   belongs to, and why, before creating anything. Place it in the correct position in `Cargo.toml` with a matching
   comment explaining what it does.

5. **Cargo.toml group structure is canonical.** The tier comments in `[workspace.members]` and
   `[workspace.dependencies]` are the source of truth for crate placement. Do not reorder or remove those comments. When
   adding a crate to either section, place it inside the correct tier group, not at the end of the list.

6. **Infrastructure is an adapter, not a foundation.** T3 crates exist to translate external formats (Tiled, disk, HTTP)
   into domain types. Domain crates (T1) must never import T3 crates. The translation is one-way: external data enters
   through T3 and is converted; domain types never flow back out as external format structs.

7. **Presentation never leaks inward.** T4 crates (rendering, audio, UI) must not be imported by T0–T3 crates for any
   reason. If a domain concept needs to _trigger_ a visual or audio effect, it does so through an **event** (a T0/T1
   data type), not by calling into T4 directly.

### Review Behavior

When reviewing code or architecture (not implementing), always check and report:

- Any crate dependency that crosses tiers in the wrong direction.
- Any `-core` / `-plugin` pair that is in different tiers.
- Any T3 type that has leaked into a T1 or T0 crate.
- Any T4 import in a T0–T3 crate.

Flag these as violations even if you are not asked to fix them.
