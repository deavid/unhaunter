# 10 — Mission End Freeze: Root Cause Analysis & Architectural Post-Mortem

- **Date:** March 1, 2026
- **Branch:** `dev-deavid`
- **Status:** Investigation complete. Fix not yet applied.
- **Symptom reported:** After pressing "End Mission" on the truck in offline (singleplayer) mode, the truck UI
  disappears, all movement stops, and the game freezes indefinitely. The expected fade-to-black and Summary screen
  transition never happen.

---

## 1. The Symptom in Detail

Three things happen when the "End Mission" hold-button is released successfully:

1. The truck UI disappears immediately.
2. All simulation motion freezes (ghost stops moving, temperature stops ticking, lighting is frozen).
3. Nothing else ever happens. No fade-to-black. No Summary screen. The game is stuck permanently.

In Host (listen-server) mode the bug does not manifest: the Summary screen is reached after approximately a 2.5-second
silent freeze on the frozen frame. The intended visual fade-to-black does not play in either mode.

---

## 2. The Intended Design

Document `07_target_architecture.md §3 "Soft Landing"` and `08_execution_plan.md §SP-5` describe the intended
mission-end sequence:

```text
1.  Player holds "End Mission" → MissionEvent::End fires.
2.  handle_mission_events: GameState → None, SimulationState → TearingDown,
                           ServerGamePhase → Concluding.
3.  calculate_rewards_and_grades fires on OnEnter(SimulationState::TearingDown) [AuthorityRole].
4.  Client (all modes including offline) observes ServerGamePhase::Concluding change →
    inserts MissionConcludingCinematic resource.
5.  tick_mission_concluding ticks a 2.5 s timer.
6.  While the timer runs, the screen fades to black (visual).
7.  Timer finishes AND SummaryData is ready → AppState → Summary.
8.  server_teardown_grace_period (server only) waits 5 s then AppState → Lobby,
    SimulationState → Unloaded.
```

This is a clean, decoupled design. The breakdown is not in the design but in two incomplete implementation gaps that
break the chain for offline mode.

---

## 3. The Complete Call Chain

### 3.1 Steps that work correctly in offline mode

**Hold-button completion** — `untruck-plugin/src/systems/truck_ui_systems.rs`

The hold-progress animation fills, `TruckUIEvent::EndMission` is written.

**Enablement gate** — `unmission-plugin/src/systems/evaluate_mission_end.rs`

`evaluate_mission_end` runs (gated on `resource_exists::<AuthorityRole>`, which IS present in offline). All active
players are in the truck → `MissionEndRequested(true)`.

**Event dispatch** — `untruck-plugin/src/systems/truck_ui_systems.rs:314`

`truckui_event_handle` reads `MissionEndRequested.0 == true` and writes `MissionEvent::End`.

**Truck UI disappears** — `OnExit(GameState::Truck)`

`handle_mission_events` in `unmission-plugin/src/systems/handle_mission_events.rs` runs (gated on `AppState::InGame`, no
`AuthorityRole` gate). It sets `GameState::None`. `OnExit(GameState::Truck)` fires, hiding the truck panel. This is the
correct and expected behavior.

**Simulation pauses** — `SimulationState::TearingDown`

The same `handle_mission_events` call sets `SimulationState::TearingDown`. The simulation tick systems all gate on
`SimulationState::Ready` or `not(SimulationState::Unloaded)`, so they stop. Again correct and expected.

**Score calculation runs** — `unsummary-plugin/src/plugin.rs:18`

`calculate_rewards_and_grades` is registered on `OnEnter(SimulationState::TearingDown)` with
`run_if(resource_exists::<AuthorityRole>)`. It fires in offline mode. `ActiveMissionEvaluator` is present (inserted in
`unclassic-mode-plugin/src/plugin.rs`). `SummaryData` is populated with a complete score breakdown. This step works
correctly.

### 3.2 The break point

Still in `handle_mission_events`, the last operation is:

```rust
for mut phase in q_server_phase.iter_mut() {
    *phase = ServerGamePhase::Concluding;
}
```

This query `q_server_phase: Query<&mut ServerGamePhase>` iterates **zero entities** in offline mode. The write is a
silent no-op. Nothing in the game is aware this write was attempted and lost.

---

## 4. Why No Entity Carries `ServerGamePhase` in Offline Mode

`ServerGamePhase` is a component that lives on the lobby entity. The lobby entity is spawned by `setup_lobby_entity` in
`unreplicon-plugin/src/systems/lobby.rs`.

Its registration:

```rust
// unreplicon-plugin/src/systems/lobby.rs (line ~49)
app.add_systems(
    OnEnter(AppState::Lobby),
    setup_lobby_entity.run_if(in_state(ServerState::Running)),
);
```

`ServerState` is a `bevy_replicon`-internal state that tracks whether a `RenetServer` transport is active. In offline
mode `connection.rs` deliberately creates no transport:

```rust
// unreplicon-plugin/src/systems/connection.rs
match &cli.net_mode {
    NetMode::Offline => {
        // Singleplayer — no transport needed.
    }
    // ...
}
```

Without a `RenetServer`, `ServerState` is permanently `Stopped`. The `.run_if(in_state(ServerState::Running))` guard on
`setup_lobby_entity` therefore **prevents the lobby entity from ever being spawned in offline mode.**

Consequences that cascade from this one missing entity:

| System                                          | Guard                      | Offline result                  |
| ----------------------------------------------- | -------------------------- | ------------------------------- |
| `setup_lobby_entity`                            | `ServerState::Running`     | Never runs → no lobby entity    |
| `ServerGamePhase` component                     | lives on lobby entity      | Does not exist in ECS           |
| `handle_mission_events`: `q_server_phase` write | no guard but empty query   | Silent no-op                    |
| `on_mission_concluding`                         | `Changed<ServerGamePhase>` | Never triggers                  |
| `MissionConcludingCinematic`                    | inserted by above          | Never inserted                  |
| `tick_mission_concluding`                       | bails on missing cinematic | Returns immediately every frame |
| `AppState::Summary` transition                  | driven by cinematic timer  | Never fires                     |

The game is comprehensively frozen in `AppState::InGame` + `GameState::None` + `SimulationState::TearingDown`.

---

## 5. The Documented Fix That Was Never Coded

`08_execution_plan.md §SP-4.4 — "Offline Mode: Ensure Universal LobbyInfo Spawning"` states:

> In offline (single-player) mode, a `LobbyInfo` entity must now always be spawned when entering the lobby phase. This
> entity is not replicated (no `Replicated` component) but must exist so that `Query<&LobbyInfo>` in UI systems
> gracefully finds data instead of panicking.

And `07_target_architecture.md §F-15` states:

> Even in `Offline` single-player mode, the orchestrator MUST spawn an entity with the `LobbyInfo` component upon
> entering the Lobby phase.

SP-4.4 was the last sub-phase of SP-4 (Lobby Identity Refactor). The lobby player data, `LobbyInfo`, and `ClientUuidMap`
were updated, but the offline spawning path — the removal of the `ServerState::Running` gate from `setup_lobby_entity` —
was never executed.

---

## 6. The Secondary Freeze: `SimulationState::TearingDown` Is Also a Dead End in Offline Mode

Even if `AppState::Summary` were somehow reached by an alternative path, the game state would remain inconsistent after
returning. The system responsible for exiting `TearingDown` is `server_teardown_grace_period`:

```rust
// unreplicon-plugin/src/systems/ghost.rs (line ~92)
app.add_systems(
    Update,
    server_teardown_grace_period
        .run_if(in_state(ServerState::Running))
        .run_if(in_state(SimulationState::TearingDown)),
);
```

This is the **only** system in the entire codebase that transitions
`SimulationState::TearingDown → SimulationState::Unloaded`. It is gated on `ServerState::Running`.

In offline mode it never runs. `SimulationState` stays in `TearingDown` permanently. The 15+ `OnExit(AppState::InGame)`
cleanup systems (thermal grid reset, sound grid reset, floor gear cache clear, entity despawning, etc.) are all waiting
for `AppState::InGame` to exit — which it never does.

This is a secondary freeze on top of the primary one: even the cleanup path is unreachable.

---

## 7. The Fade-to-Black Was Never Built

`07_target_architecture.md §3` describes the fade-to-black as an explicit part of the soft-landing:

> It initiates a 2–3 second visual fade-to-black.

`MissionConcludingCinematic` was created in `unreplicon-core/src/resources.rs` with two fields:

```rust
pub struct MissionConcludingCinematic {
    pub timer: Timer,
    pub inputs_blocked: bool,
}
```

These are the only two mentions of `inputs_blocked` in the entire codebase: the struct definition, and the one line that
sets it to `true` when the resource is inserted. No system anywhere reads `inputs_blocked`. No system anywhere reads
`MissionConcludingCinematic` except `tick_mission_concluding` — which only reads `timer`.

There is no render system, no UI overlay, no shader pass, and no sprite entity that produces a visual fade. The
2.5-second window exists as a timer delay, but during it the screen shows a completely frozen game frame. In Host mode
(where the cinematic does trigger), a player would see 2.5 seconds of a frozen frame before the hard-cut to Summary. In
offline mode they see an indefinite frozen frame.

The fade-to-black is a half-stub: the timing infrastructure was created; the rendering side was never created.

---

## 8. The `handle_mission_events` Authority Boundary Smell

`handle_mission_events` is registered with no `AuthorityRole` gate:

```rust
// unmission-plugin/src/systems/setup.rs
app.add_systems(
    Update,
    (
        handle_mission_events::handle_mission_events,            // ← no gate
        evaluate_mission_end::evaluate_mission_end
            .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
    )
        .run_if(in_state(AppState::InGame)),
);
```

The system **transitions `SimulationState`**. Any node that processes a local `MissionEvent::End` message will
transition its own `SimulationState` to `TearingDown`. In practice this is safe today because:

- `evaluate_mission_end` (which writes `MissionEvent::End` for the dead-players case) IS gated on `AuthorityRole`.
- `truckui_event_handle` (which writes `MissionEvent::End` for the End Mission path) only fires when
  `MissionEndRequested.0 == true`, which is set by `evaluate_mission_end` (also `AuthorityRole`-only) — so Join clients
  will never set `MissionEndRequested` to true, never fire `TruckUIEvent::EndMission`, and never reach the
  `MissionEvent::End` write.

However, the underlying design contract is violated: `handle_mission_events` contains authority- specific operations
(`SimulationState` transitions) but registers without the authority gate. If a future system were to write
`MissionEvent::End` on a Join client for any reason (e.g., a scripted event or a test), that client would corrupt its
own `SimulationState`. This is a MINOR authority boundary violation (class: incorrect implicit contract, not an
observable bug today) analogous to the violations documented in `06_design_audit.md §LAW 3 Findings`.

---

## 9. Why Host Mode Works

In Host (listen-server) mode, `ServerState::Running` is true throughout the session because a `RenetServer` is
instantiated. Therefore:

- `setup_lobby_entity` fires on `OnEnter(AppState::Lobby)` → lobby entity + `ServerGamePhase` are spawned. ✅
- `set_server_state_ingame` fires → `ServerGamePhase::InProgress`. ✅
- `handle_mission_events` writes `ServerGamePhase::Concluding` to the existing entity. ✅
- `on_mission_concluding` detects `Changed<ServerGamePhase>`, inserts `MissionConcludingCinematic`. ✅
- `tick_mission_concluding` ticks. After 2.5 s: `authority.is_some()` is true, `SummaryData` is always present →
  `AppState::Summary`. ✅
- `server_teardown_grace_period` fires after 5 s → `SimulationState::Unloaded`, `AppState::Lobby`. ✅

The visual freeze for 2.5 s before the hard-cut exists in Host mode too (no visual fade). It is only invisible because
the game eventually does proceed to Summary. In offline mode the same freeze is permanent.

---

## 10. Root Cause Inventory

| #        | Root Cause                                                                           | File                                                         | Consequence                                                                                                                                       |
| -------- | ------------------------------------------------------------------------------------ | ------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------- |
| **RC-1** | `setup_lobby_entity` gated on `ServerState::Running` instead of `AuthorityRole`      | `unreplicon-plugin/src/systems/lobby.rs ~L49`                | No `ServerGamePhase` entity in ECS for offline mode                                                                                               |
| **RC-2** | `handle_mission_events` `ServerGamePhase` write targets an empty query in offline    | `unmission-plugin/src/systems/handle_mission_events.rs ~L59` | `ServerGamePhase::Concluding` is never set; cinematic never starts                                                                                |
| **RC-3** | `server_teardown_grace_period` gated on `ServerState::Running`                       | `unreplicon-plugin/src/systems/ghost.rs ~L92`                | `SimulationState::TearingDown` is permanent in offline; cleanup systems never fire                                                                |
| **RC-4** | `sync_mission_result_to_net` gated on `ServerState::Running`                         | `unreplicon-plugin/src/systems/ghost.rs ~L86`                | `MissionResultNet.ready` stays false in offline (irrelevant for authority path of `tick_mission_concluding`, but breaks remote client visibility) |
| **RC-5** | SP-4.4 "Universal LobbyInfo Spawning" designed and planned but never coded           | `08_execution_plan.md §SP-4.4`                               | RC-1 was identified and documented 3 days before the symptom was reported; the fix simply was not applied                                         |
| **RC-6** | Fade-to-black rendering side was never built                                         | `unreplicon-core/src/resources.rs`, everywhere (absent)      | `inputs_blocked` is dead code; screen freezes visually for the cinematic window even when the cinematic does fire (Host mode)                     |
| **RC-7** | `handle_mission_events` missing `AuthorityRole` gate on `SimulationState` transition | `unmission-plugin/src/systems/setup.rs`                      | Implicit authority assumption; latent future breakage if any system sends `MissionEvent::End` to a Join client                                    |

---

## 11. The Architectural Pattern Causing Both This Bug and the Ghost Bug

This is the third instance of the same root architectural pattern. Document `09_ghost_movement_bug_postmortem.md`
established the pattern for ghost movement. The identical mechanism is at work here:

> **`ServerState::Running` is used as a proxy for "this node has authority", but `ServerState` is a transport lifecycle
> state, not an authority marker.**

| `ServerState::Running` when... | Intended meaning                           | Correct predicate                  |
| ------------------------------ | ------------------------------------------ | ---------------------------------- |
| `setup_lobby_entity`           | "I am the server, spawn my authority data" | `resource_exists::<AuthorityRole>` |
| `sync_mission_result_to_net`   | "I am the server, publish results"         | `resource_exists::<AuthorityRole>` |
| `server_teardown_grace_period` | "I am the server, clean up after clients"  | `resource_exists::<AuthorityRole>` |

In offline mode, `AuthorityRole` is present but `ServerState` is `Stopped`. All three systems refuse to run. The
authority node is standing next to the controls with its hands tied.

Document 09 proposed and the codebase partially adopted this LAW:

> **LAW N — No `ServerState` In Game Logic**
>
> Systems in any crate other than `unreplicon-plugin`'s internal transport plumbing must never import or match on
> `bevy_replicon::prelude::ServerState`. All topology-dependent gating must use `resource_exists::<AuthorityRole>`,
> `resource_exists::<LocalPlayerRole>`, or `resource_exists::<LobbyPresenceRole>`.

RC-1, RC-3, and RC-4 in the table above are all direct violations of this LAW that document 09 identified but did not
enumerate. The player movement systems were fixed in `SP-6.4`. The ghost replication systems were audited as the next
known gap. The lobby entity lifecycle and mission- teardown systems are the third group — not yet on any remediation
list before this investigation.

---

## 12. Inventory of `ServerState`-Gated Systems in the Mission-End Path

The following systems in `unreplicon-plugin/src/systems/ghost.rs` use `ServerState` where `AuthorityRole` is the correct
predicate:

| System                         | Current gate                | Correct gate                                      | Consequence if wrong                                        |
| ------------------------------ | --------------------------- | ------------------------------------------------- | ----------------------------------------------------------- |
| `sync_ghost_state_to_net`      | `ServerState::Running`      | `AuthorityRole`                                   | Ghost rubberbands in offline (see doc 09)                   |
| `interpolate_ghost_position`   | `not(ServerState::Running)` | pure client (`LocalPlayerRole && !AuthorityRole`) | Ghost was double-interpolated in offline (see doc 09)       |
| `sync_mission_result_to_net`   | `ServerState::Running`      | `AuthorityRole`                                   | `MissionResultNet` never populated in offline               |
| `server_teardown_grace_period` | `ServerState::Running`      | `AuthorityRole`                                   | `SimulationState` stuck in `TearingDown` forever in offline |

And the lobby entity spawning in `unreplicon-plugin/src/systems/lobby.rs`:

| System                    | Current gate           | Correct gate    | Consequence if wrong                                                                       |
| ------------------------- | ---------------------- | --------------- | ------------------------------------------------------------------------------------------ |
| `setup_lobby_entity`      | `ServerState::Running` | `AuthorityRole` | No `ServerGamePhase` entity in offline; entire cinematic chain breaks                      |
| `set_server_state_ingame` | `ServerState::Running` | `AuthorityRole` | `ServerGamePhase::InProgress` never written in offline (irrelevant since entity is absent) |

---

## 13. Why `calculate_rewards_and_grades` Does Not Expose the Bug

`calculate_rewards_and_grades` in `unsummary-plugin/src/plugin.rs` IS correctly gated on `AuthorityRole`:

```rust
app.add_systems(
    OnEnter(SimulationState::TearingDown),
    calculate_rewards_and_grades
        .run_if(resource_exists::<untypes_core::roles::AuthorityRole>),
);
```

This fires correctly in offline mode when `TearingDown` is entered, and correctly populates `SummaryData`. This is why
the Summary screen — if it were ever reached — would show correct data. The authoritative calculation is sound.

`tick_mission_concluding` also handles the authority path correctly:

```rust
let ready = if authority.is_some() {
    summary_data.is_some()     // SummaryData is always initialised; always true
} else {
    q_net.single().map(|net| net.ready).unwrap_or(false)
};
```

If the cinematic timer were ever started in offline mode, it would immediately transition to `AppState::Summary` once
the 2.5-second timer finished, because `SummaryData` is always present. The transition logic itself is correct. The
entire problem is upstream: the cinematic is never started because the `Changed<ServerGamePhase>` event that starts it
is never emitted because the entity carrying that component does not exist.

---

## 14. Summary of the Two Independent Fixes Required

### Fix A — Make `setup_lobby_entity` (and teardown) use `AuthorityRole`

Change every `ServerState`-gated system in the lobby lifecycle and mission teardown path to use
`resource_exists::<AuthorityRole>`:

- `setup_lobby_entity` registration in `lobby.rs`
- `set_server_state_ingame` registration in `lobby.rs`
- `sync_mission_result_to_net` registration in `ghost.rs`
- `server_teardown_grace_period` registration in `ghost.rs`

This is a prerequisite for everything else. All other fixes are moot without this one. It is also the literal
implementation of SP-4.4 and the LAW N prescribed in document 09.

### Fix B — Build the fade-to-black visual

`MissionConcludingCinematic` is insert-ready but has no rendering side. A system must:

1. Detect that `MissionConcludingCinematic` exists.
2. Spawn a full-screen overlay UI node (black `BackgroundColor`, initially transparent).
3. Drive its alpha from `0.0` to `1.0` over `timer.elapsed_secs() / timer.duration().as_secs_f32()`.
4. Despawn the overlay node when `MissionConcludingCinematic` is removed (i.e., on `OnExit(AppState::InGame)` or when
   `tick_mission_concluding` removes the resource).

`inputs_blocked` is also dead code. If input blocking is desired during the cinematic, a system consuming
`Res<MissionConcludingCinematic>` and checking `.inputs_blocked` must be wired into the relevant input handlers
(keyboard, truck UI interaction). Or `inputs_blocked` should be removed from the struct until the feature is actually
built.

### What NOT to change (scope guard)

- `handle_mission_events` triggering `GameState::None` and `SimulationState::TearingDown` is correct.
- The cinematic timer and `AppState::Summary` logic in `tick_mission_concluding` are correct.
- `calculate_rewards_and_grades` and `SummaryData` population are correct.
- `on_mission_concluding`/`tick_mission_concluding` registration on `LocalPlayerRole` is correct (only nodes with a
  screen need the cinematic).

---

## 15. The Broader Lesson

Every session of the refactor audit has ended with the same finding, documented now for a third time:

> Systems that should run on "nodes with authority" are gated on `ServerState::Running`. Offline mode has authority but
> no transport. The guard falsely excludes it.

The solution — role resources (`AuthorityRole`, `LocalPlayerRole`, `LobbyPresenceRole`) — was designed in document 07,
partially implemented in SP-1 and SP-6, and partially applied to player movement in SP-6.4. Every new investigation
reveals another cluster of systems that were not swept when the fix was first applied.

The only lasting cure is to enforce **LAW N** as a hard compiler/lint rule, not a documentation recommendation: ban
`ServerState` imports in all crates other than `unreplicon-plugin`'s internal transport plumbing. The ghost bug, the
player movement bug, and the mission-end freeze all share a single root: `ServerState` is ambient, intuitively named,
and wrong to use for game logic.
