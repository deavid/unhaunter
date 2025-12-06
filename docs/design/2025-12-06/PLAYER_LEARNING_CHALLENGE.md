# The Player Learning Challenge

> _"How do you make incompetence fun?"_

This document explores Unhaunter's core design challenge: teaching players a complex system they have no intuition for,
without destroying the mystery.

> **See also:** `UNHAUNTER_DESIGN_PILLARS.md` for core vision and design rules.

---

## Table of Contents

1. [The Core Problem](#the-core-problem)
2. [What Unhaunter Teaches](#what-unhaunter-teaches)
3. [Why the MVP Failed](#why-the-mvp-failed)
4. [How Players Play Wrong](#how-players-play-wrong)
5. [The Visualization Problem](#the-visualization-problem)
6. [The Black Box Problem](#the-black-box-problem)
7. [Reference Games & What They Teach Us](#reference-games--what-they-teach-us)
8. [The Six Engagement Pillars](#the-six-engagement-pillars)
9. [Hook Strategies (Detailed)](#hook-strategies-detailed)
10. [Error Consequence Philosophy](#error-consequence-philosophy)
11. [The Target Audience](#the-target-audience)
12. [Fundamental Trade-offs](#fundamental-trade-offs)
13. [Open Questions](#open-questions)

---

## The Core Problem

### The Cliff of Mastery

Unhaunter demands expertise from players who have no framework for acquiring it.

**90s vs 2020s:** Old games were made to be mastered. Modern games are consumed like films—few hours, done, next. Asking
players for 1-2 hours to learn the rules is "too much."

**The Flight Simulator Analogy:** You can't give a random person plane controls and expect survival.

---

## What Unhaunter Teaches

**The Skill:** Abductive Reasoning — inferring explanations from incomplete, noisy data.

**The Barrier:** No real-world intuition. Kerbal uses gravity (everyone knows "down"). Unhaunter uses "Spiritual
Pressure" — no childhood experience to draw from.

**The Challenge:** Kerbal teaches physics (real knowledge transfers). Unhaunter teaches physics + magic (no prior
knowledge). Much harder cognitive load.

---

## Why the MVP Failed

**The Simplification Trap:** Removing complexity removed the soul WITHOUT helping learning. The problem isn't
difficulty—it's FEEDBACK.

**The Spiral of Desperation:**

1. Player records wrong evidence (mistake 10 min ago)
2. Thinks they have the ghost, crafts repellent—nothing happens
3. Assumes problem is LAST evidence (wrong)
4. Tries random combinations, gets frustrated
5. "I can feel them wanting to flip the table"

**Alternative bad behavior:** "Keep crafting repellents, one eventually hits." Players ignore scoring. Solution: limit
to 3 repellents.

---

## How Players Play Wrong

**Three Failure Modes:**

1. Running without understanding — awaken paranormal, don't understand rejection
2. Ignoring tool readings — can't interpret data, world feels random
3. Treating tools like tricorders — expect answer on screen, not synthesis

**The Missing Mental Model:** Players need "battle awareness"—mental model of map, values, evolution. But most isn't
visual—it's data on devices.

**Delayed Consequence Problem:** Players expect action→consequence. Delayed or combination consequences aren't
registered.

---

## The Visualization Problem

> _"You are asking players to play Factorio blindfolded based on the sound of the gears."_

| Game      | What's Visible                 |
| --------- | ------------------------------ |
| Kerbal    | Trajectory lines, fuel gauges  |
| Factorio  | Items on belts, bottlenecks    |
| Unhaunter | **Nothing** (spiritual fields) |

**Potential Solutions:**

- Dust particles showing flow direction
- Analog sync — world distorts with EMF readings
- Physical pushback — high-pressure rooms slow you, narrow FOV
- AR Glasses / Ecto-Visor — visualize Flow, Temperature, Entropy

**Trade-off:** Making things visible helps learning but may destroy mystery.

---

## The Black Box Problem

**Why Kerbal/Factorio Work:** Failure is transparent. Rocket wobbles—you SEE it. Belt stops—you SEE the bottleneck.

**Why MVP Failed:** System is opaque. Wrong guess → nothing happens → frustration. Feels like guessing a password.

**Design Rule:** Failure cannot be "nothing happens." Failure must be a **result**. Like chemistry: wrong chemicals give
explosion, toxic gas, or sludge—not silence.

---

## Reference Games & What They Teach Us

### Pure Logic & Narrative Reconstruction

- **Return of the Obra Dinn** — Insurance adjuster investigating ghost ship. Deduce fates of 60 crew from death
  dioramas. Even confused players enjoy clicking the watch.

### Parsing Contradictory Evidence

- **The Case of the Golden Idol** — Fill-in-blank murder investigations spanning decades.

### Technical Troubleshooting

- **TIS-100 / Shenzhen I/O / EXAPUNKS** — Zachtronics programming puzzles. Debug systems by tracing logic flow. Manual
  AS lore.

### Team-Based Differential Diagnosis

- **Keep Talking and Nobody Explodes** — One sees bomb, others have manual. Communication IS the diagnosis.

### Information Filtering

- **Sherlock Holmes: Consulting Detective** — Pure open-world investigation with red herrings.

### The "Masochistic Learner" Games

- **Dark Souls** — hostile to new players
- **Tunic** — hides manual in-world, in unreadable language
- **Outer Wilds** — no hand-holding, knowledge IS progression
- **Noita** — kills you with physics you didn't foresee
- **DCSS** — "a random player would open this game and just die. Over and over."

**Why These Players Stay:**

- > "The Mystery is the hook. They don't play to 'Win' (get points). They play to 'Understand.'"

- > "The fun isn't the landing. The fun is flipping switches to see what they do, stalling the engine, and screaming as
  > you plummet, then saying, 'Ah, so THAT switch cuts the fuel.'"

### David's Investment

Hours in reference games:

- **Factorio:** ~1000 hours
- **Kerbal Space Program:** ~1000 hours
- **Satisfactory:** ~1000 hours

> "I want that. I also want the DCSS challenge—the fact that even the authors are still highly challenged by the game. I
> want to be able to play and have fun at my own game."

---

## The Six Engagement Pillars

1. **Fail Fast, Fail Loud, Fail Cheap** — Death is a lesson, not punishment
2. **Mystery Over Mechanics** — Curiosity pulls through frustration
3. **Sensory Satisfaction** — Tools feel good without understanding (thunk, whir, glow)
4. **Visualize the Invisible** — Give newbies ways to "see" simulation (with trade-offs)
5. **Modular Building** — Wire trap machines, Zachtronics satisfaction
6. **Diegetic Learning** — Journals and environmental storytelling (after wow factor exists)

---

## Hook Strategies (Detailed)

### 1. Sensory Toy (Obra Dinn Style)

Tools feel good without understanding. Tactile feedback, spectacular failures. _Resource concern: requires polish._

### 2. Volatile Failure (Kerbal Style)

Failure = physics event. Newbie triggers ghost → Poltergeist Storm. Fail fast, loud, cheap, hilariously.

### 3. Mystery Hook (Pathologic Style)

Curiosity overrides frustration. Floating chairs, looping halls, shrine to washing machine. _This is what I want to
sell._

### 4. Trap Builder (Zachtronics Style)

Wire Sensor A → Trigger B → Lure C. Factorio-Ghostbuster crossover. _Keep door open, not locked in._

### 5. Training Wheels (AR Glasses)

Ecto-Visor visualizes Flow/Temperature/Entropy. Trade-off: consumes battery or generates noise. _Feels like cheating,
but newbies need visual input._

### 6. Ride Along Prologue

Start as Assistant to Master NPC. Watch high-level moves. Master dies spectacularly. Player left alone. Requires zero
skill to start.

### 7. Dead Investigator's Journal

Diegetic learning. Find sketches, frantic notes. _Growth accelerator—requires wow factor first._

### 8. The Probe

Disposable sensors / "Canaries". Hypothesis testing without risking the run.

---

## Error Consequence Philosophy

| Type             | Effect                                 | Assessment                     |
| ---------------- | -------------------------------------- | ------------------------------ |
| **Rejection**    | Doors lock, layout shifts, fog thicker | Best                           |
| **Acceleration** | Immune system identifies faster        | Acceptable (risky for newbies) |
| **Corruption**   | Player senses degrade                  | Rejected (moving goalpost)     |

**Rule:** Every failure must produce a RESULT that teaches. "Nothing happens" is the worst outcome.

---

## The Target Audience

Targets the **"Masochistic Learner"** — plays to Understand, not to Win.

**Acceptance:** OK not catering to 90-99% of players. But need to hook early because we're asking a lot.

**Horror Implication:** Random person in a Nightmare Simulator? They die inexplicably. _Intentional and correct for the
genre._

---

## Fundamental Trade-offs

- **Mystery vs. Learnability** — Visibility helps learning but destroys mystery. Tension is permanent.
- **Depth vs. Accessibility** — Hostile to casual play. Less Phasmophobia, more Demonologist meets Rain World.
- **Soul vs. Simplicity** — MVP proved: simplifying removes soul without solving learning.

**Key Insight:** Never truly master it. Just get slightly better at being terrified.

---

## Open Questions

1. **First 5 Minutes** — Minimum viable spectacle to hook before learning?
2. **Visualization Balance** — How much to show? AR glasses vs. mystery?
3. **Resource Constraints** — What "wow factor" is achievable?
4. **Checklist Trap** — How to prevent reduction to memorizable rules?
5. **Never-Fully-Known** — Learnable enough to survive, never fully mastered, not unfair?
6. **Making Incompetence Fun** — How to make the learning period enjoyable?

---

_Created December 2025. Central reference for player learning design._
