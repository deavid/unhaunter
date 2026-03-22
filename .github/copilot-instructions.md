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

**Note on Markdown Linting:** Markdown lint/warning errors in the workspace are not expected to be fixed and should be
ignored. Focus exclusively on Rust workspace errors and warnings.

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

## Signal Direction & Domain Ownership

These principles govern how systems communicate and where code belongs. They take **priority over tier compliance** when
the two conflict. A crate with perfect tier compliance can still be deeply wrong if it violates these rules.

### 1. Tell, Don't Ask (Signal Direction)

The cause must emit; effects listen. A system must never reach into a foreign domain to derive meaning that the foreign
domain should be pushing.

**Violation pattern:** System A queries component B from domain B, applies thresholds or curves to compute a signal, and
uses that signal to update domain A. **Correct pattern:** Domain B computes the signal internally and writes it to a
component or event. Domain A reads that pre-computed signal.

Examples of this violation in the codebase:

- An audio plugin sampling `PlayerVitals.health` and computing heartbeat intensity. Vitals owns the knowledge of what
  "critically low health" means. It should emit that. Audio should receive it.
- A vitals plugin querying `GhostSprite.hunt_target` and `calm_time_secs` to compute damage. Ghost owns its danger
  level. Vitals should receive a signal from the ghost domain, not derive it.

**The test:** For every system, ask: _who owns the knowledge being consumed, and are they the one pushing it?_ If the
answer is "someone else owns it, and we're pulling it," that is a TDA violation.

### 2. Information Hiding

A system must not encode decisions that belong to another domain. Rendering curves, AI thresholds, persistence logic —
if the _decision_ belongs elsewhere, it is a violation even if the imports are all downward.

**Violation pattern:** A vitals system contains RGBA color math and exponent curves to drive a UI vignette. Anyone
reading the system must hold both the vitals model and the rendering model in their head simultaneously. **Correct
pattern:** The vignette component reads `PlayerVitals` and decides its own appearance. Vitals computes vitals; vignette
computes vignette.

### 3. Domain Completeness

**Check this first, before analyzing individual systems.**

A domain that does not own its types cannot enforce its own boundaries. For every plugin under review, ask:

- Is there a corresponding `-core` crate? If not, where do the primary data types actually live?
- Are the domain's primary components scattered across foreign crates (e.g., `unplayer-core`, `unreplicon-core`)?
- Does the plugin register its own replication (`app.replicate::<T>()`)? Or does a foreign crate do it?
- Does the plugin own its own events, or are its events defined in unrelated crates?

If types that conceptually belong to this domain live elsewhere, flag that as the primary structural problem. Do not
just analyze systems as if the missing types are someone else's problem.

**Evidence from system behavior:** If a plugin writes to a type that lives in a foreign crate, that is evidence the type
belongs here — not proof of a violation in the writes.

### 4. Systems Belong to One Domain

For every system in a plugin, ask:

- Is it reading AND writing this domain's own types?
- If it writes nothing in this domain, it does not belong here.

A system that only reads foreign domain types and writes to foreign domain types is misplaced regardless of tier.
Example: a death-handler that reads a network player ID, writes to a persistence profile, and populates a summary struct
— while never touching the vitals component — does not belong in the vitals plugin.

### 5. Physics Field Rule

Field crates (`unfog`, `unsoundfield`, `unlight`, `unthermal`) compute diffusion/propagation math exclusively, using
ontology components (`ThermalEmitter`, `Opaque`, etc.) as inputs. They must never import T4. If a field crate calls into
audio or rendering, it is a violation regardless of how the computation is framed.

### 6. Deletable Silo Test

A healthy domain: deleting its crates leaves holes in the simulation but not cascading failures in unrelated domains. If
unrelated domains break when you remove X, responsibilities have leaked out of X.

### Domain Audit Protocol

When asked to **audit domain X** (e.g., `unvitals`, `unghost`, `ungear`):

1. **Gather**: Read `Cargo.toml` of all `unX-*` crates to map deps. Read all source files.
2. **Check in priority order** — tier compliance is last, not first:
   - **Domain completeness:** Is there a `-core`? Do the primary types live here or in foreign crates?
   - **TDA:** For each foreign `un*` read, is that domain pushing this signal, or is this system pulling it?
   - **Information hiding:** Does any system encode decisions (thresholds, curves, colors) that belong elsewhere?
   - **System membership:** Does each system write _this_ domain's own types? If not, it is misplaced.
   - **Tier violations:** Does the crate import anything higher-tier? (flag, but weight less when recommending fixes)
3. **Output**: A written report. Flag violations. Do not modify code unless separately instructed.

Tier rules catch _import_ violations. The four checks above catch _responsibility_ violations. Both are reported, but
responsibility violations take priority in diagnosis and fix recommendations.

### Anti-Pattern: The Horizontal-Cut (Shared Bag) Trap

When a crate holds types from multiple feature domains because they were convenient to share, the instinctive "fix" is
to extract them into a new shared crate at a lower tier (e.g., "extract UI markers to `unhud-core` at T0/T1", or
"extract rendering types to `unrender-types` at T0"). **This is wrong.** It creates a smaller bag with a different
label, solving the tier check without solving the structural problem.

**The correct question is always: what feature-silo does this component belong to?** Every component has a natural owner
— the domain that writes it and is responsible for its semantics. Move it there.

A new shared crate is only justified when types meet ALL of the following:

- They have no feature affiliation (they describe no single concern)
- They would exist even if every current consumer were deleted
- They are genuinely primitive (math, markers, asset handle bags, GPU material types)

If a component fails any of those tests, it belongs in a feature domain, not a shared crate.

**Known stale audit recommendations:** The `02_t4a.md` audit recommends "extract to `unrender-types`" for `SpriteLayer`,
`GameSprite`, etc. That recommendation is superseded by the design analysis in `docs/ddd-analysis/03_concept_design.md`
(Appendix: The Horizontal-Cut Problem). Those components are simulation state that belongs in their owning domain crates
— not in a new shared rendering-types bag.

---

## Crate Tier Reference

The workspace uses a tier model to enforce dependency direction. The canonical placement of every crate is in
`[workspace.members]` inside `Cargo.toml`. **Read that block before touching any crate boundary.**

Tier compliance (import direction) is secondary to signal-direction and domain-completeness checks. Report tier
violations but do not let them dominate fix recommendations.

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

### Tier Rules

These rules apply to every task. Existing drift is tolerated; new drift is not.

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

When reviewing code, always flag: any upward tier dependency, any `-core`/`-plugin` pair in different tiers, any T3 type
in T0–T2, any T4 import in T0–T3. These are violations even if not asked to fix them.
