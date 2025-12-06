# UNHAUNTER 2.0: Design Pillars & Vision Document

> _"You are not a ghost hunter. You are a trespasser in a sleeping dragon's den."_

This document captures the essential design philosophy, mechanics, and emotional goals for Unhaunter, distilled from
extensive design exploration.

---

## Table of Contents

1. [The Soul of Unhaunter](#the-soul-of-unhaunter)
2. [Core Design Pillars](#core-design-pillars)
3. [Critical Changes from MVP](#critical-changes-from-mvp)
4. [The Mystery Hook](#the-mystery-hook)
5. [Design Anti-Patterns](#design-anti-patterns)
6. [Open Design Questions](#open-design-questions)

> **See also:** `PLAYER_LEARNING_CHALLENGE.md` for onboarding strategies and visualization solutions.

---

## The Soul of Unhaunter

### What We Are Building

**Genre:** Nightmare Simulator / Occult Forensics **NOT:** Arcade Ghost Hunting

**The Core Experience:** A slow-burn, investigative horror game where the haunting is a _simulation_ running on rules.
Players gain power through understanding, not through weapons. The transition from **Prey to Predator** happens through
engineering and data, not combat.

**The Analogy:** If Phasmophobia is _Ace Combat_, Unhaunter is _Microsoft Flight Simulator_. If Phasmophobia is a
checklist, Unhaunter is _Factorio_ meets ghost hunting.

### The Feeling We're Chasing

> _"The dawning realization that you are trespassing in a system of rules you don't understand, where the very air can
> be hostile."_

Players should feel:

- **Fragile Competence** — You are an expert operating in a hostile, irrational system
- **Curiosity stronger than fear** — The mystery pulls you through the difficulty
- **Engineering satisfaction** — "I placed this thing here, and because I understood the rules, something happened over
  there"
- **Earned mastery** — You don't level up; you get _wiser_

### What Unhaunter Teaches

**Abductive Reasoning** — inferring explanations from incomplete, noisy observations. Like _Kerbal_ teaches orbital
mechanics, **Unhaunter teaches the scientific method applied to the supernatural.**

---

## Core Design Pillars

### The Complete Pillar List

| Pillar                  | Description                                                      |
| ----------------------- | ---------------------------------------------------------------- |
| **Genre**               | Nightmare Simulator / Occult Forensics                           |
| **Player Role**         | Trespasser / Intruder (Not Technician)                           |
| **Core Loop**           | Infiltrate → Diagnose → Stage → Ritual → Escape                  |
| **Win Condition**       | Individual Escape (High Climax)                                  |
| **Multiplayer Logic**   | "Save Yourself" (Co-op Ritual, Individual Extraction)            |
| **Entity Nature**       | The Sleeping Dragon (Baseline is peace; Player action stirs)     |
| **Difficulty**          | Player-Controlled (Risk vs. Reward via agitation)                |
| **Entity Behavior**     | Dynamic Circuit / Flow (No static "Ghost Room")                  |
| **Interaction**         | Indirect Control / Herding (Manipulation via environment)        |
| **Simulation Logic**    | Systemic Fidelity (Physics + Magic Fields)                       |
| **Investigation Logic** | Black Box Diagnosis (Stimulus → Response; No Recipes)            |
| **Tool Logic**          | Illiterate Tools (Detect Physics symptoms, not Magic causes)     |
| **Data Analysis**       | Triangulation / Abductive Reasoning (Filtering noise)            |
| **Pacing**              | Atmospheric Drag (Speed creates Turbulence/Aggression)           |
| **Movement**            | Slow = Data/Safety; Fast = Noise/Blindness                       |
| **Inventory**           | Backpack System (Field deployment, No Van loops)                 |
| **Visual Style**        | Watercolor / Liminal (Clean gradients, Dream-like, No Pixel Art) |
| **Audio Style**         | The Hum of Silence (Textured silence, Low-pass filters)          |
| **Theme**               | Western Analog Tech vs. Eastern Folklore Logic                   |
| **Failure State**       | Volatile/Visible Reaction (Explosive consequences)               |
| **Solution Space**      | Infinite Valid Approaches (No single optimal strategy)           |
| **Discovery**           | Pull, Not Push (Players discover; game reveals itself)           |
| **AR Glasses**          | Core Discovery Tool (Visualize simulation data directly)         |

---

## Critical Changes from MVP

### 1. Escape, Not Cleansing, Not Identifying

**Problem:** Cleansing/Identifying = anticlimactic checklist completion.

**Solution:** Win condition is **escape after the finale**. Ritual triggers chaos, exits blocked, each player's escape
is their victory.

### 2. Player-Controlled Challenge (The Sleeping Dragon)

Players control difficulty through their actions. Experienced players provoke deliberately to control chaos. Newbies
stay careful and respectful—blasting in = quick death.

### 3. No Ghost Room

**Problem:** Players camp one room, waste map content.

**Solution:** Entity as circuit/system using whole map. Ritual objects and clues scattered throughout.

### 4. Making Slow Gameplay Interesting

**Problem:** Slow walking is tiring, van trips tedious.

**Solutions:** Backpack system (deploy in field), environment dense with cues, active scanning, satisfying feedback.

### 5. Indirect Control (Herding the Fluid)

Players cannot fight the entity—they **manipulate** it. Entity behaves like fluid/animal. Tools (Attractors, Repellents,
Barriers) shape the battlefield. Stage the environment to control haunting's flow.

### 6. Visual Style: Watercolor/Liminal

Smooth gradients, few edges, dream-like, low contrast. Exploit pareidolia. Avoid: pixel art, grunge, high contrast,
visual noise.

### 7. Infinite Solution Space

**Problem:** Players optimize the game into a single "correct" approach. Depth disappears.

**Solution:** The game must be solvable through many valid approaches. No single optimal strategy. Solutions vary by
map, entity type, player skill, and playstyle. Two expert players can approach the same haunting completely differently
and both succeed.

**Key insight:** Like Kerbal — novice and expert use the same tools, but expert attempts _harder problems_ with _more
self-imposed constraints_. The game doesn't change; the player's ambition does.

### 8. Discovery Over Teaching

**Problem:** Tutorials, manuals, and scripted teaching (like walkie-talkie voice lines) require constant maintenance.
The game changes faster than documentation can keep up. Push-based teaching is a losing battle for a solo developer.

**Solution:** Pull-based discovery. The game reveals itself when players interact with it. Discovery mechanisms are tied
to the systems themselves — when a system changes, its discovery mechanism automatically reflects the new behavior.

**The Principle:** Players should DISCOVER gameplay, not be TAUGHT it. Discovery creates ownership — they didn't learn
your game, they figured out a mystery.

**Key Distinction:** Discovery is the _entry point_, not the _content_. Unlike Portal (where discovery IS the game),
Unhaunter's depth comes AFTER discovery, from infinite solution space and emergent complexity.

### 9. AR Glasses as Core Discovery Tool

**The Solution:** Augmented Reality Glasses that visualize simulation data directly.

**Why AR Solves Multiple Problems:**

| Problem                 | How AR Solves It                                                                             |
| ----------------------- | -------------------------------------------------------------------------------------------- |
| **Maintenance burden**  | AR shows simulation data directly — when systems change, AR automatically shows new behavior |
| **Teaching complexity** | No scripting needed — the system teaches itself by displaying itself                         |
| **Scalability**         | One tool, multiple overlays (temperature, entity presence, sound, contamination)             |
| **Battle awareness**    | Players lack spatial/temporal model — AR with minimap gives them the field view              |
| **Discovery mechanism** | Players SEE patterns, form hypotheses, test them — zero maintenance required                 |

**What AR Displays:**

- **Field Overlays:** Temperature gradient, entity attention field, sound propagation, contamination levels
- **Automapper/Minimap:** Explored areas, last-known readings (fading over time), deployed equipment, detected anomalies
- **Player trail:** Where you've been, what you found there

**Critical Insight:** AR shows _your data_, not _the truth_. Old readings might be outdated. The ghost moved. The
minimap shows where it WAS. This maintains the investigation aspect — you're building a model, not receiving answers.

**Why This Works:**

1. Player puts on AR, sees temperature overlay
2. Walks around, notices gradient points toward one room
3. Goes there, finds ghost room
4. Ghost moves, gradient shifts — player discovers ghosts aren't stationary

No tutorial. No voice line. No maintenance. The player discovered the temperature system by seeing it and noticing
patterns.

**Depth Through Interpretation:** Seeing data ≠ understanding it. Temperature AND entity presence overlays sometimes
correlate, sometimes don't. Why? That's the depth — AR shows symptoms, diagnosis is still on the player.

**Trade-offs:** To be determined. Options include battery drain, noise generation, obscured normal vision, requiring
stillness to activate. Expert players use AR strategically; novices lean on it and pay costs.

**Thematic Fit:** Western tech trying to map the unmappable. Instruments struggling to quantify folklore. The AR
embodies the game's core tension.

---

## The Onboarding Problem

> _"The game has depth. Nobody experiences it because they bounce before discovering it exists."_

### What Failed (Push-Based Teaching)

- **Manuals:** Players don't read them
- **Tutorials:** Players don't want to play them; they become outdated as the game changes
- **Walkie-talkie buddy:** Works but painful to develop, can't teach everything (fluid dynamics, haunted objects)
- **Small initial maps:** Taught wrong habits (too fast, too close, too lenient)
- **Low difficulty:** Let players progress while playing incorrectly, then hit a wall

### What Partially Works

The **fake campaign** — a tutorial disguised as a campaign. A set of scenarios with suggested order, getting players
into the real game immediately. Difficulty increases from Very Easy to Normal.

### Why It Still Fails

- Ramp-up is brutal
- No scenarios for teaching individual mechanics
- Too lenient early on → players develop bad habits
- Small maps teach wrong expectations (speed, proximity)
- Players reach halfway and hit a wall — too many evidence types, wrong mental model already formed

### The Core Tension

> Build the game I want to play truly → no one will play it (needs too many hours to learn)
>
> Dumb it down → becomes another Phasmophobia clone

**This is the actual design problem.** Not "is it boring" or "is it obscure" or "is there emergence." Those are
symptoms.

### The Path Forward: Discovery Over Teaching

The AR Glasses concept addresses this by shifting from push (you tell players) to pull (players discover). The discovery
mechanism is the game itself, not a layer on top that requires maintenance.

---

## The Mystery Hook

> _"Curiosity overrides frustration."_

Create **cognitive dissonance**—present something _wrong_ that the player's brain cannot ignore. The mystery isn't "what
ghost is this?" (checklist). It's "what _happened_ here?" (narrative).

### Environmental Mystery Techniques

**Spatial Wrongness:** Room that shouldn't exist (interior doesn't match exterior). Hallway that stretches each walk.

**Objects Tell Trauma:** Nursery with adult furniture. Kitchen with no knives. Jar of teeth in teacher's desk. Mirror
facing wall.

**Behavioral Wrongness:** Entity moves only when you look AT it. Arranges objects in almost-recognizable patterns. Radio
word changes per room.

**Previous Investigators:** Equipment set up methodically. Notes that start organized, become frantic. Their failures =
your warnings.

**Recurring Symbol:** Every haunting has a signature (scratches forming same character, specific time on every clock).
Player hunts for it.

**The Contradiction:** Death certificate 1952, photo 1983. Room locked inside, no body. Footprints into wall, never out.

**The Witness Object:** Teddy bear always facing you. Painting with gouged eyes only in active rooms. Object becomes a
character.

**Rules That Don't Make Sense (Yet):** Entity never enters rooms with red objects. Passive when you're wet. At first
weird—after investigation: "The victim drowned."

### The Psychological Mechanics

**Zeigarnik Effect:** Unfinished patterns create mental tension.

**The Equation:** `Frustration from Difficulty < Curiosity about Mystery = Player Continues`

**Critical Rule:** Every failure reveals something. Death isn't "game over"—it's "so THAT'S what happens when I break
that rule."

---

## Design Anti-Patterns

| Anti-Pattern        | The Error                                             | The Fix                               |
| ------------------- | ----------------------------------------------------- | ------------------------------------- |
| **Scooby Doo**      | Ghost is "actually just" dirty electricity/infrasound | Entity is REAL. Symptoms, not causes. |
| **Wikipedia Quiz**  | Punish cultural ignorance (4=death in Japanese)       | Organic consequences, not trivia      |
| **Plumber/Janitor** | Fix HVAC to stop ghost                                | Manipulation, not repair              |
| **Fetch Quest**     | Find real salmon for Bear Spirit                      | Symbolic appeasement, not literal     |
| **Gamification**    | NE = max EMF always                                   | Subtle field bias, not recipes        |
| **19Hz Obsession**  | Use infrasound to hurt players                        | Simulate effect visually              |

**The Meta-Failure:** Making magic fully rational. Ghost = physics problem. What Unhaunter needs: **apparent
irrationality**—rationality hidden behind complexity.

**The Test:** Would this recreate "The Ring" organically, or would it be comedic?

---

## Open Design Questions

1. **Moment-to-Moment Slow Gameplay** — How to make walking/waiting genuinely engaging?
2. **Ghost Room Elimination** — Specific mechanics to ensure full map use
3. **The First 5 Minutes** — How to get players to stay 15 minutes instead of 5?
4. **Engineering vs. Observation** — How much trap-building vs. pure deduction?
5. **Multiplayer Dynamics** — What happens during ritual? Griefing prevention?
6. **The Onboarding Paradox** — How to teach without tutorials, punish bad habits without frustrating, ramp difficulty
   without walls?
7. **Legibility vs. Mystery** — How much AR visualization before it stops feeling like investigation?

---

## The Manifesto

1. **Anti-Arcade:** We simulate hostile atmospheres, not script scares
2. **Anti-Hero:** Player is a trespasser, not a Ghostbuster
3. **Anti-Tricorder:** Tools give symptoms, not answers
4. **Physics of Magic:** Magic flows like fluid, builds pressure, leaks
5. **Law of Drag:** Speed = Noise = Death. Slow is smooth, smooth is fast
6. **Win Condition:** Survive extraction, not cleanse the house
7. **Mystery:** Curiosity overrides frustration. Questions drive play
8. **Legibility:** Show state clearly, hide strategy. Symptoms visible, diagnosis earned.
9. **Infinite Solutions:** No optimal path. Many valid approaches. Players don't "solve" the game.
10. **Discovery Over Teaching:** Pull, not push. The game reveals itself. No maintenance-heavy tutorials.

---

## Reference Games

| Game                    | Lesson                                        |
| ----------------------- | --------------------------------------------- |
| Phasmophobia            | Base loop to improve, co-op dynamics          |
| Pathologic 2            | Mystery as hook, hostile systems              |
| Return of the Obra Dinn | Abductive reasoning, sensory satisfaction     |
| Kerbal / Factorio       | Fail loud, physics as gameplay, niche mastery |
| Shenzhen I/O            | Manual as lore, modular logic                 |
| The Backrooms           | Analog horror, liminal spaces, escape goal    |

---

_Created December 2025 from extended design conversation and updated with onboarding analysis and legibility
principles._
