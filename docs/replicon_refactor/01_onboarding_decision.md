# 01. The Decision to Adopt `bevy_replicon`

**Date:** February 22, 2026

This document captures the architectural journey, the friction points, and the ultimate decision to migrate Unhaunter's
custom multiplayer netcode to `bevy_replicon`. It serves as a historical record of _why_ we are undertaking this massive
refactor, the hard blockers we identified, and the specific mitigations we designed before writing a single line of
code.

---

## 1. The Catalyst: The "God Crate" Anti-Pattern

The initial investigation began with a feeling that the existing multiplayer architecture (`unnet-core` and
`unnet-plugin`) was accumulating tech debt. While the hybrid client/server-authoritative model was functionally correct
for a PvE co-op game, the _implementation_ had become a bottleneck.

### The Problem: Manual Mapping and Boilerplate

The `unnet` crates had evolved into a "God Crate." To synchronize a new entity (e.g., a `Thermometer`), a developer had
to:

1. Modify `unnet-core` to add the data to the monolithic `SnapshotMsg`.
2. Modify `unnet-plugin`'s `host_sync.rs` to extract the data from the ECS and pack it.
3. Modify `unnet-plugin`'s `client_sync.rs` to unpack the data and manually insert Bevy components.

This violated domain coupling. Network logic for a component lived far away from the component itself. It created a
fragile, boilerplate-heavy environment where forgetting a single mapping step resulted in silent multiplayer bugs (such
as movement jitter or missing audio broadcasts for remote players).

### The Goal: Streamlined Domain Coupling

We needed a streamlined process where the network code for a component lives next to the component itself. We wanted to
declare "this component should be synced" and let the underlying system handle the serialization, entity mapping
(removing the need for manual `NetworkId` lookups), and transport.

---

## 2. Evaluating `bevy_replicon`: Poking Holes and Hard Blockers

`bevy_replicon` emerged as the industry-standard solution for Bevy networking, utilizing ECS reflection to automatically
sync components. However, adopting a framework requires surrendering some control. As a reliability-focused project with
strict uptime expectations, we aggressively poked holes in the framework to ensure it could support Unhaunter's specific
requirements.

### Blocker A: The "Server-Authoritative" Paradigm vs. Local Prediction

**The Concern:** Unhaunter relies on distributed authority. A player must be able to move and cycle their inventory
(`[Q]`, `[R]`, `[Q]`, `[R]`, `[TAB]`) in quick succession to do a "full turn on" in 1 second. If `bevy_replicon` is
strictly server-authoritative, would local actions bounce or rubberband while waiting for server confirmation? If the
roundtrip with the server starts messing with the player's local state, it's a massive regression.

**The Resolution:** `bevy_replicon` supports Client Authority through a specific pattern: **Client Prediction + Server
Exclusion**.

- When a player interacts with their own gear, the local ECS updates instantly. Zero ping.
- Crucially, we use Replicon's `ClientVisibility` API to tell the server: _"Replicate this entity to everyone EXCEPT the
  client who owns it."_
- This guarantees the server will never overwrite the client's local, zero-latency prediction with older, latent data.
  The entity will not flip back and forth states continuously.

### Blocker B: Idempotency and "Six Nines" Reliability

**The Concern:** If we use "Client Messages" to tell the server what we did, sending a message like "I pressed R" (or
"Toggle") is time-sensitive and fragile. If there's a desync or a dropped packet, the same action on the client could
execute a slightly different action on the server. We need robust, reliable state updates.

**The Resolution:** We will not send "X happened" messages. We will send **Idempotent State Payloads**.

- Instead of sending `ToggleGearMessage`, the client sends `SetGearStateMessage { entity: Entity, is_on: true }`.
- If the network duplicates the packet, or it arrives late, setting `is_on: true` when it's already `true` does nothing.
  No desync. The server receives this, validates ownership, and applies it to the server's ECS.

### Blocker C: The "Thin Server" and Partial Synchronization

**The Concern:** Frameworks often encourage "syncing everything." We explicitly do _not_ want the server computing local
simulation data (like temperature maps or EMF fields) just to sync them to clients. The server must remain thin.

**The Resolution:** Replicon replicates _Components_, not Entities. This forces a healthy architectural split.

- We will split "God Components" into networked and local variants.
- Example: `GearOperativeState { is_on: bool }` is replicated. `ThermometerReading { current_temp: f32 }` is strictly
  local.
- The server remains thin, only routing the `is_on` boolean. The clients observe that boolean and compute the actual
  temperature reading locally based on their own simulation.

### Blocker D: Transport Lock-in and Future WASM Support

**The Concern:** Adopting a framework often locks you into a specific transport protocol (like raw UDP). While WASM
multiplayer isn't a priority today, it will be in 6 months. Does Replicon force a transport that breaks WASM?

**The Resolution:** `bevy_replicon` is strictly transport-agnostic. It defines a backend trait. We can use `bevy_renet`
(which supports native UDP and WASM **WebTransport**) or `bevy_matchbox` (which provides **WebRTC** built specifically
for WASM-to-Native cross-play). This actually accelerates our path to WASM compatibility without having to roll our own
signaling server.

### Blocker E: The Infrastructure Conflict (TCP Proxy vs. UDP)

**The Concern:** Unhaunter's custom infrastructure (`unprocman`) was designed to act as an in-path TCP proxy to enforce
a Proof-of-Work handshake _before_ allocating Bevy server RAM, protecting against Layer 7 DoS attacks. Standard Replicon
transports (`bevy_renet` UDP, `bevy_matchbox` WebRTC) bypass this TCP proxy entirely.

**The Resolution:** We chose to adopt standard transports (Option A) to benefit from out-of-the-box WASM support and
community maintenance. To preserve the security model without the TCP proxy, we are shifting to a **Ticket-Based
Authentication** model. The Hub REST API will issue cryptographically signed tickets after a successful PoW. The Bevy
server will validate these tickets on the UDP connection, instantly dropping unauthorized packets before they reach the
ECS or allocate game state.

### Blocker F: The "Slideshow" Problem (Tick Rate vs. Framerate)

**The Concern:** Replicon runs on its own internal tick rate (e.g., 20 or 30 ticks per second). If the game runs at 60Hz
or 144Hz, will remote players look like a slideshow?

**The Resolution:** Yes, they will, unless we interpolate. Replicon only updates the `NetworkPosition` component at the
tick rate. We will need to implement a standard interpolation pattern (e.g., replicating a `NetworkPosition` and
smoothly moving the visual `Position` towards it every frame) to ensure smooth movement for remote players.

---

## 3. Accepted Realities and Known Trade-offs

In choosing `bevy_replicon`, we are explicitly accepting the following realities:

1. **The "Late Join" Race Condition & Network Map Loading:** Replicon blasts all replicated entities to a client the
   moment they connect. Because Unhaunter relies on `.tmx` map loading, a joining client could receive entities before
   the map exists.
   - _Mitigation:_ We will build a "Joining State" UI where no systems run and nothing is in view. We will use global
     visibility masks to hide all entities from a joining client until they are ready.
   - _Pivot:_ To simplify this, we are finally pivoting to allow the server to send the map data over the network,
     rather than relying on clients to load it from disk (a design previously dismissed by AI agents, but perfectly fine
     to implement now).
2. **No Built-in Rollback:** Replicon is a state-replication framework, not a rollback engine. If a ghost catches a
   player on the server due to latency, the player will rubberband into the ghost's hands. For a PvE horror game, this
   "ghost gains powers on multiplayer woooooo" effect is acceptable. We will test and adjust.
3. **Bevy Version Lock-in:** We are tying our upgrade path to the `bevy_replicon` maintainers. We cannot upgrade Bevy
   versions until Replicon does. We are biting the apple and accepting this dependency risk.
4. **The Infrastructure Pivot:** We are abandoning the `unprocman` TCP proxy model in favor of a Ticket-Based
   Authentication model over UDP/WebRTC. This requires rewriting the Hub's security handshake but allows us to leverage
   standard, maintained transport libraries.

---

## 4. The Verdict

The current custom architecture, while functional, is too fragile and boilerplate-heavy to scale. `bevy_replicon` solves
the "God Crate" problem by allowing us to define replication rules directly inside the domain plugins (`unghost`,
`ungear`, etc.).

While it forces us to adopt strict ECS component splitting, manage visibility masks for local prediction, and write
interpolation systems, these constraints actually enforce a cleaner, more robust architecture that aligns with our
reliability goals.

We are proceeding with the migration.
