# 22 — Gear Replication: Plan of Record

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Design reference:** [21_gear_skeleton_skin_design.md](21_gear_skeleton_skin_design.md)
- **Purpose:** Actionable, ordered task list to bring all gear replication into the correct skeleton/skin architecture.
  Each task lists the exact files and functions to change.

NOTE: Status complete - not validated. Work is done but might not be doing in practice what was intended.

NOTE: This entire document is now marked as FAILED AND OBSOLETE. The source material used was WRONG when this was
created and the result is disaster a partial hotfix patch that is nearing useless. Dismiss this file entirely.

---

## Guiding Principles (from doc 21)

- **Skeleton** = server-authoritative, replicated via bevy_replicon. Exported from owning client to server via
  `ExportGearStateMessage`.
- **Skin** = client-simulated locally on every node with a human viewport. Never replicated, never sent as messages.
  `resource_exists::<LocalPlayer>()` gates all skin-compute systems.
- **`LocallyOwned`** on a gear entity means: this node (the owning client) drives the skeleton for this entity. Server
  writes (incoming replicated packets) are noop'd.
- `With<LocallyOwned>` on gear queries is the **only** correct filter for skeleton-write systems. It does NOT mean "run
  on servers" or "run on authority" — it means "run for gear this node owns."

---

## Task 0 — Grant `LocallyOwned` to Gear Entities

**Prerequisite for all other tasks.** Currently gear entities are never given `LocallyOwned`, so `With<LocallyOwned>` on
gear queries matches nothing and all noop registrations for gear components are dead code.

**Files to change:**

- `crates/unreplicon-plugin/src/systems/players.rs`

**Functions to change:**

### `handle_ownership_granted` (line ~989)

When `LocallyOwned` is inserted on the player entity, also insert it on all gear entities in their `PlayerGear`. The
player entity reference is already available (`client_entity`). The system needs to also query for the player's
`PlayerGear` component and iterate `left_hand`, `right_hand`, and all `inventory` slots.

```rust
// After: commands.entity(client_entity).insert(LocallyOwned);
// Also grant LocallyOwned to all gear entities this player holds.
if let Ok(gear) = q_player_gear.get(client_entity) {
    for gear_entity in gear.all_slots() {
        commands.entity(gear_entity).insert(LocallyOwned);
    }
}
```

The system currently does not have a `PlayerGear` query — add one.

### `setup_mission_players` (line ~244)

The host player path (the `is_host` branch, line ~345) inserts `LocallyOwned` on the player entity directly. Also insert
on all of that player's gear entities at the same time, since they are already known (they were just spawned and placed
into `player_gear` in the lines above).

### `spawn_late_joining_players` (line ~392)

Same pattern as `setup_mission_players`. If a late joiner is the host, grant `LocallyOwned` on their gear immediately.

### `fallback_player_ownership_from_uuid` (line ~930)

The fallback path inserts `LocallyOwned` on the player entity. Also insert on all gear entities. This function already
has commands access — add a `PlayerGear` query and iterate its slots.

**Note on `PlayerGear.all_slots()`**: Check whether this helper exists on `PlayerGear`. If not, inline the iteration:
`left_hand.into_iter().chain(right_hand).chain(inventory.iter().copied()).flatten()`.

---

## Task 1 — Split and Replicate `Flashlight`

**Fixes: Flashlight producing no light on remote clients, and server stomping local skin state.**

1. **Split the Component:** In `crates/ungearitems-core/src/components/flashlight.rs`, split `Flashlight` into:
   - `Flashlight` (Skeleton, Replicated): Contains ONLY `status: FlashlightStatus`.
   - `FlashlightSkin` (Skin, Not Replicated): Contains `inner_temp`, `heatsink_temp`, `frame_counter`, `rand`,
     `output_power`.
2. **Update Spawning:** Ensure `FlashlightSkin` is added to the gear bundle on clients, but NOT on the dedicated server.
3. **Register Replication:** In `players.rs`, replicate `Flashlight` and set the `LocallyOwned` noop marker.
4. **Split the Systems:**
   - `update_flashlight_skeleton`: Query `With<LocallyOwned>`. Handles trigger input -> writes `Flashlight.status`.
   - `update_flashlight_skin`: Query `(&Flashlight, &mut FlashlightSkin, ...)`. Runs under
     `resource_exists::<LocalPlayerRole>()`. Reads `status` from the skeleton, computes and stores state in
     `FlashlightSkin`, and writes to `LightEmitter`.

### `crates/ungearitems-core/src/components/flashlight.rs`

Add `Serialize, Deserialize` to `FlashlightStatus` enum (it may already have them — verify). Add
`Serialize, Deserialize` to the `Flashlight` struct derive list.

### `crates/ungearitems-plugin/src/components/flashlight.rs`

Split `update_flashlight` (line ~18) into two functions:

**`update_flashlight_skeleton`** — skeleton-write phase:

- Query: all gear with `With<LocallyOwned>` + `With<Flashlight>`
- Handles: `Triggered` input → advances `flashlight.status`. Keep the `glitch_timer <= 0.0` guard.
- Also handles: battery-dead auto-Off, overheat auto-Off. These write `flashlight.status` so they belong here.
- Does **not** write to `LightEmitter`, `GearSprite`, `StatusText`, `Battery.drain_rate`.
- Does **not** call `flashlight.update_output_power(...)`.

**`update_flashlight_skin`** — skin-compute phase:

- Query: all gear with `With<Flashlight>` (no LocallyOwned filter — remote gear needs skin too)
- Reads `flashlight.status` and `electronic.glitch_timer/glitch_intensity`
- Computes and writes: `output_power`, `LightEmitter.power`, `LightEmitter.color`, `GearSprite`, `StatusText`
- Also updates skin simulation: `Battery.drain_rate`, `inner_temp`, `heatsink_temp`, `frame_counter`

Register in the plugin's `app_setup` (wherever `update_flashlight` is currently registered):

```rust
app.add_systems(Update, (
    update_flashlight_skeleton,   // entity filter handles it; no resource gate
    update_flashlight_skin.run_if(resource_exists::<LocalPlayerRole>()),
));
```

**Verification:** After this task, a Join client should see the flashlight light up on remote players. Check with the
existing `debug_flashlight_client_state` system.

---

## Task 2 — Replicate `UVTorch`

**Fixes: No UV light visible from remote players.**

Same pattern as Task 1. `UVTorch.enabled: bool` is the skeleton field.

**Files to change:**

### `crates/ungearitems-core/src/components/uvtorch.rs`

Add `Serialize, Deserialize, Reflect` to `UVTorch` derive list.

### Task 2 — `crates/unreplicon-plugin/src/systems/players.rs`

```rust
app.replicate::<ungearitems_core::components::uvtorch::UVTorch>();
app.set_marker_fns::<LocallyOwned,
    ungearitems_core::components::uvtorch::UVTorch>(noop_write, noop_remove);
```

### `crates/ungearitems-plugin/src/components/uvtorch.rs`

Split `update_uvtorch` (line ~39) into:

**`update_uvtorch_skeleton`** — `With<LocallyOwned>`: handles `Triggered` → toggles `uvtorch.enabled`. Keep
`glitch_timer <= 0.0` guard if present.

**`update_uvtorch_skin`** — no filter, `run_if(resource_exists::<LocalPlayerRole>())`: reads `uvtorch.enabled`, computes
`output_power`, writes `LightEmitter`, `GearSprite`, `StatusText`, `Battery.drain_rate`.

---

## Task 3 — Replicate `RedTorch`

**Fixes: No red light visible from remote players.**

Identical pattern to Task 2. `RedTorch.enabled: bool` is the skeleton field.

**Files to change:**

### `crates/ungearitems-core/src/components/redtorch.rs`

Add `Serialize, Deserialize, Reflect` to `RedTorch` derive list.

### Task 3 — `crates/unreplicon-plugin/src/systems/players.rs`

```rust
app.replicate::<ungearitems_core::components::redtorch::RedTorch>();
app.set_marker_fns::<LocallyOwned,
    ungearitems_core::components::redtorch::RedTorch>(noop_write, noop_remove);
```

### `crates/ungearitems-plugin/src/components/redtorch.rs`

Split `update_redtorch` (line ~16) into `update_redtorch_skeleton` and `update_redtorch_skin` using the same pattern as
Task 2.

---

## Task 4 — Fix `Videocam`

**Fixes: No IR light visible from remote players.**

`Videocam` does not need a new replicated component. `Toggleable.is_on` is sufficient (binary, no multi-state enum). The
problem is that `update_videocam` currently runs on all nodes and stomps `Toggleable.is_on` on remote gear.

**Files to change:**

### `crates/ungearitems-plugin/src/components/videocam.rs`

Split `update_videocam` (line ~16) into:

**`update_videocam_skeleton`** — `With<LocallyOwned>`: handles `Triggered` → sets/clears `toggle.is_on`. That is the
only skeleton write; `Videocam` component itself has no skeleton fields beyond what `Toggleable` already provides.

**`update_videocam_skin`** — no filter, `run_if(resource_exists::<LocalPlayerRole>())`: reads `toggle.is_on`, computes
`output_power`, writes `LightEmitter`, `GearSprite`, `StatusText`, `Battery.drain_rate`.

No changes to `unreplicon-plugin` — `Toggleable` is already replicated with a noop registered.

---

## Task 5 — Expand `ExportGearStateMessage`

**Fixes: Only flashlight skeleton is exported to the server. All other gear stays at defaults on the server and is never
replicated to other clients.**

**Files to change:**

### `crates/unreplicon-core/src/messages.rs` (or wherever `ExportGearStateMessage` is defined)

Remove `battery: f32` and `temperature: f32` fields. The struct becomes:

```rust
pub struct ExportGearStateMessage {
    pub entity: Entity,
    pub status_level: u8,  // 0 = off/disabled, >0 = on/enabled or mode level
}
```

### `crates/unreplicon-plugin/src/systems/players.rs`

**`send_export_gear_state`** (line ~505): Expand to iterate all gear kinds in `PlayerGear`. For each gear entity,
determine `status_level` by inspecting the component for that `GearKind`:

| GearKind       | How to determine `status_level`                        |
| -------------- | ------------------------------------------------------ |
| Flashlight     | `FlashlightStatus::Off→0, Low→1, Mid→2, High→3`        |
| UVTorch        | `if enabled { 1 } else { 0 }`                          |
| RedTorch       | `if enabled { 1 } else { 0 }`                          |
| Videocam       | `if toggle.is_on { 1 } else { 0 }`                     |
| All sensors    | `if toggle.is_on { 1 } else { 0 }`                     |
| RepellentFlask | encode `qty` (0 = empty/consumed)                      |
| Salt           | encode `charges`                                       |
| QuartzStone    | encode `cracks`                                        |
| Sage           | `if consumed { 0 } else if is_active { 2 } else { 1 }` |

Use `GearKind` to dispatch — query each optional component per entity in the iterator.

**`handle_export_gear_state`** (line ~536): Expand the receiver to dispatch writes based on `GearKind`. Read `GearKind`
from the entity and match:

```rust
match gear_kind {
    GearKind::Flashlight => { flashlight.status = decode_flashlight(new_level); toggleable.is_on = new_level > 0; }
    GearKind::UVTorch    => { uvtorch.enabled = new_level > 0; toggleable.is_on = new_level > 0; }
    GearKind::RedTorch   => { redtorch.enabled = new_level > 0; toggleable.is_on = new_level > 0; }
    // ... etc.
    _                    => { toggleable.is_on = new_level > 0; }
}
```

---

## Task 6 — Replicate Consumables

**Fixes: Consumable state (qty, charges, active) never visible on remote clients.**

These are lower priority (no light rendering impact) but necessary for gameplay correctness.

### `crates/ungearitems-core/src/components/repellentflask.rs`

Add `Serialize, Deserialize, Reflect` to `RepellentFlask`.

### `crates/ungearitems-core/src/components/salt.rs`

Add `Serialize, Deserialize, Reflect` to `SaltData`.

### `crates/ungearitems-core/src/components/sage.rs`

Add `Serialize, Deserialize, Reflect` to `SageBundleData`. Check which struct holds `is_active` and `consumed` — there
are multiple structs in this file (line 4, 27, 30).

### `crates/ungearitems-core/src/components/quartz.rs`

Add `Serialize, Deserialize, Reflect` to `QuartzStoneData`.

### Task 5 — `crates/unreplicon-plugin/src/systems/players.rs`

Register replication and noops for each:

```rust
app.replicate::<RepellentFlask>();
app.set_marker_fns::<LocallyOwned, RepellentFlask>(noop_write, noop_remove);

app.replicate::<SaltData>();
app.set_marker_fns::<LocallyOwned, SaltData>(noop_write, noop_remove);

app.replicate::<SageBundleData>();
app.set_marker_fns::<LocallyOwned, SageBundleData>(noop_write, noop_remove);

app.replicate::<QuartzStoneData>();
app.set_marker_fns::<LocallyOwned, QuartzStoneData>(noop_write, noop_remove);
```

### `crates/ungearitems-plugin/src/components/{repellentflask,salt,sage,quartz}.rs`

Split each `update_*` system the same way as Tasks 1–4: skeleton writes under `With<LocallyOwned>`, skin computes under
`resource_exists::<LocalPlayerRole>()`.

---

## Task 7 — Split All Remaining `update_*` Systems

**Fixes: Server running full gear simulation (battery drain, thermal, sensor reads) for all players' gear, including
remote clients' gear which the server has no business simulating.**

The remaining systems that still need splitting after Tasks 1–6:

| System                 | File                          | Skeleton writes?   |
| ---------------------- | ----------------------------- | ------------------ |
| `update_thermometer`   | `components/thermometer.rs`   | None (sensor only) |
| `update_emfmeter`      | `components/emfmeter.rs`      | None (sensor only) |
| `update_geigercounter` | `components/geigercounter.rs` | None (sensor only) |
| `update_recorder`      | `components/recorder.rs`      | None (sensor only) |
| `update_spiritbox`     | `components/spiritbox.rs`     | None (sensor only) |
| `update_compass`       | `components/compass.rs`       | None (sensor only) |
| `update_ionmeter`      | `components/ionmeter.rs`      | None (sensor only) |
| `update_estaticmeter`  | `components/estaticmeter.rs`  | None (sensor only) |
| `update_thermalimager` | `components/thermalimager.rs` | None (sensor only) |
| `update_motionsensor`  | `components/motionsensor.rs`  | None (sensor only) |
| `update_photocam`      | `components/photocam.rs`      | None (sensor only) |

All of these are **skin-only**: they read world grids and produce sensor readings displayed on screen. They have no
skeleton-write phase. For each one:

- **No split needed** in the code structure — there is no skeleton-write half to separate.
- The entire function becomes the skin-compute phase: register it with `.run_if(resource_exists::<LocalPlayerRole>())`.

The sensor `update_*` functions live in `crates/ungearitems-plugin/src/`. The system registrations are in the plugin's
`systems/mod.rs` or equivalent `app_setup`. Add the `run_if` gate to each registration there — no change to the function
bodies.

---

## Task 8 — Remove Dead `LocallyOwned` Noop Registrations for Gear

After Task 0 grants `LocallyOwned` to gear entities, the existing dead noop registrations for `GearMarker` and
`GearKind` become live. They are correct — leave them. Verify they do not cause unexpected behavior once gear entities
actually have `LocallyOwned`.

```rust
// These are already in players.rs and become active after Task 0:
app.set_marker_fns::<LocallyOwned, GearMarker>(noop_write::<GearMarker>, noop_remove);
app.set_marker_fns::<LocallyOwned, GearKind>(noop_write::<GearKind>, noop_remove);
```

---

## Task 9 — Remove Debug Systems (cleanup after verification)

These debug systems were added to diagnose the flashlight bug and should be removed once Tasks 0–3 are verified working
in a multiplayer session:

- `debug_flashlight_client_state` — pure-client-only flashlight dump every 5 seconds
- `debug_flashlight_gear_state` — authority-side flashlight dump every 5 seconds
- The throttled `Local<HashMap<Entity, u8>>` logging in `handle_export_gear_state`

All in `crates/unreplicon-plugin/src/systems/players.rs`.

---

## Task 10 — Audit and Prevent Rogue Client Spawning

**Fixes: Duplicate entities caused by non-authority clients spawning mission/gear entities.**

1. **Audit:** Identify all systems that spawn gear or mission entities.
2. **Gate:** Ensure all spawning systems are gated by `AuthorityRole`.
3. **Enforce:** Add a debug check that logs a `CRITICAL` error if a non-authority client attempts to spawn a mission
   entity.

---

## Execution Order and Dependencies

```text
Task 0  (LocallyOwned on gear)            — prerequisite for Tasks 1, 2, 3, 4, 5, 6, 7
Task 1  (Flashlight replication)          — depends on Task 0; independently testable
Task 2  (UVTorch replication)             — depends on Task 0; independently testable
Task 3  (RedTorch replication)            — depends on Task 0; independently testable
Task 4  (Videocam system split)           — depends on Task 0; no new replication needed
Task 5  (ExportGearStateMessage expand)   — depends on Tasks 1–4 being verified; expand sender/receiver
Task 6  (Consumable replication)          — depends on Task 0; independently testable; lower priority
Task 7  (Sensor system gates)             — no dependency on other tasks; lowest risk; can be done first
Task 8  (Dead noop verification)          — do after Task 0 is live
Task 9  (Debug cleanup)                   — do last, after multiplayer verification
```

Tasks 1, 2, 3, 4 can be done in any order or in parallel. Task 5 should be done after at least Tasks 1–3 to avoid
regressions in the only working export path. Task 7 is safe to do any time as it only adds `run_if` conditions with no
logic changes.

---

## Verification Checklist

After completing all tasks, in a multiplayer session (Host + at least one Join client):

- [ ] Join client sees flashlight light on remote player when host turns it on
- [ ] Join client sees flashlight cycle through Low / Mid / High correctly
- [ ] Join client sees flashlight off when host turns it off
- [ ] Join client sees UV torch light on remote player
- [ ] Join client sees red torch light on remote player
- [ ] Join client sees IR light (videocam) on remote player
- [ ] No light leaks: remote gear starts off on Join client before host turns it on
- [ ] Offline mode: flashlight still works (Offline = AuthorityRole + LocalPlayerRole — must not regress)
- [ ] PeerHost mode: host's own flashlight still works (same concern)
- [ ] Server CPU: battery drain not running on dedicated server for remote clients' gear
- [ ] Truck UI: partner sanity value updates when partner loses sanity (via ExportStateMessage path)
