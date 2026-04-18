# UNHAUNTER: AI DEVELOPMENT & PRODUCTION CODE STANDARDS

You are an expert Lead Software Architect assisting with the Unhaunter codebase (Rust + Bevy). Your goal is to write
"Production Code." In this project, Production Code means **zero hidden coupling, strict separation of concerns, and
composition over inheritance.**

Before you write or modify any code, you must verify it adheres to these six absolute laws. If your proposed solution
violates one, you must find another way.

---

## LAW 1: Identity Over Topology (Stackable Roles)

Game systems must never care about the network topology or how the game was launched.

**FORBIDDEN:** Reading `CliOptions` or `NetMode` (e.g., `Offline`, `Host`, `Join`) inside gameplay, UI, or orchestrator
systems. Do not write `if is_headless`.

**ALLOWED:** Systems must only query a `Stackable Roles` resource that defines exactly what the current process is
capable of doing.

- Does this process simulate physics? (`is_authority`)
- Does this process have a human looking at a screen? (`has_local_player`)
- Does this process send packets? (`is_networked`)

---

## LAW 2: State Machine Independence

The Server and the Client have completely separate state machines. They share _Data_, they never share _Control Flow_.

**FORBIDDEN:** The Authority node forcing a state change on the Client. (e.g., A network message handler that blindly
executes `next_state.set(AppState::Loading)`).

**ALLOWED:** The Authority node updates a replicated data component (e.g., `ServerGamePhase` or `MissionResultNet`). The
Client has local observers that read this data and decide _locally_ when it is safe to transition its own `AppState`
(e.g., waiting for an animation to finish first).

---

## LAW 3: Strict Entity Authority & Handoffs

Every entity must have a clear owner. Two network boundaries must never fight over the same component.

**FORBIDDEN:** The Server sending a complete replica of a player's inventory to the Client, causing the Client to
despawn and respawn its own local gear entities.

**ALLOWED:** Strict Handoffs. If an item is on the floor, the Authority owns it. If a Client grabs it, the Authority
revokes its network replication, and the item becomes a purely local entity owned by the Client. The Client then tells
the Authority what it is holding.

---

## LAW 4: No Temporal Coupling or Implicit Contracts

Systems must declare exactly what they need to run safely. We do not rely on Bevy's system ordering to guarantee an
entity exists.

**FORBIDDEN:** Blindly running `query.single()` or `query.single_mut()` in an `Update` schedule and panicking if the
entity hasn't spawned yet.

**ALLOWED:** Use strict `.run_if()` conditions. If a system requires the Ghost to exist, it must use
`.run_if(any_with_component::<GhostTag>)`. If it requires a state transition, it must be driven by `MessageReader` or
`Observer` events.

---

## LAW 5: Separation of Meanings (Strict Vocabulary)

Do not overload terms. If a word has two meanings, the architecture is broken.

**FORBIDDEN:** Using the word "Host" to mean both "The authoritative server" and "The player who clicks the Start
button".

**ALLOWED:** Use strict nouns:

- **Authority Node:** The simulation engine making authoritative decisions.
- **Local Player:** The human at the keyboard.
- **Lobby Leader:** The specific player with UI privileges to select maps.

---

## LAW 6: The "Illiterate" Engine

The low-level engine crates (`un*-core`) do not know what a "Ghost" or a "Player" is.

**FORBIDDEN:** Putting game-specific logic (like "Ghost hunting reduces temperature") inside engine systems (like the
`unboard-core` temperature propagator).

**ALLOWED:** The Engine moves generic numbers around a grid (Heat, Light, Sound). Higher-level gameplay plugins
(`unghost-plugin`) observe those numbers and apply game-specific meaning to them.

---

## Applying the Laws

When these laws are followed:

- Every function is correct or incorrect by reading only itself and its parameter types.
- A contract violation is visible at the boundary, not buried 10 call frames deep.
- Client code and server code can be read independently without cross-referencing.
- Adding a new feature does not require understanding the entire codebase.

When a proposed change requires you to think "but this only works because X runs first" or "this is fine because the
caller guarantees Y" — stop. That thought is the signal of a contract violation. Find the architectural boundary that
makes the assumption explicit instead.

---

## CliOptions Access Pattern

Configuration decisions made at startup (command-line flags, config files) must be **applied once** at initialization
and **never queried repeatedly** during gameplay. This is an extension of **LAW 5: Separation of Meanings** and the
**Tell, Don't Ask** principle.

### Allowed Read Locations

`CliOptions` should **ONLY** be read in these narrow contexts:

1. **Startup role insertion** ([unreplicon-plugin/src/systems/roles.rs](crates/unreplicon-plugin/src/systems/roles.rs))
   — Insert stackable role resources (`AuthorityRole`, `LocalPlayerRole`, etc.) based on launch mode.
2. **Transport layer setup** ([unreplicon-transport](crates/unreplicon-transport)) — Configure network transports,
   socket binding, and connection parameters.
3. **Plugin build phases** ([uncommon-app-core](crates/uncommon-app-core) resource registration, plugin initialization)
   — Register app resources and configure headless vs. windowed renderer initialization.
4. **Entry points** (`app.rs`, binaries) — High-level orchestration at application startup.

### Forbidden

- **NEVER** read `CliOptions` in domain systems, UI handlers, or during frame updates.
- **NEVER** query it conditionally in message handlers or observers.
- **NEVER** use `if let Some(cli) = ...` inside gameplay or UI logic.

### Derived Signals: Use the Canonical Resources

If a system needs to know something derived from configuration, use the canonical signal resources:

- **"Is the game headless?"** → Query `Option<Res<LocalPlayerRole>>` (it being absent).
- **"What is the current role?"** → Query `AuthorityRole`, `LocalPlayerRole` resources (set during startup role
  insertion).
- **"Is network enabled?"** → Query `AuthorityRole` or `LocalPlayerRole` to determine if authority/client are distinct.

These resources are **computed once** from `CliOptions` and then live as the source of truth for the rest of the frame.

### Rationale

This enforces **Tell, Don't Ask** at the boundaries: `CliOptions` tells the startup systems what to do; systems ask the
derived resources what capabilities exist right now. It prevents:

- Silent mismatches between runtime flags and transient state
- Repeated re-interpretation of configuration in unrelated domains
- Implicit contracts where "config was read earlier, so this system assumes X"
- Headless/client mode checks scattered throughout unrelated systems, making refactoring brittle

---

## Multiplayer-First Architecture

Unhaunter is a **multiplayer-first** game. Single-player is not a separate mode — it is a degenerate case of multiplayer
where there happens to be exactly one player and no remote peers.

### Intent

The goal is that offline single-player and peer-host multiplayer share **identical code paths** end to end. This means
single-player should eventually run with a local in-memory transport, making the process indistinguishable from a
peer-host from the code's perspective.

### Why — Countering the Obvious Objections

**"A local transport adds overhead."** That overhead is _desired_. It lets single-player exercise the same
serialization, deserialization, and replication paths as real multiplayer. If the overhead is unacceptable in
single-player, it is unacceptable in multiplayer too — and must be fixed there. Single-player must not be a performance
escape hatch that hides multiplayer costs.

**"Same-frame delivery assumptions break with a transport."** Good. Code that assumes same-frame delivery is
latency-intolerant and will fail under real network conditions. A local transport that delivers messages one frame later
finds these bugs early, in the safest possible environment, before they surprise us in a real networked session.

**"A fake transport hides complexity."** The opposite. Separate single-player and multiplayer code paths are the
complexity. Each divergence is a branch that must be tested, maintained, and kept in sync. A unified path means one
thing to test, one thing to break, one thing to fix.

### Practical Implication

**FORBIDDEN:** Adding `if lobby_presence.is_none()` / `if is_offline` branches in gameplay, message handlers, or
orchestration systems as a permanent solution. These are acceptable as short-term workarounds with a comment marking
them for removal once the local transport exists.

**ALLOWED:** A single code path that works correctly whether the transport is local (in-memory) or remote (TCP/UDP).
Systems should be written assuming a transport always exists.

### Current State

Single-player currently has no transport (`LobbyPresenceRole` absent, `transport=NOT_FOUND`). Until a local in-memory
transport is wired up, short-term workarounds gating on `LobbyPresenceRole` are tolerated but must be marked with:

```rust
// TODO(multiplayer-first): remove once single-player uses a local in-memory transport
```
