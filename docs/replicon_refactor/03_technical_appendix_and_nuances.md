# 03. Technical Appendix, Nuances, and Rejected Alternatives

**Date:** February 23, 2026

This document serves as a technical appendix to the `bevy_replicon` migration, capturing low-level nuances, historical
tech debt discovered during the research phase, and alternative architectural patterns that were considered and
rejected. The goal is to achieve "six nines" of reliability by design through explicit authority management and
idempotent state updates.

---

## 1. Authority Boundaries and Phase Transition

The success of the "Local Prediction" model depends on a rigid definition of when an entity transitions from
Server-Authoritative to Client-Authoritative.

- **The Floor (Server-Auth):** Items in the world, on the floor, or in the van are strictly owned by the server. No
  client prediction is performed for grabbing or moving these items. This prevents "ghost items" where two players think
  they grabbed the same object.
- **The Transition (E/F/G):** Interactions like Interaction ([E]), Grab ([F]), and Drop ([G]) are server-validated.
- **The Hand (Client-Auth):** Once the server confirms a grab, the entity's authority is handed off to the client. This
  is implemented via `ClientVisibility::set_visibility(Hidden)` for the owner.
- **Handling Rejection:** Since grabbing is not predicted, there is no "ghost item" flicker for the initial grab.
  However, if a client attempts an action that the server rejects (e.g., due to a race condition), the client's local
  state must be idempotent and programmed to be overwritten by the next authoritative server update without crashing or
  desyncing.
- **Force Overrides:** While the owner has authority, the server can still "force" state changes (e.g., an EMP ghost
  disabling a flashlight) by sending a targeted RPC event (e.g., `ForceGearStateEvent`) which the client is programmed
  to respect.

---

## 2. Phasing Acceptance (Movement vs. Environment)

A deliberate design choice was made to prioritize **Local UX over Absolute State Consistency**.

- **Scenario:** A player walks through a door that is closed on the server but was seen as open on the client due to
  latency.
- **Decision:** The server will accept the client's position as authoritative for their own feet.
- **Result:** The player will "ghost" or "phase" through the closed door on the server's view and other clients' views.
  This prevents jarring rubber-banding and ensures movement always feels tight and responsive for the local player.

---

## 3. Distributed Item Trading

Trading items between players requires a surgical visibility flip to prevent one-frame flickering.

- When Player A gives an item to Player B:
  1. The server must set visibility to `Visible` for A (so they see the official hand-off).
  2. The server must simultaneously set visibility to `Hidden` for B (so they take over local prediction immediately).
- If these are not synced in the same network tick, the item may briefly teleport to the world origin or flicker out of
  existence.

---

## 4. Audio Prediction and the `exclude_player` Requirement

To achieve zero-latency feedback for actions like clicking a flashlight or toggling a switch, sounds are often played
locally by the client immediately.

- **The Problem:** If the server receives the state change and broadcasts a `SoundEvent` to all clients, the original
  player will hear the sound twice (once locally, once from the server).
- **The Requirement:** The `SoundEvent` system must be expanded to include an optional
  `exclude_player: Option<ClientId>` field. The server will use this field to skip the original actor when broadcasting
  environmental sounds that were already predicted locally.

---

## 5. Extensibility: Voice and Chat

A key requirement for Unhaunter is the future support for integrated voice and text chat.

- **Non-ECS Data:** While `bevy_replicon` handles component replication, the underlying transport backends (like
  `bevy_renet`) support multiple independent channels.
- **Architectural Fit:** We will open dedicated unreliable channels for voice data and reliable channels for chat
  messages that bypass the ECS replication logic entirely. This ensures that high-bandwidth voice data doesn't interfere
  with critical game state updates.

---

## 6. Rejected Alternatives

Before settling on `bevy_replicon`, two other conceptual paths were evaluated:

### Concept 1: The "Dumb Pipe" (Domain-Specific Messages)

- **Idea:** `unnet` becomes a generic transport layer for binary blobs. Each plugin (e.g., `unghost`) writes its own
  serialization logic and calls `network.broadcast()`.
- **Rejection Reason:** While it solves decoupling, it requires manual serialization boilerplate for every single
  feature, leading to the same "Human Compiler" errors we wanted to escape.

### Concept 2: The Payload Builder (Generic Atomic Snapshot)

- **Idea:** Keep the single `SnapshotMsg` but make its contents a `HashMap<String, Vec<u8>>`. Plugins "register" their
  own payloads into the map.
- **Rejection Reason:** This is a middle ground that still requires manual entity-to-network-ID mapping and manual
  buffer management. It lacks the ergonomic "just add a component" flow of Replicon.

---

## 6. Historical Tech Debt: The "Double-Apply" Jitter

During the investigation, a critical bug was identified in the legacy `unnet` system's movement logic:

- **The Bug:** On the host, the `player_movement_system` was calculating movement for _all_ players (local and remote).
  However, the host was _also_ receiving the exact position from clients and overwriting the ECS position.
- **The Result:** The host was applying movement twice for remote players—snapping them to the client's position and
  then adding the client's input movement on top again—causing a constant forward jitter.
- **Lesson Learned:** Any movement system in the new architecture must strictly distinguish between "Input-driven"
  movement (Local Player) and "Telemetry-driven" movement (Remote Players).
