# Workspace Refactor Summary: Core vs. Plugin Architecture

**Date:** January 14, 2026

## Overview

A major architectural refactor was completed to decouple game data from system logic across the entire workspace. The
project migrated from a mixed-responsibility crate structure to a strict **Core vs. Plugin** model.

## The "Why" (Objectives)

- **Compile Times:** By separating logic (fast-changing) from data (stable), we minimize recompilation of downstream
  crates.
- **Architectural Clarity:** Enforced a clear dependency graph where logic plugins are "leaf nodes" that nothing else
  depends on.
- **Decoupling:** Eliminated "God Crates" (`uncore-components`, `uncore-resources`) that acted as central bottlenecks
  for the entire project.
- **Encapsulation:** Ensured that plugins only expose a single `Plugin` struct, making the internal implementation
  details of features private.

## Work Done

### 1. Crate Reorganization

- **`un*-core`**: Low-level data crates containing only components, resources, and events. Stripped of all systems.
- **`un*-plugin`**: Logic layer containing systems and observers. These are now leaf nodes.
- **`unrender-std`**: Centralized rendering data (materials, bundles) to prevent circular dependencies between core
  crates and rendering logic.

### 2. Monolith Dissolution

- **`uncore-components` & `uncore-resources`**: Completely dissolved. Their contents were distributed to domain-specific
  `-core` crates (e.g., `ungear-core`, `untypes-core`, `uninteraction-core`).

### 3. Major Splits

- High-complexity crates were split to resolve deep dependency chains:
  - `unrender` -> `unrender-std` / `unrender-plugin`
  - `ungear` -> `ungear-core` / `ungear-plugin`
  - `unghost`, `unplayer`, `unmetrics`, `unsettings`, etc., followed the same split pattern.

### 4. Visibility Enforcement

- Internal systems and modules within plugins were changed to `pub(crate)` or private to enforce strict API boundaries.

## Result

The project now consists of a robust foundation of data crates (`-core`) and a modular set of logic plugins (`-plugin`)
that are easily maintainable and follow a predictable dependency flow.
