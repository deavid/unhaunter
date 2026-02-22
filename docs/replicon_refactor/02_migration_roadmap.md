# Migration Roadmap: Custom Networking to Bevy Replicon

- **Date:** 2026-02-22
- **Status:** Draft / Roadmap
- **Depends on:** `01_onboarding_decision.md`, `docs/hub/09_full_security_solution.md`

---

## 1. Executive Summary

This document outlines the phased migration of Unhaunter's custom `unnet-*` multiplayer architecture to `bevy_replicon`.

The primary goal is to eliminate the "God Crate" anti-pattern, allowing networked components to be defined alongside
their game logic, while preserving the project's strict requirements for:

1. **Zero-Latency Local Prediction:** Clients must see their actions immediately (e.g., movement, gear toggles).
2. **Thin Server Architecture:** The server should not run heavy simulation logic (e.g., temperature calculations)
   unless necessary for authority.
3. **Robust Security:** The infrastructure must defend against Layer 7 DoS attacks targeting the RAM-constrained
   dedicated servers.

### The Infrastructure Pivot (Option A)

We have chosen to adopt standard `bevy_replicon` transports (`bevy_renet` for UDP, `bevy_matchbox` for WebRTC/WASM).
This requires a fundamental shift in the Hub security model:

- **Old Model:** `unprocman` acted as an in-path TCP proxy, enforcing a Proof-of-Work (PoW) handshake before allocating
  Bevy server RAM.
- **New Model:** We will use a **Ticket-Based Authentication** system. The Hub issues cryptographically signed tickets
  (after PoW). The Bevy server validates these tickets on the UDP connection. Unauthorized packets are dropped instantly
  by the transport layer before reaching the ECS.

---

## 2. Phase 1: Infrastructure & Transport Foundation

Before touching game logic, we must establish the new transport layer and secure the dedicated server lifecycle.

### 1.1. Introduce `bevy_replicon` and `bevy_renet`

- Add `bevy_replicon` and `bevy_replicon_renet` to the workspace.
- Create a new `unreplicon-plugin` to encapsulate the Replicon setup and transport configuration.
- Configure `bevy_renet` to listen on UDP ports assigned by `unprocman`.

### 1.2. Implement Ticket-Based Authentication

- **Hub Side:** Modify the Hub REST API to issue a signed JWT (or similar cryptographic ticket) upon successful PoW and
  room allocation.
- **Client Side:** Update the client to request this ticket and pass it to the `bevy_renet` connection configuration.
- **Server Side:** Implement a Renet authenticator that validates the ticket signature. Connections without a valid
  ticket are rejected at the transport layer, protecting the ECS from unauthorized load.

### 1.3. Adapt `unprocman` Lifecycle

- `unprocman` will no longer act as a TCP proxy. It will revert to a pure process manager.
- Implement the "Fast Expiry" rule: If a dedicated server does not receive a valid, ticketed connection within 5 seconds
  of allocation, `unprocman` kills it and reclaims the port.

---

## 3. Phase 2: Core State & Map Loading

With the secure transport in place, we migrate the foundational game state.

### 2.1. Replicate AppState and GameState

- Use `bevy_replicon`'s state replication features to synchronize `AppState` (Lobby, Loading, InGame) and `GameState`
  (None, Normal, Hunting) from the server to clients.
- Ensure clients transition states based on server authority.

### 2.2. Migrate Network Map Loading

- The server remains authoritative over the selected map.
- When the server transitions to `AppState::Loading`, it replicates a `SelectedMap` component.
- Clients observe this component, load the corresponding `.tmx` file locally, and spawn the static map geometry.
- **Crucial:** The server does _not_ replicate the thousands of static map tiles. It only replicates the map identifier.

---

## 4. Phase 3: Player Entities & Prediction

This is the most complex phase, requiring the "Client Prediction + Server Exclusion" pattern.

### 3.1. Player Spawning & Ownership

- The server spawns player entities and assigns ownership using `bevy_replicon`'s `ClientId`.
- Replicate core player components: `Position`, `PlayerName`, `PlayerColor`.

### 3.2. Implement Client Prediction for Movement

- **Client:** When the local player moves, update the local `Position` immediately (zero latency). Send a
  `PlayerMoveEvent` (Client Event) to the server.
- **Server:** Receive the event, validate the movement (speed limits, collision), and update the authoritative
  `Position`.
- **Replication:** Use `ClientVisibility` (Server Exclusion) to replicate the authoritative `Position` to _all other
  clients except the owner_. The owner relies on their local prediction.
- **Correction (Optional):** Implement a mechanism for the server to force-correct the owner's position if they desync
  significantly (e.g., rubber-banding).

### 3.3. Migrate Gear & Interactions

- Apply the same prediction pattern to gear toggles (e.g., turning on a flashlight).
- **Component Splitting:** Separate networked state (`GearOperativeState`) from local simulation state
  (`ThermometerReading`). The server replicates the operative state; the client computes the reading locally based on
  the map.

---

## 5. Phase 4: Ghost AI & World State

Migrate the remaining dynamic entities.

### 5.1. Ghost Replication

- The server runs the Ghost AI (behavior trees, navigation).
- Replicate the Ghost's `Position`, `GhostState` (Idle, Hunting), and visible effects.
- Clients interpolate the Ghost's position for smooth rendering.

### 5.2. Interactive Objects (Doors, Switches)

- Replicate the state of interactive map objects (e.g., `DoorOpen`, `LightSwitchOn`).
- Clients send interaction events to the server; the server validates and updates the replicated state.

---

## 6. Phase 5: Cleanup & Decommissioning

Once all systems are migrated and verified:

### 6.1. Remove `unnet-*` Crates

- Delete `unnet-core` and `unnet-plugin`.
- Remove all custom serialization and manual packet routing logic.

### 6.2. Final Security Audit

- Verify that the Ticket-Based Authentication effectively prevents unauthorized RAM allocation.
- Load test the Hub and dedicated servers to ensure the new architecture meets the 213-room capacity baseline.

---

## 7. Success Criteria

- The "God Crate" is gone; networked components are defined in their respective feature plugins.
- Local player movement and actions feel instantaneous (zero latency).
- The server remains "thin," offloading static map logic and sensory calculations to clients.
- The infrastructure survives simulated Layer 7 DoS attacks without exhausting server RAM.
