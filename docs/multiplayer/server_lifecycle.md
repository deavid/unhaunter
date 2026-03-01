# Dedicated Server — State Machine Lifecycle

This document describes the full lifecycle of an Unhaunter **dedicated server** (headless, `AuthorityRole` only, no
`LocalPlayerRole`) as a state machine. It is the authoritative reference for F-17 (§ SP-7 of
`docs/replicon_refactor/08_execution_plan.md`).

---

## Role Resources Present on a Dedicated Server

| Resource            | Present? | Notes                                       |
| ------------------- | :------: | ------------------------------------------- |
| `AuthorityRole`     |    ✓     | Server runs authoritative simulation logic. |
| `LocalPlayerRole`   |          | No human screen; all UX systems are skipped.|
| `LobbyPresenceRole` |    ✓     | Server participates in a networked lobby.   |

---

## Full State Progression

```text
Process Start
  │
  ├─ BootState::Loading          (bevy_asset_loader scans assets;
  │                                headless server loads map index only)
  │
  └─ BootState::Ready            (Res<Maps> is non-empty;
                                  set by set_boot_ready_when_maps_loaded)

─────────────────────────────── AWAITING CONNECTIONS ────────────────────────────────

  AppState:       (stays at EngineBoot — server never transitions via asset-loader)
  SimulationState: Unloaded
  ServerGamePhase: Lobby

  Clients can now connect. The server processes lobby messages:
    • handle_request_select_map       (leader only)
    • handle_request_select_difficulty (leader only)
    • handle_request_start_mission    (leader only) ──────────────┐
                                                                   │
─────────────────────────────── MISSION LOADING ─────────────────▼──────────────────

  Trigger: RequestStartMission received from lobby leader.

  └─ SimulationState::Loading    (LoadLevelEvent consumed; map geometry parsed,
     │                            board arrays allocated)
     │
     └─ SimulationState::Spawning (fields fully allocated;
        │                          ghost entities spawned with Replicated;
        │                          all ghost entities exist before Ready)
        │
        └─ SimulationState::Ready (arrays valid; simulation invariants met;
           │                       ticking begins; replicated state flows to clients)
           │
           └─ AppState::InGame    (server perspective only — not a UX concept;
                                   client-facing UX transitions separately)

─────────────────────────────── MISSION IN PROGRESS ────────────────────────────────

  AppState:        InGame
  SimulationState: Ready
  ServerGamePhase: InProgress

  Normal gameplay: ghost AI, sanity drain, hunt state machine, etc.

─────────────────────────────── MISSION END ────────────────────────────────────────

  Trigger: MissionEvent::End (player extraction, TPK, or admin command).

  └─ SimulationState::TearingDown   (ticking stops; compute scores;
     │                               populate SummaryData + MissionResultNet;
     │                               calculate_rewards_and_grades runs here)
     │
     ├─ ServerGamePhase::Concluding  (clients start their fade-to-black cinematic)
     │
     └─ ServerGamePhase::Ended       (results fully published and available)

  After a grace period (≥ 5 seconds via server_teardown_grace_period):

  └─ SimulationState::Unloaded      (board entities despawned, arrays zeroed)

  └─ ServerGamePhase::Lobby         (back to awaiting next mission)
```

---

## `AppState` Variants the Dedicated Server **Never** Enters

The dedicated server has **no `LocalPlayerRole`**, so all UX-facing states are unreachable. Systems that transition into
these states are either gated by `resource_exists::<LocalPlayerRole>` or are registered only in crates the dedicated
server binary does not include.

| `AppState` Variant | Why the server never enters it                                            |
| ------------------ | ------------------------------------------------------------------------- |
| `MainMenu`         | Driven by `bevy_asset_loader`; headless server skips visual asset loading.|
| `MissionLoading`   | Client-only UX wait screen; server drives loading via `SimulationState`.  |
| `Summary`          | UX screen for the local player; server goes directly back to lobby.       |
| `SettingsMenu`     | UI-only; requires `LocalPlayerRole`.                                      |
| `MapHub`           | UI-only; requires `LocalPlayerRole`.                                      |
| `UserManual`       | UI-only; requires `LocalPlayerRole`.                                      |
| `PreplayManual`    | UI-only; requires `LocalPlayerRole`.                                      |
| `MissionSelect`    | UI-only selection screen; requires `LocalPlayerRole`.                     |
| `Hub`              | UI-only; requires `LocalPlayerRole`.                                      |

The only `AppState` variants that are meaningful on the dedicated server are:

- `EngineBoot` — initial state; server stays here or transitions to `InGame` when a mission loads.
- `InGame` — server is running a mission.
- `Lobby` — server is between missions, accepting connections and lobby messages.

---

## Key Invariants

1. **`BootState::Ready` is one-directional.** Once set, it is never reset to `Loading`. Systems that depend on
   `Res<Maps>` being populated must `.run_if(in_state(BootState::Ready))`.

2. **Ghost entities exist before `SimulationState::Ready`.** The `setup_ghost_entities` system runs in/before
   `SimulationState::Spawning`. When `SimulationState::Ready` is entered, all ghost entities with `Replicated` are
   guaranteed to already exist.

3. **Simulation ticks stop before score calculation.** `SimulationState::TearingDown` is entered before
   `calculate_rewards_and_grades` runs. No gameplay system may mutate ghost or player state after `TearingDown` begins.

4. **No `LocalPlayerRole` = no UX systems.** All systems gated by `resource_exists::<LocalPlayerRole>` are entirely
   absent from the dedicated server's system graph. This is enforced at registration time, not at runtime.

5. **`load_level_handler` guards against missing asset handles.** If a requested map path is not present in
   `Res<Maps>` (common on headless servers that skip visual asset loading), the handler emits `warn!()` and returns
   without panicking (F-04 guard).

---

## Related Files

| File | Role |
| ---- | ---- |
| `crates/untypes-core/src/states.rs` | All state enum definitions (`AppState`, `SimulationState`, `BootState`, `GameState`) |
| `crates/untypes-core/src/roles.rs` | `AuthorityRole`, `LocalPlayerRole`, `LobbyPresenceRole` marker resources |
| `crates/unengine-plugin/src/plugin.rs` | Role insertion at startup; `BootState` registration and transition |
| `crates/unreplicon-plugin/src/systems/ghost.rs` | `SimulationState::TearingDown` entry, score calculation, grace-period teardown |
| `crates/unreplicon-plugin/src/systems/lobby.rs` | Lobby entity spawn, mission start handler, `SimulationState` transitions |
| `crates/unmission-plugin/src/systems/handle_mission_events.rs` | `MissionEvent::End` → `TearingDown` + `ServerGamePhase::Concluding` |
| `crates/untmxmap-plugin/src/load_level.rs` | Map load handler with F-04 missing-asset guard |
