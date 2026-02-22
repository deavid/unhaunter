# Simulation Boundaries and Code Health

**Date:** 2026-02-15

**Follows:** `01_dedicated_with_multi_mission_support_analysis.md`

NOTE: Status as of 2026-02-22 - this is deemed as not wanted.

---

## 1. Reframing: What Is This Actually About?

The multi-mission feasibility analysis (01) cataloged ~30 Resources that are per-mission, ~15 systems with `Local<T>`
leakage, unscoped entity queries, and nuclear cleanup. The original question was: "Can a single Bevy App host N
concurrent missions?"

But talking through the results, the more useful question turned out to be:

> **Does the codebase have clean simulation boundaries?**

The answer is no. And that matters regardless of multi-mission support.

---

## 2. What Even IS "The Game"?

There's a pattern in how games get built. You start by making the simulation — the thing that runs. Player walks, ghost
moves, fields propagate, doors open. That's the game. It works. You're already "in the place."

Then you retrofit everything else: menus, state machines, loading screens, options, UX. These are necessary for a
shipped product, but they're not the game. They're the **packaging around the simulation.**

Consider: Factorio's main menu has the game running in the background. Duke Nukem 3D's menu plays a demo. The simulation
is the fundamental thing. Everything else is scaffolding.

This maps directly to the engine-thought architecture work:

- The **Engine** is the simulator. Board, fields, actors, physics.
- The **Game Mode** defines what the simulation means — win conditions, ghost behavior, evidence.
- The **UI/States/Menus** are UX packaging that launches, configures, and presents the simulation.

The current codebase doesn't quite respect this hierarchy. `AppState` and `GameState` are Bevy States that gate
simulation systems, but they mix simulation lifecycle with UX flow. The simulation doesn't have its own concept of "I
exist" and "I'm done" — it's driven by external state machine transitions that also drive menus, loading screens, and
summary screens.

### The Simulator Lens

If you look at the code through the lens of "what is the simulator, and what is packaging," several things clarify:

| Current Pattern                     | Through the Simulator Lens                                         |
| ----------------------------------- | ------------------------------------------------------------------ |
| `AppState::InGame` gates systems    | The simulation is running                                          |
| `AppState::Loading` loads a map     | A simulation is being constructed                                  |
| `AppState::Summary` shows results   | The simulation has ended; UX is presenting its output              |
| `AppState::Lobby` waits for players | No simulation exists yet; UX is gathering configuration            |
| Resources hold per-mission state    | These are simulation state, not app state                          |
| `headless_summary_reset_system`     | Simulation teardown — but it doesn't know what "the simulation" is |

The simulation has no identity. It's not a thing you can point at. It's just "whatever entities and resources happen to
exist right now." That's why cleanup is nuclear, why resources are unscoped, and why loading a second map doesn't
perfectly clean up the first.

---

## 3. The Multi-Mission Analysis as Code Health Exercise

The exercise of asking "what if two missions ran simultaneously?" is valuable even if the answer is "we never do that."
It surfaces every piece of state that is implicitly scoped but not explicitly scoped.

Every item in the 01 document's catalog is also a code smell in the single-mission case:

| Multi-Mission Finding                | Single-Mission Code Health Issue                                        |
| ------------------------------------ | ----------------------------------------------------------------------- |
| ~30 Resources that are per-mission   | No clear boundary between "simulation state" and "app infrastructure"   |
| `Local<T>` leakage in ~15 systems    | System state that should be on entities (ghost timers, iteration state) |
| `.single()` assumptions              | Crashes if a second player/ghost is added (multiplayer, future modes)   |
| Unscoped entity queries              | No explicit "what context does this entity belong to?"                  |
| Nuclear cleanup                      | Map reload leaves leftovers; no scoped teardown                         |
| Entity handles in Resources          | Relational data in global bags instead of on entities                   |
| Global `AppState` driving simulation | Simulation lifecycle tangled with UX flow                               |

### Where This Connects to Prior Work

- **DDD Audit** (2026-01-30): Identified "God Resource" crates, PlayerSprite carrying ControlKeys, the God Factory in
  unmapload. These are all symptoms of missing domain boundaries — which are also missing simulation boundaries.

- **Engine-Thought** (Dec 2025): The "mechanisms vs meaning" split and three-tier model (Engine → Shared Mod → Game
  Mode) define where the simulation boundary should be. Multi-mission just makes that boundary mandatory instead of
  aspirational.

- **`.single()` Elimination** (DDD Plan 05): Removing `.single()` calls prepares systems for multiple entities.
  Multi-mission is the next ring: multiple independent groups of entities.

These aren't separate initiatives. They're concentric views of the same underlying question: **does the code know what
it's scoping over?**

---

## 4. The Query Strategy

A key advantage of multi-mission in a single Bevy World (vs. SubApps or multiple Worlds) is that systems process all
missions in one pass:

```rust
// One query, one iteration, all players across all missions
fn update_positions(mut q: Query<(&mut Transform, &Velocity), With<Player>>) {
    for (mut transform, velocity) in &mut q {
        transform.translation += velocity.0;
    }
}
```

For systems that need mission context — ghost AI querying nearby players — you bring the `MissionId` component along:

```rust
fn ghost_ai(
    q_ghost: Query<(&Ghost, &Transform, &MissionId)>,
    q_players: Query<(&Transform, &MissionId), With<Player>>,
) {
    for (ghost, ghost_transform, ghost_mission) in &q_ghost {
        for (player_transform, player_mission) in &q_players {
            if ghost_mission != player_mission { continue; }
            // ...
        }
    }
}
```

The query runs once across all missions. The mission filtering happens inside the loop, not by dispatch. This is ECS
doing what ECS is good at: data-oriented, cache-friendly iteration.

### Caveat: O(n×m) Queries

When missions share a single World, cross-entity queries (ghost vs. all players) become O(ghosts × all_players) instead
of O(ghosts × players_in_this_mission). With 8 concurrent missions of 4 players each, that's 32 players iterated per
ghost instead of 4. This is a ~8× overhead for those specific queries.

In practice this is fine for 8 missions. It would not be fine for 100. The target is small — roughly 8 concurrent
missions — and the iteration cost of filtering `MissionId` on 32 players is negligible compared to the actual
computation (pathfinding, field lookups) that happens per relevant player.

If this ever became a problem, the optimization path is clear: index players by `MissionId` in a lookup structure (e.g.,
`HashMap<MissionId, Vec<Entity>>` maintained by an observer). But that's premature optimization for the foreseeable
scale.

---

## 5. What We Actually Need (Minimum Viable Boundary)

Full multi-mission may never ship. But the insight from the analysis points to something smaller and more immediately
valuable:

### The "Simulation" as a First-Class Concept

Even for single-mission, the codebase would benefit from:

1. **A simulation root entity** — A single entity that represents "a mission is happening." Per-mission Resources become
   components on this entity. When the mission ends, `despawn_recursive` on this entity cleans up everything that
   belongs to it.

2. **Explicit simulation lifecycle** — Instead of `AppState` transitions driving simulation start/stop, the simulation
   has its own init and teardown that can be triggered by any context (state transition, network message, hot-reload).

3. **Scoped cleanup** — Right now, loading a map after another is "not 100% clean." Leftovers remain because there's no
   boundary that says "everything spawned between init and teardown belongs to this simulation." A root entity with
   children (or a marker component like `SimulationScoped`) would make cleanup deterministic.

This is essentially **hot-reload support for missions** — the ability to fully tear down one simulation and stand up
another without restarting the process. The dedicated server already needs this (clients disconnect, new lobby forms,
new mission starts). The client needs it too (player finishes a mission, goes back to lobby, starts another).

### What This Looks Like in Practice

```
Before:                              After:
┌─────────────────────┐              ┌─────────────────────┐
│       Bevy App      │              │       Bevy App      │
│                     │              │                     │
│  Res<BoardTopology> │              │  Entity: SimRoot    │
│  Res<ThermalGrid>   │              │    ├─ BoardTopology  │  (component)
│  Res<LightGrid>     │              │    ├─ ThermalGrid    │  (component)
│  Res<HauntState>    │              │    ├─ LightGrid      │  (component)
│  Res<RoomDB>        │              │    ├─ HauntState     │  (component)
│                     │              │    ├─ RoomDB         │  (component)
│  [scattered entities]│              │    └─ [child entities]│
│                     │              │                     │
│  cleanup: despawn   │              │  cleanup: despawn   │
│  everything tagged  │              │  SimRoot recursively │
└─────────────────────┘              └─────────────────────┘
```

The "After" model works for single-mission. It also works for multi-mission — you'd just have N `SimRoot` entities. But
the value is in the single-mission case: deterministic cleanup, clear boundaries, explicit lifecycle.

---

## 6. Relationship to Engine-Thought Architecture

The engine-thought work asked: "If Unhaunter-the-game were a mod of Unhaunter-the-engine, where would the seam be?"

The simulation boundary IS that seam, viewed from a different angle:

| Engine-Thought Framing  | Simulation Framing                                          |
| ----------------------- | ----------------------------------------------------------- |
| Engine provides Board   | The simulation root owns the Board                          |
| Engine provides Fields  | Fields are components on the simulation root                |
| Game Mode provides Loop | The game mode decides when to create/destroy the simulation |
| Mechanisms vs Meaning   | The simulation is mechanisms; the game mode gives meaning   |

The three-tier model (Engine → Shared Mod → Game Mode) describes what code goes where. The simulation boundary describes
**what runtime state belongs together.** They're complementary views.

If the simulation root entity pattern is adopted, it also answers the engine-thought question about "how do game modes
register": a game mode is the thing that creates a simulation root, attaches the right components, spawns the right
entities as children, and defines when the simulation ends.

---

## 7. What This Means for the Dedicated Server

The dedicated server's lifecycle is:

1. Start process → Lobby (no simulation)
2. Players connect, select map/difficulty
3. Mission starts → Simulation created
4. Mission ends → Simulation destroyed
5. Back to Lobby → goto 2

With an explicit simulation boundary, step 3 creates a `SimRoot` entity and everything mission-related is a child or
carries a marker. Step 4 is `despawn_recursive` on the `SimRoot`. No nuclear cleanup, no resource removal guessing, no
leftovers.

For multi-mission (if ever pursued), multiple `SimRoot` entities coexist. Each client connection is bound to a
`SimRoot`. Snapshots are built per-`SimRoot`. The lobby itself might optionally be a lightweight simulation (as
discussed in the architecture tensions doc).

---

## 8. Summary

| Question                                   | Answer                                                          |
| ------------------------------------------ | --------------------------------------------------------------- |
| Is multi-mission worth building now?       | Probably not. Optimization is reducing the need.                |
| Was the analysis useful?                   | Yes — as a code health audit, not a feature spec.               |
| What's the actionable takeaway?            | The codebase needs explicit simulation boundaries.              |
| Does this connect to engine-thought / DDD? | Deeply. Same underlying issue viewed from different angles.     |
| What's the minimum viable change?          | A simulation root entity + scoped cleanup.                      |
| Does that block on multi-mission?          | No. It's valuable for single-mission. Multi-mission is a bonus. |

### Deferred

- Multi-mission as a feature: parked. The optimization work is reducing room overhead enough that single-mission-per-
  process may suffice for the foreseeable scale (~120 rooms on an 8-vCore VPS).
- The full Resource→Component migration for 30+ types: not needed until multi-mission is actually pursued.
- `MissionId` component on all entities: unnecessary for single-mission.

### Worth Doing Regardless

- Continue the `.single()` elimination (DDD Plan 05) — makes systems robust for multiplayer and future modes.
- Continue the `Local<T>` → entity component migration where natural — cleaner code, testable state.
- Investigate a `SimRoot` or `SimulationScoped` marker for mission entities — enables deterministic cleanup even for
  single-mission.
- Audit map reload for leftover state — the "not 100% clean" problem is a concrete bug, not a theoretical concern.
