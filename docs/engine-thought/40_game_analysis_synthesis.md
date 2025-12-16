# Synthesis: Engine Insights from Game Analysis

This document consolidates the engine design insights discovered by analyzing 25 games through the lens of "could this
be an Unhaunter mod?"

---

## Fit Summary

| Game                | Fit | Category                        |
| ------------------- | --- | ------------------------------- |
| Phasmophobia        | 95% | Core genre template             |
| Demonologist        | 90% | Core genre                      |
| Ghost Watchers      | 90% | Core genre                      |
| Ghost Exorcism Inc  | 85% | Core genre + meta               |
| Forewarned          | 80% | Core genre, different skin      |
| Mortuary Assistant  | 70% | Investigation + job tasks       |
| Devour              | 70% | Escape mode covers this         |
| Pacify              | 60% | Simplified action variant       |
| Lethal Company      | 50% | Multiple entities, scavenging   |
| Pools/Anemoiapolis  | 50% | Minimal engine test             |
| Obra Dinn           | 45% | Pure deduction, no threat       |
| Outlast Trials      | 45% | Stealth emphasis                |
| Silent Hill 2       | 40% | Atmosphere match, combat breaks |
| White Noise 2       | 40% | Asymmetric multiplayer          |
| LIMINAL SHROUD      | 40% | Pure atmosphere                 |
| P.T.                | 35% | Scripted vs simulated           |
| Alan Wake 2         | 35% | Case board concept              |
| Control             | 25% | Combat breaks genre             |
| Dead by Daylight    | 20% | Different genre (PvP)           |
| Stories Untold      | 20% | Interface-as-world              |
| Stanley Parable     | 15% | Branching narrative             |
| Home Safety Hotline | 15% | Remote diagnosis                |
| Midnight Ghost Hunt | 15% | Prop hunt PvP                   |
| Video Nasty         | 10% | FMV medium                      |
| Her Story           | 10% | Database interface              |

---

## Core Engine Capabilities (Confirmed)

These emerged as definitely-engine across all analyses:

| Capability               | Evidence                                               |
| ------------------------ | ------------------------------------------------------ |
| **Space / Board**        | Every spatial game needs this                          |
| **Light propagation**    | Critical for mood, used by most games                  |
| **Sound propagation**    | Critical for atmosphere                                |
| **Player movement**      | Universal                                              |
| **Entity existence**     | Most games have a threat                               |
| **Gear/tools framework** | Investigation games need equipment                     |
| **Field system**         | Readings are core to investigation                     |
| **Hiding mechanics**     | Exist in Unhaunter, validated by Outlast/stealth games |

---

## Validated Game Mode Concepts

These are confirmed as game-mode territory (not engine):

| Concept                   | Evidence                                          |
| ------------------------- | ------------------------------------------------- |
| **Entity behavior rules** | Every game configures differently                 |
| **Win/lose conditions**   | Varies wildly                                     |
| **Evidence meaning**      | What readings mean is game-specific               |
| **Specific gear items**   | EMF, thermometer, etc. are content                |
| **Loop structure**        | Investigate→identify vs collect→escape vs survive |
| **Theming**               | Ghosts, mummies, demons, aliens — same mechanics  |

---

## New Engine Capabilities Suggested

### High Priority (multiple games suggest this)

| Capability                        | Suggested by                   | Notes                                               |
| --------------------------------- | ------------------------------ | --------------------------------------------------- |
| **Multiple entities per mission** | Lethal Company, Ghost Watchers | Engine shouldn't assume exactly one entity          |
| **Entity binding modes**          | Mortuary Assistant             | Free roam / anchored to room / anchored to object   |
| **No-entity mode**                | Pools, LIMINAL SHROUD          | Entity should be optional                           |
| **Time-based escalation**         | Pacify, Lethal Company         | Entity can escalate on time, not just player action |
| **Fetch objectives**              | Devour, Lethal Company         | "Bring N items to location X" as objective type     |
| **Secondary objectives**          | Forewarned, Phasmophobia       | Optional goals for bonus rewards                    |

### Medium Priority (interesting but not essential)

| Capability                              | Suggested by           | Notes                                  |
| --------------------------------------- | ---------------------- | -------------------------------------- |
| **Case board / evidence visualization** | Obra Dinn, Alan Wake 2 | Visual connection of clues             |
| **Job/task system**                     | Mortuary Assistant     | Player has duties beyond investigation |
| **Day/night or time cycles**            | Lethal Company         | Environmental phases                   |
| **Procedural map generation**           | Forewarned             | Or procedural room arrangement         |
| **Stationary interaction mode**         | Stories Untold         | Locked to desk/terminal                |
| **Larger team support**                 | Ghost Exorcism Inc     | 6+ players                             |

### Low Priority (niche or outside core scope)

| Capability                          | Suggested by       | Notes                           |
| ----------------------------------- | ------------------ | ------------------------------- |
| **Asymmetric multiplayer**          | White Noise 2      | Player as entity                |
| **Voice interaction**               | Phasmophobia       | Spirit box voice recognition    |
| **Flashback/past viewing**          | Obra Dinn          | See past events at locations    |
| **Multiple identification targets** | Obra Dinn          | 60 people vs 1 ghost            |
| **Business management meta**        | Ghost Exorcism Inc | Between-mission company running |

---

## Confirmed Boundaries (Outside Scope)

These are explicitly NOT part of the engine:

| Boundary                | Why                    | Evidence                              |
| ----------------------- | ---------------------- | ------------------------------------- |
| **Combat**              | Breaks the genre       | Control, Alan Wake 2, Silent Hill 2   |
| **PvP/Competitive**     | Different genre        | Dead by Daylight, Midnight Ghost Hunt |
| **Branching narrative** | Different architecture | Stanley Parable                       |
| **FMV/video as core**   | Different medium       | Video Nasty, Her Story                |
| **Human enemies**       | Paranormal focus       | Outlast Trials                        |
| **Scripted horror**     | Simulation philosophy  | P.T.                                  |

---

## Key Philosophical Insights

### 1. Scripted vs Simulated

> **P.T. is scripted horror. Unhaunter is simulated horror. These are opposite philosophies.**

The engine provides honest simulation. Scares emerge from rules, not authored moments. This is a CHOICE that defines
Unhaunter's identity.

### 2. Investigation vs Objectives

> **Some games investigate (what is this?). Some games complete objectives (do this thing). Both are valid.**

Unhaunter's core is investigation. But the engine should support objective-based modes (Escape, Devour-style) where the
entity is known and the goal is action.

### 3. The Minimal Engine

> **LIMINAL SHROUD and Pools are Unhaunter with everything removed. What remains: space + player + light + sound.**

This confirms the true engine core. Everything else (entity, gear, evidence, objectives) is game mode.

### 4. Entity is Optional

> **"No entity" is valid. The haunting can be purely atmospheric.**

The engine shouldn't require an entity. Some horror is about absence.

### 5. Horror from Absence

> **Pools is terrifying because nothing is there. Presence isn't required for horror.**

The engine provides atmosphere. Whether something is actually there is game mode choice.

### 6. Movement is Core

> **The engine assumes the player MOVES. Games about being stationary (Stories Untold) are poor fits.**

Spatial exploration is fundamental. Interface-only games are outside scope.

### 7. Theming is Content

> **Egyptian tombs (Forewarned) work on the same mechanics as haunted houses.**

The engine doesn't care if it's ghosts, mummies, or aliens. Entity behavior and identification work the same.

---

## The Three-Tier Model (Refined)

The game analyses confirm and refine the three-tier model:

```
┌─────────────────────────────────────────────────────────────┐
│                     GAME MODE LAYER                         │
│  Entity behavior rules, win conditions, specific gear       │
│  Evidence meaning, loop structure, theming                  │
│  Examples: Classic, Escape, Phasmo-style, Devour-style      │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                    SHARED MOD LAYER                         │
│  Temperature field, EMF field, common gear framework        │
│  Evidence types shared between modes                        │
│  Ghost-hunting-specific but not mode-specific               │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│                     ENGINE LAYER                            │
│  Board/space, Light, Sound, Player movement                 │
│  Entity framework (not behavior), Gear framework            │
│  Field system, Hiding spots, Objectives framework           │
│  Time, Sanity container, Health container                   │
└─────────────────────────────────────────────────────────────┘
```

---

## What This Means for Engine Design

### The engine should provide:

1. **Space** — Board, positions, spatial queries
2. **Fields** — Light, sound as mandatory; framework for custom fields
3. **Entity framework** — Entities can exist, but behavior is configurable:
   - Zero, one, or many entities
   - Free roam, room-anchored, or object-anchored
   - Time-based or event-based escalation
4. **Player** — Movement, sanity/health containers, inventory
5. **Gear framework** — Items can read fields, affect space, be placed
6. **Objectives framework** — Hooks for win/lose, goal completion, fetch quests
7. **Hiding** — Spots where player is safe/hidden

### The engine should NOT provide:

1. Specific entity types or behaviors
2. Specific equipment implementations
3. Specific evidence meanings
4. Combat systems
5. PvP/competitive infrastructure
6. Branching narrative systems
7. Voice recognition

### The game mode provides:

1. What entity exists and how it behaves
2. What equipment does and what readings mean
3. What the player is trying to do
4. How you win or lose
5. The feel, pacing, and theming

---

## Next Steps

This analysis suggests the engine is well-scoped for its genre. The main extensions to consider:

1. **Multiple entities** — Remove the one-entity assumption
2. **Entity binding modes** — Support anchored entities
3. **Objective framework** — Generalize beyond investigation
4. **Case board** — Visual evidence connection (nice-to-have)

The boundaries are clear: no combat, no PvP, no scripted horror. The engine is for **cooperative investigation of
paranormal phenomena in simulated spaces**.
