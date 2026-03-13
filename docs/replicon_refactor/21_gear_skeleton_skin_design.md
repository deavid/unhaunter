# 21 — Gear Skeleton/Skin Design & Multiplayer Replication Plan

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Purpose:** Canonical reference for the skeleton/skin split across all gear types. This document defines what must be
  replicated (skeleton), what must be simulated locally per-client (skin), and the full reorganization plan needed to
  make all gear work correctly in multiplayer.

---

## 1. The Core Design Contract

Every piece of gear has two distinct layers:

### Skeleton (server-authoritative, replicated)

The minimum state needed for all clients to know what is happening in the world:

- **Which gear entity exists** — `GearMarker`, `GearKind`, `Owner` (already replicated)
- **Where it is held** — expressed through `PlayerGear` on the player entity (already replicated)
- **Is it on?** — `Toggleable.is_on` (already replicated, but currently being stomped)
- **What operational mode is it in?** — e.g. `FlashlightStatus`, torch enabled/disabled. This is the **missing piece**.

### Skin (client-simulated, never replicated)

Everything a client computes locally for any gear entity it can see — whether that gear is locally owned or remote:

- Internal thermal state (`inner_temp`, `heatsink_temp`)
- Battery level and drain rate
- Frame counters, random seeds, animation state
- Sensor readings (EMF value, temperature reading, sound level, Geiger CPM)
- Output power / light power (derived from skeleton)
- Sprite selection, status text, audio output
- Glitch effects (`Electronic.glitch_timer`, `glitch_intensity`)
- Alert/hint blinking state

**The rule:** If a viewer client needs it purely for display, it is skin. If another player's actions change it and
every client needs to know, it is skeleton.

### The server's role

The server:

- Stores the skeleton for all entities
- Receives skeleton updates from owning clients via `ExportGearStateMessage`
- Replicates skeleton to all other clients
- Does **not** run skin simulation for any gear (no battery drain, no thermal, no sensor reads)

---

## 2. Shared Components: Skeleton vs. Skin Classification

These components are used by most or all gear. Their classification is global.

| Component          | Fields             | Classification | Notes                                                               |
| ------------------ | ------------------ | -------------- | ------------------------------------------------------------------- |
| `Toggleable`       | `is_on`            | **Skeleton**   | Already replicated. Sufficient for binary gear.                     |
| `Battery`          | `level`            | **Skin**       | Client-owned simulation. Server must not track or replicate.        |
| `Battery`          | `drain_rate`       | **Skin**       | Derived from gear mode; computed locally.                           |
| `Electronic`       | `sensitivity`      | **Skeleton**   | Fixed at spawn, never changes.                                      |
| `Electronic`       | `glitch_timer`     | **Skin**       | Ghost interaction delivered separately (event or replicated field). |
| `Electronic`       | `glitch_intensity` | **Skin**       | Derived from `glitch_timer` locally.                                |
| `LightEmitter`     | `power`            | **Skin**       | Derived from skeleton status locally on each client.                |
| `LightEmitter`     | `color`            | **Skin**       | Derived from glitch intensity locally.                              |
| `LightEmitter`     | `light_type`       | **Skeleton**   | Fixed at spawn, never changes.                                      |
| `GearSprite`       | all                | **Skin**       | Display only; derived locally.                                      |
| `StatusText`       | all                | **Skin**       | Display only; derived locally.                                      |
| `PerceivedClarity` | all                | **Skin**       | Display/evidence hint; derived locally.                             |
| `ItemName`         | all                | Static         | Set at spawn, never changes.                                        |
| `ItemDescription`  | all                | Static         | Set at spawn, never changes.                                        |

---

## 3. Per-Gear Skeleton Fields (the missing replicated keys)

These are the operational-mode fields that currently exist only in gear-specific components and are **not replicated**.
Each one must become a skeleton field — either replicated directly or delivered via a `GearStateMessage`.

| GearKind       | Component         | Skeleton field(s)                 | Skin fields                                                            |
| -------------- | ----------------- | --------------------------------- | ---------------------------------------------------------------------- |
| Flashlight     | `Flashlight`      | `status: FlashlightStatus`        | `inner_temp`, `heatsink_temp`, `frame_counter`, `rand`, `output_power` |
| UVTorch        | `UVTorch`         | `enabled: bool`                   | `output_power`                                                         |
| RedTorch       | `RedTorch`        | `enabled: bool`                   | `output_power`                                                         |
| Videocam       | `Videocam`        | _(toggleable only)_               | `output_power`                                                         |
| Thermometer    | `Thermometer`     | _(toggleable only)_               | `temp`, `temp_l2`, `temp_l1`, `frame_counter`, `blinking_hint_active`  |
| EMFMeter       | `EMFMeter`        | _(toggleable only)_               | all fields                                                             |
| GeigerCounter  | `GeigerCounter`   | _(toggleable only)_               | all fields                                                             |
| Recorder       | `Recorder`        | _(toggleable only)_               | all fields                                                             |
| SpiritBox      | `SpiritBox`       | _(toggleable only)_               | all fields                                                             |
| Compass        | `Compass`         | _(toggleable only)_               | _(stateless)_                                                          |
| IonMeter       | `IonMeter`        | _(toggleable only)_               | _(stateless)_                                                          |
| EStaticMeter   | `EStaticMeter`    | _(toggleable only)_               | _(stateless)_                                                          |
| ThermalImager  | `ThermalImager`   | _(toggleable only)_               | _(stateless)_                                                          |
| MotionSensor   | `MotionSensor`    | _(toggleable only)_               | _(stateless)_                                                          |
| Photocam       | `Photocam`        | _(toggleable only)_               | _(stateless)_                                                          |
| RepellentFlask | `RepellentFlask`  | `liquid_content`, `qty`, `active` | _(effects handled via events)_                                         |
| QuartzStone    | `QuartzStoneData` | `cracks: u8`                      | `cracked_time`, `energy_absorbed`                                      |
| Salt           | `SaltData`        | `charges: u8`                     | _(effects handled via events)_                                         |
| Sage           | `SageBundleData`  | `is_active`, `consumed`           | `burn_timer`, `smoke_produced`                                         |

**Key observation:** Most gear only needs `Toggleable.is_on` as its skeleton because they are sensors — they read the
world, they do not change it visibly for other players. The exceptions are:

1. **Light-emitting gear** (Flashlight, UVTorch, RedTorch, Videocam): other clients must know their operational level to
   compute `LightEmitter.power` locally. `Toggleable.is_on` is insufficient for multi-state devices (Flashlight).
2. **Consumables** (RepellentFlask, Salt, Sage, QuartzStone): the consumed/active state matters to other clients for
   visual effects and ghost interactions.

---

## 4. The `ExportGearStateMessage` Pattern (Temporary Bridge)

`ExportGearStateMessage` is a **client→server skeleton batch export** — a **temporary bridge** used while native
component replication is not yet fully wired up for the client-to-server direction. The direction (owning client →
server → other clients via replication) is correct, but this message-based approach is **not** the intended long-term
pattern. See Section 7 for the architectural rationale and replacement plan. **Do not add new fields to this message; it
should be shrunk and eventually replaced, not grown.**

Currently it carries:

```rust
pub struct ExportGearStateMessage {
    pub entity: Entity,
    pub status_level: u8,   // skeleton ✓
    pub battery: f32,       // SKIN — must be removed
    pub temperature: f32,   // SKIN — must be removed
}
```

Two problems:

1. **Skin fields must be removed.** `battery` and `temperature` are skin. The server must never receive or track them.
   The `TODO: battery: 100.0` comment already flags this as known-wrong.
2. **Coverage is flashlight-only.** The message currently only exports `Flashlight.status` via `status_level`. Every
   other gear type with a skeleton field (`UVTorch.enabled`, `RedTorch.enabled`, consumable states) is not covered. The
   message must be expanded to export all gear skeleton state per entity.

`status_level` is skeleton and correct to send. For the flashlight this encodes `FlashlightStatus` as u8. For binary
gear it doubles as `Toggleable.is_on`.

---

## 5. The `update_*` Systems: What Runs Where

Currently every `update_*` system runs on every node (authority, host, dedicated server, pure client) for every gear
entity, with no differentiation between locally-owned and remote gear.

### What must change

Each `update_*` system mixes skeleton-reads, skin-simulation, and output-writes. These must be split:

**For skin simulation (currently all inside `update_*`):**

- Runs on **every client** (local and remote gear both need skin)
- Must read from skeleton fields (e.g. `Flashlight.status`, `Toggleable.is_on`) rather than deriving skeleton
- Must **not** write back to skeleton fields
- The server must **not** run this part

**The server must not run skin simulation at all, and must not replicate skin fields.** Components that currently mix
skeleton and skin fields (like `Flashlight`, `UVTorch`, `RedTorch`) **MUST be split into two separate components**:

1. A replicated skeleton component (e.g., `Flashlight` containing only `status`)
2. A local, non-replicated skin component (e.g., `FlashlightSkin` containing `output_power`, `inner_temp`,
   `frame_counter`, etc.)

_Architectural Rule:_ You cannot replicate a mixed component. Replicating a component that contains local stateful
accumulators (like temperature or smoothed power) will cause the server to constantly overwrite and destroy the client's
local simulation. The bundles must be refactored so the server only holds and replicates the skeleton components.

**Required system split — NOT a simple gate:**

Each `update_*` system must be split into two distinct pieces, because the right execution condition differs:

| Responsibility                 | Runs on                                | Query filter                            | Resource gate                      |
| ------------------------------ | -------------------------------------- | --------------------------------------- | ---------------------------------- |
| Skeleton-write (input → mode)  | Only the owning player's node          | `With<LocallyOwned>` on the gear entity | n/a (filter is per-entity)         |
| Skin-simulate (mode → display) | Every node with a local human viewport | none                                    | `resource_exists::<LocalPlayer>()` |

**Prerequisite: gear entities must carry `LocallyOwned`.** Currently gear entities are spawned with `Owner` +
`Replicated` but never receive `LocallyOwned`. The `With<LocallyOwned>` filter on gear queries will match nothing until
this is fixed. When `LocallyOwned` is inserted onto a player entity (both in `handle_ownership_granted` for remote
clients and in the host path in `setup_mission_players`), it must also be inserted onto every gear entity referenced in
that player's `PlayerGear` (left_hand, right_hand, all inventory slots). Everything in the player's possession is
locally owned by the controlling client. — exactly the nodes that have a human player and need to render gear. It does
**not** exist on a dedicated server. `AuthorityRole` is the **wrong** gate for skin simulation: both Offline
(`Authority + LocalPlayer`) and PeerHost (`Authority + LocalPlayer + LobbyPresence`) are authority nodes that still need
to run skin simulation for their own local player's gear.

So the correct split for `update_flashlight` is:

```rust
// Skeleton-write function: only for own gear
fn update_flashlight_skeleton(q: Query<..., With<LocallyOwned>>) { ... }

// Skin-compute function: for all visible gear, only on nodes with a viewport
fn update_flashlight_skin(q: Query<...>) { ... }

app.add_systems(Update, (
    update_flashlight_skeleton,  // no resource gate — entity filter handles it
    update_flashlight_skin.run_if(resource_exists::<LocalPlayer>()),
));
```

The server's `Toggleable.is_on` is kept in sync by `handle_export_gear_state`, which already writes it directly from the
`ExportGearStateMessage`. The skin-compute system never needs to run on a dedicated server.

---

## 6. Proposed Replicated Component Set (after reorganization)

Add to `app.replicate::<T>()`:

| Component         | Why                                                                                                                                                                               |
| ----------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Flashlight`      | Carries `status` (multi-state skeleton). **Must be stripped of all skin fields** (`output_power`, `inner_temp`, etc.), which move to a non-replicated `FlashlightSkin` component. |
| `UVTorch`         | Carries `enabled` (skeleton). `output_power` overwritten locally.                                                                                                                 |
| `RedTorch`        | Carries `enabled` (skeleton). `output_power` overwritten locally.                                                                                                                 |
| `RepellentFlask`  | Carries `liquid_content`, `qty`, `active` (all skeleton).                                                                                                                         |
| `QuartzStoneData` | Carries `cracks` (skeleton). `cracked_time` and `energy_absorbed` overwritten locally.                                                                                            |
| `SageBundleData`  | Carries `is_active`, `consumed` (skeleton). `burn_timer`, `smoke_produced` overwritten locally.                                                                                   |
| `SaltData`        | Carries `charges` (skeleton).                                                                                                                                                     |

Add to `LocallyOwned` noop markers (so owning client ignores server's replicated skin state):

- `Flashlight`, `UVTorch`, `RedTorch`, `RepellentFlask`, `QuartzStoneData`, `SageBundleData`, `SaltData`

Do **not** replicate: `Battery`, `Electronic`, `LightEmitter`, `GearSprite`, `StatusText`, `Thermometer`, `EMFMeter`,
`GeigerCounter`, `Recorder`, `SpiritBox`, `PerceivedClarity`.

---

### 7. The `ExportGearStateMessage` Pattern (Temporary Hack)

`ExportGearStateMessage` is a **client→server skeleton batch export**.

_Note: The long-term goal is to send the actual skeleton components directly via `bevy_replicon`'s native replication.
However, because `bevy_replicon` currently requires a more complex setup for client-to-server component replication, we
are temporarily using `ExportGearStateMessage` as a bridge._

Two changes are required:

1. **Remove skin fields.** Strip `battery` and `temperature`. The message must carry only skeleton state.
2. **Expand to all gear.** Currently only `Flashlight.status` is exported. Every gear type with a non-trivial skeleton
   field must be covered. The message sends one record per gear entity per frame. The receiver
   (`handle_export_gear_state`) reads the `GearKind` of each entity and writes the appropriate skeleton field(s).

`status_level` is a temporary hack to encode `FlashlightStatus` (u8) and binary toggle states (0 = off, 1 = on). We will
replace this with native component replication as soon as the skeleton/skin split is stable.

---

## 8. Implementation Order (safest first)

### Step 0 (prerequisite for all steps) — Grant and Revoke `LocallyOwned` on gear entities

When `LocallyOwned` is inserted onto a player entity — both in `handle_ownership_granted` (for remote clients) and in
the host path in `setup_mission_players` — immediately also insert `LocallyOwned` on every gear entity in that player's
`PlayerGear` (left_hand, right_hand, all inventory slots).

Without this, `With<LocallyOwned>` on gear queries matches nothing, and all `set_marker_fns` noop registrations for gear
components are dead code.

**`LocallyOwned` must also be removed when gear leaves a player's possession.** See Section 13 for the full dropped-item
policy. Short rule: when an item is dropped or transferred, `LocallyOwned` is removed from the previous holder
immediately. On a pickup, `LocallyOwned` is inserted for the new holder. The system that modifies `PlayerGear` slots
(pick-up, drop, transfer) is responsible for both insertions and removals.

### Step 1 — Split and Replicate `Flashlight` (lowest risk, most visible)

**You cannot replicate `Flashlight` until the struct is actually split. Step 1a must happen before 1b.**

**All replication registration and hydration observers for flashlight belong in `ungearitems-plugin`, not in
`unreplicon-plugin` or `players.rs`. Each crate is responsible for the networking of its own things.**

1a. **Split the `Flashlight` component struct** in `crates/ungearitems-core/src/components/flashlight.rs`: -
`Flashlight` (Skeleton, replicated): Contains **ONLY** `status: FlashlightStatus`. Add `Serialize, Deserialize, Reflect`
derives. - `FlashlightSkin` (Skin, never replicated): Contains `inner_temp`, `heatsink_temp`, `frame_counter`, `rand`,
`output_power`. Do **not** add `Serialize` or `Deserialize`. - Move `update_output_power()` out of the `Flashlight` impl
block; its logic moves into the skin system. - Update all construction sites that use `Flashlight { ... }` struct
literals.

1b. **In `crates/ungearitems-plugin`** (e.g., the plugin's `plugin.rs` or a dedicated `registration.rs`): -
`app.replicate::<Flashlight>()` - `app.set_marker_fns::<LocallyOwned, Flashlight>(noop_write, noop_remove)` - Register
the `OnAdd<Flashlight>` hydration observer (see Section 12).

1c. Remove `battery` and `temperature` from `ExportGearStateMessage` (or stop sending them and keep field for compat).

1d. **Split `update_flashlight` into two functions** in `crates/ungearitems-plugin/src/components/flashlight.rs`: -
`update_flashlight_skeleton`: handles trigger input → writes `flashlight.status`. Query with `With<LocallyOwned>`. No
resource gate — entity filter handles it. - `update_flashlight_skin`: query `(&Flashlight, &mut FlashlightSkin, ...)`.
Reads `status` from the skeleton, computes and stores all accumulators into `FlashlightSkin`, writes
`LightEmitter.power`, sprite, text. Register with `.run_if(resource_exists::<LocalPlayerRole>())` (excludes dedicated
server, includes Offline/PeerHost/Join).

Expected result: remote clients receive `Flashlight { status: Mid }` via replication, skin-compute system runs against
`FlashlightSkin` local accumulators, light renders. Offline and PeerHost modes unaffected.

### Step 2 — Fix UVTorch and RedTorch (same pattern, simpler — only `enabled: bool`)

Same split approach. Replicate the component, add noop. Split `update_uvtorch` / `update_redtorch` into skeleton-write
(`With<LocallyOwned>`) and skin-compute (`resource_exists::<LocalPlayer>()`).

### Step 3 — Fix Videocam

`Videocam.output_power` is skin; `Toggleable.is_on` is sufficient as skeleton (binary only). Split `update_videocam` the
same way: skeleton-write handles trigger input with `With<LocallyOwned>`, skin-compute derives power from toggle and
runs under `resource_exists::<LocalPlayer>()`.

### Step 4 — Fix consumables (RepellentFlask, QuartzStone, Salt, Sage)

These are lower-priority for visual correctness but matter for gameplay correctness. Replicate skeleton fields.

### Step 5 — Remove Battery + temperature from ExportGearStateMessage

After the above steps are verified, clean up the message.

### Step 6 — Split all remaining skin simulations away from server

Go through each remaining `update_*` system and apply the same skeleton/skin split. The pattern is always:
skeleton-write with `With<LocallyOwned>`, skin-compute with `resource_exists::<LocalPlayer>()`. Verify that the server
still correctly stores and replicates skeleton state through `handle_export_gear_state` alone.

---

## 9. Design Questions — Answered

**Q — Should gear entities get `LocallyOwned` when the player does?**

**A — Yes. Everything in the player's possession is locally owned by the controlling client.** The player entity, all
gear in left_hand, right_hand, and inventory — all get `LocallyOwned` when ownership is granted. The owning client
drives the state of these entities with full authority. The server receives their skeleton state via
`ExportGearStateMessage` and replicates it to other clients. Other clients never write back to these entities on the
owning node. This is the invariant that makes `With<LocallyOwned>` safe as the skeleton-write query filter.

---

**Q — Are `PlayerSprite.health`, `PlayerSprite.sanity`, and `Stamina.current` skeleton or skin?**

**A — Skeleton.** The controlling client computes them locally (full ownership), then exports them to the server via
`ExportStateMessage`. The server replicates them to other clients so visible effects (Truck UI sanity display, future
visual/audio cues for wounded/exhausted states) work correctly for all observers. Like all skeleton fields on
locally-owned entities, the owning client ignores incoming replicated values for its own entity — `LocallyOwned` noop
guards on `PlayerSprite` and `Stamina` already handle this correctly.

---

**Q — Videocam skeleton:** Is `Toggleable.is_on` sufficient for the Videocam, or does `output_power` need to be
replicated?

**A — Videocam uses fixed power when on.** The Videocam emits a fixed output power when enabled; there is no multi-level
mode enum like `FlashlightStatus`. Therefore `Toggleable.is_on` (already replicated) is sufficient as the skeleton for
the Videocam. `update_videocam` only needs to be gated from authority so it stops stomping `Toggleable` on remote
clients. No new component replication is needed for Videocam.

---

**Q — Ghost-triggered glitch (`Electronic.glitch_timer`):** Ghost interactions set `glitch_timer` on gear. This affects
visuals on all clients. Is this a skeleton (must replicate) or skin (each client simulates independently)?

**A — Glitch is skin.** Ghost interactions on the server/authority do drive `glitch_timer`, but viewing clients are
expected to compute their own glitch effects independently from local ghost proximity data. Divergence in exact glitch
timing between clients is acceptable — it's a visual effect, not a gameplay-critical state. `glitch_timer` and
`glitch_intensity` remain skin fields and are never replicated. See Section 10 for the design note on keeping the
physical-switch state clean from glitch influence.

---

**Q — Sensor readings (EMF, temperature, Geiger, Recorder, SpiritBox):** Each client reads local world grids. Do
readings diverge between clients?

**A — Sensor divergence is accepted by design.** Each client independently simulates sensors from its local
`ThermalGrid`, `SoundGrid`, `MiasmaGrid`, etc. Readings will diverge between clients. This is known and intentional. The
grids are not replicated. Evidence detection is therefore a per-client judgment, which is the intended design.

---

## 10. The Glitch Architecture Principle

Ghost interactions temporarily set `Electronic.glitch_timer > 0` on nearby gear. While glitching, `update_*` systems
reduce `output_power` and play visual/audio effects. This is firmly **skin** — it is a locally computed visual layer.

The critical constraint: **glitch must never write into skeleton fields.**

Verified via code inspection:

- `Flashlight.status` — NOT modified when `glitch_timer > 0`. The trigger handler is gated by
  `if electronic.glitch_timer <= 0.0`. Battery-off and overheat status transitions are also gated the same way.
  `output_power` is reduced by `update_output_power(..., glitch_timer)`, which is skin. ✅
- `UVTorch.enabled` — NOT modified by glitch. Glitch only affects `output_power` computation inside
  `calculate_output_power`. ✅
- `RedTorch.enabled` — NOT modified by glitch. Glitch modifies `new_power` before it is written to `output_power`,
  leaving `enabled` untouched. ✅

**The pattern to maintain:** Every `update_*` system has two logically separate responsibilities:

1. **Skeleton-write phase:** Handle user trigger input → write the physical-switch skeleton field (`FlashlightStatus`,
   `enabled`, etc.). This phase must be gated: `if glitch_timer <= 0.0`.
2. **Skin-compute phase:** Derive all display outputs (power, color, sprite, status text, audio) from the skeleton
   field, battery level, and glitch effects. This phase runs regardless of glitch state.

Currently both responsibilities live inside the same `update_*` function, which is acceptable. The gating of trigger
handling by `glitch_timer <= 0.0` correctly prevents glitch from corrupting the skeleton. When refactoring toward
explicit skeleton/skin phases, this distinction should be preserved and made explicit.

---

## 11. Summary of What Is Currently Broken in Multiplayer

| Gear          | Problem                                | Root cause                                                                  |
| ------------- | -------------------------------------- | --------------------------------------------------------------------------- |
| Flashlight    | No light seen from remote              | `Flashlight.status` not replicated, `update_flashlight` stomps `Toggleable` |
| UVTorch       | No UV light from remote                | Same: `UVTorch.enabled` not replicated                                      |
| RedTorch      | No red light from remote               | Same: `RedTorch.enabled` not replicated                                     |
| Videocam      | No IR light from remote                | `Toggleable` stomped by `update_videocam` reading wrong `output_power`      |
| All sensors   | Readings are local only                | Correct by design — they read local grids                                   |
| Consumables   | Wrong qty/state on remote              | Skeleton fields not replicated                                              |
| Battery drain | Server draining all clients' batteries | `update_*` runs on server for all gear                                      |

---

## 12. Skin Hydration: How Skin Components Reach Entities

When the server replicates a gear entity to a joining or observing client, it sends only the skeleton components
(`Flashlight`, `UVTorch`, etc.). Skin components (`FlashlightSkin`, etc.) are never in the server's world — the server
does not hold them, so they cannot be replicated.

The client must **hydrate** skin components when it first receives the replicated entity. The correct mechanism is a
Bevy observer reacting to the `OnAdd<SkeletonComponent>` trigger for the **specific skeleton component** of that gear
type — one observer per gear type, registered in `ungearitems-plugin`:

```rust
// In ungearitems-plugin — each gear type registers its own hydration observer:
app.observe(|trigger: Trigger<OnAdd, Flashlight>, mut commands: Commands| {
    commands.entity(trigger.entity()).insert(FlashlightSkin::default());
});

app.observe(|trigger: Trigger<OnAdd, UVTorch>, mut commands: Commands| {
    commands.entity(trigger.entity()).insert(UVTorchSkin::default());
});
// ... one per gear type that has a skin component
```

This is a clean 1:1 mapping: each skeleton component's arrival triggers exactly one skin component insertion. No `match`
statement on `GearKind`, no central dispatch, no cross-crate dependency.

**Rules:**

1. **Each observer lives in its gear's own crate** (`ungearitems-plugin`), alongside the systems that use the skin
   component. Not in `unreplicon-plugin`. Not in `players.rs`.
2. **The observer fires on every node** — including the dedicated server. So the observer body must check for
   `LocalPlayerRole` before inserting, or be registered only on nodes that have the `LocalPlayerRole` resource. The
   simplest approach: wrap the insert in `if commands.get_resource::<LocalPlayerRole>().is_some()` or use a system-set
   condition on registration.
3. Skin components start at `default()`. The skin-compute system fills them from skeleton state within a frame or two.
   No special initialisation from the server is needed or possible.
4. The observer fires on the owning client when gear is spawned locally too (the entity receives `Flashlight` on spawn).
   This is correct and desired: the owner also needs `FlashlightSkin` to run its own skin simulation.

**Anti-pattern — server spawning skin:** The server must never hold skin components. If the server inserts
`FlashlightSkin` during gear spawn, it either gets accidentally replicated (bug) or accumulates forever in server memory
with no system to drive it. Server holds only skeleton components.

---

## 13. Unowned Gear: Skin Simulation and Battery Drain for Dropped Items

When a player drops a flashlight on the floor, it becomes **unowned**: it is no longer held in any `PlayerGear` slot.
The design here is simple and follows directly from the core contract established in Section 1.

### `LocallyOwned` is removed immediately on drop

When an item leaves a player's possession (drop or transfer), `LocallyOwned` is stripped from that gear entity
immediately. The system that modifies `PlayerGear` slots is responsible for removing it. On a pickup, `LocallyOwned` is
inserted for the new holder.

The skeleton of the dropped item is **frozen** at whatever state it held when dropped (e.g., `status: Mid`). No client
has `LocallyOwned` on it, so no `update_*_skeleton` system will process it. The item cannot be triggered, toggled, or
have automatic transitions applied to it. It stays exactly as dropped until someone picks it up.

### Who drains the battery of a dropped, active flashlight?

**Every client. All of them. In parallel.**

`update_flashlight_skin` has no `LocallyOwned` filter. It runs on every piece of gear visible in the local viewport,
ownership irrelevant. Each client sees `Flashlight { status: Mid }`, concludes the flashlight is on, and independently
drains its own local battery simulation.

Battery level will diverge across clients. Client A may think the battery dies after 47 seconds; Client B after 51
seconds. This is expected and accepted by design (see Section 1: skin divergence is by design). The flashlight emits
light on every client proportional to each client's local simulated `output_power` — which is fine.

### When the battery hits zero on a client's local skin simulation

The skin system computes `output_power → 0` when battery is depleted locally. The light goes dark on that client. No
skeleton write happens, and no replication event is sent. Other clients may see the light for a bit longer until their
own battery simulation also drains to zero. The discrepancy is small in practice and accepted.

The skeleton field `status` stays at `Mid` even after the battery is locally dead on a given client — the skeleton is
frozen and owned by nobody. This is correct: the skeleton records the physical-switch state ("the switch is in the Mid
position"), not the energy availability.

**Summary:**

- `LocallyOwned` is removed from gear immediately when dropped.
- Skeleton is frozen at the drop value for the entire time the item is unowned.
- Skin simulation (battery drain, thermal, output power) runs independently on every client for every visible item.
- Skin divergence between clients for dropped gear is expected and accepted by design.

---

## 14. Client Spawning Prohibition

**Non-authority clients must never spawn mission entities (gear, players, or environment objects).**

If a non-authority client spawns a gear entity locally, `bevy_replicon` has no record of it. This creates:

- A ghost entity invisible to all other players and the server.
- A duplicate entity once the server later replicates the real entity to that client.
- Invisible desync that is nearly impossible to trace in logs.

**Rule: every system that spawns gear, player entities, or any mission-scoped entity must be gated by
`resource_exists::<AuthorityRole>()`.**

Audit checklist:

- `setup_mission_players` — player spawning; verify it is `AuthorityRole`-gated.
- Gear spawning and inventory initialisation — verify `AuthorityRole`-gated.
- Observers or event handlers that create new entities in response to game events — verify gate.
- Equipment use that spawns effect entities (throwing salt, deploying sensors) — the spawning of replicated entities
  must be `AuthorityRole`-gated; client-local VFX/SFX effects that carry no `Replicated` marker may run on any node.

The only exception is transient, never-replicated client-local entities (particle effects, UI indicators) that are
spawned and despawned entirely within one client's lifecycle and are never given a `Replicated` marker.
