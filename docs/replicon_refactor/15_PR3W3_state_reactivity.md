# PR 3 Specification: State Reactivity & Client Authority (Workstream 3)

> **Document status:** Authored against the live codebase (March 2026). All file paths, struct names, field names,
> system names, function signatures, and component lists have been verified by direct inspection of the source.
> Line-number references are indicative and may drift; the symbol names are authoritative.

---

## 1. Objective

Fix two bugs that cause incorrect game-state outcomes in multiplayer:

**Bug #1 — Invincible remote players ("the overwrite bug"):**

The server's `ghost_enrage` system (already correctly gated to `AuthorityRole`) runs `handle_hunting_phase`, which
reduces `player.health` on the server's copy of all player entities. However, every client sends an `ExportStateMessage`
every frame containing its own **locally-undamaged** `PlayerSprite.health`. The server's `handle_export_state` handler
applies `sprite.health = msg.message.health`, completely overwriting the ghost's damage. Remote players are effectively
invincible.

**Bug #2 — Per-instance ghost sound-field desync:**

`sound_update` in `unsound-plugin` runs on **every** instance (offline, host, pure client) with no role guard. Once in
approximately 30 frames (`gn == 0`), it injects random sound vectors near all `SoundEmitter` entities into the
`SoundGrid` resource. `SoundGrid` feeds `lose_sanity` in `unplayer-plugin`. Because every instance rolls its own
independent RNG, each player experiences different sanity-drain pressure over time, even when standing in identical
positions.

**The fix (two-part):**

1. **Path A — Client authority for health:** Remove health reduction from server ghost AI. Add a new client-side system
   that reads the replicated `GhostSprite` and applies damage to the locally-owned player. The existing
   `ExportStateMessage` mechanism already carries health back to the server; no protocol changes are needed.

2. **Centralize ghost sound-field triggers:** Guard `sound_update`'s RNG block to `AuthorityRole` only. Broadcast a new
   `GhostSoundFieldBroadcast` message to all pure clients so they inject the same sound-field pulse at the same time.

---

## 2. Scope Boundaries

| In scope                                               | Out of scope                           |
| :----------------------------------------------------- | :------------------------------------- |
| `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs` | Map loading / tile stitching           |
| `crates/unplayer-plugin/src/systems/sanityhealth.rs`   | UI vignette modifications              |
| `crates/unplayer-plugin/Cargo.toml`                    | `health_regen` authority/client split  |
| `crates/unsound-plugin/src/systems.rs`                 | `detect_and_apply_death` guard changes |
| `crates/unsound-plugin/src/plugin.rs`                  | `server_apply_client_sanity` changes   |
| `crates/unsound-plugin/Cargo.toml`                     |                                        |
| `crates/unreplicon-core/src/messages.rs`               |                                        |
| `crates/unreplicon-plugin/src/systems/ghost.rs`        |                                        |

---

## 3. Background: Key Types and Architecture

Read this section fully before modifying any file.

### 3.1 Role resources (verified — `crates/untypes-core/src/roles.rs`)

```rust
pub struct AuthorityRole;    // Resource — present on server/host instances only
pub struct LocalPlayerRole;  // Resource — present on any instance running a local human player
```

Roles are inserted once at startup (see `crates/unengine-plugin/src/systems.rs`):

- **Offline:** `AuthorityRole` + `LocalPlayerRole`
- **PeerHost:** `AuthorityRole` + `LocalPlayerRole` + `LobbyPresenceRole`
- **Pure Join client:** `LocalPlayerRole` + `LobbyPresenceRole` (no `AuthorityRole`)
- **Dedicated server:** `AuthorityRole` + `LobbyPresenceRole` (no `LocalPlayerRole`)

```rust
// Also defined in untypes-core/src/roles.rs — use this instead of manual not() logic:
pub fn is_pure_client(
    local: Option<Res<LocalPlayerRole>>,
    authority: Option<Res<AuthorityRole>>,
) -> bool {
    local.is_some() && authority.is_none()
}
```

### 3.2 `LocallyOwned` (verified — `crates/unreplicon-core/src/ownership.rs`)

`LocallyOwned` is a marker component (not a resource) inserted onto the **one specific entity** owned by the local
player. Every instance has exactly one `LocallyOwned` player entity when in-game.

- On offline/host: inserted during `setup_mission_players`.
- On pure client: inserted by `handle_ownership_granted` (or the UUID fallback) once the server's `OwnershipGranted`
  message arrives. At that point `Replicated` is also removed from the entity, so the server stops replicating that
  `PlayerSprite` back to the owning client. After this point the only authoritative source of health data for that
  entity is the client's own `ExportStateMessage`.

**Use `With<LocallyOwned>` in queries** when a system must only affect the local player's entity.

### 3.3 `GhostSprite` replication (verified)

`GhostSprite` is registered with `app.replicate::<GhostSprite>()` in `unreplicon-plugin/src/systems/ghost.rs`. This
means every client receives a continuously updated copy of the ghost's full state, including `hunt_target`, `hunting`,
`calm_time_secs`, and `hunt_time_secs`.

**Important caveat on `hunt_time_secs`:** This field is set on the server using `time.elapsed_secs()` at the moment the
hunt starts. The server's elapsed time starts when the server process was launched. Pure clients have their OWN
`Time::elapsed_secs()` counter that starts when the client process was launched — which will be a different absolute
value. Computing `time.elapsed_secs() - ghost.hunt_time_secs` on the client will almost certainly produce a _negative_
or wildly incorrect value, clamped to 0, yielding `ghost_strength = 0.0` and zero damage. **Do not use
`ghost.hunt_time_secs` for client-side ramp-up.** Use a `Local<f32>` timer instead.

### 3.4 `SoundGrid` and `SoundEmitter` — two distinct types

These two names appear in different contexts. Do not confuse them:

- **`unsound_core::resources::SoundGrid`** — a `Resource` holding `HashMap<BoardPosition, Vec<Vec2>>`. This is the
  spatial sanity-pressure field that `lose_sanity` reads. It is what `sound_update` writes to.

- **`unboard_core::components::physics::SoundEmitter`** — a marker `Component` placed on entities that represent
  spatially active sound sources. In the current codebase, exactly **two** entities carry this component per mission:
  the ghost breach and the ghost entity itself (both inserted in `unclassic-mode-plugin/src/systems/orchestrator.rs`).
  The `qe` query in `sound_update` iterates these.

- **`unsound_core::emitter::SoundEmitter`** — a completely different `SystemParam` struct used in ghost AI (`enrage.rs`)
  as a convenience wrapper around time, asset server, and audio playback. Unrelated to the above.

### 3.5 Current data flow for player health

```
[Server: ghost_enrage]          [Client: LocallyOwned PlayerSprite]
       |                                    |
  player.health -= dmg              (no ghost damage here yet)
       |                                    |
  (Replicated PlayerSprite                  |
   — but client has removed                 |
   Replicated, so server does               |
   NOT push this back to owner)             |
                                            |
  [handle_export_state on server] <-- ExportStateMessage (every frame)
  sprite.health = msg.health           health = undamaged value
  --------------- ← THE BUG ─────────────────────────────────────────
```

After this PR the flow becomes:

```
[Client: client_ghost_aura_damage]         [Server: ghost_enrage]
  reads GhostSprite (replicated)             reads players for rage calc only
  if ghost.hunt_target: applies dmg          does NOT touch player.health
  writes LocallyOwned player.health          |
       |                                     |
  [send_export_state]                        |
  ExportStateMessage.health = damaged  ──→  [handle_export_state on server]
                                             sprite.health = msg.health ✓
```

---

## 4. Step 1: Remove Health Damage from the Server Ghost AI

**File to edit:** `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs`

**No Cargo.toml changes required for this step.**

### 4.1 Remove the damage loop from `handle_hunting_phase`

Inside `handle_hunting_phase`, locate and **delete** the entire damage block:

```rust
    let ghost_strength = (time.elapsed_secs() - ghost.hunt_time_secs).clamp(0.0, 2.0);

    // Apply player damage during hunt
    // Damage all players based on distance to ghost
    for (mut player, player_pos, _) in q_player.iter_mut() {
        let dist2 = calculate_weighted_distance_squared(ghost_position, player_pos) + 2.0;
        let dmg = dist2.recip() * difficulty.0.health_drain_rate;
        let damage_to_apply = dmg * dt * 30.0 * ghost_strength / (1.0 + ghost.calm_time_secs / 5.0);
        player.health -= damage_to_apply;
    }
```

Both the `ghost_strength` variable and the `for` loop must be removed together. After removal `handle_hunting_phase` no
longer calls `.iter_mut()` on `q_player` at all.

### 4.2 Update query mutability in `handle_hunting_phase`

`handle_hunting_phase`'s `q_player` parameter must change from `&mut Query<(&mut PlayerSprite, ...),  ...>` to
`&Query<(&PlayerSprite, ...),  ...>`. After the damage loop is removed there are no more writes to `PlayerSprite` inside
this function.

The current signature:

```rust
pub(crate) fn handle_hunting_phase(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    q_player: &mut Query<
        (&mut PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
    time: &Res<Time>,
    difficulty: &Res<CurrentDifficulty>,
    dt: f32,
) -> HuntingResult {
```

Must become:

```rust
pub(crate) fn handle_hunting_phase(
    ghost: &mut GhostSprite,
    ghost_position: &Position,
    q_player: &Query<
        (&PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
    time: &Res<Time>,
    difficulty: &Res<CurrentDifficulty>,
    dt: f32,
) -> HuntingResult {
```

> **Note:** The `time` and `difficulty` parameters are now unused after removing `ghost_strength` and the damage loop.
> Remove them from the signature too (and from the call site in `ghost_enrage`). If any other code in
> `handle_hunting_phase` still needs `dt` (it does — for rage reduction), then `dt` may stay. `time` and `difficulty`
> are no longer needed.

### 4.3 Update `ghost_enrage` to match

In `ghost_enrage`, the `q_player` field and the call to `handle_hunting_phase` must change to reflect the immutable
query. Find the query definition:

```rust
    mut q_player: Query<
        (&mut PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
```

Change it to:

```rust
    q_player: Query<
        (&PlayerSprite, &Position, Option<&Hiding>),
        (
            Without<PlayerSpectating>,
            Without<PlayerDisconnected>,
            Without<PlayerInactive>,
            Without<InTruck>,
        ),
    >,
```

The call site in `ghost_enrage` changes from:

```rust
            let hunt_result = handle_hunting_phase(
                &mut ghost,
                ghost_position,
                &mut q_player,
                &gs_audio.time,
                &difficulty,
                dt,
            );
```

To:

```rust
            let hunt_result = handle_hunting_phase(
                &mut ghost,
                ghost_position,
                &q_player,
                dt,
            );
```

(Removing the `&gs_audio.time` and `&difficulty` arguments, matching the updated signature.)

### 4.4 What must NOT change in `enrage.rs`

- `calculate_rage_update` reads `player_sprite.sanity` — this is only a read, never a write. Leave it untouched.
- `calculate_min_player_distance` reads `player.health` to filter dead players. Leave it untouched.
- All ghost state mutations (rage, hunting, hunt*target, hunt_warning*\*, etc.) must remain.
- The `ghost_enrage` system's `AuthorityRole` gate in `ghost_ai/mod.rs` must remain; do not touch that file.

---

## 5. Step 2: Add Client-Side Ghost Aura Damage

### 5.1 Add Cargo.toml dependencies for `unplayer-plugin`

Edit **`crates/unplayer-plugin/Cargo.toml`**. Add these two lines in the `[dependencies]` section:

```toml
unghost-core = { path = "../unghost-core" }
untags-core  = { path = "../untags-core" }
```

Circular-dependency check: `unghost-core/Cargo.toml` depends only on `unspatial-core` and `unfoundation-core` — neither
of which depends on `unplayer-core` or `unplayer-plugin`. There is no cycle.

`unreplicon-core` (which provides `LocallyOwned`) is already listed:

```toml
unreplicon-core = { workspace = true }
```

### 5.2 Add the system to `sanityhealth.rs`

**File:** `crates/unplayer-plugin/src/systems/sanityhealth.rs`

#### 5.2a Add imports

Add these three imports to the existing `use` block at the top of the file (alongside the existing imports, maintaining
the `un*` crate ordering convention):

```rust
use unghost_core::components::ghost_sprite::GhostSprite;
use unreplicon_core::ownership::LocallyOwned;
use untags_core::tags::GhostTag;
```

All other needed types (`Position`, `PlayerSprite`, `Time`, `CurrentDifficulty`, `InTruck`, `PlayerSpectating`) are
already imported.

#### 5.2b Write the system

Insert this function **before** `pub(crate) fn app_setup`:

```rust
/// Client-side: apply ghost aura damage to the locally-owned player.
///
/// Replaces the server-side health damage removed from `handle_hunting_phase`.
/// Runs only on instances with a local player (`LocalPlayerRole`).
/// Queries only the locally-owned entity (`With<LocallyOwned>`) so that on a
/// PeerHost the host's own player is damaged, but not the server copies of
/// remote player entities.
///
/// `GhostSprite` is replicated from the server, so the client has a current
/// copy of `hunt_target`, `hunting`, and `calm_time_secs`.
///
/// The `Local<f32> hunt_start` timer avoids using `ghost.hunt_time_secs`
/// (a server-absolute timestamp) with the client's local `Time::elapsed_secs()`.
/// See architecture notes for the reason this subtraction is incorrect.
fn client_ghost_aura_damage(
    mut q_local_player: Query<
        (&Position, &mut PlayerSprite),
        (
            With<LocallyOwned>,
            Without<PlayerSpectating>,
            Without<InTruck>,
        ),
    >,
    q_ghost: Query<(&Position, &GhostSprite), With<GhostTag>>,
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    mut hunt_start: Local<f32>,
) {
    let dt = time.delta_secs();

    let Ok((player_pos, mut player)) = q_local_player.single_mut() else {
        return;
    };

    // Check if ANY ghost is hunting before entering the per-ghost loop.
    // This must be computed outside the loop to avoid the following bug:
    // if ghost A is hunting and ghost B is not, ghost B's iteration would
    // reset *hunt_start = 0.0 on every frame, preventing ghost_strength
    // from ever ramping up and making ghost A deal zero damage.
    let any_hunting = q_ghost.iter().any(|(_, g)| g.hunt_target);

    if !any_hunting {
        // No ghost is currently hunting: reset the ramp-up timer.
        *hunt_start = 0.0;
    }

    for (ghost_pos, ghost) in q_ghost.iter() {
        if !ghost.hunt_target {
            continue;
        }

        // Record the client-local time at which we first observed any hunt_target==true.
        // We deliberately avoid time.elapsed_secs() - ghost.hunt_time_secs here
        // because hunt_time_secs is a server-side absolute timestamp that cannot
        // be compared meaningfully to the client's elapsed time.
        if *hunt_start == 0.0 {
            *hunt_start = time.elapsed_secs();
        }
        let ghost_strength = (time.elapsed_secs() - *hunt_start).clamp(0.0, 2.0);

        // Inline of calculate_weighted_distance_squared from unghost-plugin/enrage.rs.
        // That function is pub(crate) within unghost-plugin and not accessible here.
        // Logic is identical: Z distance is multiplied by 10 when on different floors
        // to make the ghost less effective at damaging players across floors.
        let dx = player_pos.x - ghost_pos.x;
        let dy = player_pos.y - ghost_pos.y;
        let ghost_floor = ghost_pos.z.round();
        let player_floor = player_pos.z.round();
        let dz = if ghost_floor != player_floor {
            (player_pos.z - ghost_pos.z) * 10.0
        } else {
            player_pos.z - ghost_pos.z
        };
        let dist2 = dx * dx + dy * dy + dz * dz + 2.0;

        let dmg = dist2.recip() * difficulty.0.health_drain_rate;
        let damage_to_apply =
            dmg * dt * 30.0 * ghost_strength / (1.0 + ghost.calm_time_secs / 5.0);
        player.health -= damage_to_apply;
    }
}
```

**Why `q_ghost.iter()` and not `q_ghost.single()`:** There can be zero ghosts (before mission starts, or during
teardown) or two ghosts (during development/testing). `single()` panics on zero or two results. Iterating is always
safe.

**Why `Without<InTruck>` and `Without<PlayerSpectating>`:** The server damage loop also filters these out. Spectating
players are already dead and should not take further damage. Players in the truck are outside the mission area.

### 5.3 Register the system in `app_setup`

In `pub(crate) fn app_setup(app: &mut App)` at the bottom of `sanityhealth.rs`, add `client_ghost_aura_damage` to the
`.add_systems(Update, (...).run_if(in_state(SimulationState::Ready)))` call:

```rust
    app.add_message::<PlayerDiedEvent>().add_systems(
        Update,
        (
            lose_sanity.run_if(resource_exists::<LocalPlayerRole>),
            recover_sanity,
            health_regen.run_if(resource_exists::<AuthorityRole>),
            server_apply_client_sanity.run_if(resource_exists::<AuthorityRole>),
            visual_health.run_if(resource_exists::<LocalPlayerRole>),
            update_player_stamina,
            client_ghost_aura_damage.run_if(resource_exists::<LocalPlayerRole>),  // ← ADD
            detect_and_apply_death,
            update_profile_death_stats.run_if(resource_exists::<LocalPlayerRole>),
            debug_kill_spectator,
        )
            .run_if(in_state(SimulationState::Ready)),
    );
```

`resource_exists::<LocalPlayerRole>` ensures the system runs on offline, PeerHost, and pure client instances — but NOT
on a dedicated server (which has no local player). On PeerHost, `detect_and_apply_death` (ungated) and `health_regen`
(Authority-gated) continue to run, and `client_ghost_aura_damage` (LocalPlayer-gated) also runs. This is correct: the
host's own player now gets its health from the client-side system, not from the ghost AI.

### 5.4 Offline / PeerHost correctness

In both offline and PeerHost modes:

- `ghost_enrage` runs (AuthorityRole present) but no longer writes `player.health`.
- `client_ghost_aura_damage` runs (LocalPlayerRole present) and applies damage via the inlined formula.
- `health_regen` runs (AuthorityRole present) and applies regeneration to all server-side player copies. On the host's
  own player entity, regen and damage run in the same frame — this is identical behaviour to before this PR for
  single-player (offline) mode.
- `send_export_state` runs (LocalPlayerRole present) and exports the locally-reduced health to the server, where
  `handle_export_state` applies it. On PeerHost the host's player entity has `LocallyOwned` and also lives on the
  authority; `handle_export_state` skips it (`Without<LocallyOwned>` filter).

---

## 6. Step 3: Centralize Ghost Sound-Field Events

### 6.1 Define the network message

**File:** `crates/unreplicon-core/src/messages.rs`

`unreplicon-core` already has `serde` as a dependency and re-exports `bevy::prelude::*` via its own
`use bevy::prelude::*`. No Cargo.toml changes are needed for this file.

Append at the end of `messages.rs`:

```rust
/// Sent by the Authority when the ghost-talk RNG fires in `sound_update`
/// (the `gn == 0` block). All pure clients receive this and inject a matching
/// sound-field pulse into their local `SoundGrid`, synchronizing ghost-induced
/// sanity pressure across all instances.
///
/// One message is sent per `SoundEmitter` entity that exists at the time of
/// the trigger (normally: ghost breach + ghost entity = two messages per event).
#[derive(Debug, Clone, Serialize, Deserialize, Message)]
pub struct GhostSoundFieldBroadcast {
    /// World-space position of the emitter entity (breach or ghost).
    pub position: [f32; 3],
}
```

No `Reflect`, `Default`, or `MapEntities` implementation is needed; `position` is plain `f32` data.

### 6.2 Register the message

**File:** `crates/unreplicon-plugin/src/systems/ghost.rs`

`bevy_replicon`, `unghost-core`, and `untags-core` are already in `unreplicon-plugin/Cargo.toml`. No Cargo changes
needed here.

Add `GhostSoundFieldBroadcast` to the existing import from `unreplicon_core::messages`:

```rust
// Before:
use unreplicon_core::messages::{
    RequestJournalEvidenceToggle, RequestJournalGhostToggle, SpawnParticleNetEvent,
};

// After:
use unreplicon_core::messages::{
    GhostSoundFieldBroadcast, RequestJournalEvidenceToggle, RequestJournalGhostToggle,
    SpawnParticleNetEvent,
};
```

Add the registration line inside `pub(super) fn app_setup`, immediately after the `SpawnParticleNetEvent` registration:

```rust
    // Register server → client messages.
    app.add_server_message::<SpawnParticleNetEvent>(Channel::Ordered);
    app.add_server_message::<GhostSoundFieldBroadcast>(Channel::Ordered);  // ← ADD
```

### 6.3 Add Cargo.toml dependencies for `unsound-plugin`

**File:** `crates/unsound-plugin/Cargo.toml`

`unsound-plugin` currently has no `bevy_replicon` or `unreplicon-core` dependency. Add both:

```toml
unreplicon-core = { workspace = true }
bevy_replicon   = { workspace = true }
```

Both are already declared as workspace dependencies in the root `Cargo.toml` (`bevy_replicon = "0.39"`, and
`unreplicon-core` is in the workspace).

Do NOT add `unghost-core` or `untags-core` to `unsound-plugin`. The `qe` query in `sound_update` iterates ALL
`SoundEmitter` (component) entities, which is the correct existing behavior — both the ghost breach and the ghost entity
have `SoundEmitter`. Filtering by `GhostTag` would change behavior by excluding the breach.

### 6.4 Refactor `sound_update` in `unsound-plugin/src/systems.rs`

**File:** `crates/unsound-plugin/src/systems.rs`

Add these imports at the top of the file alongside the existing imports:

```rust
use bevy_replicon::prelude::{MessageWriter, SendMode, ToClients};
use unreplicon_core::messages::GhostSoundFieldBroadcast;
use untypes_core::roles::AuthorityRole;
```

Expand the signature of `sound_update` to add two new parameters:

```rust
// Before:
pub fn sound_update(
    mut sound_grid: If<ResMut<SoundGrid>>,
    roomdb: Res<RoomDB>,
    qe: Query<(&SoundEmitter, &Position)>,
) {

// After:
pub fn sound_update(
    mut sound_grid: If<ResMut<SoundGrid>>,
    roomdb: Res<RoomDB>,
    qe: Query<(&SoundEmitter, &Position)>,
    is_authority: Option<Res<AuthorityRole>>,
    mut ev_broadcast: MessageWriter<ToClients<GhostSoundFieldBroadcast>>,
) {
```

> **Why `Option<Res<AuthorityRole>>` but NOT `Option<MessageWriter<...>>`:** `MessageWriter` (and `EventWriter`,
> `Commands`, `Query`) are active system parameters that Bevy registers globally when `app.add_server_message::<T>()` is
> called. That call happens unconditionally in plugin setup, so the event queue exists on every instance (server AND
> client). Therefore `MessageWriter` is always available and must NOT be wrapped in `Option` — doing so is invalid Bevy
> syntax and will panic at compile time. The `Option<Res<AuthorityRole>>` guard is all that is needed: pure clients have
> the writer but the `is_authority.is_some()` check ensures they never call `.write()` on it.

Change the `if gn == 0 {` block from:

```rust
    if gn == 0 {
        // Ghost talk once in a while
        for (_, pos) in qe.iter() {
            ...
        }
    }
```

To:

```rust
    if gn == 0 && is_authority.is_some() {
        // Ghost talk once in a while — Authority only.
        // Applies locally (so the host/offline instance hears it),
        // then broadcasts to pure clients.
        for (_, pos) in qe.iter() {
            let pos: &Position = pos;
            let bpos = pos.to_board_position();

            // Apply to local SoundGrid (unchanged inner loop)
            for _ in 0..16 {
                let mut v = Vec2::new(rng.random_range(-2.0..2.0), rng.random_range(-2.0..2.0));
                let l = v.length();
                if l < 0.02 {
                    continue;
                }
                let loudness = rng.random_range(0.3_f32..2.0).powi(2);
                v *= loudness * 4.5 / l;
                let vn = v.normalize() * 1.5;
                let newbpos = Position {
                    x: (bpos.x as f32 + vn.x),
                    y: (bpos.y as f32 + vn.y),
                    z: bpos.z as f32,
                    visual_priority: 0.0,
                }
                .to_board_position();
                sound_grid.sound_field.entry(newbpos).or_default().push(v);
            }

            // Broadcast to pure clients.
            // ev_broadcast is always present (registered globally in plugin setup),
            // safe to call unconditionally here — the is_authority guard above
            // ensures this block is never entered on a pure client.
            ev_broadcast.write(ToClients {
                mode: SendMode::Broadcast,
                message: GhostSoundFieldBroadcast {
                    position: [pos.x, pos.y, pos.z],
                },
            });
        }
    }
```

The rest of `sound_update` (the sound propagation / diffusion loop that processes `sound_grid.sound_field`) must remain
completely unchanged. Only the `if gn == 0 { }` block changes.

### 6.5 Add the client listener system

**File:** `crates/unsound-plugin/src/systems.rs`

Add this new function at the end of the file:

```rust
/// Client-side: receive `GhostSoundFieldBroadcast` from the server and inject
/// the sound-field pulse into the local `SoundGrid`.
///
/// Must NOT run on the Authority (offline or host): `sound_update` already
/// applies the pulse locally on those instances.
/// Registered with `.run_if(is_pure_client)` in `UnhaunterSoundPlugin`.
///
/// Note: The 16-iteration random-vector loop is re-run with local RNG, so the
/// exact vector magnitudes differ between server and client. The trigger timing
/// (when the pulse occurs) is synchronized; the per-vector randomness is
/// intentionally not synchronized. This is acceptable for sanity simulation.
pub fn handle_ghost_sound_field_broadcast(
    mut reader: MessageReader<GhostSoundFieldBroadcast>,
    mut sound_grid: If<ResMut<SoundGrid>>,
) {
    for msg in reader.read() {
        let pos = Position {
            x: msg.position[0],
            y: msg.position[1],
            z: msg.position[2],
            visual_priority: 0.0,
        };
        let bpos = pos.to_board_position();
        let mut rng = random_seed::rng();
        for _ in 0..16 {
            let mut v = Vec2::new(rng.random_range(-2.0..2.0), rng.random_range(-2.0..2.0));
            let l = v.length();
            if l < 0.02 {
                continue;
            }
            let loudness = rng.random_range(0.3_f32..2.0).powi(2);
            v *= loudness * 4.5 / l;
            let vn = v.normalize() * 1.5;
            let newbpos = Position {
                x: (bpos.x as f32 + vn.x),
                y: (bpos.y as f32 + vn.y),
                z: bpos.z as f32,
                visual_priority: 0.0,
            }
            .to_board_position();
            sound_grid.sound_field.entry(newbpos).or_default().push(v);
        }
    }
}
```

The `MessageReader` import is already available through `bevy::prelude::*` (Bevy re-exports replicon's message types
into the prelude in this project's setup). The `GhostSoundFieldBroadcast` import added in §6.4 covers the new type.

### 6.6 Register the listener in `plugin.rs`

**File:** `crates/unsound-plugin/src/plugin.rs`

Add the import:

```rust
use untypes_core::roles::is_pure_client;
```

Register the new system:

```rust
// Before:
impl Plugin for UnhaunterSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sound_update)
            .add_systems(Update, init_sound_grid)
            .add_systems(OnExit(AppState::InGame), reset_sound_grid);

        metrics::register_all(app);
    }
}

// After:
impl Plugin for UnhaunterSoundPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Update, sound_update)
            .add_systems(Update, init_sound_grid)
            .add_systems(
                Update,
                crate::systems::handle_ghost_sound_field_broadcast
                    .run_if(is_pure_client),
            )
            .add_systems(OnExit(AppState::InGame), reset_sound_grid);

        metrics::register_all(app);
    }
}
```

`is_pure_client` (defined in `untypes_core::roles`) returns `true` only when `LocalPlayerRole` exists AND
`AuthorityRole` does not. This correctly targets pure Join clients only:

| Mode             | `LocalPlayerRole` | `AuthorityRole` | `is_pure_client` | listener runs?                         |
| :--------------- | :---------------- | :-------------- | :--------------- | :------------------------------------- |
| Offline          | ✓                 | ✓               | false            | No (applies locally in `sound_update`) |
| PeerHost         | ✓                 | ✓               | false            | No (applies locally in `sound_update`) |
| Pure Join client | ✓                 | –               | true             | **Yes** ✓                              |
| Dedicated server | –                 | ✓               | false            | No (no local player anyway)            |

---

## 7. Files Modified — Summary

| File                                                   | Change summary                                |
| :----------------------------------------------------- | :-------------------------------------------- |
| `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs` | Remove damage loop; update query mutability   |
| `crates/unplayer-plugin/Cargo.toml`                    | Add `unghost-core`, `untags-core`             |
| `crates/unplayer-plugin/src/systems/sanityhealth.rs`   | Add `client_ghost_aura_damage`; register it   |
| `crates/unreplicon-core/src/messages.rs`               | Add `GhostSoundFieldBroadcast` struct         |
| `crates/unreplicon-plugin/src/systems/ghost.rs`        | Register `GhostSoundFieldBroadcast` message   |
| `crates/unsound-plugin/Cargo.toml`                     | Add `unreplicon-core`, `bevy_replicon`        |
| `crates/unsound-plugin/src/systems.rs`                 | Guard `gn==0`; add broadcast; add listener    |
| `crates/unsound-plugin/src/plugin.rs`                  | Register `handle_ghost_sound_field_broadcast` |

---

## 8. Success Criteria Checklist

Run `cargo clippy` (global — no `-p` flag) before submitting. All items below must pass.

**Step 1:**

- [ ] No code in `enrage.rs` reads `time` or `difficulty` inside `handle_hunting_phase`.
- [ ] `handle_hunting_phase` signature takes `&Query<(&PlayerSprite, ...)>` (not `&mut`).
- [ ] `ghost_enrage` passes `&q_player` (not `&mut q_player`) to `handle_hunting_phase`.
- [ ] No `player.health -= ...` expression exists anywhere in `enrage.rs`.

**Step 2:**

- [ ] `unplayer-plugin/Cargo.toml` lists `unghost-core` and `untags-core` as path dependencies.
- [ ] `client_ghost_aura_damage` uses `With<LocallyOwned>`, `Without<InTruck>`, `Without<PlayerSpectating>`.
- [ ] `client_ghost_aura_damage` uses a `Local<f32>` hunt timer, not `time.elapsed_secs() - ghost.hunt_time_secs`.
- [ ] `client_ghost_aura_damage` uses `q_ghost.iter()`, not `q_ghost.single()`.
- [ ] `client_ghost_aura_damage` is registered with `.run_if(resource_exists::<LocalPlayerRole>)`.
- [ ] `client_ghost_aura_damage` is inside the same `.run_if(in_state(SimulationState::Ready))` gate as the other
      systems.

**Step 3:**

- [ ] `GhostSoundFieldBroadcast` is defined in `unreplicon-core/src/messages.rs`.
- [ ] `GhostSoundFieldBroadcast` is registered with
      `app.add_server_message::<GhostSoundFieldBroadcast>(Channel::Ordered)` in
      `unreplicon-plugin/src/systems/ghost.rs`.
- [ ] The `gn == 0` block in `sound_update` has `&& is_authority.is_some()` as an additional condition.
- [ ] `handle_ghost_sound_field_broadcast` is registered in `UnhaunterSoundPlugin` with `.run_if(is_pure_client)`.
- [ ] `unsound-plugin/Cargo.toml` lists `unreplicon-core = { workspace = true }` and
      `bevy_replicon = { workspace = true }`.

**General:**

- [ ] `cargo clippy` produces zero errors and zero warnings.
- [ ] No `pub use` re-exports introduced anywhere.
- [ ] No `despawn_recursive()` calls introduced.
- [ ] `mod.rs` and `lib.rs` files contain only `mod` statements (no new code added to them).

---

## 9. Out of Scope (do not touch)

- **`health_regen`:** Authority-only; effectively a no-op for remote clients (server regen'd value is not replicated
  back). Pre-existing limitation, tracked separately.
- **`detect_and_apply_death`:** Runs ungated on all instances. Pre-existing design choice; not part of this PR.
- **`server_apply_client_sanity`:** Pre-existing server consolidation for sanity values from `PlayerInput`.
- **Actual game audio playback** (`SoundEmitter` system param in `enrage.rs`, roar sounds): This PR does not change any
  audio playback. Only the `SoundGrid` (sanity simulation field) trigger is being synchronized.
- **Map loading, stitching, breach spawning, lobby flow.**
