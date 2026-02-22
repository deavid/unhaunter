# Unhaunter 3D Transition Analysis

> _"The presentation is actively fighting the design."_

This document analyzes the decision to transition Unhaunter from 2D isometric pixel art to first-person 3D. This is a
major architectural decision that addresses a fundamental problem: **players approach the game with the wrong mindset
because the visual language tells them it's a different genre.**

NOTE: As of the latter docs/design/2026-02-21/core_mechanics_and_vision.md, we are going to try to stick to 2D for
a while, to give it an extra push to see if we can fix the problems correctly.

---

## Table of Contents

1. [The Problem Statement](#the-problem-statement)
2. [Why 2D Isometric Was Chosen Originally](#why-2d-isometric-was-chosen-originally)
3. [Why It's Not Working](#why-its-not-working)
4. [What First-Person 3D Changes](#what-first-person-3d-changes)
5. [Technical Feasibility](#technical-feasibility)
6. [The Duke Nukem 3D Approach](#the-duke-nukem-3d-approach)
7. [Risk Assessment](#risk-assessment)
8. [Prototype Requirements](#prototype-requirements)
9. [Success Criteria](#success-criteria)
10. [Decision Framework](#decision-framework)

---

## The Problem Statement

### The Symptom

Players consistently play Unhaunter "wrong":

- They rush through rooms instead of reading them
- They treat it as an action/arcade game requiring reflexes
- They ignore environmental cues (visible miasma gets "oh cool" instead of dread)
- They fill gaps in understanding with assumptions from other genres
- The onboarding teaches basics, but players learn the wrong mental model

### The Diagnosis

**The 2D isometric pixel art presentation actively miscommunicates the game's nature.**

When players see:

- 2D isometric view
- Pixel art aesthetic
- WASD movement
- Top-down perspective

Their brain imports expectations from hundreds of similar-looking games where:

- Speed and reflexes matter
- Action is the core loop
- Environmental details are decorative, not informational
- "Getting there" is the goal, not "understanding here"

**The game's systems are irrelevant if players never engage with them correctly.** No amount of polish on the current
presentation fixes this fundamental mismatch.

### The Evidence

After releasing v0.3.0 with extensive onboarding work (fake campaign, walkie-talkie buddy):

- Players still don't get the game concept
- Onboarding succeeds at teaching basics but trains wrong behaviors
- The game is "too flexible, too lenient" — players pass without proper understanding
- Adding atmospheric elements (miasma, effects) doesn't change player behavior
- Six months of burnout followed the realization that this path leads to infinite work

---

## Why 2D Isometric Was Chosen Originally

### Initial Reasoning

1. **Perceived feasibility:** 3D seemed "too much" for a solo developer
2. **Mental model:** The game was imagined as top-down 3/4 view
3. **Simulation architecture:** Tile-based systems seemed natural for 2D
4. **Lighting control:** Dislike of standard 3D lighting; wanted custom tile-based illumination

### Evolution

The initial top-down view hit limitations:

- Walls are crucial in horror games
- Wall contents (objects, details) are even more important
- Top-down shows only floors
- Moved to isometric to get 2 usable walls (out of 4)

This was a workaround, not a solution. The fundamental problem remained: the player is looking DOWN AT the space, not
BEING IN it.

---

## Why It's Not Working

### Genre Vocabulary Mismatch

First-person horror games have established expectations:

- Slow, deliberate movement
- Environmental examination as gameplay
- Atmosphere is inescapable (you're IN it)
- Dread and tension are embodied experiences
- "Reading the room" is the default behavior

2D isometric games have different expectations:

- Efficiency and speed
- Overview and control
- Action and reaction
- Environmental details as decoration
- "Getting to the objective" is the goal

**Unhaunter's design pillars assume first-person behaviors in a presentation that triggers the opposite.**

### The Onboarding Trap

The current approach creates a vicious cycle:

1. Players enter with wrong expectations (action game)
2. Game is lenient enough to let them progress
3. They learn to play incorrectly
4. They hit walls later when wrong behaviors fail
5. They blame the game or leave

Teaching players to play correctly requires fighting their assumptions constantly. This is unsustainable.

### Specific Failures

| Design Intent                 | 2D Isometric Result                     |
| ----------------------------- | --------------------------------------- |
| "Trespasser in hostile space" | Player feels like omniscient overseer   |
| "Reading the room"            | Rooms are small sprites to pass through |
| "Environmental wrongness"     | Details too small to notice or feel     |
| "Atmospheric dread"           | Atmosphere is observed, not experienced |
| "Slow, deliberate movement"   | WASD + isometric = "why so slow?"       |
| "Liminal spaces"              | Hallways are paths, not experiences     |

---

## What First-Person 3D Changes

### Definitively Better

| Aspect                   | Why First-Person Helps                                               |
| ------------------------ | -------------------------------------------------------------------- |
| **Player mindset**       | First-person horror = slow, careful, atmospheric (genre expectation) |
| **Environmental impact** | You SEE details at eye level, not as tiny sprites                    |
| **Atmosphere immersion** | You're IN the dark hallway, not looking at it                        |
| **"Reading the room"**   | Natural behavior in first-person; forced behavior in isometric       |
| **Liminal/wrongness**    | Hallway stretching, spatial wrongness are visceral in first-person   |
| **Sound design**         | Directional audio, immersive soundscape                              |
| **Trespasser feeling**   | Embodied experience, not abstract concept                            |
| **Tool usage**           | Holding a thermometer vs. selecting a sprite                         |

### Probably Better

| Aspect                   | Why It Might Help                                           |
| ------------------------ | ----------------------------------------------------------- |
| **Player patience**      | First-person horror players expect and accept slow movement |
| **AR Glasses**           | HUD overlay is native to first-person games                 |
| **Multiplayer presence** | Seeing another player in the space is more impactful        |
| **Object examination**   | Looking at objects directly vs. clicking sprites            |

### Neutral or Unchanged

| Aspect                   | Why It Doesn't Change                                                |
| ------------------------ | -------------------------------------------------------------------- |
| **System legibility**    | Still need AR Glasses — first-person doesn't show temperature fields |
| **Onboarding mechanics** | Still need discovery mechanisms for game systems                     |
| **Gameplay depth**       | Underlying simulation unchanged                                      |
| **Core loop**            | Infiltrate → Diagnose → Stage → Ritual → Escape remains              |

### Potentially Worse

| Aspect                       | Concern                                                  |
| ---------------------------- | -------------------------------------------------------- |
| **Spatial awareness**        | Isometric gives map overview; first-person is local only |
| **Multiplayer coordination** | Harder to know where teammates are                       |
| **Development velocity**     | 3D assets take longer, even with shortcuts               |
| **Differentiation**          | Lose "distinctive 2D look" in a genre dominated by 3D    |

---

## Technical Feasibility

### What Transfers Directly

The underlying systems are already 3D:

- **Tile-based simulation:** Tiles represent 3D volumes (60cm × 60cm × 220cm boxes)
- **Fluid dynamics:** Temperature, pressure systems are computed in 3D
- **Multi-floor support:** Systems already handle vertical space (stairs, levels)
- **Lighting engine:** Custom tile-based illumination (not standard 3D lighting)

**The simulation survives intact. Only rendering changes.**

### What Needs Building

| Component               | Scope                                                  |
| ----------------------- | ------------------------------------------------------ |
| **First-person camera** | Standard FPS camera controller                         |
| **3D tile rendering**   | Convert tile data to 3D geometry (cubes)               |
| **Sprite billboarding** | Objects/entities as sprites facing camera (DN3D style) |
| **Basic 3D models**     | Furniture volumes, player model for multiplayer        |
| **Input adaptation**    | Mouse look, interaction system                         |
| **UI/HUD**              | Tool displays, AR overlay system                       |

### What Stays the Same

- All game logic
- All simulation systems
- Map data structure
- Entity behavior
- Multiplayer networking (mostly)
- Save/load systems
- Audio (with spatial upgrade)

---

## The Duke Nukem 3D Approach

### The Philosophy

Not competing with AAA 3D horror. Creating a **stylized, tile-based first-person game** that leverages constraints as
aesthetic choices.

### Concrete Approach

| Element          | Implementation                                                    |
| ---------------- | ----------------------------------------------------------------- |
| **Walls/Floors** | Textured cubes/planes from tile data                              |
| **Objects**      | Sprites that billboard toward camera                              |
| **Furniture**    | Simple box volumes OR sprites, depending on importance            |
| **Entities**     | Sprites (ghost, NPCs) — semi-transparent, facing camera           |
| **Players (MP)** | Simple 3D models OR detailed sprites                              |
| **Lighting**     | Keep custom tile-based system, render as vertex colors or overlay |

### Reference Games

| Game              | What to Learn                                                        |
| ----------------- | -------------------------------------------------------------------- |
| **Dusk**          | Stylized low-poly horror FPS, atmosphere through design not fidelity |
| **Gloomwood**     | Immersive sim horror with deliberate lo-fi aesthetic                 |
| **Duke Nukem 3D** | Sprites in 3D space, sector-based level design                       |
| **Prodeus**       | Modern game with intentionally retro 3D aesthetic                    |

### Why This Works for Solo Dev

- Cubes are trivial to generate from tile data
- Sprites already exist (current 2D assets can be reused/adapted)
- No complex 3D modeling required for MVP
- Aesthetic is intentional, not placeholder
- Matches the tile-based simulation philosophy

---

## Risk Assessment

### Risks of Switching

| Risk                            | Likelihood | Impact | Mitigation                                   |
| ------------------------------- | ---------- | ------ | -------------------------------------------- |
| **Time investment fails**       | Medium     | High   | Prototype first, validate before full commit |
| **Art pipeline struggles**      | Medium     | Medium | DN3D approach minimizes 3D art needs         |
| **Doesn't fix mindset problem** | Low        | High   | Prototype specifically tests player behavior |
| **Performance issues**          | Low        | Medium | Tile-based means bounded complexity          |
| **Loss of distinctive look**    | Certain    | Low    | New look can be distinctive in different way |

### Risks of NOT Switching

| Risk                           | Likelihood | Impact | Mitigation                    |
| ------------------------------ | ---------- | ------ | ----------------------------- |
| **Continued player confusion** | Certain    | High   | None — this is the status quo |
| **Infinite onboarding work**   | High       | High   | None — already demonstrated   |
| **Developer burnout**          | High       | High   | Already happened once         |
| **Game never achieves vision** | High       | High   | None — fundamental mismatch   |

### Risk Asymmetry

- **If switch succeeds:** Fundamental problem solved, clear path forward
- **If switch fails:** 2-3 months lost, but definitive answer obtained
- **If don't switch:** Continue in demonstrated unsustainable state

---

## Prototype Requirements

### Purpose

The prototype tests ONE hypothesis:

> **"Do players naturally slow down, look around, and engage with the environment in first-person?"**

It does NOT test:

- Full feature parity
- Art quality
- Performance optimization
- All game systems

### Minimum Viable Prototype

| Component              | Requirement                                          |
| ---------------------- | ---------------------------------------------------- |
| **Map**                | One small existing map converted to 3D               |
| **Rendering**          | Textured cubes for walls/floors, sprites for objects |
| **Camera**             | Standard first-person controller                     |
| **Lighting**           | Basic implementation of tile-based system            |
| **One system visible** | Temperature OR entity presence (to test "reading")   |
| **Basic interaction**  | Open doors, pick up objects                          |
| **Entity**             | Ghost sprite, basic behavior                         |

### What to Skip

- Multiplayer
- Full tool set
- All maps
- Polish
- UI refinement
- Save/load
- Campaign structure

### Estimated Timeline

**8-12 weeks** for solo developer, potentially less with effective AI assistance.

---

## Success Criteria

### The Core Test

Put players in first-person Unhaunter and observe:

**Failure indicators:**

- Players sprint through rooms
- Environmental details ignored
- "Where do I go?" mindset
- Same behaviors as 2D version

**Success indicators:**

- Players creep forward
- Players check corners, look around
- Players notice environmental details unprompted
- Players express tension/unease from atmosphere alone
- "What is this?" curiosity rather than "Where to?" efficiency

### Secondary Indicators

- Does holding a tool feel meaningful?
- Does the atmospheric dread land?
- Do players naturally slow down?
- Does the space feel hostile to inhabit?

### The Honest Bar

If players STILL play it like an action game in first-person — the problem is deeper than presentation and first-person
won't solve it. But based on how first-person horror games are typically received, this is unlikely.

---

## Decision Framework

### Proceed with Prototype If

- You accept 8-12 weeks of development time for validation
- You're willing to adopt the DN3D aesthetic approach
- You understand this solves the mindset problem, not the system legibility problem
- You have capacity to evaluate honestly whether the prototype succeeds

### Do Not Proceed If

- You need certainty before investing time
- The DN3D aesthetic feels like unacceptable compromise
- You believe 2D can still work with more polish
- Development capacity is too limited for experimentation

### After Prototype

**If successful:**

- Commit to full 3D transition
- Plan phased migration of remaining systems
- Develop art pipeline for full game
- Consider parallel 2D maintenance vs. full sunset

**If unsuccessful:**

- Document what specifically failed
- Evaluate if partial 3D (hybrid?) makes sense
- Return to 2D with new understanding
- Consider other radical alternatives

---

## Summary

### The Core Argument

The 2D isometric presentation creates a **genre vocabulary mismatch**. Players' brains categorize the game as "action"
and behave accordingly. The game's systems — no matter how deep — are irrelevant if players never engage correctly.

First-person 3D **leverages existing genre expectations** rather than fighting them. Players enter first-person horror
games expecting to move slowly, examine environments, and feel dread. This is exactly what Unhaunter's design requires.

### What This Solves

- Player mindset on entry
- Environmental storytelling impact
- Atmospheric immersion
- Natural "reading the room" behavior
- The trespasser feeling as embodied experience

### What This Doesn't Solve

- System legibility (still need AR Glasses)
- Discovery mechanisms (still need pull-based learning)
- Wow factor for early engagement (separate problem)
- Gameplay depth (unchanged — that's good)

### The Recommendation

**Build the prototype.** 8-12 weeks is acceptable investment for a decision of this magnitude. The risk of not changing
(continued unsustainable state) exceeds the risk of changing (time investment with definitive answer either way).

The DN3D approach makes this feasible for a solo developer. You're not trying to make Phasmophobia. You're making
Unhaunter visible in the form it was always meant to take.

---

_Created December 2025 from design discussion._
