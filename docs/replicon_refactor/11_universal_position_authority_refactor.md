# 11 — Universal Position Authority Refactor: Investigation, Design Intent & Solution

- **Date:** March 1, 2026
- **Branch:** `dev-deavid`
- **Status:** Design complete. Implementation not yet started.
- **Depends on:** `08_execution_plan.md`, `09_ghost_movement_bug_postmortem.md`, `10_mission_end_freeze_postmortem.md`

---

## 1. Origin: Why This Document Exists

After applying the replicon refactor (documented in `01`–`08`), the game was not playable in offline (singleplayer)
mode. Two glaring bugs were reported immediately:

- **Bug 09** — Ghost rubberbands toward spawn and stalls.
- **Bug 10** — Pressing "End Mission" freezes the game indefinitely after the truck UI disappears.

The instinct from the project owner was **not** to patch these symptoms individually. The request was to step back,
understand the underlying architectural cause, and determine whether there were more failures of the same class waiting
to be discovered — and to fix the root, not the symptom.

This document records that investigation, the design discussion that followed, all decisions made by the project owner,
and the agreed solution.

---

## 2. The Core Architectural Flaw Discovered

### 2.1 The Fallacy: Topology as Proxy for Authority

The replicon refactor (phases 1–4, documented in `04`) replaced the old networking module. In doing so, it introduced
`bevy_replicon`'s internal `ServerState` enum as a gate for game-logic systems. The mistake:

> **`ServerState::Running` was used as a proxy for "I am the simulation authority."** **`not(ServerState::Running)` was
> used as a proxy for "I am a dumb client."**

`ServerState::Running` has one and only one accurate meaning: _a network transport socket is open and listening._ It is
a topology fact, not a capability declaration.

This works accidentally in `Host` and `Dedicated` modes, because those modes have _both_ transport and authority. It
fails completely in `Offline` (singleplayer), which has authority but no transport.

The role system introduced in `SP-1` of `08_execution_plan.md` (`AuthorityRole`, `LocalPlayerRole`, `LobbyPresenceRole`)
was the correct replacement. But the sweep that applied roles to player movement (SP-6 / F-19) was never extended to
ghost systems and mission orchestration systems. Those remained on `ServerState`.

### 2.2 The Four-Node Matrix

Every system that gates on `ServerState` must be evaluated against all four target deployment nodes:

| Node      | `AuthorityRole` | `LocalPlayerRole` | `ServerState::Running` |
| --------- | :-------------: | :---------------: | :--------------------: |
| Offline   |        ✓        |         ✓         |           ✗            |
| Host      |        ✓        |         ✓         |           ✓            |
| Dedicated |        ✓        |         ✗         |           ✓            |
| Join      |        ✗        |         ✓         |           ✗            |

`ServerState::Running` is only `true` for Host and Dedicated. Offline — the primary single-player experience — always
has `ServerState::Stopped`. Any authority-side game system gated on `ServerState::Running` silently does nothing in
singleplayer. Any client-side system gated on `not(ServerState::Running)` incorrectly fires in Offline mode, fighting
the local simulation.

### 2.3 The is_pure_client Concept

The correct guard for "I am viewing this entity from the network, not simulating it" is:

```rust
fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}
```

This is `true` only for Join clients. Offline, Host, and Dedicated all fail it for the correct reasons. No system in the
codebase currently uses this condition. All existing "client-only" guards use `not(ServerState::Running)`, which
incorrectly includes Offline.

---

## 3. The ServerState Sweep — Complete Findings

A full `grep` of every `ServerState` usage in the codebase was conducted. All 22 matches are in exactly four files.

### Group 1 — Correctly gated (leave alone)

These systems process `FromClient<T>` or `ToClients<T>` replicon messages. Without an active transport, these messages
physically cannot exist. `ServerState::Running` is the precise correct gate here.

| System                                                           | File                                                                                       |
| ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------ |
| `handle_request_select_map/difficulty/start_mission`             | `lobby.rs:63`                                                                              |
| `handle_journal_evidence_toggle` / `handle_journal_ghost_toggle` | `ghost.rs:75`                                                                              |
| `handle_player_move`, `broadcast_*`                              | `players.rs:113`                                                                           |
| `handle_truck_loadout_message`                                   | `players.rs:120`                                                                           |
| `auto_start_headless_lobby`                                      | `lobby.rs:39` (already has `BootState::Ready` guard; internal role check handles the rest) |

### Group 2 — Authority-side systems broken in Offline (fix: `ServerState` → `AuthorityRole`)

| System                                             | File               | Offline consequence                                                         |
| -------------------------------------------------- | ------------------ | --------------------------------------------------------------------------- |
| `setup_lobby_entity`                               | `lobby.rs:46`      | No lobby entity → no `ServerGamePhase` in ECS → Bug 10 RC-1                 |
| `set_server_state_ingame`                          | `lobby.rs:52`      | `ServerGamePhase` never reaches `InProgress`                                |
| `sync_ghost_state_to_net` + `sync_evidence_to_net` | `ghost.rs:79`      | `NetworkPosition` frozen at spawn → Bug 09 root                             |
| `sync_mission_result_to_net`                       | `ghost.rs:87`      | `MissionResultNet.ready` stays false → Bug 10 RC-4                          |
| `server_teardown_grace_period`                     | `ghost.rs:93`      | `SimulationState::TearingDown` permanent → cleanup never runs → Bug 10 RC-3 |
| `reset_floor_gear_cache` (enter & exit InGame)     | `players.rs:66,70` | Floor gear cache never reset → state leak between missions                  |

### Group 3 — Client-side systems incorrectly running in Offline (fix: `not(ServerState)` → `is_pure_client`)

| System                        | File              | Offline consequence                                                                                              |
| ----------------------------- | ----------------- | ---------------------------------------------------------------------------------------------------------------- |
| `interpolate_ghost_position`  | `ghost.rs:107`    | Lerps ghost `Position` toward frozen `NetworkPosition`; fights AI → Bug 09 primary                               |
| `apply_ghost_state_net`       | `ghost.rs:107`    | Overwrites `GhostSprite` from stale frozen net data                                                              |
| `apply_evidence_net`          | `ghost.rs:107`    | Overwrites `GhostGuess` from stale net data                                                                      |
| `apply_mission_result_net`    | `ghost.rs:114`    | Dormant in practice (internally gated on `net.ready == true` which stays false), but structurally wrong          |
| All `apply_*_net` for players | `players.rs:154`  | Dormant in practice (`Changed<>` never fires since authority-side sync also doesn't run), but structurally wrong |
| `apply_remote_movable_motion` | `animation.rs:21` | Dormant in practice (no `MovableMotionBroadcast` events in Offline), but structurally wrong                      |

### Group 4 — Query scope collision (not a ServerState bug, but discovered in the same sweep)

| System                       | File              | Problem                                                                                                                                                                                      |
| ---------------------------- | ----------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `interpolate_remote_players` | `players.rs:~832` | Filter is `Without<MainPlayer>`. After Phase 4 added `NetworkPosition` to ghost entities, the ghost matches this query and gets interpolated every frame — contributing to Bug 09 secondary. |

---

## 4. The Deeper Architectural Issue: Two Directions for One Arrow

Beyond the `ServerState` gate problem, investigating Bug 09 exposed a more fundamental design flaw: the direction of the
`NetworkPosition ↔ Position` data flow is **reversed between authority and clients**.

- **On authority**: `Position` is ground truth. `sync_ghost_state_to_net` copies `Position → NetworkPosition` outward.
- **On clients**: `NetworkPosition` (received via replication) is ground truth. `interpolate_ghost_position` copies
  `NetworkPosition → Position` inward.

This two-direction model is the root cause of the "three systems fighting over one field" pattern described in
`09_ghost_movement_bug_postmortem.md §2`. It also makes any universal system impossible to write, because the direction
of the arrow depends on which node you are on.

---

## 5. The Project Owner's Design Intent (Recorded Verbatim)

The following decisions were made explicitly by the project owner during the design discussion that produced this
document. They are recorded here as authoritative design intent.

### 5.1 One Code Path

> "I want all modes to behave the same. I don't want any architectural decision that leads to 'I need to test every
> single mode combination.' I want simplicity, a single codepath, reusability of the code — same systems for
> everything."

This is the guiding axiom. Any solution that works differently per-mode, or that requires a different code path for
Offline vs. Host vs. Join, violates this axiom and must be rejected.

### 5.2 The Universal Position Chain

The correct, universal data flow for any entity that participates in replication is:

```text
NetworkPosition (RW) → Position (RO) → Transform (RO)
```

For entities that do not participate in replication:

```text
Position (RW) → Transform (RO)
```

**If an entity has `NetworkPosition`, then `Position` is read-only. All writes must go to `NetworkPosition`. No
exceptions. If a system needs to write `Position` directly, `NetworkPosition` must not be on that entity.**

### 5.3 The Universal Interpolation System

The `NetworkPosition → Position` copy must be performed by one single system that runs for all entities with
`NetworkPosition`, on all nodes, with the same logic. The lerp is uniform — same alpha, same system. There is no special
alpha for the authority node. There is no separate "authority sync" system that runs in the opposite direction.

This system lives in `unreplicon-plugin` (for now, consistent with current organization).

### 5.4 The Local Player Exception

The local player — the one the human controls — must never have `Position` driven by network data. The experience of
seeing your own character rubberbanding toward a server-received position is unacceptable.

**The local player entity does not have `NetworkPosition`.** The universal interpolation system therefore never touches
it. `Position` is written directly from input processing. This is not a special code path — it is the natural
consequence of the component not being present.

The local player propagates its position to the server via `PlayerMoveMessage`. On the server (or authority),
`handle_player_move` writes the received position into that player entity's `NetworkPosition`. The universal lerp then
converges `Position` toward `NetworkPosition` on all other clients. A 1–2 frame lag on the local player's own position
display is explicitly accepted as the trade-off for this simplicity.

### 5.5 "Make It Right, Not Just Make It Work"

> "Since the replicon refactor I haven't been able to play the game proper — so nothing to lose. We make things right
> first and now, we make things work later."

The project owner explicitly rejected hotfixing individual symptoms. The scope of this work is to repair the
architectural root causes, even if that means a larger refactor than strictly necessary to unblock singleplayer.

### 5.6 Scope Extends to All Applicable Entities

> "My principles should extend to everything applicable. NPCs, movable objects. I try to avoid exceptions, so if you
> find anything and think 'should this also apply to ABC?' — the answer is probably yes."

The universal position chain applies to every entity that has `NetworkPosition`. This includes the ghost, any future
NPCs, and any AI-driven movable objects. It is not ghost-specific.

### 5.7 On `unreplicon-core` as a Dependency

The concern was raised that `unghost-plugin` depending on `unreplicon-core` (for `NetworkPosition`) creates a game-logic
→ networking coupling. The project owner explicitly accepted this dependency and reframed it:

> "`NetworkPosition` is not really a network concept. It is a positioning/board concept. The real problem is the
> reverse: it is the network module that should not be aware of ghost, gear, thermometer, temperatures — but it does.
> That is a problem for another day. For now, `unghost-plugin` depending on `unreplicon-core` is perfectly reasonable."

---

## 6. The Agreed Solution

### 6.1 Phase A — Gate Corrections (no architectural change, just broken guards)

Fix every Group 2 and Group 3 system from §3 above. These are mechanical one-line changes: swap `ServerState::Running`
for `resource_exists::<AuthorityRole>()` on authority systems; add `is_pure_client` condition on client-interpolation
systems.

This does not fix the two-direction arrow problem. It makes the _existing_ architecture correct for all four nodes. It
is a prerequisite for Phase B.

Specific changes:

| System                                             | Current gate                             | Correct gate                                  |
| -------------------------------------------------- | ---------------------------------------- | --------------------------------------------- |
| `setup_lobby_entity`                               | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `set_server_state_ingame`                          | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `sync_ghost_state_to_net` + `sync_evidence_to_net` | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `sync_mission_result_to_net`                       | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `server_teardown_grace_period`                     | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `reset_floor_gear_cache`                           | `in_state(ServerState::Running)`         | `resource_exists::<AuthorityRole>`            |
| `interpolate_ghost_position`                       | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| `apply_ghost_state_net`                            | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| `apply_evidence_net`                               | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| `apply_mission_result_net`                         | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| All `apply_*_net` for players                      | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| `apply_remote_movable_motion`                      | `not(in_state(ServerState::Running))`    | `is_pure_client`                              |
| `interpolate_remote_players`                       | no `ServerState` gate, wrong query scope | add `Without<GhostTag>` or migrate to Phase B |

`is_pure_client` should be defined once as a named `Condition` in `unreplicon-plugin` and reused at every call site:

```rust
pub fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}
```

### 6.2 Phase B — Universal Position Authority (the architectural repair)

This phase eliminates the two-direction arrow and `sync_ghost_state_to_net` entirely.

**Step B-1: Ghost AI writes `NetworkPosition` directly**

In `unghost-plugin/src/systems/ghost_ai/movement.rs`, the `ghost_movement` system currently writes `pos.x/y/z`. Change
this to write `net_pos.x/y/z` instead. Add `unreplicon-core` as a dependency of `unghost-plugin` in its `Cargo.toml`.

The query changes from `&mut Position` to `(&mut NetworkPosition, &mut Position)` where `Position` is now read-only
(used only for reading current visual state, not for writing movement).

**Step B-2: Authority writes `NetworkPosition` in `handle_player_move`**

In `unreplicon-plugin/src/systems/players.rs`, `handle_player_move` currently writes the received `Position` directly.
Change it to write `NetworkPosition` on the target player entity instead.

**Step B-3: Universal `NetworkPosition → Position` lerp system**

In `unreplicon-plugin`, add a new system that runs in `Update` for all entities with `NetworkPosition`:

```rust
fn apply_network_position(
    mut q: Query<(&NetworkPosition, &mut Position)>,
    time: Res<Time>,
) {
    const LERP_SPEED: f32 = 15.0;
    let alpha = (LERP_SPEED * time.delta_secs()).min(1.0);
    for (net_pos, mut pos) in q.iter_mut() {
        pos.x += (net_pos.x - pos.x) * alpha;
        pos.y += (net_pos.y - pos.y) * alpha;
        pos.z += (net_pos.z - pos.z) * alpha;
    }
}
```

This system has no node gate. It runs on every node, for every entity with `NetworkPosition`, without exception.

#### Step B-4: Delete the now-redundant systems

Once B-1 through B-3 are in place:

- `sync_ghost_state_to_net` (the `Position → NetworkPosition` bridge): **deleted**
- `interpolate_ghost_position` (ghost-specific lerp): **deleted**
- `interpolate_remote_players` (player-specific lerp with `Without<MainPlayer>` query): **deleted**

The universal system from B-3 replaces all three.

**Step B-5: Confirm local player entity has no `NetworkPosition`**

Audit `setup_mission_players` in `unreplicon-plugin/src/systems/players.rs`. Confirm that the entity tagged `MainPlayer`
(the local player) does not receive a `NetworkPosition` component. If it does, remove it. The universal lerp system must
not touch the local player's position.

### 6.3 Checkpoint

After both phases, `cargo clippy` must pass with zero new warnings and the following must hold:

- Singleplayer mission can be started, played, ended via truck, and the Summary screen is reached.
- Ghost moves to its waypoints without rubberbanding.
- Returning to Lobby after a mission works (no permanent `SimulationState::TearingDown`).
- Host (listen-server) mode behaves identically to Phase A baseline (no regression).

---

## 7. Items Explicitly Out of Scope for This Document

The following problems were acknowledged during the investigation but are **not part of this work**. They require
separate design sessions:

| Item                                          | Description                                                                                                                                                                                  |
| --------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `unreplicon-core` god-crate problem           | `unreplicon-core` knows about ghost, gear, thermometer, and many other game concepts. The direction of coupling should be reversed. Deferred.                                                |
| Visual fade-to-black on mission end           | `MissionConcludingCinematic` has a timer but no render system produces a visual fade. The 2.5-second window produces a frozen frame. The stub exists; the rendering side does not. Deferred. |
| `D-01/F-18` full TMX visual/logic separation  | Structural complexity; deferred from `08_execution_plan.md`.                                                                                                                                 |
| `D-02/F-03` `LocalPlayer` resource redesign   | No agreed design; deferred from `08_execution_plan.md`.                                                                                                                                      |
| `O-01/F-20` Dedicated server / Lobby coupling | Needs own design session.                                                                                                                                                                    |

---

## 8. Open Questions Resolved During This Discussion

**Q: Should Offline mode start a local replicon server to satisfy `ServerState`?** A: No. The design explicitly uses
roles (`AuthorityRole`, `LocalPlayerRole`) to express capability. `ServerState` is a topology fact and should never be
used for game logic gating. The role system is the correct abstraction.

**Q: Should the ghost have `NetworkPosition` in Offline mode at all?** A: Yes. The rule is: if an entity has
`NetworkPosition`, the universal position chain applies. There is no per-mode discrimination of which components get
added. Offline runs the same systems as Host. The components are the same. The only difference is that `bevy_replicon`
never transmits anything over the wire because there is no transport.

**Q: Does the universal position principle apply to players too?** A: Yes. `handle_player_move` on the authority will
write `NetworkPosition` on the target player entity. The universal lerp system will drive `Position` from that. The
local player's own entity has no `NetworkPosition` at all, so it is never touched by the lerp system.

**Q: Should the authority use alpha=1.0 (instant copy) to avoid lag?** A: No. The same lerp, same alpha, runs
everywhere. The authority sees the same 1–2 frame convergence as clients. For the ghost this is irrelevant (nobody
directly controls the ghost; a frame or two of visual lag is imperceptible). For the local player this is a non-issue
because the local player has no `NetworkPosition` at all.
