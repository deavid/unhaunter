# Plan 26: Thin Dedicated Server — Feasibility Analysis

**Date:** 2026-02-12 **Status:** Analysis complete, not yet implemented **Depends on:** Plans 21–25 (N-client
multiplayer, lobby, session layer — all complete)

---

## Context and Motivation

### Where We Are

Multiplayer is working end-to-end as of Plans 21–25:

- N-client TCP connections, host-authoritative
- Lobby with map/difficulty selection
- Late join, heartbeat, disconnect resilience
- Multi-mission sessions with persistent connections

The host currently runs the **full game client** — rendering, audio, input, AND simulation. We want a headless variant
that runs **only the simulation** so we can deploy on cheap VPSes.

### Why a Dedicated Server

1. **NAT traversal.** The developer is behind CGNAT. A dedicated server on a VPS sidesteps NAT entirely — all players
   connect outbound.
2. **Fair hosting.** Currently, one player must always be the host. That forces role assignment based on network
   topology rather than preference.
3. **Availability.** A VPS can stay up 24/7. Players can join anytime.
4. **Cost model.** This is a FOSS game with a solo developer. The simulation is tile-based 2D — CPU cost per instance
   should be negligible on even a $5/mo VPS.

### The Key Question

The original plan was to build a "fat" dedicated server — one that runs all the simulation (thermal diffusion, light
propagation, sound grid, fog/miasma, gear computations) and sends results to clients. This works, but it means the
server carries the heaviest computational systems for no benefit: clients also have all the inputs needed to compute
these locally.

**Can we push environmental simulation to clients entirely and make the server as thin as possible?**

The answer, after analysis, is **yes**. This document records the reasoning.

---

## The Thin Server Model

### Core Principle

The server owns exactly two things:

1. **The ghost** — its AI decisions, movement, evidence clarity values, hunt state.
2. **Shared mutable map state** — door toggles, light switches, breaker state, object positions.

Everything else — thermal diffusion, light propagation, sound grid, fog/miasma, gear readings, sanity, health — is
**derived from data the server already sends to clients**. Clients compute these locally from identical inputs and reach
equivalent results.

### Why This Works for Unhaunter Specifically

Several properties of the game make this feasible:

1. **Co-operative, not competitive.** There's no adversarial relationship between players. If a player tampers with
   their local simulation (e.g., reporting false sanity), they hurt their own team. The security model can be relaxed.

2. **Players observe their own instruments.** Player A reads their own thermometer; Player B reads theirs. Nobody
   compares readings in real-time. Small divergences between clients' thermal grids are invisible to gameplay.

3. **Evidence judgment is server-authoritative.** The server decides "freezing temperature evidence is present" (via
   ghost behavior dynamics / Perlin noise clarity values). The specific temperature a player's thermometer shows is
   flavor derived from that judgment, not the other way around.

4. **Environmental grids converge.** Thermal diffusion is driven toward equilibrium by ghost emitters. Even with
   different frame rates causing different iteration counts, all clients converge to the same steady state. Divergence
   is transient, not cumulative.

5. **Light state is event-triggered.** The light grid rebuilds when room state changes (door toggle, breaker trip, light
   switch). These events arrive simultaneously via the snapshot's room state sync. The only variable inputs are
   flashlight positions, which are synced.

---

## What the Snapshot Already Carries

Before analyzing what moves where, it's critical to understand what data the server already sends to every client every
frame via `SnapshotMsg` (defined in `crates/unnet-core/src/messages.rs`):

### Ghost State (per ghost)

- Position (x, y, z)
- Ghost class (type → determines which 3 evidences it has)
- Warp intensity
- Hunt warning active/intensity
- Hunt target flag, calm time
- Repellent hits/misses (delta and total)
- **All 8 evidence clarity values:** `freezing_temp_clarity`, `floating_orbs_clarity`, `uv_ectoplasm_clarity`,
  `emf_level5_clarity`, `evp_recording_clarity`, `spirit_box_clarity`, `rl_presence_clarity`, `cpm500_clarity`
- `visual_alpha_multiplier`, `rage_tendency_multiplier`

### Player State (per player)

- Position, orientation
- Is hiding, in truck, spectating, running
- Stamina, health, sanity
- Animation frame

### Map/World State

- Room states (name → on/off) — every room's power state
- Map tile changes (delta per frame, full sync on join)
- Breach position
- Haunted objects (position, tileset, influence type)
- Movable objects (position, held-by)

### Gear State (per gear item)

- Position, kind, on/off, deployed, deployed direction
- Battery level
- Gear-specific details:
  - Flashlight: status
  - Thermometer: temperature reading
  - EMF: level
  - Spirit Box: charge, ghost_answer
  - Sage: consumed, active, remaining time
  - Repellent Flask: quantity, active, liquid content

### Other

- Evidences found/missing (journal state)
- Ghost type guess, discarded ghosts
- Mission result (when in Summary state)
- Transient events (sounds, particles)
- App state, game state, can_end_mission flag

**Key insight:** The snapshot carries ghost evidence clarity values AND ghost position AND room states. These are the
three inputs that drive thermal emitters, sound emitters, and fluid emitters. Clients already have everything they need
to reconstruct the environmental grids locally.

---

## System-by-System Authority Analysis

### Currently Gated as Host-Only (`.run_if(is_host)`)

These systems already only run on the host. They would remain server-only:

| System                                  | Plugin            | File                                 |
| --------------------------------------- | ----------------- | ------------------------------------ |
| `ghost_movement`                        | `unghost-plugin`  | `systems/ghost_ai/mod.rs`            |
| `ghost_enrage`                          | `unghost-plugin`  | `systems/ghost_ai/mod.rs`            |
| `ghost_fade_out_system`                 | `unghost-plugin`  | `systems/ghost_ai/mod.rs`            |
| `update_ghost_warning_field`            | `unghost-plugin`  | `systems/ghost_ai/mod.rs`            |
| `ghost_scale_glitch_system`             | `unghost-plugin`  | `systems/ghost_ai/mod.rs`            |
| `ghost_interaction_selection_system`    | `unghost-plugin`  | `systems/gis/selection.rs`           |
| `ghost_interaction_execution_system`    | `unghost-plugin`  | `systems/gis/execution.rs`           |
| `update_ghost_behavior_dynamics_system` | `unghost-plugin`  | `systems/dynamic_behavior_update.rs` |
| `sync_ghost_emitters`                   | `unghost-plugin`  | `systems/dynamic_behavior_update.rs` |
| `grab_object`, `drop_object`, etc.      | `unplayer-plugin` | `systems/grabdrop.rs`                |
| `initialize_truck_gear`                 | `untruck-plugin`  | `truckgear.rs`                       |

### Currently Checked Inside System Body (`if is_host { ... }`)

These systems run on all instances but branch based on authority:

| System                          | Plugin                 | What Happens                                                          |
| ------------------------------- | ---------------------- | --------------------------------------------------------------------- |
| `interaction_event_handler`     | `uninteraction-plugin` | Uses `Authority::Host` vs `Authority::Client` for tile interactions   |
| `player_movement_system`        | `unplayer-plugin`      | On client, only runs for MainPlayer                                   |
| `player_gear_usage_system`      | `unplayer-plugin`      | Different behavior per role                                           |
| `update_thermometer`            | `ungearitems-plugin`   | Host computes temp reading from ThermalGrid; client uses synced value |
| `update_emfmeter`               | `ungearitems-plugin`   | Host computes EMF from grids; client uses synced value                |
| `update_spiritbox`              | `ungearitems-plugin`   | Mixed — charge accumulation and ghost_answer logic                    |
| `sage`/`repellentflask` systems | `ungearitems-plugin`   | Host controls state transitions                                       |
| `evaluate_mission_end`          | `unmission-plugin`     | Returns early if not host                                             |
| Truck manager systems           | `untruck-plugin`       | Check `is_host` inside                                                |

### NOT GATED — Run Identically Everywhere

**These are the systems at the heart of the thin server decision:**

| System                                    | Plugin                  | What It Does                            | Server Needs It?        |
| ----------------------------------------- | ----------------------- | --------------------------------------- | ----------------------- |
| `temperature_update`                      | `unthermal-plugin`      | Thermal diffusion over ndarray grid     | **NO**                  |
| `init_thermal_grid_*`                     | `unthermal-plugin`      | Initialize thermal grid from map        | **NO** (clients do it)  |
| `rebuild_lighting_field`                  | `unlight-plugin`        | Light propagation (wave edges, sources) | **NO**                  |
| `player_visibility_system`                | `unlight-plugin`        | Raycasting visibility per-player        | **PARTIAL** (see below) |
| `apply_lighting_to_tiles_system`          | `unlight-plugin`        | Apply light values to sprite rendering  | **NO**                  |
| `apply_lighting_to_sprites_system`        | `unlight-plugin`        | Same                                    | **NO**                  |
| `sound_update`                            | `unsound-plugin`        | Sound grid simulation                   | **NO**                  |
| `update_miasma`                           | `unfog-plugin`          | Fog/miasma Navier-Stokes sim            | **NO**                  |
| `spawn_miasma` / `animate_miasma_sprites` | `unfog-plugin`          | Visual particles                        | **NO**                  |
| `lose_sanity` / `recover_sanity`          | `unplayer-plugin`       | Sanity from light+temp+sound            | **NO**                  |
| `visual_health`                           | `unplayer-plugin`       | Screen damage overlay                   | **NO**                  |
| `ghost_visual_sync`                       | `unghost-plugin`        | Copy ghost state to render components   | **NO**                  |
| `ghost_influence_visual_sync`             | `unghost-plugin`        | Same for haunted objects                | **NO**                  |
| `spawn_ghost_orb_particles`               | `unghost-plugin`        | Visual particle spawning                | **NO**                  |
| `apply_perspective`                       | `unrender-plugin`       | Isometric transform for rendering       | **NO**                  |
| All UI systems                            | Multiple                | Menu, truck UI, summary, hints          | **NO**                  |
| All input systems                         | `unplayer-plugin`       | Keyboard, mouse                         | **NO**                  |
| Camera spawning                           | Multiple                | Camera2d creation                       | **NO**                  |
| Picking                                   | `unpicking-plugin`      | Mouse picking backend                   | **NO**                  |
| Evidence perception                       | `unclassic-mode-plugin` | Determines what evidence player "sees"  | **NO**                  |
| Evidence decay                            | `unghost-plugin`        | Clarity decay over time                 | **NO** (client-local)   |

---

## Verified: Ghost AI Does NOT Read Environmental Grids

This is the critical finding that makes the thin server viable.

The ghost AI systems (`ghost_movement`, `ghost_enrage`, `ghost_fade_out_system`, `update_ghost_warning_field`,
`ghost_scale_glitch_system`) were checked exhaustively. **None of them query `ThermalGrid`, `LightGrid`, `SoundGrid`, or
`MiasmaGrid`.**

What the ghost AI actually reads:

| Resource/Component        | Used By                          | Purpose                                  |
| ------------------------- | -------------------------------- | ---------------------------------------- |
| `PlayerSprite.sanity`     | `ghost_enrage`                   | Rage calculation: low sanity → more rage |
| `PlayerSprite.mean_sound` | `ghost_enrage`                   | Sound proximity → more rage              |
| `PlayerSprite.health`     | `ghost_enrage`                   | Health factor in rage formula            |
| `Position` (players)      | `ghost_movement`, `ghost_enrage` | Distance calculations                    |
| `Hiding` component        | `ghost_movement`, `ghost_enrage` | Hidden players are harder to target      |
| `BoardCollisionField`     | `ghost_movement`                 | Pathfinding/wall avoidance               |
| `BoardTopology`           | `ghost_movement`                 | Map size bounds                          |
| `RoomDB`                  | `ghost_enrage`                   | Player-in-room check for rage bonus      |
| `CurrentDifficulty`       | Both                             | Difficulty multipliers                   |
| `GhostBehaviorDynamics`   | `ghost_enrage`                   | `rage_tendency_multiplier`               |
| `PerlinNoise`             | `dynamic_behavior_update`        | Evidence clarity computation             |
| `VisibilityData`          | `gis/selection`                  | Prefer visible interactions (see note)   |

**Note on `VisibilityData`:** The ghost interaction selection system uses player visibility data to bias interactions
toward tiles the player can see (70% preference for "dramatic" interactions). On the dedicated server,
`player_visibility_system` operates on entities with the `Viewer` component, which is only attached to `MainPlayer`. A
dedicated server has no `MainPlayer`, so visibility data would be empty. The fallback behavior is fine: the ghost still
interacts with objects, just without the visibility bias. If desired, the server could add `Viewer` to all player
entities and run the visibility system (it's cheap — just raycasting through `BoardCollisionField`, no lighting needed).

The ghost interaction execution system (`gis/execution.rs`) similarly only reads `BoardCollisionField`, `BoardTopology`,
`Behavior`, and `Position`. No grids.

### What Ghost Behavior Dynamics Computes

`update_ghost_behavior_dynamics_system` in `dynamic_behavior_update.rs` uses Perlin noise to generate evidence clarity
values. Its inputs are:

- `Time` (elapsed seconds)
- `PerlinNoise` (resource — deterministic from seed)
- `GhostSprite.class` (determines which evidences are real)
- `CurrentDifficulty` (`evidence_visibility` parameter)
- Per-ghost noise offsets (initialized at spawn, constant thereafter)

**This is pure math with no environmental dependencies.** It produces the clarity values that are then sent to clients
in the snapshot and used by `sync_ghost_emitters` to configure `ThermalEmitter`, `FluidEmitter`, and `SoundEmitter`
components.

On the thin server, `sync_ghost_emitters` is irrelevant — there are no grids to drive. But the clarity values are still
needed in the snapshot.

---

## The Sanity Circularity Problem

There is one dependency loop between server and client:

```text
Server: ghost_enrage reads PlayerSprite.sanity, .mean_sound, .health
Client: lose_sanity computes sanity from LightGrid, ThermalGrid, SoundGrid
```

If sanity computation moves to the client, the server needs those values back.

### Solution: Split Sanity and Health Authority

**Health stays server-authoritative.** Ghost damage (`player.health -= damage_to_apply` in `ghost_enrage`) is applied on
the server. Health regeneration is also trivial arithmetic that the server can run without grids:

```rust
// From lose_sanity — no grid reads needed for this part
ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
    * difficulty.0.health_recovery_rate;
```

The server sends authoritative `health` in the snapshot, and clients display it. Clients never report `health` back —
this avoids the "invincible player" bug where client-reported health could overwrite server-applied damage.

**Sanity moves to client.** Only sanity computation requires environmental grids (light, thermal, sound). The
`PlayerInput` message (client → server, every frame) currently carries:

```rust
PlayerInput {
    player_id, o_position, movement, run, interact,
    use_right_hand, use_left_hand, target_right_hand, target_left_hand,
    target_position, aim_direction,
}
```

**Add `sanity: f32` and `mean_sound: f32` fields to `PlayerInput`.** The client computes these locally from its
environmental grids and reports them to the server. The server uses the client-reported values for ghost rage
computation.

The server also runs the simple health regen formula locally (no grids needed) and applies ghost damage. The combined
flow:

1. Client computes `sanity` from local grids, sends it in `PlayerInput`.
2. Server stores client-reported `sanity` on the player entity.
3. Server runs `ghost_enrage`, reads `sanity` → computes rage → applies damage to `health`.
4. Server runs health regen (trivial arithmetic).
5. Server sends authoritative `health` + relayed `sanity` in snapshot.
6. Client displays snapshot `health` for all players. For the local player, the locally-computed sanity is used directly
   (snapshot sanity is for other players' HUDs).

### Is Client-Reported Sanity Safe?

In a competitive game, no — clients could lie about their sanity. But Unhaunter is co-op. Faking high sanity makes the
ghost less aggressive, which hurts your own team's challenge. Faking low sanity makes the ghost hunt more, which is
dangerous for everyone.

For extra safety, the server could do bounds-checking (sanity must be 0–100, mean_sound must be non-negative) and
rate-of-change validation. This is a future nicety, not a blocker.

### How Stale Is the Data?

`PlayerInput` is sent every frame (~60Hz). Ghost rage computation uses sanity in a continuous accumulation formula. 16ms
of staleness (one frame) is negligible compared to the time constants in the rage system (calm_time is in seconds, rage
decays via `1.01^dt` and adds via `dt * 5.2 * inv_sanity`).

---

## What the Server Can Drop

### Plugins to SKIP entirely on dedicated server

| Plugin                        | Why Safe                                                    |
| ----------------------------- | ----------------------------------------------------------- |
| `UnhaunterFpsPlugin`          | FPS display — visual only                                   |
| `UnhaunterUiPlugin`           | UI asset loading, themes                                    |
| `UnhaunterSettingsPlugin`     | User settings persistence (video, audio)                    |
| `UnmetricsPlugin`             | Diagnostics display (keep if you want server metrics)       |
| `CustomSpritePickingPlugin`   | Mouse picking                                               |
| `UnhaunterMenuPlugin`         | Main menu UI                                                |
| `UnhaunterMenuSettingsPlugin` | Settings menu UI                                            |
| `UnhaunterMapHubPlugin`       | Map selection UI (host selects via CLI/config)              |
| `UnhaunterSummaryPlugin`      | Summary screen UI (server sends MissionSummary via network) |
| `UnhaunterManualPlugin`       | In-game manual UI                                           |
| `UnhaunterCampaignPlugin`     | Campaign progression (not relevant for dedicated missions)  |
| `UnhaunterProfilePlugin`      | Local player profile (no local player)                      |
| `UnhaunterCoreMenuPlugin`     | Menu templates                                              |
| `ThermalPlugin`               | **Thermal grid simulation — clients compute locally**       |
| `UnhaunterLightPlugin`        | **Light propagation — clients compute locally**             |
| `SoundPlugin`                 | **Sound grid — clients compute locally**                    |
| `UnhaunterFogPlugin`          | **Fog/miasma — clients compute locally**                    |
| `UnhaunterGearItemsPlugin`    | **Per-item gear simulation — clients compute locally**      |
| `UnhaunterWalkiePlugin`       | Walkie triggers — client-local                              |
| `UnhaunterNPCPlugin`          | NPC UI — no local player on server; hydration is inert      |

### Plugins to KEEP (simulation + network)

| Plugin                       | What Server Uses                                              |
| ---------------------------- | ------------------------------------------------------------- |
| `UnhaunterEnginePlugin`      | State machine (`AppState`, `GameState`), cleanup              |
| `UnhaunterNetPlugin`         | All networking                                                |
| `UnhaunterTmxMapPlugin`      | Map loading (custom `TmxMap`/`TsxSheet` asset loaders)        |
| `UnhaunterMapLoadPlugin`     | Map entity spawning (creates `Behavior`, `Position`, room DB) |
| `UnhaunterGhostPlugin`       | Ghost AI, behavior dynamics, interaction selection/execution  |
| `UnhaunterInteractionPlugin` | Door/switch/breaker toggle arbitration                        |
| `UnhaunterDifficultyPlugin`  | Difficulty configuration                                      |
| `MissionPlugin`              | Mission lifecycle evaluation                                  |
| `UnhaunterLobbyPlugin`       | Lobby state management (no UI)                                |

Note: `UnhaunterMapLoadPlugin` has a rendering dependency on `Assets<CustomMaterial1>` which must be resolved (see Phase
3 and Appendix D). `UnhaunterNPCPlugin` is moved to the SKIP list — NPC interactions are not meaningful without a local
player.

### Plugins to KEEP PARTIALLY (need surgery)

| Plugin                  | Keep                                                                                                           | Skip                                                                                      |
| ----------------------- | -------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------- |
| `UnhaunterRenderPlugin` | `BoardTopology`, `BoardCollisionField`, `SpriteDB`, `RoomDB` resources, `board_sync` systems, entity hydration | `apply_perspective`, `Material2dPlugin`, `UiMaterialPlugin`, `set_window_icon`            |
| `UnhaunterPlayerPlugin` | Movement, position tracking, interaction dispatch                                                              | Input systems, styling, mouse, camera, sanity/health, viewer sync, walk target indicators |
| `UnhaunterGearPlugin`   | `GearSpawnerRegistry`, gear entity management                                                                  | Gear rendering, sound events                                                              |
| `ClassicModePlugin`     | `classic_mode_orchestrator` (spawns ghost/breach/players), `spawn_joined_player`                               | Evidence perception, game UI, hints, looking gear                                         |
| `UnhaunterTruckPlugin`  | Truck gear initialization, loadout management, journal state                                                   | Truck UI, truck camera                                                                    |

---

## What Moves to Clients

For each system that moves, here's the analysis of whether its inputs are available client-side from the snapshot data:

### Thermal Diffusion (`unthermal-plugin`)

**Inputs needed:**

- Map geometry (collision field, room structure) → identical on all clients (loaded from same `.tmx`)
- `ThermalEmitter` components → driven by ghost behavior dynamics

**How it works:** `sync_ghost_emitters` sets `ThermalEmitter.target_temp` and `ThermalEmitter.power` based on
`freezing_temp_clarity` from the ghost's behavior dynamics. The snapshot sends `freezing_temp_clarity` to all clients.
Clients can reconstruct the emitter parameters identically.

**Divergence risk:** The diffusion algorithm iterates over tiles in a rolling-budget pattern. Different frame rates →
different iteration counts → slightly different temperatures. But thermal diffusion converges to equilibrium. The ghost
emitter pushes toward a target temperature; all clients converge to the same steady state within seconds.

**Impact of divergence:** Each player reads their own thermometer. Nobody compares readings in real time. Even a 0.5°C
difference between clients is unnoticeable.

### Light Propagation (`unlight-plugin`)

**Inputs needed:**

- Room on/off states → synced in snapshot (RoomSync)
- Static light sources → from map (identical)
- Flashlight positions/directions → synced in snapshot (GearSyncState)
- Collision field → identical (from map)

**How it works:** `rebuild_lighting_field` runs when `BoardTopologyToRebuild` fires (door toggle, breaker trip, light
switch). These events originate from room state changes which arrive via the snapshot. All clients receive the same room
states at the same tick, so they rebuild the light field from identical inputs.

**Divergence risk:** Effectively none for static lighting. Flashlight contributions may vary by one frame of position
data — negligible.

### Sound Grid (`unsound-plugin`)

**Inputs needed:**

- Ghost position → synced in snapshot
- `SoundEmitter` component → driven by ghost behavior dynamics (synced)

**How it works:** `sound_update` generates random sound impulses from the ghost's position and propagates them through
room tiles.

**Divergence risk:** Uses `random_seed::rng()` which is likely not synchronized between clients. Sound impulses will
differ between clients. But `mean_sound` (which feeds into ghost rage) is a smoothed average over time, so different
random patterns converge.

**Important:** If we move sanity computation to clients, `mean_sound` is computed locally and reported to the server.
Different RNG → slightly different `mean_sound` → slightly different ghost rage calculation. This is acceptable for a
co-op game with slow-moving averages.

### Fog/Miasma (`unfog-plugin`)

**Inputs needed:**

- Ghost breach position → synced in snapshot
- `FluidEmitter` component → driven by ghost behavior dynamics (synced)

**Divergence risk:** Similar to sound — iterative fluid simulation will diverge between clients. Miasma is currently
visual-only (no gameplay systems read `MiasmaGrid` except the EMF meter). The EMF meter uses `miasma_pressure` in its
reading calculation, but EMF values are already somewhat noisy by design. Minor divergence is invisible.

### Gear Reading Computation (`ungearitems-plugin`)

**Thermometer:** Currently host-only computation. The host samples `ThermalGrid` at the thermometer's position, runs a
smoothing filter, produces a temperature reading, and sends it in the snapshot as `GearDetails::Thermometer { temp }`.
In the thin model, clients compute their own thermal grid and sample it locally. The smoothing filter is deterministic
given the same inputs. Each client displays their own reading.

**EMF Meter:** Similar — host samples `ThermalGrid`, `SoundGrid`, and `MiasmaGrid` plus uses
`HauntState.ghost_dynamics.emf_level5_clarity` (synced in snapshot). Client can do this locally. The
`emf_level5_clarity` value from the snapshot is the authority; the local grid values are derived flavor.

**Spirit Box:** Partially client-safe. The charge accumulation reads local `SoundGrid`, `ThermalGrid`, and `LightGrid` —
clients can do this. **BUT** the `ghost_answer` decision (whether the ghost responds) involves threshold checking + RNG.
Two clients with slightly different charge values might disagree on whether the threshold was reached. **`ghost_answer`
must remain server-authoritative.** Currently it's sent in the snapshot as `GearDetails::SpiritBox { ghost_answer }`.
Keep this.

**Other gear (flashlight, sage, salt, UV, etc.):** Already run on all instances or are trivial state machines. No grid
dependencies.

### Sanity/Health (`unplayer-plugin/sanityhealth.rs`)

**Sanity inputs (client-side):**

- `LightGrid` at player position → client-local grid
- `ThermalGrid` at player position → client-local grid
- `SoundGrid` at player position → client-local grid
- `RoomDB` → identical (from map)
- `CurrentDifficulty` → identical

**Health inputs (server-side):**

- Ghost damage → applied by `ghost_enrage` on server
- Health regen formula → trivial arithmetic, no grids needed

**Sanity computation** runs ONLY on clients. The client reports `sanity` and `mean_sound` to the server via
`PlayerInput`.

**Health computation** stays on the server. Ghost damage and health regen are both applied server-side. The server sends
authoritative `health` in the snapshot, as today.

Both `sanity` and `health` remain in `PlayerState` within the snapshot:

- `health`: server-authoritative (server computes damage + regen, clients display it)
- `sanity`: server-relayed (server stores client-reported value, broadcasts to all clients)

### Walkie (`unwalkie-plugin`)

Walkie triggers fire based on ghost state (synced), timers, and player profile. All inputs are available client-side.
The walkie is informational — it doesn't affect simulation. Clients can run it independently.

### Evidence Perception (`unclassic-mode-plugin`)

`update_current_evidence_readings_from_player_perception_system` determines what evidence the local player "perceives"
based on their gear's proximity and the light grid. This is inherently client-local — each player perceives based on
their own gear and position. The system reads `LightGrid` for visibility checks (can I see my thermometer's reading?).
Stays client-side.

### Evidence Decay (`unghost-plugin/evidence_decay.rs`)

`decay_evidence_clarity_system` reduces `CurrentEvidenceReadings` clarity over time when not being updated by gear. This
is a client-local system that affects the player's evidence HUD. Stays client-side.

---

## Late Joiner Thermal Convergence

When a player late-joins, `init_thermal_grid_content` initializes their local thermal grid at ambient temperature ±3°C.
If the ghost has been freezing a room, the late joiner's grid starts warm while other clients have converged grids.

**How fast does convergence happen?** The ghost's `ThermalEmitter` has high power when `freezing_temp_clarity` is
elevated. The diffusion algorithm runs thousands of tile iterations per frame (rolling budget). A typical room is 30–80
tiles. At high emitter power, the temperature drops to near-target within seconds, not minutes.

**Gameplay impact:** For 5–10 seconds, the late joiner's thermometer reads warm in a "freezing" room. This is acceptable
— the player just connected and is orienting themselves. By the time they walk to the ghost room and pull out their
thermometer, the grid has converged.

**If faster convergence is needed (future optimization):**

- Send per-room average temperatures in the late-join snapshot. Client seeds its grid with room-specific values instead
  of ambient.
- Or run the emitter at 10× power for the first N frames after join.
- Or perform extra diffusion iterations during the loading screen (before the player gains control).

This is a minor UX quirk, not a blocker. It only affects late joiners, only for a few seconds, and only for temperature
readings.

---

## Snapshot Changes for Thin Server

### Fields to Keep (server computes, sends to clients)

- Ghost position, all clarity values, hunt state, rage state, visual_alpha, rage_tendency
- Room states, map tile changes
- Gear positions, on/off, deployed, battery
- `GearDetails::SpiritBox { ghost_answer }` — server decides responses
- Player positions (from client-reported or server-tracked)
- Player alive/dead/spectating
- Journal state, mission result, transient events
- Breach position, haunted objects, movable objects

### Fields to Reconsider

**`PlayerState.health`:**

- **Stays server-authoritative.** Server applies ghost damage AND health regen (no grids needed).
- Server sends authoritative health in the snapshot, as it does today.
- Clients never report health. This prevents the "invincible player" bug.

**`PlayerState.sanity`:**

- Server no longer computes this (requires grid reads).
- Server receives client-reported sanity from `PlayerInput` and relays it in the snapshot.
- Each client sees the server-relayed sanity for other players (for HUD display).
- For the local player, the client uses its own locally-computed sanity.

**`GearDetails::Thermometer { temp }` / `GearDetails::EMF { level }`:**

- Two options:
  - **Option A:** Remove from snapshot. Each client computes their own. They'll diverge slightly but nobody notices.
  - **Option B:** Keep in snapshot, but the server no longer computes them. The gear-owning client reports readings
    back. Server relays.
- **Recommendation: Option A.** Simpler. No new protocol fields needed. The slight divergence is acceptable.

### Fields to Add to `PlayerInput`

```rust
PlayerInput {
    // ... existing fields ...
    sanity: f32,         // NEW: client-computed sanity
    mean_sound: f32,     // NEW: client-computed sound average
}
```

---

## The Minimal Server: What Remains

After removing all environmental simulation, the dedicated server's responsibilities are:

### 1. Ghost AI Brain

- **Behavior dynamics:** Perlin noise → evidence clarity values (pure math, no grids)
- **Rage calculation:** From player-reported sanity/mean_sound + distance + difficulty
- **Hunt decision:** Rage threshold → pre-warning → warning → hunt
- **Movement:** Target selection (player tracking, room wandering), pathfinding via `BoardCollisionField`
- **Interaction selection:** Choose what object to toggle/throw/slam (uses `VisibilityData` optionally, falls back
  gracefully without it)
- **Interaction execution:** Apply the interaction to map state

### 2. Shared Mutable State Arbitration

- **Door/switch/breaker toggles:** Two players toggle the same switch → server decides order
- **Grab/drop conflicts:** Two players reach for same item → server decides who gets it
- **Object positions:** Movable/throwable objects have server-authoritative positions
- **Room state:** On/off state of every room (affects map tile rendering for everyone)

### 3. Mission Lifecycle

- **Mission evaluation:** Are all players in the truck? Is the ghost unhaunted? → end mission
- **Player death:** Server decides when ghost damage kills a player
- **Scoring and summary:** Server computes final score

### 4. Network Hub

- **Snapshot broadcast:** Compile and send state to all clients (~20–60Hz)
- **Input processing:** Receive `PlayerInput` from all clients, apply to player entities
- **Session management:** Lobby state, heartbeat, disconnect handling, late join
- **Event relay:** Transient events (sounds, particles) from server decisions → all clients

### What Does NOT Run on the Server

- No thermal grid
- No light grid (no `rebuild_lighting_field`, no `player_visibility_system`\*)
- No sound grid
- No miasma/fog grid
- No gear reading computation
- No sanity/health computation
- No rendering, sprites, materials, cameras
- No audio
- No UI
- No input handling
- No picking
- No evidence perception or decay

\* The server could optionally run `player_visibility_system` (cheap raycasting via `BoardCollisionField`) to provide
ghost interaction visibility bias. Without it, ghost interactions still work but lose the "prefer visible to player" 70%
bias. This is a minor behavioral difference, easily acceptable.

---

## CPU Cost Estimate

### Before (Fat Server)

The heaviest systems per the metrics infrastructure:

1. **Thermal diffusion** (`temperature_update`): Iterates rolling budget of tiles (~6K iterations/frame on a 100×100×3
   map at quality 1.0)
2. **Light propagation** (`rebuild_lighting_field`): Wave propagation from sources through collision field
3. **Sound grid** (`sound_update`): Sparse HashMap iteration + propagation
4. **Fog/miasma** (`update_miasma`): Navier-Stokes-like velocity field iteration
5. **Ghost AI**: Perlin noise sampling + rage math
6. **Network I/O**: Snapshot serialization

Systems 1–4 are **removed** from the thin server.

### After (Thin Server)

What remains:

- Ghost AI: Perlin noise for ~10 values per ghost per frame. Negligible.
- Rage calculation: Arithmetic over player list. Negligible.
- Ghost movement: Position updates + collision checks. Negligible.
- Interaction selection: Iterate nearby entities, random selection. Rare (fires every few seconds, not every frame).
- Board topology / collision field: Initialized once at map load. Updated on interaction events. Negligible ongoing
  cost.
- Snapshot serialization: JSON serialization of ~2–5KB per frame. Light.
- Network I/O: TCP read/write for N clients. Light.

**Estimated CPU per instance:** <5% of a modern vCPU at 60Hz tick rate.

### Memory per Instance (variable, per-mission)

- Map entities: ~30K tiles × ~200 bytes each ≈ 6MB
- `BoardCollisionField`: 30K × ~32 bytes ≈ 1MB
- `RoomDB`: Small (dozens of rooms)
- Ghost entities: <1KB
- Player entities: <1KB each
- Network buffers: ~10KB per client
- **Total variable per instance: ~10–15MB**

### Memory per Process (fixed overhead)

In addition to per-mission data, each dedicated server **process** carries fixed costs:

| Fixed Cost                      | Size       | Notes                                 |
| ------------------------------- | ---------- | ------------------------------------- |
| PerlinNoise lookup table        | **64 MB**  | 4000×4000×f32, precomputed at startup |
| Thread pool stacks (2-core VPS) | ~10 MB     | ~5 threads × 2 MiB default stacks     |
| Binary private data/statics     | ~15 MB     | Initialized data, allocator metadata  |
| Bevy ECS World/schedules        | ~3 MB      | System graphs, archetype tables       |
| Asset infrastructure            | ~2 MB      | `AssetServer`, `Assets<T>` storages   |
| SpriteDB / tileset templates    | ~3 MB      | Pre-built tile component templates    |
| **Total fixed per process**     | **~97 MB** |                                       |

**Total per process = ~112 MB** (97 fixed + 15 variable). On a 1 GB VPS: ~8 concurrent mission processes.

### The PerlinNoise Optimization

The `PerlinNoise` resource (in `unnoise-core/src/perlin.rs`) allocates a 4000×4000 `Vec<Vec<f32>>` = 64 MB. This
dominates the fixed overhead. On the server, ghost behavior dynamics samples it ~10 times per ghost per frame. The
`noise::Perlin` crate function computes a value in sub-microsecond time. The precomputed table is a CPU cache
optimization important for client-side systems (thermal/light/fog simulation over entire grids) but unnecessary for the
server's sparse sampling pattern.

**Options (any one suffices):**

1. **Compute on-the-fly on server:** Replace `PerlinNoise::get()` with direct `noise::Perlin::get()` calls. Zero memory.
   Server-only change, clients keep the table.
2. **Shrink table:** 1000×1000 = 4 MB. Still instant lookups, just wraps around 4× more.
3. **mmap a shared file:** Pre-generate the table to disk, `mmap` read-only. Shared across processes via OS page cache.

With option 1 or 2, fixed overhead drops to ~33–37 MB. Per-process total: ~48–52 MB. Capacity on 1 GB: **~18–20
instances.**

---

## Deployment Model: Per-Process vs Fork vs Multi-Room

### Option 1: One Process Per Mission (Recommended)

Each mission spawns a new dedicated server process. Simplest architecture.

| Metric                | With Perlin table | After Perlin optimization |
| --------------------- | ----------------- | ------------------------- |
| Memory per process    | ~112 MB           | ~48 MB                    |
| Instances on 1 GB VPS | ~8                | ~20                       |
| Complexity            | Trivial           | Trivial                   |

A meta-server (see next section) handles room creation and spawns/kills child processes. Standard Unix process
management. No shared state concerns.

### Option 2: Fork Model

Pre-allocate the PerlinNoise table and parsed tileset data in a parent process, then `fork()` per mission. Child
processes inherit the 64 MB table as copy-on-write shared pages.

**Why it doesn't help much:**

1. **SpriteDB contains Bevy `Handle<Image>` / `MeshMaterial2d`** — can't be built before the Bevy App exists, so it's
   post-fork. Only `PerlinNoise` and raw parsed tilesets (~1 MB) benefit from CoW sharing.
2. **Thread pools don't survive fork.** POSIX: forking a multi-threaded process is unsafe. Must fork _before_ Bevy
   creates `TaskPoolPlugin`, then recreate pools in the child.
3. **`unsafe` + Linux-only.** Fine for production Linux VPS, but adds complexity.
4. **Achieves the same density as shrinking the Perlin table** — ~20 instances on 1 GB. One is a constant change; the
   other is an architecture overhaul.

**Verdict:** Fork saves ~64 MB per child, but eliminating the Perlin table saves the same 64 MB with zero architectural
complexity. Fork is not worth the tradeoff.

### Option 3: Multi-Room Single Process

One Bevy App hosts N concurrent missions in the same ECS World. All fixed costs are shared.

| Metric            | Value         |
| ----------------- | ------------- |
| Fixed overhead    | ~82 MB (once) |
| Per mission       | ~15 MB        |
| Instances on 1 GB | ~62           |

**Much higher capacity**, but requires:

- Namespacing all ECS queries by "room" or "session" (new component/filter on every entity)
- Ensuring zero cross-room state leakage
- Major refactor of every system that iterates over entities

**Verdict:** Not justified unless you need 50+ concurrent rooms on a single VPS. Defer until demand proves the need. The
per-process model with Perlin optimization gives ~20 rooms, which is plenty for early deployment.

---

## Headless Bevy Configuration

### Approach: `MinimalPlugins` + Explicit Additions

```rust
// Pseudocode for dedicated server app setup
app.add_plugins(MinimalPlugins.set(
    ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
));
app.add_plugins(AssetPlugin::default());  // Needed for .tmx loading
app.add_plugins(TransformPlugin);          // Position/Transform used in game logic
app.add_plugins(HierarchyPlugin);          // Parent-child relationships
app.add_plugins(StatesPlugin);             // AppState/GameState
app.add_plugins(DiagnosticsPlugin);        // Optional, for server metrics
// ... add simulation + network plugins only
```

**DO NOT** use `DefaultPlugins` minus renderers. `DefaultPlugins` bundles rendering, audio, windowing, input together
with inter-plugin dependencies. Removing subsets risks panics.

### What Breaks Without a Window

1. **`Material2dPlugin<CustomMaterial1>`** — requires render pipeline. The server must NOT load
   `UnhaunterRenderPlugin`'s material/mesh setup. Any code path creating `MeshMaterial2d<CustomMaterial1>` handles will
   panic.

2. **`Camera2d` spawning** — several plugins spawn cameras (lobby, map hub, summary). These must be skipped on the
   dedicated server.

3. **Sprite/animation systems** — `apply_perspective`, animation timers for rendering. The server may still need
   `AnimationTimer` on entities (it's queried in `host_send_snapshots` for the `frame` field), but doesn't need the
   actual animation tick systems.

4. **Asset loading works headless** — `AssetPlugin` reads files from disk without GPU. The custom asset loaders
   (`TmxMapLoader`, `TsxSheetLoader`, `AssetIdxLoader`) are file I/O based. `bevy_asset_loader`'s `LoadingState` works
   with `AssetPlugin`.

5. **No `bevy_ecs_tiled` or `bevy_ecs_tilemap`** — confirmed not used. Map loading is entirely custom. Good — no
   third-party rendering dependency.

6. **`bevy_persistent`** (profile/settings plugins) — file I/O based, works headless. But the profile plugin is skipped
   on the dedicated server anyway.

---

## Implementation Plan

### Phase 1: Add `--dedicated` flag and `is_headless` run condition

- Add `dedicated: bool` to `CliOptions`
- Create `pub fn is_headless(cli: Res<CliOptions>) -> bool`
- Add `sanity: f32` and `mean_sound: f32` to `PlayerInput` (protocol change)
- Update `ghost_enrage` to prefer client-reported sanity when available
- Move health regen to server side (extract from `lose_sanity` into standalone system; trivial arithmetic, no grids)

This is the minimal protocol change. No plugin restructuring needed yet.

### Phase 2: Create the dedicated server binary

- New file: `unhaunter/src/bin/dedicated.rs`
- Uses `MinimalPlugins` + `ScheduleRunnerPlugin` + `AssetPlugin`
- Loads only: engine, net, tmxmap, mapload, render (partial), ghost, interaction, difficulty, player (partial), gear
  (partial), mission, lobby (partial), classic-mode (partial), npc
- Skips everything listed in "Plugins to SKIP"

### Phase 3: Split mixed plugins

For plugins that have both simulation and rendering systems (unrender-plugin, unlight-plugin, unghost-plugin,
unplayer-plugin, unclassic-mode-plugin, untruck-plugin):

- Modify `app_setup` to check `is_headless` and skip visual systems
- OR split into sub-modules: `app_setup_simulation(app)` vs `app_setup_rendering(app)`

Priority order:

1. `unrender-plugin` — skip materials, perspective, window icon
2. `unplayer-plugin` — skip input, styling, sanity, mouse, camera
3. `unghost-plugin` — skip visual sync, ghost orbs
4. `unclassic-mode-plugin` — skip evidence perception, game UI
5. `untruck-plugin` — skip UI
6. `unlobby-plugin` — skip UI

### Phase 4: Validate

- Start dedicated server with `--dedicated --host 5000 --map <map> --difficulty <diff>`
- Connect with regular client
- Verify: map loads, player spawns, ghost spawns, interactions work, mission completes
- Check for panics from missing resources or render-dependent code paths

### Phase 5: Optimize

- Reduce snapshot rate to 20Hz (currently ~60Hz). Player movement is client-predicted.
- Consider `FixedUpdate` for remaining server systems
- Profile with `unmetrics` on the VPS
- Test with 4–9 concurrent clients

---

## Meta-Server Architecture

The thin server analysis so far covers the **mission server** — a headless Bevy process running one game room. But
players need a way to discover, create, and join rooms. This requires a **meta-server** that sits in front of mission
servers.

### The Two Meanings of "Host"

The current codebase uses "host" to mean two distinct things:

1. **Lobby Leader** — the player who chooses the map, difficulty, and decides when to start the mission. This is a
   social/UI role.
2. **Simulation Authority** — the instance that runs ghost AI, arbitrates interactions, and broadcasts snapshots. This
   is a technical role.

In the current peer-hosted model, these are always the same player. In the dedicated server model, they split:

| Role                 | Peer-Hosted                     | Dedicated Server                |
| -------------------- | ------------------------------- | ------------------------------- |
| Lobby leader         | The host player                 | The player who created the room |
| Simulation authority | The host player's game instance | The dedicated server process    |

The current `NetMode::Host` conflates both. For the dedicated server, the simulation authority is the server process,
while the lobby leader is a regular client with elevated privileges (the "room owner").

### Preserving the Current UX

**Key constraint:** The current host-client model must keep working unchanged. Players who host locally (LAN, direct
connect) should see no difference. The dedicated server is an additional option, not a replacement.

Current user journeys that must remain:

1. **Player hosts a game** → opens port, friends connect directly → works as today
2. **Player joins a game** → enters IP:port → connects as client → works as today
3. **Player creates a room on the dedicated server** → new journey (see below)
4. **Player joins a room via code** → new journey (see below)

### Connection Flow with Meta-Server

```text
                    ┌─────────────────────┐
                    │    Meta-Server      │
                    │  unhaunter.com:5000 │
                    │                     │
                    │  - Room registry    │
                    │  - Room codes       │
                    │  - Process spawner  │
                    └────┬───────────┬────┘
                         │           │
                    spawn│      spawn│
                         ▼           ▼
              ┌──────────────┐ ┌──────────────┐
              │ Mission Srv  │ │ Mission Srv  │
              │ Room: ABCD   │ │ Room: EFGH   │
              │ Port: 5001   │ │ Port: 5002   │
              └──────────────┘ └──────────────┘
                 ▲  ▲  ▲          ▲  ▲
                 │  │  │          │  │
              Players A,B,C    Players D,E
```

**Step by step:**

1. **Client connects to meta-server** at `unhaunter.com:5000`.
2. **Create room:** Client sends `CreateRoom` request. Meta-server:
   - Generates a room code (e.g., `ABCD`)
   - Spawns a mission server process on a free port (e.g., `5001`)
   - Returns `{ room_code: "ABCD", address: "unhaunter.com:5001" }`
   - The creating player is marked as "room owner" (lobby leader)
3. **Join room:** Another client sends `JoinRoom { code: "ABCD" }`. Meta-server:
   - Looks up room code → returns `{ address: "unhaunter.com:5001" }`
4. **Client connects to mission server** at `unhaunter.com:5001`.
   - From here, the existing lobby → map selection → mission flow takes over.
   - The room owner has lobby leader privileges (select map, start mission).
   - The mission server is the simulation authority.
5. **Mission ends:** Players return to lobby within the same mission server. Room owner can start another mission. The
   process stays alive.
6. **Room persists** as long as the room has connected players. The mission server process stays alive across multiple
   missions — players can return to the lobby and start another map. When the last player disconnects, the process
   exits. Meta-server cleans up the room code.

### Meta-Server Is NOT Bevy

The meta-server is a lightweight TCP/HTTP service. It does NOT run a Bevy App. Its responsibilities:

- Maintain a registry of active rooms (code → port → player count → status)
- Spawn and kill mission server child processes
- Assign ports from a pool
- Health-check child processes (detect crashes, reap zombies)
- Optionally: authentication, rate limiting, ban list

This can be a simple Rust binary using `tokio` (or even synchronous I/O — the load is negligible). It holds no game
state. It's a process manager with a room registry.

**Protocol decision: HTTP REST.**

HTTP REST was chosen over raw TCP + JSONL for the meta-server protocol. Reasons:

- WASM compatibility for a future browser client — regular HTTP calls work natively in WASM.
- Easy to deploy and scale with `tokio` + any HTTP framework (axum, warp, actix-web).
- curl-friendly for debugging and monitoring.
- The meta-server handles very few requests per session (create/join/list), so HTTP overhead is negligible.
- No persistent connection needed — the meta-server is stateless from the client's perspective.

### Room Owner vs Simulation Authority

The mission server process runs `NetMode::Host` — it is the simulation authority. But it has no local player. The room
owner is a regular `NetMode::Join` client with a flag marking them as the lobby leader.

New concept needed: **`RoomOwner`** — a `NetworkId` stored on the mission server. The room owner's client gets special
privileges:

- Select map and difficulty in the lobby
- Start the mission
- (Maybe) kick players

These are UI/lobby actions, not simulation actions. The simulation authority is always the mission server.

**Important:** The room owner is a privilege flag on any `NetworkId`. It is NOT tied to player numbering — Player 1 does
not need to be the owner. Any connected player can hold the owner flag.

**Owner only matters in the lobby, not during missions.** When a mission is ongoing and the owner disconnects, the
mission continues unaffected. There is no disruption. The owner role only becomes relevant when players return to the
room lobby (e.g., to select the next map or start another mission).

**When the room owner disconnects:**

The next connected player (by join order) is automatically assigned as the new room owner. This happens silently. If the
original owner reconnects later, they join as a regular player — there is no automatic re-promotion.

### What Changes in the Current Protocol

**Minimal changes needed:**

1. **New `NetMode` variant** (or a flag): `NetMode::Dedicated { port, bind }` — like `Host` but with no local player.
   Alternatively, reuse `Host` with a `headless: bool` flag (already planned in Phase 1).

2. **Room owner tracking:** The mission server needs to know which `NetworkId` is the lobby leader. Currently the host
   player is implicitly the leader because they run the host instance. In dedicated mode, the first connected client (or
   the one the meta-server designates) becomes the leader.

3. **Lobby actions need authority checks:** `host_apply_input_system` currently accepts lobby actions (map selection,
   start mission) from any client because only the host runs them. In dedicated mode, these actions arrive as network
   messages and must be checked against the room owner's `NetworkId`.

4. **Meta-server messages:** New message types between meta-server and client:
   - `CreateRoom` → `RoomCreated { code, address }`
   - `JoinRoom { code }` → `RoomFound { address }` / `RoomNotFound`
   - `ListRooms` → `RoomList { rooms: Vec<RoomInfo> }` (optional, for a server browser)

   These are meta-server protocol messages, NOT part of the mission server's `NetworkMessage` enum. Different
   connection, different protocol.

### The Current Host-Client Model Stays Intact

The mission server process, from its own perspective, is just a `NetMode::Host` instance. It runs the exact same
systems, snapshot broadcasting, input processing. The only difference is:

- No `MainPlayer` entity (no local input, no camera)
- No rendering, audio, or UI plugins
- Room owner is tracked by `NetworkId` instead of being implicit

Clients connecting to a dedicated server see no difference from connecting to a player-hosted game. The `NetworkMessage`
protocol is identical. The snapshot format is identical. The join/late-join flow is identical.

**From the player's perspective:** They join a room via code, end up in a lobby, someone picks a map, they play the
mission. Whether the simulation runs on another player's machine or on a VPS is invisible.

---

## Risk Register

| Risk                                                                      | Severity | Likelihood | Mitigation                                                                                                                        |
| ------------------------------------------------------------------------- | -------- | ---------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `Material2dPlugin` panic without render pipeline                          | High     | High       | Don't load `UnhaunterRenderPlugin`'s material setup; ensure no code path creates material handles on server                       |
| Systems querying `Camera` components with no camera                       | Medium   | Medium     | Gate systems behind `is_headless`; provide no-op fallbacks for systems that optionally query cameras                              |
| `SoundEmitter` (SystemParam) used in ghost AI but audio plugin not loaded | Medium   | High       | `SoundEmitter` wraps `AssetServer` for audio loading — server needs stub or skip audio playback calls                             |
| Client RNG divergence in sound grid → different `mean_sound`              | Low      | Certain    | Accepted — smoothed average converges; co-op game                                                                                 |
| Thermal grid divergence between clients                                   | Low      | Certain    | Accepted — each player reads own instruments; evidence authority is server-side                                                   |
| Late joiner thermal grid starts warm in frozen room                       | Low      | Certain    | Converges within 5–10 seconds from emitter power; can seed room temps on join if needed                                           |
| Player sanity cheating via `PlayerInput`                                  | Low      | Low        | Accepted for co-op; add bounds checking later if needed                                                                           |
| Spirit box charge desync between clients                                  | Medium   | Likely     | Keep `ghost_answer` server-authoritative; charge is display-only                                                                  |
| Ghost visibility bias lost (VisibilityData empty)                         | Low      | Certain    | Acceptable degradation — interactions still fire, just without "dramatic" bias; can add `Viewer` to all players on server cheaply |
| `AnimationTimer` queried in snapshot but animation systems not loaded     | Medium   | Medium     | Keep `AnimationTimer` component on entities; skip animation tick systems                                                          |
| `bevy_asset_loader` `LoadingState` requires all asset collections present | Medium   | Medium     | Register ghost/player/gear assets even on server (they're image handles but won't be rendered)                                    |

---

## Summary

The thin server model works because:

1. Ghost AI doesn't read environmental grids (verified)
2. Environmental grids are deterministically derivable from synced data
3. Evidence authority is already separate from environmental readings
4. The game is co-operative, relaxing the security model
5. The snapshot already carries all the inputs clients need

The server becomes: **ghost brain + interaction arbiter + network hub**. CPU and memory costs drop to near-zero per
instance.

---

## Appendix: External Review Findings

Three concerns were raised by external analysis. Each was verified against the codebase:

### A. Gear Position Sync on Server — NOT a problem

**Claim:** The server doesn't update held gear positions to match the holding player, causing flashlights to appear
stuck at `(0,0,0)` for remote observers.

**Verification:** `sync_held_gear_position` in `unplayer-plugin/systems/grabdrop.rs` runs on **all instances** (no
`.run_if(is_host)` gating). It copies the player's `Position` to all held gear entities every frame. This system runs on
the server, on the host, and on all clients. The server's `host_send_snapshots_system` reads the gear's `Position`
component (already updated) and includes it in `GearSyncState`.

The app_setup in `grabdrop.rs` registers:

```rust
app.add_systems(Update, (sync_held_gear_position, update_held_object_position));
app.add_systems(Update, (grab_object, drop_object, ...).run_if(is_host));
```

Position sync is ungated. Only grab/drop mutation is host-gated. **No issue.**

### B. PlayerInput→PlayerSprite Sanity Propagation — Obvious implementation detail

**Claim:** There is no code that copies `PlayerInput.sanity` to `PlayerSprite.sanity` on the server.

**Verification:** This is correct — because `PlayerInput` doesn't have `sanity` or `mean_sound` fields yet. Those fields
are part of Phase 1 of the implementation plan. When we add them to `PlayerInput`, we also need a server-side system to
copy `PlayerInput.sanity` → `PlayerSprite.sanity` and `PlayerInput.mean_sound` → `PlayerSprite.mean_sound`. This is a
trivial 5-line system. It's an implementation detail, not a design gap.

Currently `host_apply_input_system` in `host_input.rs` copies all `PlayerInput` fields from the network message to the
component. When `sanity`/`mean_sound` are added, the same function will copy those too, and a new system will propagate
them to `PlayerSprite`.

### C. Late Joiner Initial Lighting — Already handled

**Claim:** A late joiner might see darkness in a lit house because `RoomStateSyncEvent` only fires on room state
_changes_, and if the local defaults match the server, no event fires and no light rebuild happens.

**Verification:** This is wrong. The initialization chain handles this correctly:

1. `level_finalization::after_level_ready` fires both `RoomStateSyncEvent` AND `RoomChangedEvent` unconditionally on
   first load (including late join).
2. `prebake_lighting_on_level_ready` fires on `LevelReadyEvent` and runs the full lighting prebake.
3. `roomchanged_event` (in `unclassic-mode-plugin`) fires `BoardTopologyToRebuild` on `RoomChangedEvent`, which triggers
   `rebuild_lighting_field`.

Late joiners go through the same `after_level_ready` → `RoomStateSyncEvent` → `room_state_sync_system` →
`BoardTopologyToRebuild` → `rebuild_lighting_field` chain.

Additionally, the first snapshot includes ALL room states. Rooms default to `Off`; any server room that is `On` will
differ from the client's default, triggering `RoomStateSyncEvent`. Even if the first snapshot arrives after
`after_level_ready` has already fired, it provides a second sync pass.

If the server also has all rooms `Off`, no event fires — but that's correct (the map is dark on both sides, e.g.,
breaker is off). **No issue.**

### D. Map Plugin Rendering Dependency — Valid concern (already in risk register)

**Claim:** `UnhaunterTmxMapPlugin` / `UnhaunterMapLoadPlugin` is heavily coupled to rendering. `process_and_spawn_tile`
creates `MeshMaterial2d<CustomMaterial1>` handles. On a headless server without `Material2dPlugin<CustomMaterial1>`, the
`Assets<CustomMaterial1>` resource won't exist and the `LoadLevelSystemParam` SystemParam will panic.

**Verification:** This is correct. `LoadLevelSystemParam` requires `ResMut<Assets<CustomMaterial1>>` and
`ResMut<Assets<Mesh>>`. These resources are initialized by `Material2dPlugin<CustomMaterial1>` (registered in
`unrender-plugin`) and Bevy's rendering pipeline respectively.

On a headless server that skips the render plugin, the SystemParam will fail to resolve. The `process_and_spawn_tile`
function also reads material assets, clones them, and creates new handles — all of which depend on the material asset
storage being initialized.

This was already identified in the risk register as the highest-severity risk ("`Material2dPlugin` panic without render
pipeline"). The mitigation is:

- **Option 1:** Register `Assets<CustomMaterial1>` and `Assets<Mesh>` as empty resources on the headless server (via
  `app.init_resource::<Assets<CustomMaterial1>>()`). The materials are never rendered, but the handles exist and don't
  panic. The `.get()` call on line 71 of `tile_spawning.rs` would return `None` and panic — this needs a fallback.
- **Option 2 (better):** Gate the visual parts of `process_and_spawn_tile` behind `is_headless`. The server only needs
  the logical components (`Behavior`, `Position`, `BoardPosition`, `HydrationStage`, etc.), not the `MeshMaterial2d`,
  `Transform`, `Visibility`, or `GameSprite` components.

This is implementation work for Phase 3 (Split mixed plugins). Not a design gap — it's an acknowledged part of the plan.

### E. NPC Plugin on Server — Low risk, easy to skip

**Claim:** `UnhaunterNPCPlugin` is almost entirely UI and will break on a headless server.

**Verification:** The NPC plugin has two parts:

- `hydration_npc_system`: Inserts `Pickable`, `NpcHelpDialog`, `Interactive`, and `FloorItemCollidable` components on
  NPC entities. These are plain ECS components with no rendering dependency. `Pickable` is inert without a picking
  backend.
- `npchelp` UI systems: `setup_ui` only runs `OnEnter(GameState::NpcHelp)`. A dedicated server never enters this state.
  `UIPanelMaterial` is used only in `setup_ui`.

The NPC plugin is safe to keep on the server (the UI code paths are never reached). However, it's also safe to move to
the SKIP list since NPC interactions on a dedicated server are not meaningful — there's no local player to trigger them.
**Recommendation: move to SKIP list for cleanliness.**
