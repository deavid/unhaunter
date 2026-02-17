# Hub Implementation Plan

This document outlines the phased implementation of the Unhaunter Hub ecosystem, as proposed in `01_proposal.md`.

## Phase 1: Foundation & Shared Logic
- [x] Create `crates/unhub-client` library.
- [x] Implement Safe-Vocal alphabet (Base-20).
- [x] Implement Room Code generation and validation.
- [x] Implement the Codename system (Adjective-Noun generator).
- [x] Define shared protocol messages (JSONL format).

## Phase 2: The Hub Service (`unhub`)
- [x] Initialize `crates/tools/unhub` (Axum + Tokio).
- [x] Implement in-memory state (`DashMap`) for Rooms and ProcMans.
- [x] Implement RON persistence for `HubConfig` (Official server keys, Bans).
- [x] Implement ProcMan TCP listener and handshake.
- [x] Implement REST API:
    - `POST /v1/rooms/create`
    - `POST /v1/rooms/join/:code`
    - `GET /health`
- [x] Implement heartbeat and stale room cleanup.

## Phase 3: The Process Manager (`unprocman`)
- [x] Initialize `crates/tools/unprocman`.
- [x] Implement outbound TCP connection to Hub with auto-reconnect.
- [x] Implement `undedicated` process spawning and management.
- [x] Implement stdin/stdout communication channel with `undedicated`.
- [x] Implement idle pool maintenance (default 1 idle server).
- [x] Forward events (PlayerJoined, StateChanged) from `undedicated` to Hub.

## Phase 4: Dedicated Server Adaptations
- [x] Update `unhaunter_dedicated` CLI to support `--procman-channel stdin`.
- [x] Implement message loop for ProcMan commands (`AssignRoom`, `Shutdown`).
- [x] Send lifecycle events to ProcMan (`Ready`, `PlayerJoined`, `Exiting`, etc.).
- [x] Implement Room Secret validation in `unnet` handshake.
- [x] Implement 5-minute idle timeout in the dedicated server logic.

## Phase 5: Game Client Integration
- [x] Integrate Hub discovery into `AppState::MainMenu`.
- [x] Add "Play Online (Hub)" menu option.
- [x] Implement background Hub connectivity status.
- [x] Implement Hub Menu UI with code input display.
- [x] Display Room Code in the Lobby UI for the host.

## Phase 6: Polishing & Validation
- [x] Comprehensive `cargo clippy` and `cargo check` across all crates.
- [x] Removed duplicate ProcMan channel implementation.
- [x] Cleaned up `unhub-plugin` to follow project conventions.
- [x] Final review and checklist generation.
