# Multiplayer Vision Summary & Reflection (Feb 2026)

This document summarizes the core philosophy, technical evolution, and design thoughts regarding Unhaunter's multiplayer
transition from a 1:1 CLI prototype to a multi-client dedicated server architecture.

---

## 1. Core Philosophy: The "Attention" Game

The soul of Unhaunter is a rejection of "arcade" horror. The player's primary skill is **paying attention**.

- **Simulation-First:** Everything (thermal diffusion, light, EMF, ghost emotions) is simulated, not scripted.
- **Ambiguity as Gameplay:** Sensors are logarithmic and directional; the environment provides false positives (e.g.,
  fridges emitting EMF).
- **Multiplayer Identity:** In multiplayer, this transforms the game into a **conversation about evidence**. The "SWAT
  team" fantasy emerges not from twitch reflexes, but from coordinated deployment and shared interpretation of ambiguous
  data.

## 2. The Social Vision: "Place" over "Program"

The goal is for the game to feel like a persistent world/life rather than a software utility.

- **The Lobby as a Physical Space:** The lobby is not a menu; it's a map (the player's "house" or "agency office"). It
  serves as a social staging area and a tutorial where players learn mechanics in safety.
- **The Van as an Ops Center:** The van operator is a high-information role, not a hiding spot. It's the perfect
  onboarding role for new players to learn systems by watching sensor readouts.
- **Asynchronous Presence:** Because the player base is small and spread across timezones, the game needs "bulletin
  boards" and "scheduled sessions" rather than just real-time matchmaking.
- **Fluid Agencies:** "Agencies" are contact lists with shared history, not rigid guilds.

## 3. Technical Evolution (Plans 21-25)

In a rapid burst of development, the following was achieved:

- **N-Client Plumbing:** Moved from 1:1 to 1:N TCP connections.
- **Dynamic Spawning:** Host spawns only itself; remote players spawn on handshake.
- **Lobby Protocol:** Split `Welcome` into `LobbyWelcome`, `LobbyState`, and `StartMission`.
- **UI Polish:** Rebuilt the lobby using `unmenu-core` templates with mouse support and sub-states.
- **Session Layer:** Implemented heartbeats, late-join support, and disconnect resilience (entities "hide" on
  disconnect).

## 4. The Dedicated Server Decision

The decision was made to skip a "dumb relay" (TURN) and go straight to a **Dedicated Server** for several reasons:

- **Abuse Prevention:** A relay can be used as a general proxy; a dedicated server only speaks the game protocol.
- **NAT Traversal:** Solves the CGNAT problem for the host without requiring a player to have a public IP.
- **Fairness:** Centralizes game logic on a neutral VPS.
- **Identity:** Moves toward a keypair-based identity system (Public Key = ID) to allow banning/reputation without
  requiring traditional accounts.

## 5. Current Status & Tensions

The dedicated server is now built and functional, but there is a lingering dissatisfaction.

- **Compute Concerns:** The simulation (thermal, light, AI) might be heavier than expected for a headless instance.
- **Authority Mess:** Networking logic is currently "messy" regarding who controls what and how conflicts are resolved.
- **The "Feeling":** Despite the technical success, the transition to a dedicated server has introduced new friction or
  lost some of the "organic" feel of the original host/client model.

---

_Document compiled on Feb 14, 2026, following the completion of the initial multiplayer roadmap._
