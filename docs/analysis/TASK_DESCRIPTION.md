# Task Description: Project `unhaunter` Architectural Analysis

## 1. Overarching Goal & Context

The primary long-term objective is to refactor the `unhaunter` codebase to support a significant pivot from a 2D
isometric game to a full 3D game. A secondary goal is to evolve the architecture towards a more modular "ghost game
engine," where the core systems are generic and different game modes (like `unhaunter` itself) can be built on top as if
they were mods.

This requires a deep understanding of the current codebase's structure, data flows, and, most importantly, its areas of
tight coupling and modularity issues.

## 2. Immediate Task: Codebase Mapping

The immediate task is to perform a comprehensive architectural analysis of the entire codebase to create a "map" of its
structure. This involves systematically analyzing each crate in the `crates/` directory to understand its role and
responsibilities.

The output of this task is a series of markdown files, one for each crate analyzed, stored in the `analysis/` directory.

## 3. Analysis Methodology

For each crate being analyzed, the following process is being followed:

1. **Exploration:** The crate's `lib.rs` and `Cargo.toml` files are read. Its module structure is explored recursively
   by reading `mod.rs` files and listing directory contents to identify all public APIs (structs, enums, components,
   resources, events, systems, etc.).

2. **Documentation:** The findings for the crate are synthesized and saved into a dedicated markdown file (e.g.,
   `analysis/crate-name.md`).

3. **Analysis Criteria:** Each analysis document is structured to answer the following key architectural questions,
   based on the overarching goal:
   - **Core Responsibilities:** What is the crate's main purpose?
   - **Public API:** What key features does it expose to the rest of the workspace?
   - **Dependencies:** What other crates does it depend on? This is crucial for mapping the dependency graph.
   - **SOLID Principles:** How well does the crate adhere to principles like Single Responsibility (SRP)? This helps
     identify "God crates" or areas with low cohesion.
   - **2D/3D Coupling:** How tightly is the crate coupled to the current 2D isometric rendering pipeline? This is a
     critical question for the 3D pivot. We look for use of 2D vectors, sprite-specific logic, hardcoded isometric
     projection math, etc.
   - **Game Logic vs. Engine Logic:** Is the code specific to the _Unhaunter_ game, or is it a generic, reusable
     "engine" feature? This informs the "ghost game engine" goal.
   - **Other Notes:** Any other significant findings, such as performance optimizations, architectural patterns, or
     major issues like file duplication.

## 4. Key Findings So Far (Summary)

- **High 2D Coupling:** A major theme is the deep coupling to a 2D isometric rendering pipeline, especially in
  `uncore-board` (isometric projection math), `uncore-components` (components with 2D logic), and `uncore-assets` (Tiled
  map format loaders).
- **"God Crates":** `uncore-components` and `uncore-resources` have been identified as having very low cohesion, acting
  as "grab bags" for many disparate concerns. They are prime candidates for being decomposed into more focused crates.
- **File Duplication:** A critical issue was discovered where core spatial components (`BoardPosition`, `Position`,
  etc.) are duplicated between `uncore-board` and `uncore-components`. This is a major violation of DRY and must be
  addressed in a refactor.
- **Clear Engine-level Patterns:** Despite issues, some crates show strong engine-level design, such as `uncore-assets`
  (with its generic asset loaders and performance optimizations) and `uncore-events` (which provides excellent
  decoupling).

## 5. Current Status & Next Steps

We are systematically working through the `uncore-*` family of crates. The following crates have been analyzed and have
corresponding documents in the `analysis/` directory:

- `uncore-foundation`
- `uncore-types`
- `uncore-components`
- `uncore-board`
- `uncore-events`
- `uncore-resources`
- `uncore-assets`
- `uncore-systems`
- `unstd`
- `untmxmap`
- `unmapload`
- `unmaphub`
- `uncampaign`
- `uncoremenu`
- `undifficulty`
- `unfog`

The immediate next step is to continue the analysis of the remaining crates in the `crates/` directory.

**Remaining Crates to Analyze:**

- `ungame`
- `ungear`
- `ungearitems`
- `unghost`

The immediate next step is to continue the analysis of the remaining crates in the `crates/` directory.

**Remaining Crates to Analyze:**

- `unlight`
- `unmenu`
- `unmenusettings`
- `unnpc`

The immediate next step is to continue the analysis of the remaining crates in the `crates/` directory.

**Remaining Crates to Analyze:**

- `unplayer`
- `unprofile`
- `unsettings`
- `unsummary`
- `untruck`
- `unwalkie`
- `unwalkie_types`
- `unwalkiecore`
