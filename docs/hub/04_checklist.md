# Hub Implementation Checklist

## Foundation
- [x] `unhub-client` crate created.
- [x] Safe-Vocal alphabet implemented.
- [x] Codename system implemented.
- [x] Typed protocols defined.

## Hub Service
- [x] `unhub` binary implemented.
- [x] REST API for create/join.
- [x] ProcMan TCP orchestration.
- [x] Persistence (RON).
- [x] Heartbeats and cleanup.

## Process Manager
- [x] `unprocman` binary implemented.
- [x] Server pool management.
- [x] Stdin/Stdout relaying.
- [x] Hub connectivity.

## Dedicated Server
- [x] ProcMan channel integration.
- [x] Room secret validation.
- [x] Idle timeout (5 mins).
- [x] State synchronization.

## Game Client
- [x] Hub plugin with async worker.
- [x] Main Menu integration.
- [x] Hub UI for room codes.
- [x] Lobby room code display.

## Verification
- [x] `cargo check` clean.
- [x] `cargo clippy` clean.
- [x] No duplicate logic.
- [x] Project conventions followed.
