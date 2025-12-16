# Unhaunter Engine Architecture: Design Document

> _"If Unhaunter-the-game were a mod of Unhaunter-the-engine, where would the seam be?"_

---

## Document Purpose

This document captures the architectural vision for separating Unhaunter into an **engine** and **game mode** structure.
It is the result of extensive design exploration focused on understanding where boundaries should exist—not for external
modding, but as a **design constraint** to force proper code organization.

The goal: **The game becomes a mod of itself.**

### A Note on Process

This document emerged from iterative design conversations. At several points, the discussion veered into over-abstract
territory or generated "plausible-sounding" frameworks that didn't actually connect to reality. The owner (David)
repeatedly called this out, forcing the conversation to step back and consolidate.

The frameworks here are the ones that **survived** that scrutiny—the ideas that mapped to real code decisions and real
game design questions. Speculative layers and speculative systems that seemed reasonable but didn't connect were
discarded.

---

## Table of Contents

1. [Design Pillars (Context)](#design-pillars-context)
2. [The Problem](#the-problem)
3. [The Vision](#the-vision)
4. [Core Principle: Mechanisms vs Meaning](#core-principle-mechanisms-vs-meaning)
5. [What the Engine Provides](#what-the-engine-provides)
6. [What Game Modes Provide](#what-game-modes-provide)
7. [The Three-Tier Model](#the-three-tier-model)
8. [The Field System](#the-field-system)
9. [Key Decisions](#key-decisions)
10. [The Genre as Negative Space](#the-genre-as-negative-space)
11. [Current Codebase Analysis](#current-codebase-analysis)
12. [Open Questions](#open-questions)
13. [Appendix: What This Is Not](#appendix-what-this-is-not)

---

## Design Pillars (Context)

Before diving into architecture, it's worth noting Unhaunter's core design philosophy. These pillars explain _why_
certain boundaries exist:

| Pillar               | Description                                                       |
| -------------------- | ----------------------------------------------------------------- |
| **Genre**            | Nightmare Simulator / Occult Forensics (NOT arcade ghost hunting) |
| **Player Role**      | Trespasser / Intruder (not technician)                            |
| **Simulation Logic** | Systemic Fidelity (Physics + Magic Fields, honest rules)          |
| **Tool Logic**       | Illiterate Tools (detect physics symptoms, not magic causes)      |
| **Data Analysis**    | Triangulation / Abductive Reasoning (filtering noise)             |
| **Discovery**        | Pull, Not Push (players discover; game reveals itself)            |
| **Anti-Arcade**      | We simulate hostile atmospheres, not script scares                |
| **Solution Space**   | Infinite Valid Approaches (no single optimal strategy)            |

The engine should **enable** these pillars without **mandating** them. A game mode that wanted a more arcade feel could
still use the engine—but the engine's design leans toward simulation fidelity.

---

## The Problem

The Unhaunter codebase has grown organically. While there's reasonable intuition about separation (simulation in one
place, ghost logic in another), several "core" crates have become dumping grounds:

- **`uncore-components`** — A "God Component" crate mixing spatial primitives with game-specific components
- **`uncore-resources`** — A "God Resource" crate bundling unrelated global state
- **`uncore-types`** — A grab bag of miscellaneous types

These crates blur the boundary between engine-level infrastructure and game-specific logic. When everything depends on
everything, making changes becomes risky, compile times suffer, and reasoning about the system becomes difficult.

### Why "Engine vs Game Mode"?

This framing isn't primarily about supporting external modders. It's a **mental exercise** to discover natural
boundaries in the code.

- **First benefit:** Forces the code into proper organization
- **Second benefit:** Enables future game modes (Classic, Escape, others)
- **Third benefit:** If a community forms, they can fork and customize more easily

The "modder" in this context is a programmer—likely David himself. "Mods" are built into the main distribution, not
external plugins.

---

## The Vision

### The Central Question

> **What is the MAXIMUM the engine can provide while still allowing fundamentally different games to be built?**

Not the minimum—that would just be Bevy or Unity. The goal is to pack in enough features that making liminal horror
games becomes easy, even if specific games need to work around some constraints.

### The Framing

The engine simulates a **space that feels haunted**, without knowing what "haunted" means.

- The engine provides **mechanisms**: fields propagate, actors exist, players hold things
- The game mode provides **meaning**: this reading means danger, this entity is the ghost, this is how you win

### Two Planned Game Modes

1. **Unhaunter Classic** — Locate the entity, find its room, classify it, create the repellent. Investigative.
2. **Unhaunter Escape** — Perform a ritual, the entity traps you, escape to win. Survival/action.

Both share significant infrastructure. The differences are in the loop, entity behavior, and objectives.

#### Classic Mode (Current Implementation)

The player investigates a haunted location to identify what kind of entity is present:

- Use tools to gather evidence
- Triangulate readings to classify the entity
- Build the correct repellent
- The challenge is **diagnosis**—understanding what you're dealing with

#### Escape Mode (Planned)

The player performs a ritual that awakens/angers the entity:

- Prepare the location before triggering the ritual
- The ritual causes the entity to trap the player (exits block)
- The player must escape the location
- The challenge is **survival after provocation**

The entity behavior is fundamentally different: In Classic, the entity is a passive puzzle to be diagnosed. In Escape,
the entity becomes an active threat to be evaded.

---

## Core Principle: Mechanisms vs Meaning

This is the fundamental architectural insight:

|            | Engine (Mechanisms)                       | Game Mode (Meaning)                      |
| ---------- | ----------------------------------------- | ---------------------------------------- |
| **Fields** | Values propagate across the grid          | "High EMF means the ghost is near"       |
| **Actors** | Things exist and move in space            | "This actor is the ghost, it hunts"      |
| **Tools**  | Items can read field values               | "The EMF reader beeps when reading > 50" |
| **Health** | A value exists on the player              | "Health < 0 means death, game over"      |
| **Doors**  | Interactive object with open/closed state | (Doors are common enough to be engine)   |

The engine is **illiterate**—it provides the vocabulary without understanding the sentences.

---

## What the Engine Provides

### Definite Engine Territory

| Component                    | Description                                                             |
| ---------------------------- | ----------------------------------------------------------------------- |
| **The Board**                | 3D discrete grid, coordinates, spatial queries, adjacency               |
| **Light Propagation**        | How light spreads through space—tied to rendering, mandatory            |
| **Sound Propagation**        | How sound spreads through space—tied to audio, mandatory                |
| **Field Framework**          | Generic system for defining fields, their types, propagation algorithms |
| **Actor Framework**          | Things can exist in space, move, be rendered                            |
| **Player Basics**            | Movement, holding items, basic interaction                              |
| **Inventory Framework**      | The concept of holding things, not specific items                       |
| **Gear Framework**           | The `GearUsable` trait, inventory slots—not specific equipment          |
| **Map Loading**              | TMX parsing, level loading infrastructure                               |
| **Common Objects**           | Doors, switches, stairs—common enough to warrant engine support         |
| **Settings/Persistence**     | Save/load mechanisms, settings management                               |
| **Asset Loading**            | Infrastructure for loading and managing assets                          |
| **Health/Sanity Containers** | The values exist; behavior is game mode territory                       |

### Why Light and Sound Are Special

Light and sound are **engine-level** because they're tied to rendering and audio output. They define the mood
infrastructure that makes the genre work.

A game mode that doesn't want light-based gameplay would need to create proxy fields, not disable the light system. The
engine defines _how_ light behaves and renders.

---

## What Game Modes Provide

### Definite Game Mode Territory

| Component                  | Description                                            |
| -------------------------- | ------------------------------------------------------ |
| **The Entity/Ghost**       | AI, behaviors, types—or no entity at all               |
| **Specific Equipment**     | EMF reader, thermometer, what they do                  |
| **Evidence System**        | What evidence exists, how it's gathered, what it means |
| **The Loop**               | Objectives, victory conditions, failure states         |
| **Sanity/Health Behavior** | What affects these values, what thresholds trigger     |
| **Mission Structure**      | Campaigns, progression, scoring                        |
| **Game-Specific UI**       | Truck interface, journal, summary screens              |
| **NPCs**                   | Tutorial helpers, any non-player non-entity characters |
| **Walkie-Talkie System**   | Communication mechanics specific to Unhaunter          |

### The Loop, Defined

A loop is the **set of action types the player must perform to advance or succeed**.

The engine doesn't know what the loop is. It provides hooks—game state transitions, events, the ability to declare "game
is done." The game mode orchestrates these into a coherent experience.

The engine knows _when_ the game is done (the game mode signals it). The engine doesn't know _what_ "done" means.

---

## The Three-Tier Model

Analysis revealed not just engine vs game mode, but a middle layer:

```
┌─────────────────────────────────────────────────┐
│              GAME MODE LAYER                    │
│  Ghost AI, specific gear, loop, objectives      │
│  Walkie system, truck UI, mission structure     │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│              SHARED MOD LAYER                   │
│  Temperature field, EMF field, entity presence  │
│  Fields shared between Classic and Escape       │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│              ENGINE LAYER                       │
│  Board, Light, Sound, Player basics             │
│  Gear framework, map loading, doors             │
└─────────────────────────────────────────────────┘
                      │
                      ▼
┌─────────────────────────────────────────────────┐
│              BEVY / RUST                        │
└─────────────────────────────────────────────────┘
```

### Why Three Tiers?

- **Light and Sound** are engine-level (tied to rendering/audio)
- **Temperature, EMF, entity presence** are shared between game modes but not engine-core
- Both Classic and Escape use the same fields; they differ in what those fields _mean_

This allows game modes to share substantial infrastructure while remaining fundamentally different.

---

## The Field System

Fields are central to Unhaunter's architecture. The design doc "Parallel Simulator Architecture" describes two
simulation engines running in parallel:

1. **Physics Engine** — Light, Sound, Heat, Motion
2. **Folklore Engine** — The "Magic" Logic

The game IS the friction between them.

This duality is **engine-level** in the sense that the engine provides facilities for both kinds of fields and their
interactions. But it's **mod-level** in that a game mode chooses which fields to use and what they mean. A simple mode
might only use physics fields. A complex mode might layer mystical fields on top.

### Engine Responsibilities

- Provide the field _framework_ (how to define a field, how fields attach to the board)
- Provide specific built-in fields: **Light, Sound** (mandatory for rendering/audio)
- Provide propagation algorithms (wave propagation, fluid simulation, raycast + floodfill)
- Expose field types (scalar, vector, etc.) that affect how fields behave
- Support field interactions (when one field affects another)

### Game Mode Responsibilities

- Define which additional fields exist (temperature, EMF, entity presence)
- Define what emits into each field
- Define what reads from each field
- Define what field values _mean_

### The "Illiterate Tools" Principle

From the design docs:

> "Tools aren't broken (random), they're **illiterate** (measuring the wrong thing). EMF beeps = Magnetic Field, not
> Ghost."

The engine knows there's a field. A tool reads the field. The game mode defines that high readings mean something is
near. The engine provides symptoms; the game mode provides diagnosis.

**The Calibration Moment:**

A key teaching pattern emerges from this architecture:

1. Player sees a chair slide
2. Motion sensor doesn't trigger
3. Realization: "The force has no physical mass/heat"
4. Lesson: Eyes are primary. Tools are filters.

This teaching happens _because_ the engine provides honest simulation. The engine doesn't script the lesson—the lesson
emerges from mechanism meeting meaning.

### Field Configurability

Game modes need deep control over fields:

1. **Use existing field as-is**
2. **Configure existing field** (different decay curve, range)
3. **Define new field using existing propagation**
4. **Define new field with custom propagation** (advanced)

Most game modes use levels 1-3. Level 4 is for advanced modifications.

---

## Key Decisions

### Doors: Engine

Doors are common enough that the engine should provide how they work. Base door behavior (interactive, toggleable state,
blocks collision when closed) is engine-level. The engine already has utilities to allow new door types.

### "Threat" / AI Director: Does Not Exist

There is no "threat level" or pacing management system. This was a misunderstanding based on games like Left 4 Dead (AI
Director) and Alien: Isolation (Menace System).

These systems explicitly manage tension and pacing, creating "rubber-banding" to manipulate difficulty invisibly.

**This goes against Unhaunter's design philosophy:**

> "Anti-Arcade: We simulate hostile atmospheres, not script scares."

Tension in Unhaunter emerges from **honest simulation**. The ghost does what physics and its AI dictate. The environment
responds according to rules. The player feels tension from _understanding the system_, not from invisible manipulation.

No threat system needed. Tension emerges from simulation.

### Entity: Game Mode Concept

At the engine level, there's no "Entity" or "Ghost" concept. There are actors—things that exist and move in space.

**Actor vs Entity Distinction:**

An "Actor" is something that performs, that feels alive. An NPC might be an actor. A turret that seeks and fires is an
actor. But a machinegun lying on the ground is just an item—not an actor.

An "Entity" (the monster/ghost) is an actor with special significance to the game mode. But that significance is
entirely mod-defined. The engine doesn't know one actor is "the ghost" and another is "the helper NPC."

A game mode might:

- Designate one actor as "the ghost" with specific AI
- Have multiple entities
- Have no discrete entity (the location itself is haunted)
- Make the map the entity

The engine provides actor infrastructure. What actors _mean_ is game mode territory.

### Health and Sanity: Engine-Defined, Mod-Controlled

The pattern: **the engine provides the container, the game mode provides the rules**.

- Health exists as a value on the player (engine)
- What causes damage, what health < 0 means (game mode)
- Sanity exists as a value (engine, maybe)
- What affects sanity, what low sanity does (game mode)

Many horror games treat health as binary (alive/dead). That's just health = 100, any attack deals > 100 damage.

### Game Mode Registration: Deferred

How game modes register their systems is lower priority than separating components properly.

Rough direction:

- `ungame` becomes `ungameclassic`
- A new crate adds the concept of "game mode"
- Ability to spin systems up/down based on active mode
- Single binary, mode selection determines which plugins load

This will be designed after component boundaries are clear.

---

## The Genre as Negative Space

An important insight from design exploration:

> "The game genre seems to be defined by the negative space, of what it lacks is what defines it. The game is more like
> a hole."

Things that **break** the ghost hunting genre:

- Adding weapons, fights
- Adding fast-paced dexterity (double dash, wall jumps)
- Adding in-game music
- Adding too much texture/detail (removes liminality)

What **varies** within the genre is still being explored. Analog horror, liminal horror, and Backrooms-style games share
common ground.

The engine should avoid providing things that would break the genre. It enables the negative space.

---

## Current Codebase Analysis

### The "God Crates" Problem

Three crates became dumping grounds:

| Crate               | Problem                                             |
| ------------------- | --------------------------------------------------- |
| `uncore-components` | Player, Ghost, UI, and spatial components all mixed |
| `uncore-resources`  | BoardData alongside GhostGuess and WalkiePlay       |
| `uncore-types`      | GhostType, Evidence mixed with generic field types  |

### Clearly Engine (from existing crates)

- `uncore-foundation` — Primitive types, constants, math
- `unspatial` — Position, Direction, BoardPosition (created in recent refactor)
- `untags` — Marker components (created in recent refactor)
- `uninteraction` — Generic interaction primitives (created in recent refactor)
- `unlight` — Light propagation
- `unfog` — Fluid/fog simulation
- `ungear` — Gear framework (not specific items)
- `unmapload`, `untmxmap` — Map loading infrastructure
- `unsettings`, `unmenusettings` — Settings management

### Clearly Game Mode (from existing crates)

- `unghost` — Ghost AI and behaviors
- `ungearitems` — Specific equipment implementations
- `uncampaign` — Mission/campaign structure
- `untruck` — Truck interface
- `unsummary` — End-of-mission summary
- `unwalkie`, `unwalkie_types`, `unwalkiecore` — Walkie system
- `unnpc` — NPCs

### Mixed (need conceptual splitting)

- `unplayer` — Basic movement (engine) + sanity/hiding behavior (game mode)
- `undifficulty` — Framework (engine) + specific parameters (game mode)
- `uncore-events` — Generic events (engine) + specific events (game mode)
- `uncore-board` — Grid (engine) + specific behaviors (varies)
- `unmenu`, `uncoremenu` — Basic menu (engine) + specific content (game mode)
- `unprofile` — Save/load mechanism (engine) + what gets saved (game mode)

### Progress Made

Phases 1 and 2 of a previous refactoring effort created `unspatial`, `untags`, and `uninteraction`. This was the right
direction—extracting clean, low-level components from the God crates.

---

## Open Questions

### What is the minimal game mode?

From discussion: If you just provided a map to load, you'd get a player that can walk around and open doors. No threat,
no win, no lose.

The **minimal** game mode needs:

- A win condition (event to tell the engine the game succeeded)
- Optionally, a lose condition
- Some scoring mechanism

Example: Place an item randomly in the location. Detect when the player gets outside with the item. Trigger success.
That's a crude game loop, but it's a game.

This suggests the engine needs:

- Hooks for "game is done" (win/lose)
- The ability to query player state (position, inventory)
- Scoring output channels

### How do components compose?

The initial thinking was hierarchical layers. But game modes might want to "grab stuff from layer 7 skipping layer 6."

Better model: **components with dependencies**. Most depend on Board. Some depend on each other. A game mode picks which
components it needs.

### What propagation algorithms does the engine provide?

Currently the codebase has:

- Fluid simulation (potential energy + speed)
- Light (raycast + floodfill)
- Wave propagation (vector-based)

These may be variations of the same underlying approach, optimized differently. Whether the engine provides all of these
or a framework for defining them is undecided.

### What about "proxy fields"?

If a game mode needs tools to read something other than light/sound, but wants the values to correlate with light
sources, there may need to be a way to have light sources automatically emit into both the light field and a custom
field.

This "proxy field" concept needs design work.

### How do game modes handle procedural variation?

The design docs describe a "Virology Model" for keeping veteran players scared: the rules themselves can vary between
runs. Not just random maps—randomized cause and effect:

- Run A: Mirrors are dangerous
- Run B: Mirrors are the only safe place
- Run C: Mirrors are inert, windows are dangerous

This procedural variation of rules is **entirely game mode territory**. The engine doesn't know what "dangerous" means.
But the engine needs to provide:

- Field configurability sufficient for rule variation
- No hardcoded assumptions about what fields mean
- The ability for a game mode to parameterize its own behavior

The Virology Model is why the "mechanisms vs meaning" split matters: if the engine embedded meaning, procedural
variation of meaning would be impossible.

---

## Appendix: What This Is Not

### Not an AI Director

Unhaunter does not and should not have a system that manages tension or pacing. Games like Left 4 Dead and Alien:
Isolation use invisible directors to manipulate difficulty. This creates artificial feeling and goes against
simulation-based horror.

Tension emerges from the simulation being honest. The player's relationship with information creates fear.

### Not a General Engine

The goal is NOT to build something as general as Unity or Godot. The engine is deliberately **specific** to liminal
horror. It knows about grids, light propagation, and atmospheric mood. A WW2 tank game wouldn't make sense on this
engine.

Specificity is the feature. It makes building games in this genre easier.

### Not a Reskin System

Game modes can change the **rules**, not just the content. The evidence system, the entity behavior, the loop itself can
be replaced. This is harder than reskinning but enables fundamentally different games.

### Not an External Plugin System

"Mods" are not downloaded plugins. They're compiled into the game. A "modder" is a programmer who forks the repo or adds
crates. Distribution is through the main game binary with mode selection.

### Multiplayer Considerations

The design docs mention "Save Yourself" multiplayer logic: co-op ritual, but individual extraction. Each player's escape
is their own victory.

This suggests:

- The engine provides multi-player infrastructure (multiple players in the same space)
- The game mode defines whether players cooperate, compete, or have individual win conditions
- The engine doesn't assume "team win" or "individual win"—it provides hooks for both

---

## Summary

The Unhaunter engine architecture is built on one core insight:

> **The engine knows about MECHANISMS but not MEANING.**

The engine provides:

- Spatial infrastructure (board, positions, grid)
- Simulation systems (light, sound, field propagation)
- Actor framework (things exist and move)
- Player basics (movement, inventory, interaction)
- Common objects (doors, switches)
- Game state hooks (transitions, events)

The game mode provides:

- What fields mean
- What actors represent
- What the player is trying to do
- How to win or lose
- Everything that makes it _this_ game rather than _a_ game

The three-tier model (Engine → Shared Mod → Game Mode) acknowledges that some things (temperature, EMF) are shared
between modes but aren't engine-core.

The existing codebase has the right intuition but suffers from "God crates" that blur boundaries. Ongoing refactoring is
extracting clean components. The abstract understanding documented here should guide that work.

---

_Document synthesized from design exploration conversations, December 2025._
