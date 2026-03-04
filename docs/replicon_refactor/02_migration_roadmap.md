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
- **Bevy 0.18 Note:** Ensure `bevy_replicon` events are consumed using `MessageReader<T>` and `MessageWriter<T>` to
  align with the project's strict Message vs Event (Observer) separation.

### 1.2. Implement Ticket-Based Authentication

- **Hub Side:** Modify the Hub REST API to issue a signed cryptographic ticket (postcard + HMAC-SHA256) upon successful
  PoW and room allocation.
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
- **Bevy 0.18 Pattern:** Clients use an Observer (`OnAdd<SelectedMap>`) to trigger the local `.tmx` file load and spawn
  the static map geometry, rather than polling in a system.
- **Crucial:** The server does _not_ replicate the thousands of static map tiles. It only replicates the map identifier.

---

## 4. Phase 3: Player Entities & Prediction

This is the most complex phase, requiring the "Client Prediction + Server Exclusion" pattern.

### 3.1. Player Spawning & Ownership

- The server spawns player entities and assigns ownership using `bevy_replicon`'s `ClientId`.
- Replicate core player components: `Position`, `PlayerName`, `PlayerColor`.

### 3.2. Implement Client Prediction and Interpolation for Movement

- **The Interpolation Problem:** Replicating `Position` directly at the network tick rate (e.g., 30Hz) will cause remote
  players to stutter on 60Hz+ clients.
- **Component Split:** Replicate a `NetworkPosition` component instead of the core `Position`.
- **Client (Local Player):** Updates their local `Position` immediately (zero latency). Sends a `PlayerMoveMessage`
  (Client Message) to the server.
- **Server:** Receives the message, validates movement, and updates the authoritative `NetworkPosition`.
- **Replication:** Uses `ClientVisibility` (Server Exclusion) to replicate `NetworkPosition` to _all other clients
  except the owner_.
- **Client (Remote Players):** An interpolation system smoothly moves the visual `Position` towards the replicated
  `NetworkPosition` every frame.

### 3.3. Migrate Gear & Interactions

- Apply the same prediction pattern to gear toggles (e.g., turning on a flashlight).
- **Component Splitting:** Strictly separate networked state from local visual/audio state.
  - Example: `FlashlightNet { is_on: bool }` is replicated.
  - **Bevy 0.18 Pattern:** Use an Observer (`OnChanged<FlashlightNet>`) to toggle the local `Flashlight` component
    (which handles the actual `SpotLight` and meshes). The server _never_ spawns the local `Flashlight`.
- The server replicates the operative state; the client computes sensory readings (like `ThermometerReading`) locally
  based on the map.

---

## 5. Phase 4: Ghost AI & World State

Migrate the remaining dynamic entities.

### 5.1. Ghost Replication

- The server runs the Ghost AI (behavior trees, navigation).
- Replicate the Ghost's `Position`, `GhostState` (Idle, Hunting), and visible effects.
- Clients interpolate the Ghost's position for smooth rendering.

### 5.2. Interactive Objects (Doors, Switches)

- Replicate the state of interactive map objects (e.g., `DoorOpen`, `LightSwitchOn`).
- Clients send interaction messages to the server; the server validates and updates the replicated state.
- **Bevy 0.18 Pattern:** Use Observers (`OnChanged<DoorOpen>`) to trigger the local door animation and sound effects.

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
