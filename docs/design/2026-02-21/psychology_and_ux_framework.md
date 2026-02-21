# The Psychology & UX Framework: Curing the "On-Call" Simulator

**Date:** February 21, 2026

**Authors:** David Martínez Martí & Gemini 3.1 Pro

> _"You can't sprint up a mountain and then complain it's slippery."_

## 1. Introduction: The Symptom vs. The Disease

Unhaunter was designed to be an intricate, systemic paranormal simulator. The ultimate design goal was "fragile
competence"—a slow-burn investigation where players use science and deduction to understand a hostile environment. It
was meant to be a chill, atmospheric experience where players relax, observe, and piece together a puzzle.

Then the playtests happened.

Players were sweating. They were grinding their teeth. They were speedrunning through the maps, desperately juggling
items, looking like they were trying to disarm a bomb with two seconds left on the clock. But when asked if they were
scared?

> _"Eh? No, no, not at all."_

They weren't terrified of the entity. They were experiencing pure, unadulterated anxiety. This led to a harsh but vital
architectural realization: Unhaunter was failing to be a liminal horror game because it was inadvertently triggering the
exact psychological profile of an IT incident response. Like a veteran sysadmin having flashbacks to a breaking server
migration, we realized we had accidentally built a **PagerDuty Simulator**.

This document is the collaborative root-cause analysis of _why_ players feel that way, and the UX rules required to fix
it.

## 2. The Cognitive Conflict (Prefrontal Cortex vs. Amygdala)

Unhaunter demands "abductive reasoning." The game asks players to read fluctuating thermal dynamics, triangulate Geiger
counter clicks, and diagnose a paranormal entity based on noisy data. That requires deep logic and focus (the prefrontal
cortex).

But the game asks them to do this inside a haunted house that they believe wants to kill them (the amygdala).

Here is the biological truth of human behavior: **Pressure destroys the ability to reason.**

When humans are put under survival pressure, complex logical reasoning degrades. When a new player enters the game,
their baseline stress goes up. If the game then throws complex data at them while applying perceived survival pressure,
their brain panics. They can't do the math. So they revert to the lizard brain: they run, they spam items, they
speedrun.

**They aren't speedrunning because they want to win fast; they are speedrunning to escape the cognitive pressure.**

## 3. The Information Vacuum & The Missing "Danger Gauge"

If the game isn't meant to be played fast, why do players feel like there's a ticking clock? Because Unhaunter currently
lacks a "Gear 3."

- **Gear 1 (Boring):** Nothing is happening.
- **Gear 5 (Panic):** The screen is red, the ghost is roaring.

There is no middle ground where the player feels _creeped out but safe enough to think_. In a good horror game, tension
comes from anticipation—knowing exactly how close the monster is (e.g., the motion tracker in _Alien_).

Because of the 2D isometric view and the lack of prominent, localized environmental audio (footsteps, creaks), the house
feels dead. The entity might be clicking switches and opening doors, but the player is often deaf to it. Because players
have no continuous "Danger Gauge" telling them the entity is at a low anger level, they are forced to assume it is
_always_ at a 9.9/10. They are trapped in an Information Vacuum, waiting for a jump scare. You cannot relax in a vacuum.

## 4. The UI of Threat (The Leniency Paradox)

The original design intent was to avoid the instant-death mechanics of other games. To be lenient, the ghost was
programmed to roar and turn the screen red _way_ before it actually attacks. The assumption was that players would
realize they had plenty of time to casually walk out.

Instead, it gave them heart attacks.

The lizard brain doesn't read source code; it reads sensory input. A red screen is the universal video game language for
_"YOU ARE TAKING DAMAGE AND DYING right now."_ A roaring monster means _"The predator has caught you."_

To the developer, it was a polite warning. To the player, **the warning itself was the punishment.** The game didn't
become more lenient; it just stretched out the execution.

## 5. Sensory Overload: The 3 Pagers & The Walkie-Talkie

Liminal horror requires silence. The eerie feeling of an empty hallway at 2 AM relies on the sound of wind, settling
floorboards, and your own breathing.

Right now, a player can hold three tools at once. When an EMF reader activates, it sounds like: _Pi... pi... pipipi...
PI PI PI PI._ What else makes that high-frequency, synthetic, rhythmic beeping? An ICU heart monitor crashing. A bomb
timer hitting zero. A server outage pager.

Compounding this is the **Walkie-Talkie Bombardment**. While the walkie-talkie is a brilliant onboarding tool, new
players tend to trigger _all_ the introductory messages in rapid succession.

By filling the audio spectrum with synthetic gadget noise and a barking radio, the liminal atmosphere is completely
drowned out. The player stops listening to the environment and starts listening to the tools. They suffer from choice
paralysis and FOMO (_"Am I holding the wrong tool? Did I miss the evidence?"_), creating a high-APM nightmare that feels
like a noisy office cubicle, not a ghost hunt.

## 6. The Epistemology of Tools (The False Negative Trap)

The design goal of Unhaunter is _Reality²_. Tools should give false negatives. An EMF reader should spike because
someone left the microwave on.

But gamers are conditioned by 30 years of binary game design. To them, EMF = 0 means "The ghost has no EMF." When they
later find out the ghost _did_ have EMF, they don't think, _"Ah, I should have tested the room longer."_ They think,
_"The tool is broken. The game lied to me."_

This breaks player trust. If a player is in an Information Vacuum (the house is silent) and their only life raft (the
tool) lies to them, they feel cheated.

We have to earn the right to lie to the player. The house must "speak" to them first. If they can feel the temperature
drop and hear the floorboards creak, but the EMF reader is silent, they can logically deduce: _"Okay, the ghost is here,
but it doesn't emit a magnetic field."_

## 7. Architectural Directives (The New UX Rules)

To cure the On-Call Simulator, future UI/UX development must adhere to these rules:

1. **Kill the Pagers:** Tools must be silent by default. They should only emit sound when actively returning positive,
   undeniable evidence. Do not drown the liminal atmosphere in synthetic beeps.
2. **Pace the Walkie-Talkie:** Voice lines must be rigorously paced to avoid bombarding the player, especially in the
   early game. Silence must be prioritized.
3. **The House Must Speak First:** Escalation of threat (Gear 3) must be communicated through the environment _before_
   UI changes occur. We need visual distortions (shadows warping outside the vision cone, frost on the screen) and
   localized audio (creaks, footsteps) to serve as an organic Danger Gauge.
4. **Warnings are not Executions:** Never use the sensory language of death (red screens, loud roars) for a warning.
   Save the red screen for actual, lethal failure.
5. **Blind the Player (The Cone of Vision):** 2D Isometric views grant omniscience, which encourages speedrunning. We
   must implement a strict visibility function (Cone of Vision) so the player is blind to what is behind them. The human
   brain naturally slows down when walking into the dark.
6. **Punish the Speedrun (The Mechanical Inverse):** Currently, doing the same tasks faster triggers the ghost _less_.
   This accidentally rewards the speedrunning panic. We must invert this logic: mechanical actions done rapidly and
   loudly must generate _more_ entity agitation than actions done slowly and methodically. Give breathing room to slow
   play.
7. **Identify the Real Phantom Timers:** During playtesting, players were found to be completely ignorant of the Sanity
   drain because the UI hides it well. Therefore, the feeling of a "ticking clock" isn't coming from the Sanity
   mechanic—it is coming entirely from the Information Vacuum and the frantic audio design. Do not over-fix mechanics
   that players aren't actually perceiving as the threat.
