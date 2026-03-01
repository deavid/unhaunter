# 09 — Ghost Movement Bug: Root Cause Analysis & Architectural Post-Mortem

- **Date:** March 1st, 2026
- **Branch:** `dev-deavid`
- **Status:** Investigation complete. Fix not yet applied.

---

## 1. The Symptom

In offline (singleplayer) mode and presumed also in Host (listen-server) mode, the ghost exhibits a visible but defeated
motion: it appears to attempt movement toward its waypoint, then snaps back toward its spawn point. This "rubberbanding"
pattern repeats until the ghost stalls near spawn. The ghost AI is running and computing movement correctly; the
position changes are being undone by other systems in the same frame.

---

## 2. The Three Systems In Conflict

Every frame in offline mode, three systems all write to the ghost's `Position` component:

### System A — `ghost_movement`

- **File:** `crates/unghost-plugin/src/systems/ghost_ai/movement.rs:38`
- **Gate:** `.run_if(resource_exists::<AuthorityRole>)`
- **Action:** Advances `pos.x/y/z` toward the current waypoint (`ghost.target_point`) at a speed controlled by
  `difficulty.ghost_speed`. This is the intended locomotion.
- **Runs in offline?** ✅ Yes — `AuthorityRole` is inserted for offline mode.

### System B — `interpolate_ghost_position`

- **File:** `crates/unreplicon-plugin/src/systems/ghost.rs:254`
- **Gate:** `.run_if(in_state(AppState::InGame)).run_if(not(in_state(ServerState::Running)))`
- **Action:** Lerps `pos` toward `net_pos` (the frozen `NetworkPosition` component) at `LERP_SPEED = 15.0`. At 60 fps,
  `alpha ≈ 0.22` per frame. Query filter: `With<GhostStateNet>`.
- **Runs in offline?** ✅ Yes — `ServerState` is `Stopped` because no `RenetServer` is ever started in offline mode.

### System C — `interpolate_remote_players`

- **File:** `crates/unreplicon-plugin/src/systems/players.rs:830`
- **Gate:** `.run_if(in_state(AppState::InGame))` — **no ServerState guard at all**
- **Action:** Lerps `pos` toward `net_pos` at `LERP_SPEED = 15.0`. Query filter: `Without<MainPlayer>`. The ghost
  satisfies this filter after the refactor added `NetworkPosition` to it.
- **Runs in offline?** ✅ Yes — always runs in InGame, for every entity that has `NetworkPosition` and not `MainPlayer`.

### Per-Frame Position Math

Let `D` be the displacement `ghost_movement` writes in a given frame, and `d` be the current distance from spawn (the
distance to the frozen `NetworkPosition`). The net position change after all three systems execute is approximately:

```text
After System A:   pos += D              (away from spawn)
After System B:   pos += (net_pos - pos) * 0.22   (toward spawn)
After System C:   pos += (net_pos - pos) * 0.22   (toward spawn, again)
```

Combined, the ghost is pulled back by roughly `0.22 + 0.22*(1-0.22) ≈ 0.39` of the current displacement from spawn each
frame. Once the ghost is far enough from spawn that `0.39 * d > D`, the net movement is _toward_ spawn. At 60fps with
normal ghost speed settings, this equilibrium is reached within a few seconds, causing the stall.

---

## 3. Why `NetworkPosition` Is Frozen At Spawn

`NetworkPosition` is set to the ghost's spawn position in `setup_ghost_entities`:

- **File:** `crates/unreplicon-plugin/src/systems/ghost.rs:136`
- **Gate:** `OnEnter(SimulationState::Spawning).run_if(resource_exists::<AuthorityRole>)`
- **Action:** Inserts `NetworkPosition { x: pos.x, y: pos.y, z: pos.z }` — correctly initialized to spawn position.

The system responsible for updating `NetworkPosition` as the ghost moves is `sync_ghost_state_to_net`:

- **File:** `crates/unreplicon-plugin/src/systems/ghost.rs:179`
- **Gate:** `.run_if(in_state(ServerState::Running)).run_if(in_state(AppState::InGame))`
- **Action:** Copies `pos.x/y/z → net_pos.x/y/z` every frame.
- **Runs in offline?** ❌ No — offline never enters `ServerState::Running`.

So `NetworkPosition` is written once at spawn and **never updated again** in offline mode. It is a permanently frozen
anchor at the ghost's origin.

---

## 4. Why Offline Mode Never Enters `ServerState::Running`

`bevy_replicon`'s `ServerState` is a state machine managed by the presence of a `RenetServer` resource. It transitions
to `Running` only when a transport is active.

In `crates/unreplicon-plugin/src/systems/connection.rs:43`:

```rust
match &cli.net_mode {
    NetMode::Offline => {
        // Singleplayer — no transport needed.
    }
    NetMode::Host { port, .. } => {
        // ...
        commands.insert_resource(RenetServer::new(connection_config));
        // ...
    }
    NetMode::Join { .. } => {
        // ...
        commands.insert_resource(RenetClient::new(...));
        // ...
    }
}
```

Offline inserts neither `RenetServer` nor `RenetClient`. There is no replicon transport. `ServerState` remains `Stopped`
for the entire session.

Contrast this with the role system in `crates/unengine-plugin/src/systems.rs:103`:

```rust
NetMode::Offline => {
    commands.insert_resource(AuthorityRole);
    commands.insert_resource(LocalPlayerRole);
}
```

Offline receives **both** `AuthorityRole` (it runs the simulation) and `LocalPlayerRole` (it renders a screen). This is
the correct role assignment per the design in `08_execution_plan.md §SP-1.4`.

The problem is that `ServerState` is **not equivalent to `AuthorityRole`** and never was:

| Mode      | `AuthorityRole` | `LocalPlayerRole` | `ServerState::Running` |
| --------- | :-------------: | :---------------: | :--------------------: |
| Offline   |       ✅        |        ✅         |           ❌           |
| Host      |       ✅        |        ✅         |           ✅           |
| Join      |       ❌        |        ✅         |           ❌           |
| Dedicated |       ✅        |        ❌         |           ✅           |

`ServerState::Running` is only true for **Host** and **Dedicated**. In offline mode — which has `AuthorityRole` and is
the primary singleplayer experience — it is always false.

---

## 5. The Secondary Bug: `interpolate_remote_players` Catches The Ghost

This system was written for remote _players_:

```rust
fn interpolate_remote_players(
    time: Res<Time>,
    mut q_remote: Query<(&NetworkPosition, &mut Position), Without<MainPlayer>>,
)
```

Its exclusion filter is `Without<MainPlayer>`. The comment says "Only runs for entities that do NOT have `MainPlayer`
(i.e., remote players)." The mental model was: entities with `NetworkPosition` are players; non-local players lack
`MainPlayer`.

After the Phase 4 refactor added `NetworkPosition` to ghost entities via `setup_ghost_entities`, the ghost became an
**invisible participant** in this query. The ghost has `NetworkPosition` (added for replication) and never has
`MainPlayer`. It matches perfectly. No one caught this because the audit document (`05_audit.md §3.2`) reviewed
`interpolate_remote_players` in the context of player movement and marked it ✅ — the ghost was simply not in scope at
the time.

---

## 6. The Third Remaining Instance: `apply_remote_movable_motion`

A third system in this family also uses `ServerState` as a proxy for "client-side only":

- **File:** `crates/unghost-plugin/src/systems/gis/animation.rs:21`
- **Gate:** `.run_if(not(in_state(ServerState::Running)))`
- **System:** `apply_remote_movable_motion`

This receives `MovableMotionBroadcast` server messages and applies door/object tweens. In offline mode, this runs
because `ServerState` is `Stopped`. Since there are no `MovableMotionBroadcast` events in offline (they are only
produced by the network broadcast path), this is likely dormant — but it represents the same structural error: a
client-only system gated on `ServerState` rather than on roles.

---

## 7. How We Got Here: The Architectural Story

### 7.1 The Stated Goal

The user's consistent intent throughout the replicon refactor was:

> "I want all modes to behave the same. If one is broken, all should be broken. I don't want any architectural decision
> that leads to 'I need to test every single mode combination.' I want simplicity, a single codepath, reusability of the
> code — same systems for everything."

### 7.2 The Planning Documents (01–04)

Documents `01_onboarding_decision.md` through `04_phased_execution_plan.md` — the foundational planning layer — do **not
contain the concepts** of `AuthorityRole`, `LocalPlayerRole`, or `ServerState` at all. These concepts didn't exist yet.

More critically, offline/singleplayer appears in these documents **only** as a regression baseline:

> _"Singleplayer must remain playable at every checkpoint."_ — `04_phased_execution_plan.md`, repeated at the end of
> every phase.

This framing — singleplayer as canary, not as equal citizen — silently embedded the first architectural fault line.
Offline was something to preserve, not to design for.

The decision about whether offline would start a replicon server was **never addressed** in these documents. Phase 0
removed the old net plugin; Phase 1 added `unreplicon-plugin`; whether `unreplicon-plugin` would start a transport for
offline was left implicit. The implementation chose "no transport for offline," and this was accepted without examining
what that implied for `ServerState`-gated code.

### 7.3 The First Audit (05)

`05_audit.md` is a checkpoint document written after Phases 1–4 were implemented. It reviews each system and marks it ✅
or notes issues.

For ghost replication (`§4.1`):

> "`interpolate_ghost_position` lerps ghost position with a lerp speed of 15.0. ✅"

For player interpolation (`§3.2`):

> "`interpolate_remote_players` lerps remote entities' `Position` toward `NetworkPosition` every frame with a lerp speed
> of 15.0, excluding `MainPlayer` entities. ✅"

Both are marked correct. The audit was done from an online-multiplayer lens: in online play with an actual running
server, `sync_ghost_state_to_net` _does_ execute and `NetworkPosition` _is_ kept current. The interpolation systems work
correctly in that context. The offline case — where `NetworkPosition` is frozen because `ServerState` never reaches
`Running` — was not tested during this audit.

### 7.4 The Redesign Audit (06–08)

The second audit layer (`06_design_audit.md`, `06_redesign_plan.md`, `07_target_architecture.md`,
`08_execution_plan.md`) identified eighteen architectural violations (F-01 through F-19) and specified repairs.

**F-19** in `07_target_architecture.md` is the most directly relevant. It identifies exactly the problem described in
this document, but only for _player movement_:

> "The fix is: 1. Delete `sync_player_state_to_net` entirely. 2. Remove the topology guard
> `.run_if(not(in_state(ServerState::Running)))` from `send_local_player_position`. Every process with a
> `LocalPlayerRole` unconditionally sends `PlayerMoveMessage` every frame. 3. The server-side `handle_player_move`
> handler receives the message."
>
> "The result: **one function, one contract, zero topology checks.**"

This is precisely the right principle. F-19 was implemented in `players.rs` and verified in
`08_execution_plan.md §SP-6.4`. Player movement now uses `LocalPlayerRole` correctly.

**The ghost was never included in this sweep.** No F-XX was filed for:

- `sync_ghost_state_to_net` being gated on `ServerState::Running`
- `interpolate_ghost_position` being gated on `not(ServerState::Running)`
- `interpolate_remote_players` having no entity-type exclusion

The reform applied the right principle to players and stopped. Ghost systems remained on the old `ServerState`-gating
pattern.

### 7.5 Why The Ghost Systems Were Skipped

Several contributing factors:

1. **Different plugin ownership.** Player movement systems live in `unreplicon-plugin/src/systems/players.rs`. Ghost
   replication systems live in `unreplicon-plugin/src/systems/ghost.rs` — same plugin, different file, different PR
   review context.

2. **The interpolation systems seemed symmetric.** `interpolate_remote_players` (players) and
   `interpolate_ghost_position` (ghost) look structurally identical. When F-19 was filed for players, the ghost
   equivalent wasn't noticed because it wasn't on the inspection path.

3. **The `interpolate_remote_players` query bug is invisible from the player audit.** The audit for `§3.2` was about
   remote players. The ghost entity was not a player. Nobody was looking for non-player entities inside a system named
   "interpolate remote _players_."

4. **The offline case was never a test target.** Both audits (`05_audit.md` and `06_design_audit.md`) were clearly
   conducted with networked multiplayer in mind. The systems work in Host mode (where `ServerState::Running` is true and
   `sync_ghost_state_to_net` fires). The failure mode only appears in offline mode, which was protected as a "canary"
   baseline rather than designed as a first-class test scenario.

---

## 8. The Correct Mental Model

For any system that reads from or writes to a `NetworkPosition` component, there are exactly two categories:

**Category 1 — "I own this entity's simulation"** The local process is running the authoritative logic for this entity
(ghost AI, player AI, physics). This is indicated by `resource_exists::<AuthorityRole>`. The local `Position` is ground
truth. `NetworkPosition` should be written _from_ `Position` to propagate state to remote viewers.

**Category 2 — "I am viewing this entity from the network"** The local process receives `NetworkPosition` updates from a
remote authority and must interpolate the local `Position` toward them for visual smoothness. This only applies to
**pure join clients**: `resource_exists::<LocalPlayerRole>` AND `NOT resource_exists::<AuthorityRole>`.

The condition `not(in_state(ServerState::Running))` was intended to express Category 2. It fails because:

- Offline mode has `AuthorityRole` (it owns ghost simulation) but `ServerState` is `Stopped`.
- Offline mode therefore appears as Category 2, receives interpolation, and fights its own AI.

The correct guard for Category 2 — client-only interpolation — is:

```rust
.run_if(resource_exists::<LocalPlayerRole>)
.run_if(not(resource_exists::<AuthorityRole>))
```

Or equivalently, a dedicated condition:

```rust
fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}
```

This correctly maps to Join clients only. Offline, Host, and Dedicated all fail this condition for the correct reasons.

For `interpolate_remote_players` catching ghost entities, the additional fix is an entity-type exclusion filter
`Without<GhostTag>` in the query — or equivalently, only matching on `With<PlayerNetInfo>` which ghosts never carry.

---

## 9. Inventory Of Affected Systems

The following systems contain `ServerState`-based gating that is semantically wrong for the same reason described in
this document:

| System                        | File                                          | Line | Current Gate                            | Correct Gate                       |
| ----------------------------- | --------------------------------------------- | :--: | --------------------------------------- | ---------------------------------- |
| `sync_ghost_state_to_net`     | `unreplicon-plugin/src/systems/ghost.rs`      |  79  | `ServerState::Running`                  | `resource_exists::<AuthorityRole>` |
| `interpolate_ghost_position`  | `unreplicon-plugin/src/systems/ghost.rs`      | 107  | `not(ServerState::Running)`             | pure client condition              |
| `apply_ghost_state_net`       | `unreplicon-plugin/src/systems/ghost.rs`      | 107  | `not(ServerState::Running)`             | pure client condition              |
| `handle_spawn_particle`       | `unreplicon-plugin/src/systems/ghost.rs`      | 107  | `not(ServerState::Running)`             | pure client condition              |
| `apply_mission_result_net`    | `unreplicon-plugin/src/systems/ghost.rs`      | 114  | `not(ServerState::Running)`             | pure client condition              |
| `interpolate_remote_players`  | `unreplicon-plugin/src/systems/players.rs`    | 830  | `Without<MainPlayer>` (no server guard) | add `Without<GhostTag>` exclusion  |
| `apply_remote_movable_motion` | `unghost-plugin/src/systems/gis/animation.rs` |  21  | `not(ServerState::Running)`             | pure client condition              |

Several related server-side systems in `unreplicon-plugin/src/systems/players.rs` (lines 113, 120) also use
`ServerState::Running` where `AuthorityRole` is the correct check. These are less likely to cause observable bugs in
offline mode because offline mode does have `AuthorityRole` and the player-side AI systems are already gated on
`AuthorityRole` separately, but they represent the same structural smell.

---

## 10. What A Correct Architecture Looks Like

Given the user's goal of a unified codepath, the target architecture should uphold this invariant:

> **No system anywhere in the codebase gates behavior on `ServerState` or `NetMode`.** All topology decisions flow
> exclusively from `AuthorityRole`, `LocalPlayerRole`, and `LobbyPresenceRole`.

`ServerState` is an internal `bevy_replicon` state that reflects transport lifecycle. It is appropriate for
replicon-internal plumbing (e.g., when to start accepting client connections). It is **not** appropriate for game logic
gating, because it does not map cleanly to the game's conceptual authority model.

The fully correct registration block for ghost replication would look like this:

```rust
// Server side: sync simulation state → network components
app.add_systems(
    Update,
    (
        sync_ghost_state_to_net,
        sync_evidence_to_net,
        handle_journal_evidence_toggle,
        handle_journal_ghost_toggle,
    )
        .run_if(resource_exists::<AuthorityRole>)  // ← not ServerState::Running
        .run_if(in_state(AppState::InGame)),
);

// Client side: apply replicated state to local world
// "pure client" = has LocalPlayerRole but not AuthorityRole (Join mode only)
app.add_systems(
    Update,
    (
        interpolate_ghost_position,
        apply_ghost_state_net,
        apply_evidence_net,
        handle_spawn_particle,
    )
        .run_if(in_state(AppState::InGame))
        .run_if(resource_exists::<LocalPlayerRole>)       // ← renders a screen
        .run_if(not(resource_exists::<AuthorityRole>)),   // ← is NOT the simulation owner
);
```

This would mean:

- **Offline:** `sync_ghost_state_to_net` runs (keeps `NetworkPosition` current for... nobody, but harmlessly).
  Interpolation systems do NOT run. Ghost AI drives `Position` directly. Ghost moves correctly.
- **Host:** Same as offline for ghost. `NetworkPosition` is kept current and replicated to join clients.
- **Join:** Interpolation runs. Ghost `Position` is smoothed toward the received `NetworkPosition`.
- **Dedicated:** `sync_ghost_state_to_net` runs. No screen, no interpolation. Correct.

All four modes now behave from a single coherent set of rules. If the ghost breaks in one mode, it breaks in all modes
that share the same role combination.

---

## 11. Broader Lesson: `ServerState` As A Leaky Abstraction

`bevy_replicon`'s `ServerState` leaking into game logic is a broader design smell that this post-mortem has documented
in three separate systems. The architectural reform in documents `07_target_architecture.md` and `08_execution_plan.md`
correctly identified this smell (F-19 for players) and prescribed the cure (role resources). The cure was applied
inconsistently — player movement was fixed, ghost movement was not.

The underlying reason the smell keeps appearing is that `ServerState` is _available_ and _named intuitively_ ("is the
server running?"). Developers reaching for a server/client split naturally reach for `ServerState`. The fix is not just
correcting the existing instances, but establishing a hard LAW — analogous to the existing LAW 1 (no `CliOptions` in
game logic) — that bans `ServerState` from game system gating entirely:

> **LAW N — No `ServerState` In Game Logic**
>
> Systems in any crate other than `unreplicon-plugin`'s internal transport plumbing must never import or match on
> `bevy_replicon::prelude::ServerState`. All topology-dependent system gating must use
> `resource_exists::<AuthorityRole>`, `resource_exists::<LocalPlayerRole>`, or `resource_exists::<LobbyPresenceRole>`.
> Violations are treated as bugs of the same severity as reading `CliOptions` in game logic (LAW 1).

This LAW, had it been stated in the planning documents, would have made the ghost bug impossible to introduce. The F-19
fix for player movement would have surfaced the ghost case in the same pass.

---

## 12. Summary of Root Causes

| #    | Root Cause                                                                      | Where Introduced                                        | When Noticed                                           |
| ---- | ------------------------------------------------------------------------------- | ------------------------------------------------------- | ------------------------------------------------------ |
| RC-1 | `ServerState` used as proxy for `AuthorityRole` in `sync_ghost_state_to_net`    | Phase 4 implementation                                  | This document                                          |
| RC-2 | `ServerState` used as proxy for "pure client" in `interpolate_ghost_position`   | Phase 4 implementation                                  | This document                                          |
| RC-3 | `interpolate_remote_players` has no `Without<GhostTag>` filter                  | Phase 4 implementation (ghost gained `NetworkPosition`) | This document                                          |
| RC-4 | Offline mode never starts `RenetServer`, so `ServerState` is always `Stopped`   | Phase 1 design choice                                   | `05_audit.md §1.3` (noted but consequences not traced) |
| RC-5 | F-19 fix for player movement was not applied to ghost systems in the same pass  | `08_execution_plan.md §SP-6.4`                          | This document                                          |
| RC-6 | Planning docs (01–04) treated offline as regression baseline, not equal citizen | Phase 0 planning                                        | This document                                          |
| RC-7 | No explicit LAW banning `ServerState` from game logic gating                    | Never established                                       | This document                                          |

The ghost is trying to move. The architecture built a net around it.
