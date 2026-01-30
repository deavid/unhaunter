# Task: Domain-Driven Architectural Analysis

## 1. Objective

The goal of this task is to perform a deep-dive architectural audit of the `unhaunter` workspace. We aim to evaluate the
codebase through the lens of **Domain-Driven Design (DDD)** and **Clean Architecture** to ensure the project remains
maintainable, testable, and ready for future technological pivots (such as a 3D rendering engine or a moddable "Ghost
Engine").

## 2. Why This Analysis?

While the project follows a modular structure, software systems naturally accumulate "semantic leakage" over time. We
need to identify:

- **Domain Boundaries:** Are our crates organized around game concepts (e.g., "Ghost behavior", "Player movement") or
  technical implementations?
- **Data vs. Logic:** Does the current "Core vs. Plugin" split successfully isolate raw data from systems, or do core
  crates still carry hidden logic?
- **Visual/Infrastructure Coupling:** How much do our "Game Rules" know about specific rendering details (like sprite
  indices) or infrastructure details (like keyboard controls)?
- **Engine Readiness:** Could the core systems be extracted into a generic "Ghost Engine" where the game itself is just
  a "mod"?

## 3. Methodology

We will systematically analyze every crate in the workspace. For each crate, we will document:

1. **Bounded Context:** What "part of the world" does this crate own? (e.g., The Player domain, the Board domain).
2. **Responsibility (SRP):** What is the single reason this crate exists?
3. **Encapsulation Audit:**
   - For `-core` crates: Are they pure data? Does any logic leak in?
   - For `-plugin` crates: Are systems private? Is the `Plugin` struct the only public export?
4. **Semantic Leakage Check:**
   - **Visuals:** Does this crate contain asset handles, sprite indices, or UI layouts?
   - **Infrastructure:** Does it know about keyboard keys, file paths, or network details?
5. **Dependency Flow:** Does it follow a strict Directed Acyclic Graph (DAG)? Does it depend on things "above" its
   layer?

## 4. Expected Outcomes

- **The Artifacts:** A series of markdown documents (one per crate) providing a "X-Ray" view of the current
  architecture.
- **The Audit Report:** A final summary identifying "Architectural Debt" and providing a prioritized list of refactors.
- **Engine Blueprint:** A clear plan for what needs to change to achieve a generic Entity-Orchestration map loader.
