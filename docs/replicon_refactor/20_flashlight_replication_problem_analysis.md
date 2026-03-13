# 20 — Flashlight Replication Problem Analysis

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Purpose:** Capture everything known about the "join clients can't see other players' flashlights" bug, what the logs
  have revealed, what the open design questions are, and what the agreed-upon design intent is.

Note: This analysis is superseded by the implementation plan in 21_gear_skeleton_skin_design.md.

---

## 1. The Symptom

On a join client (pure client, not the host), the flashlights of other players are invisible — they produce no light.
The local player's own flashlight works correctly.

---

## 2. What the Debug Logs Show

### Server (FLASHLIGHT-AUTH, every 5 s)

```text
gear=2323v0 status=Mid toggle=true power=10.00 | holder=2326v0 pos=(21.9,10.4,1.0) dir=(340.97,115.48)
gear=2328v0 status=Mid toggle=true power=10.00 | holder=2331v0 pos=(27.3,11.1,1.0) dir=(...)
```

**Server state is correct.** Both flashlights are `status=Mid`, `toggle=true`, with valid position and direction. The
server is doing its job.

### Join Client (FLASHLIGHT-CLIENT, every 5 s)

```text
gear=57v0  status=Off  toggle_on=false  power=0.00  => RENDERABLE=false   (t=08:01:40)
gear=57v0  status=Off  toggle_on=true   power=0.00  => RENDERABLE=false   (t=08:01:50)  ← brief window
gear=57v0  status=Off  toggle_on=false  power=0.00  => RENDERABLE=false   (t=08:01:55)
```

Key observations:

- `Flashlight.status` is always `Off` on the client for remote gear. It never changes.
- `Toggleable.is_on` flickers: occasionally arrives as `true` (replication packet delivered it), then drops back to
  `false` within seconds.
- `LightEmitter.power` is always `0.00` even when `toggle_on` momentarily becomes `true`.

### Player entities on the client (PLAYER, every 2 s)

The remote player whose UUID maps to `ea39e84e-...` shows up as **multiple entities with the same owner**:

| Entity                 | dir | spr | anim | gear | inp | stam |
| ---------------------- | --- | --- | ---- | ---- | --- | ---- | ------------------------------------- |
| `63v0`                 | ✓   | ✓   | ✓    | ✓    | ✓   | ✓    | — fully hydrated "main" player entity |
| `57v0`, `59v0`, `61v0` | ✗   | ✗   | ✗    | ✗    | ✗   | ✗    | — skeleton-only, only `pos=true`      |

This means there are multiple entities for the same remote player, and only one of them is fully hydrated. The
flashlight gear (`57v0`) is held by `63v0` (the fully hydrated one), so the position/direction lookup in the debug
system does find valid pos+dir — but the gear entity itself is still failing to render.

`fallback_player_ownership_from_uuid` logs keep printing, suggesting ownership assignment for the remote player is still
running periodically — the client keeps trying to find the right entity.

---

## 3. Immediate Technical Cause

`update_flashlight` (in `ungearitems-plugin`) runs every frame on **all** entities that have a `Flashlight` component,
without any gate for whether the entity is locally owned or remote.

Every frame it writes:

```rust
toggle.is_on = flashlight.status != FlashlightStatus::Off;
flashlight_render.power = flashlight.output_power;
```

Since `Flashlight.status` is **not replicated**, it defaults to `Off` after client hydration and never changes for
remote gear. This means every frame the system forcefully sets `toggle.is_on = false` and `power = 0`, overwriting
whatever replication just delivered.

The replication of `Toggleable` does occasionally arrive as `true`, but it is overwritten almost immediately by this
system on the very next frame.

---

## 4. Design Decisions (confirmed by user)

### A1 — Authority model: confirmed

> Each client runs its **own** full gear simulation for **its own** gear. The server does not run gear simulation at all
> — it only stores a thin "skeleton" state (on/off, position, direction). Other clients receive that skeleton state and
> are expected to render from it, but they do **not** run or receive the full internal simulation of remote gear.

**Additional nuance:** Each client also runs the skin simulation for **remote** gear — i.e. for another player's
flashlight, the viewing client is responsible for computing the heat effects, sprite animation, audio, etc. The server
only provides the skeleton; the client provides the skin locally.

`Toggleable.is_on` alone is insufficient for flashlights because they are multi-state (Off/Low/Mid/High). The full
`Flashlight.status` is needed so the client can derive power and visual state correctly.

### A2 — `update_flashlight` must run on remote gear: confirmed

Remote clients run the skin simulation for remote gear. `update_flashlight` should run on all gear — local and remote.
The current code does this (no gate). The problem is not that the system runs; the problem is that it has no valid
skeleton input (`Flashlight.status` is never delivered to the client), so it always produces wrong output.

### A3 — `LightEmitter.power` authority: replicate `Flashlight.status` (option c)

The client should compute `LightEmitter.power` from the replicated `Flashlight.status`, identical to how it treats
locally-owned gear. This is the cleanest approach because `update_flashlight` already contains the correct computation;
it just needs the right skeleton input.

### A4 — Multiple entities in the PLAYER debug log: this is a diagnostic artifact

The `debug_player_entities` system queries `Query<(Entity, &Owner, ...)>` **without any filter for player-vs-gear
entities**. Gear entities also have `Owner` + `Position` + `Replicated`, so they appear in the output. The four
"entities" for the remote player are:

- `63v0` — the actual player entity, fully hydrated (has `PlayerSprite`, `PlayerGear`, `Direction`, etc.)
- `57v0`, `59v0`, `61v0` — the remote player's gear entities (flashlight, thermometer, inventory item), which only have
  `Owner` + `Position` after replication and show up as skeleton-stubs in the query

This is **not a spawning bug**. The join client correctly does not spawn any entities of its own
(`setup_mission_players` is gated `run_if(resource_exists::<AuthorityRole>)` and join clients have no `AuthorityRole`).
The fix is to add `With<PlayerSprite>` (or similar) to the `debug_player_entities` query so gear entities don't pollute
the output.

`fallback_player_ownership_from_uuid` keeps logging because it runs a periodic diagnostic every 3 seconds looking for
unowned player entities. The local player (`64v0`) is already `LocallyOwned` and `Without<Replicated>`, so the query
(filtered `Without<LocallyOwned>`) correctly never matches it. The system is working but the log is noisy. Not a bug.

### A5 — Skeleton/skin split: confirmed design direction

This flashlight fix is the **first step** of a larger reorganization. All gear is effectively broken in multiplayer for
the same underlying reason. The skeleton/skin split applies across all gear types.

We are free to reorganize in whatever order makes sense — including breaking things and fixing them properly. The
flashlight is a good place to start because it has the most visible and testable effect.

---

## 5. Precise Root Cause

`Flashlight.status` is the skeleton field that drives the entire gear state machine. It is:

- Written locally on the owning client by user input (via `Triggered` → `update_flashlight`)
- Exported to the server via `send_export_gear_state` → `ExportGearStateMessage`
- Applied on the server by `handle_export_gear_state` → `flashlight.status = Mid` etc.
- **NOT replicated** — `app.replicate::<Flashlight>()` is never called

Because `Flashlight.status` never reaches other clients, `update_flashlight` on those clients always reads `status=Off`
(the default after hydration) and writes `toggle.is_on=false` and `power=0` every frame — stomping whatever
`Toggleable.is_on=true` replication occasionally delivers.

---

## 6. Known Secondary Issue: Server Running Skin Simulation for Remote Gear

`update_flashlight` has no `run_if` gate and runs everywhere — including on the dedicated server. The dedicated server
spawns gear for all players via `gear_registry.spawn()` (full bundle including `Battery`, `Electronic`, `Flashlight`,
`LightEmitter`, etc.). This means:

- Server runs battery drain for remote clients' gear
- Server computes temperatures for remote clients' gear
- When we replicate `Flashlight`, the server will replicate its own (server-simulated, potentially divergent) skin state
  alongside the skeleton

This is a pre-existing issue and **does not block the immediate fix** — the client's local `update_flashlight`
overwrites skin fields immediately on the next tick anyway. But it is the next thing to address in the reorganization:
the server should not run the skin simulation for remote clients' gear.

---

## 7. Proposed Fix

### Minimal (immediate)

In `unreplicon-plugin/src/systems/players.rs`:

1. Add `app.replicate::<Flashlight>()` — this delivers `Flashlight.status` (and skin fields, accepted as a temporary
   impurity) to all clients.
2. Add `app.set_marker_fns::<LocallyOwned, Flashlight>(noop_write::<Flashlight>, noop_remove)` — this ensures that
   locally-owned gear ignores the server's replication of `Flashlight`, preventing the server from overwriting the
   owning client's locally-computed skin state.

**Why this works:** The server receives `ExportGearStateMessage` from the join client, sets `flashlight.status = Mid` on
the gear entity, and replicates it. Viewing clients receive `Flashlight { status: Mid, ... }`. Their local
`update_flashlight` reads `status=Mid`, computes `power=16.0`, sets `toggle.is_on=true`. Light renders.

**The skin stomping:** The server also replicates temperatures etc., but the viewing client's `update_flashlight` runs
from the correct skeleton immediately and re-derives skin from scratch — the server's skin values are ignored within one
frame.

### Broader reorganization (next steps)

1. Stop the server from running `update_flashlight` on remote gear (gate with
   `Or<(With<LocallyOwned>, Without<Replicated>)>` or similar, server-side).
2. Fix `debug_player_entities` query to filter on `With<PlayerSprite>` to avoid gear entity noise.
3. Consider splitting `Flashlight` into `FlashlightSkeleton { status }` (replicated) and keeping the rest as
   unreplicated skin — cleaner architecture, avoids skin-stomping entirely.
4. Audit all other gear types for the same skeleton/skin problem.

---

## 8. What We Must NOT Do

- Do not gate `update_flashlight` with `With<LocallyOwned>` — remote clients must run it for skin simulation.
- Do not replicate `Flashlight` without also adding the `LocallyOwned` noop guard — the owning client must not have its
  local skin simulation overwritten by the server.
- Do not chase the "multiple player entities" as a spawning bug — they are gear entities appearing in the wrong query.
- Do not add skin fields (temperatures, output_power) to `ExportGearStateMessage` — those are client-simulated.
