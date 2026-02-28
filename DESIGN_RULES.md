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
