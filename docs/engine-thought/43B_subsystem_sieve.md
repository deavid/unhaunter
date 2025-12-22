# Subsystem Sieve: Abstract Architecture for Unhaunter

This document sieves the abstract **Subsystems** (collections of functionality) required for Unhaunter.

- **✅ Definitely Needed** — Core architectural pillars.
- **❓ In Doubt** — Useful but maybe too complex or niche.
- **❌ Not Needed** — Outside scope.

---

## ENGINE LAYER — Genre-Agnostic Simulation

### ✅ Definitely Needed

#### `Spatial Subsystem`
**Scope:** Grid coordinates, transforms, directions, map loading (TMX), spatial queries ("what is in this tile?").
**Why:** The foundation of a tile-based 3D game.
**Crate:** `unspatial`, `untmxmap`

#### `Sensory Physics Subsystem` (Light & Sound)
**Scope:** Light propagation (floodfill/raycast), visibility calculation, sound propagation (occlusion/distance), audio playback.
**Why:** "Illiterate Tools" need physical data to read. Atmosphere relies on lighting and sound.
**Crate:** `unlight`, `unsound` (or `unsensory`)

#### `Thermodynamics Subsystem`
**Scope:** Heat values per tile, heat sources, cooling rates, propagation.
**Why:** "Freezing Temperatures" is a core evidence type. The engine simulates heat; the game interprets it.
**Crate:** `unthermal`

#### `Inventory Subsystem`
**Scope:** Item slots, containers, pickup/drop logic, hands/equipped state.
**Why:** Players need to carry tools. Ghosts might interact with items.
**Crate:** `uninventory`, `ungear` (framework)

#### `Interaction Subsystem`
**Scope:** Raycasting for selection, "Use" verb, hold-to-interact logic, stateful objects (Doors, Switches).
**Why:** The primary way players affect the world.
**Crate:** `uninteraction`

#### `Locomotion Subsystem`
**Scope:** Actor movement, collision detection, pathfinding (NavMesh/A*), speed modifiers.
**Why:** Things need to move.
**Crate:** `unactor` / `unmovement`

#### `Atmosphere Subsystem`
**Scope:** Zone-based post-processing, fog density, color grading, particle emitters (dust/spores).
**Why:** Liminal horror relies on visual mood.
**Crate:** `unatmosphere`

---

### ❓ In Doubt

#### `Fluid Dynamics Subsystem`
**Scope:** Simulation of gas/liquid flow (Miasma).
**Why:** Cool for "Miasma" mechanic, but is it overkill? Can Miasma just be a static zone in `Atmosphere`?
**Verdict:** Keep simple for now. Maybe part of Atmosphere.

#### `Physics Object Subsystem` (Rigidbodies)
**Scope:** Real-time physics simulation for throwing objects.
**Why:** Poltergeist activity.
**Verdict:** Bevy has physics (Rapier/Avian). We just need to integrate it. Don't build a custom one.

---

## SHARED MOD LAYER — Paranormal Investigation

### ✅ Definitely Needed

#### `Sanity Subsystem`
**Scope:** Mental health tracking, drain logic (darkness, events), hallucination triggers.
**Why:** The "Health Bar" of psychological horror.
**Crate:** `unsanity`

#### `Forensics Subsystem` (Evidence)
**Scope:** Definition of Evidence types (EMF 5, Orbs), logic for detecting them, journal tracking.
**Why:** The core loop of "Classic" mode.
**Crate:** `unforensics`

#### `Entity Identity Subsystem`
**Scope:** Generation of ghost details: Name, Age, Death Cause, Shy/Aggressive traits.
**Why:** Gives the ghost personality and narrative weight.
**Crate:** `unidentity`

---

## GAME MODE LAYER — Specific Loops

### ✅ Definitely Needed

#### `Ghost AI Subsystem`
**Scope:** State machines (Idle, Roam, Hunt), behavior trees, sensory perception logic.
**Why:** The antagonist needs a brain.
**Crate:** `unghost`

#### `Mission Subsystem`
**Scope:** Objectives, win/loss states, campaign progression, difficulty scaling.
**Why:** The game needs a structure.
**Crate:** `unmission` / `uncampaign`

---

## Summary of Changes from "Component" View

| Old "Component" View | New "Subsystem" View |
| -------------------- | -------------------- |
| `ThermalMass`        | **Thermodynamics Subsystem** (includes Mass, Sources, Propagation) |
| `GridTransform`      | **Spatial Subsystem** (includes Grid, Map, Queries) |
| `Sanity`             | **Sanity Subsystem** (includes Drain, Effects, Recovery) |
| `Inventory`          | **Inventory Subsystem** (includes Slots, UI, Containers) |

This shift moves us from thinking about **data structs** to thinking about **architectural pillars**.
