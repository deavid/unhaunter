# Core Mechanics & Vision: Interfacing with the Paranormal

**Date:** February 21, 2026

**Authors:** David Martínez Martí & Gemini 3.1 Pro

> _"You are asking players to play Factorio blindfolded based on the sound of the gears."_

## 1. Introduction: Bridging Simulation and Experience

Unhaunter is powered by a majestic, complex backend. We are simulating fluid dynamics, thermal fields, radiation, and
audio decay. However, a complex simulation is meaningless if the player cannot properly interface with it.

Historically, our mechanics suffered from a disconnect between what the engine was doing and what the player perceived.
This document outlines the core mechanical pillars and physical rules designed to ground the player in the simulation,
enforce the correct pacing, and establish the unique, blue-collar "bureaucratic horror" flavor of the game.

## 2. The 2D Vision Constraint (Curing Omniscience)

There is a persistent temptation to transition the game to a 3D First-Person perspective to force players to slow down
and experience the horror. However, maintaining the 2D Isometric engine is a deliberate, highly beneficial architectural
choice—provided we fix its greatest flaw: **Omniscience**.

A traditional isometric camera floats near the ceiling. It allows the player to see over walls, behind their character's
back, and into adjacent hallways. When a player has perfect situational awareness, their brain optimizes for speed and
efficiency. They treat the game like an action RPG.

**The Solution:** We do not need 3D to slow players down; we need to blind them. By implementing a strict, oppressive
**Cone of Vision**, we weaponize the human brain's self-preservation instinct. The area behind the player's character
must be shrouded in darkness or desaturated obscurity. If a player hears a floorboard creak behind them, they must be
forced to physically turn their character around to see the threat. When running blindly means running into the
terrifying unknown, players will inherently stop holding down the `W` key.

## 3. The CRT Van Monitor (Solving the "Batman" Problem)

Because our simulation is invisible (spiritual pressure, temperature gradients), players previously felt like they were
guessing passwords in a black box. The initial idea to fix this was "AR Glasses" that visualized these fields.

However, AR Glasses introduce the "Batman Detective Mode" trap: if players have a tool that highlights all the answers
in neon wireframes, they will never take it off. The analog, liminal horror of the house is destroyed. Furthermore, AR
tech pushes the game into an unwanted Sci-Fi aesthetic.

**The Solution:** We take the visualization system out of the player's hands and bolt it to the inside of the Van as a
chunky, 90s-style **CRT Topological Monitor**.

- **Deployed Data Only:** The CRT Monitor is not magic; it only visualizes data where players have physically dropped
  sensors inside the house. If a thermometer is dropped in the kitchen, the CRT draws a localized 5-meter thermal
  gradient around that specific sensor.
- **The Ops Center Role:** In multiplayer, this creates a dedicated, high-information role. The "Van Operator" stares at
  the glowing fluid dynamics on the CRT, screaming over the walkie-talkie: _"The thermal gradient just shifted into the
  hallway, get out of there!"_ Meanwhile, the field agents remain in the terrifying, analog dark.
- **The Solo Loop:** For solo players, it enforces perfect pacing. The player must run into the dark house _blind_,
  deploy their sensors, and retreat to the safety of the Van to analyze the topographical flow. They deduce the entity's
  location, then leave the safety of the screen to execute their plan.

## 4. The Mixer: "Streamer Chaos Mode"

Matchmaking in small, indie multiplayer games often suffers from the "5-Player Problem" (empty lobbies and dead server
browsers). The Mixer was originally conceived as a chaotic way to group random players together, but its true identity
is much more valuable.

The Mixer is **Streamer Chaos Mode**. It is a server-controlled, fast-paced queue where players do not choose the map or
difficulty. For content creators on Twitch or YouTube, managing Discord roles, hiding lobby codes, and manually kicking
players is a massive hassle. The Mixer allows a streamer to simply tell their chat, _"Queue up for the Mixer,"_ and the
server handles rotating the audience in and out of their lobbies mission by mission.

It acts as an unfarmable "Proof of Work" proving ground for new players, an anti-troll mechanism (via vote-kicking and
server authority), and a zero-friction community growth engine.

## 5. Bureaucratic Horror: The Insurance Deductible

Progression in centralized games (like _Phasmophobia_) relies on a secure database tracking XP and money. Because
Unhaunter uses a federated, FOSS architecture where progression is strictly client-side, we must approach economic
tension differently.

Death in Unhaunter shouldn't just mean losing a flashlight. It means navigating **Insurance Deductible Horror**.

- If a player passes out or gets attacked, they face hospital bills, ambulance fees, therapy costs for sanity recovery,
  and the fee for the extraction team to recover the Van.
- If a team fails catastrophically, their insurance premiums and required "security deposits" for future missions
  increase.

This perfectly captures the vibe of broke, gig-economy ghost hunters. The tension shifts from a generic _"Don't die"_ to
a highly relatable _"We literally cannot afford the deductible for another hospital visit; we have to abort this mission
right now and eat the gas costs."_

## 6. Speculative Horizons (The "Rent-Free" Ambitions)

As the game matures, our architecture allows for incredibly ambitious mechanics that push the boundaries of the genre.
These are our long-term north stars:

- **Gradient Descent Entity AI:** Instead of a simple behavioral state machine (`if sanity < 50% -> hunt`), the entity's
  mood is modeled as a rolling point on a 30-dimensional topographical map (axes of Mischief, Solemnity, Aggression,
  etc.). An entity can get "stuck" in an emotional local minimum (e.g., endlessly organizing salt piles), requiring the
  players to actively perform actions to "nudge" it out of its behavioral valley to get a reaction.
- **Non-Euclidean 4D Isometric Geometry:** Because the game relies on a tile-based grid graph rather than static 3D
  meshes, the Entity can dynamically rewrite the topological routing at runtime. Walking through the kitchen door might
  suddenly route the player to the basement, or the entire grid might silently rotate 90 degrees—meaning the map feels
  completely wrong but is mathematically identical. This induces true spatial madness without requiring complex 3D
  rendering tricks.
