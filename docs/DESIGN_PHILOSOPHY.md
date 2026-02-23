# Unhaunter Design Philosophy: The Core Pillars

**Date:** February 23, 2026

This document outlines the core psychological, architectural, and operational principles that define Unhaunter. It
serves as a guide for developers to ensure that new features align with the game's vision and avoid common pitfalls
identified during development.

---

## 1. UX & Psychology: Curing the "PagerDuty Simulator"

### The Symptom: Stress-Induced Speedrunning

Early playtests revealed that while Unhaunter was designed as a slow-burn "liminal horror" game, players were
experiencing extreme anxiety and speedrunning through maps. They weren't terrified of the entity; they were experiencing
the same psychological pressure as a sysadmin during a server outage. We inadvertently built a **PagerDuty Simulator**.

### The Cause: Cognitive Overload

- **Prefrontal Cortex vs. Amygdala:** The game asks for deep logical deduction (reading thermal dynamics, Geiger clicks)
  inside a hostile environment. Survival pressure destroys the ability to reason.
- **The Information Vacuum:** Without localized audio cues (creaks, footsteps), the house felt dead, forcing players to
  assume the threat was always at 10/10.
- **Synthetic Noise:** Tool sounds (beeping EMFs) and constant walkie-talkie bombardment drowned out the liminal
  atmosphere, sounding like ICU monitors or bomb timers.

### The Pillar: Fragile Competence

To fix this, Unhaunter must adhere to the principle of "Fragile Competence":

- **Gear 3 (The Danger Gauge):** Provide continuous, low-level sensory feedback about the entity's presence so players
  can gauge threat and relax enough to think.
- **Silence as a Tool:** Minimize synthetic gadget noise. Prioritize environmental "Liminal Audio" to build tension
  through anticipation, not jump scares.
- **Leniency through Transparency:** Ensure that "warnings" (roars, screen tints) don't feel like punishment, but like
  data points the player can use to make informed tactical retreats.

---

## 2. Networking: The "Zero-Ops" & Friction Architecture

### The Universe & The Domain

To achieve frictionless matchmaking without centralizing liability, we use a federated "Universe" directory.

- **Domains** (Hub + Dedicated Servers) are operated by trusted entities.
- **Area Codes** (e.g., `U-XXXX`) inform the client of who owns the server, shifting liability to independent operators
  and fulfilling GDPR consent without intrusive popups.

### Statelessness & Zero-Ops

The Hub and Dedicated Servers are designed to be **RAM-only**. No player session history or IP logs are persisted to
databases. This eliminates migration dread; moving a server is a 45-second script, not a three-day database migration.

### The SFU Imperative

Proximity voice chat is a core gameplay loop. To protect player privacy and prevent IP-based DDoS attacks common in P2P
setups, all audio is routed through a **Selective Forwarding Unit (SFU)** on the server. This masks player IPs and
ensures legal compliance (GDPR Data Minimization) at the cost of slight latency.

---

## 3. Security & Abuse Mitigation: Friction as Defense

In a FOSS game, bans are useless. We use **The Friction Architecture**: taxing a troll's time instead of their money.

- **Cryptographic Identity:** Players are identified by local Public/Private Keypairs. No accounts, no emails, minimal
  GDPR footprint.
- **The Reputation Economy:** New keys must earn Reputation in **The Mixer** (a server-controlled quick-play queue)
  before they can host public lobbies.
- **Shadowbanning (The Hell Lobby):** Flagged trolls are silently routed to low-resource "Hell Mode" servers with 5Hz
  tick rates and physics degradation, quarantining them with other trolls without alerting them to generate a new key.
- **Network Asymmetry:** Every server reservation requires a stateful TCP handshake and a 10-second fast-expiry clock to
  prevent lightweight REST-based DoS attacks from allocating heavy Bevy RAM slots.

---

## 4. Multiplayer Authority: Distributed Ownership

Unhaunter follows a **Distributed Authority** model enabled by `bevy_replicon`:

- **Client Authority:** Players serve their own movement and inventory state. Cycling inventory (`[Q]`) or activating
  gear (`[R]`) must be instantaneous. We use Replicon's `ClientVisibility` to hide owned items from the server's
  replication back to the owner, preventing "state bouncing."
- **Server Authority:** The environment (doors, switches, ghosts) and items on the floor are server-authoritative.
  Interactions (`[E]`, `[F]`, `[G]`) require server confirmation.
- **Phasing Acceptance:** We prioritize local movement UX over global consistency. If a client walks through a door that
  just closed on the server, the server accepts the position. The player "ghosts" through the door on other screens,
  ensuring the local player never feels rubber-banding.

---

## 5. Development Workflow: Strict Modularity

To minimize compile times and separation of concerns:

- **`un*-core`**: Zero game logic, zero systems, zero plugins. Pure data contracts.
- **`un*-plugin`**: Encapsulated gameplay mechanics. The only export is the `Plugin`.
- **Replication Registry**: Add new networked features in 5 minutes by adding `app.replicate::<T>()` to the domain
  plugin, rather than modifying central networking code.

---

## 6. Event-Driven Messaging: The `add_message` Pattern

To decouple disparate systems, Unhaunter uses a unified event-driven communication layer.

- **Explicit Registration:** Systems must register their events using `app.add_message::<T>()`. This acts as a clear
  data contract between systems.
- **The `#[derive(Message)]` Convention:** Any struct intended for cross-system communication should use this derive (or
  implement the corresponding marker trait). This ensures that messages are compatible with both local systems and
  potential future network replication.
- **Broadcast by Default:** When a system writes a message that should be heard by other clients in multiplayer (e.g., a
  `SoundEvent` at a location), the message struct includes a `broadcast: bool` field. The networking layer observes this
  and routes the message accordingly, maintaining the "Thin Server" principle where the server only relays what is
  necessary.
