# Unhaunter Multiplayer Project: Comprehensive Architecture & Roadmap (2026-02-14)

This document consolidates and supersedes all previous multiplayer investigation and design documents (01-20). It
represents the current authoritative state of the Unhaunter multiplayer architecture, the technical decisions made, and
the remaining challenges.

---

## 1. Core Philosophy & Vision

Unhaunter's multiplayer is designed as a **"Game of Attention"** and a **"SWAT Team Fantasy."**

- **The Experience:** Co-operative investigation where players coordinate evidence gathering and surveillance. The goal
  is to dominate a haunting through knowledge and team coordination rather than reflexes.
- **The "Place" vs. "Program":** The game should feel like a persistent social space. The lobby is a physical staging
  area (the player's "house") that doubles as a safe tutorial zone.
- **FOSS Values:** Lightweight infrastructure, no mandatory accounts (identity via UUID/Keypairs), and support for
  community-run servers.
- **Frictionless Entry:** Default path is Host + Relay/Hub (no port forwarding), but direct `--host`/`--join` always
  remains for off-grid play.

---

## 2. Technical Architecture: The "Thin Server" Model

Unhaunter uses a **Host-Authoritative Star Topology** with a "Thin Server" optimization.

### 2.1 Authority Split

- **Server (Host/Dedicated):** Owns the "Arbiter" logic.
  - Ghost AI decisions (state machine, target selection, hunt triggers).
  - Shared mutable map state (door toggles, light switches, breaker state).
  - Item ownership and interaction validation.
  - Mission phase transitions.
- **Client:** Owns the "Simulation & Rendering" logic.
  - Environmental Grids (Thermal, Light, Sound, Fog/Miasma) are computed locally based on synced ghost/player positions.
  - Visual animations, interpolation, and particle effects.
  - Sanity computation (reported back to server for ghost rage logic).

### 2.2 Networking Stack

- **Transport:** TCP + JSONL (JSON Lines) serialization.
- **Frequency:** 60Hz snapshots (can be throttled to 5-10Hz in Lobby).
- **Identity:** Installation UUID generated on first run. No accounts.
- **NAT Traversal:** Handled by a lightweight **Hub/Relay** service (Tokio-based) on a VPS.

---

## 3. Current Implementation Status

### ✅ Completed Features

- **N-Client Support:** Host handles up to 8 remote clients (9 players total).
- **Dynamic Spawning:** Players spawn on join; Host is always ID 1 (or ID 0 for dedicated).
- **Multiplayer Lobby:** Functional `AppState::Lobby` with map/difficulty selection and sub-states.
- **Session Layer:** Roster management, heartbeats, and late-join support.
- **Disconnect Resilience:** Disconnected players automatically hide from the ghost.
- **Headless Server:** `unhaunter-server` binary for Linux VPS deployment.
- **Thin Server Optimization:** Environmental simulations (Thermal, Fog, Light, Sound) removed from server; computed
  locally by clients.
- **Authority Refinement:** Separation of "Simulation Authority" (server) from "Room Owner" (lobby leader).
- **Sanity/Health Split:** Sanity is client-authoritative (reported via input); Health is server-authoritative.

### 🚧 Known Issues & Tensions

- **Van Entry Logic:** Currently global; needs to be per-player using the `InTruck` component.
- **Map Sync Gaps:** Initial synchronization of breach location and haunted objects needs verification.
- **Architectural Debt:** The `SnapshotMsg` has grown to 22+ fields; needs modularization.
- **Lobby Control:** Transitioning from local UI writes to a request/response protocol for dedicated servers.

---

## 4. The "Lobby Boundary" & Social Layer

The Lobby is a physical Bevy game space.

- **Room Ownership:** The first client to connect to a dedicated server becomes the `RoomOwner`, gaining privileges to
  select maps and start missions.
- **Solo-as-Multiplayer:** Solo play is a private multiplayer session. If opted-in, it appears in the Hub's room
  browser.
- **The Hub (Discovery Service):** A non-Bevy Rust service (axum + SQLite) that manages room codes, server lists, and
  moderation (ban lists by UUID).
- **Mixer Mode:** A proposed "Quick Play" mode for streamers/events to cycle players through missions.

---

## 5. Roadmap: Next Steps

### Phase 1: The Hub & Frictionless UI

- Build the `unhaunter-hub` service for room code registration.
- Replace IP-based joining in the UI with 6-character Room Codes.
- Implement the public room browser.

### Phase 2: Van Entry & Per-Player State

- Refactor `GameState::Truck` to be per-player using the `InTruck` component.
- Allow independent mission-end for clients while the host continues.

### Phase 3: Optimization & Dedicated Server Polish

- Profile and optimize the headless server (single-threaded mode, adaptive tick rates).
- Target: ~240 concurrent rooms on a €10/month VPS.
- Implement lightweight Perlin noise to reduce memory footprint.

### Phase 4: Social Connective Tissue

- Asynchronous presence (bulletin boards, "I was here" traces).
- Contact lists with history (Agencies).
- **[Text Chat System](35_diegetic_chat_system.md)** (Accessibility & Tension fallback).
- **[Voice Chat System](36_voice_chat_system.md)** (Immersive, EMI-affected dual-mode comms).

---

## 6. Hard Truths & Lessons Learned

- **Treat the Disease, not the Symptoms:** Avoid adding fields to the snapshot to fix logic bugs; fix the authority
  model instead.
- **State Triplication is Death:** Ensure a single source of truth for item ownership (PlayerGear vs.
  EquipmentPosition).
- **The God File:** `unnet-plugin/src/systems.rs` must be split into `transport`, `protocol`, and `sync` modules.
- **Documentation vs. Code:** The vision is clear in docs; the code must be disciplined to follow it.
