# The Simulation as Data and Hidden Domains

**Date:** 2026-03-25

**Follows:** `02_simulation_boundaries_and_code_health.md`

**Status:** Conceptual exploration and architectural compass. No prescriptive implementation required at this time.

---

## 1. How We Got Here: The Dependency Trashcan

The concept of "Multi-Mission Support" was originally researched (in Docs 01 and 02) primarily as a networking scaling
optimization—amortizing the Bevy scheduler CPU cost across multiple investigations on a single cheap headless VPS. At
the time, it was shelved as "too complex for the current scale."

However, the topic resurfaced unexpectedly while auditing the dependency graph (DAG) and analyzing why "trashcan" crates
(like `uncommon-app-core` and `uncommon-app-core`) were perpetually accumulating state enums and marker components.

We discovered that deep architectural pain points—like domain bleeding between `unmission` (the meta-game objectives),
`unreplicon` (the networking session), and the core Engine physics—all stem from a single root assumption: **The
codebase currently treats the physical "Simulation" as a global Application State rather than a piece of Instantiated
Data.**

When `GameState` or `SimulationState` are global enums, every domain—from the UI loading screen to the Ghost's AI—must
couple to a centralized orchestrator. If we instead flip the paradigm to treating a "World" or "Dimension" as a distinct
chunk of data (e.g., a conceptual `SimulationRoot` or `WorldIndex`), the boundary between the "Meta-game" and the
"Engine" instantly solidifies.

The Meta-game simply loads a World. The Engine simply ticks the Worlds it possesses. The architectural friction
dissolves. We are exploring multi-world concepts not just to put multiple teams on one server, but to find the perfect
**Clean Architecture Boundaries** for the engine today.

---

## 2. The Silent Correctness Threat: Cross-Contamination

Document 02 touched on the performance caveat of O(n×m) entity queries. Further investigation revealed that flat,
unscoped ECS queries are not merely performance overheads—they are latent **correctness failures** waiting for multiple
dimensions to exist.

Consider these existing query patterns:

1. **Global Aggregation:** Systems that calculate damage by checking `q_ghost` and applying it to
   `q_local_player.single_mut()`. In a multi-world state, this logic would inadvertently allow a ghost in Mission A to
   murder a player standing in the hub of Mission B.
2. **Table Scans for Spatial Checks:** Interaction systems that iterate every `Interactive` object in the ECS just to
   find out if the player is standing next to a door.
3. **The `.single()` Assumption:** Assuming there is exactly one map, one thermal grid, or one local player globally
   available.

Currently, these queries work because the external orchestration strictly enforces that only one universe exists in
memory at a time. But this hardcodes the assumption of a single-reality into every ghost behavior, every item script,
and every player input system.

---

## 3. Conceptual Avenues (Not Recipes)

While we do not have—and do not need—the exact technical implementation yet, the goal is to align future refactors
toward patterns that naturally survive in a multi-universe architecture.

When refactoring heavy systems or moving components, we should encourage thought experiments around **Data Routing and
Spatial Indexing**:

- **Indexes over Iteration:** If a player needs to know what is nearby, relying on global query iterations
  (`O(Total_Entities²)`) will fail at scale. Solutions might eventually involve a Spatial Hash, a localized Map Index,
  or an observer that maintains `Vec<Entity>` rosters for specific spaces/worlds.
- **Decoupling the Grids:** Dense data like `ThermalGrid`, `LightGrid`, or `RoomDB` are currently singletons (`Res<T>`).
  In the future, the Engine may need to manage a collection of these (e.g., contiguous arrays or Boxed mappings).
  Refactoring efforts should isolate how gameplay logic accesses these grids so they can eventually be swapped or routed
  based on an Entity's contextual ID without tearing up the core physics.
- **Isolating System State:** Systems leaking contextual data into Bevy `Local<T>` accumulators (like roar cooldowns or
  ghost wander logic) blur the line between "the system's rules" and "the actor's state." Moving this data into
  explicitly owned Components ensures that state belongs to the Entity inside the Dimension, rather than globally shared
  across the CPU thread.

---

## 4. Future Proofing without YAGNI Violations

We are not building an MMO backend or dynamic "Nether dimensions" today. Over-engineering a bespoke dimension-routing
manager now would violate "You Aren't Gonna Need It."

However, we can adopt code hygiene rules _today_ that cost nothing but keep the door unlocked for those futures:

1. **Avoid `.single_mut()` for Engine Actors:** On authoritative simulation logic, prefer iterating a query (even if it
   currently yields one result) rather than enforcing a `.single()` panic. Leave `.single()` for purely client-side/UX
   concepts.
2. **Query Spaces, Not the World:** When writing logic that requires proximity or environment awareness, consider if the
   system is scanning the entire database instead of querying a localized context.
3. **Guard the DAG:** Keep treating upward tier violations as the "canary in the coal mine." If a pure Engine component
   suddenly needs to know about `unreplicon` or `unmission` state to function, it means we are leaking meta-game
   orchestration into the physical simulation again.

The dream of a seamless, dimension-hopping Engine begins with strictly defining what an entity belongs to. By separating
"The Rules" from "The Reality," we build a healthier, decoupled codebase for today's single-mission needs.
