# Client → Server Component Export — Design Plan

> Part of the Unhaunter DDD audit. Index: [02_crate_audit.md](02_crate_audit.md)

---

## The Problem

bevy_replicon is server→client only. It has no built-in mechanism for a client to push component state upstream to the
server. But clients own their player simulation: position, vitals, gear state. That state must reach the server so
replicon can distribute it to everyone else.

The game currently fills this gap with hand-rolled messages: `ExportStateMessage` (a flat struct containing every player
field) and `ExportGearStateMessage` (an entity + a closed enum listing every gear type). Both live in `unreplicon-core`,
which means the networking crate knows the shape of every player stat and every gear item in the game.

This creates two problems:

1. **Wrong home.** Adding a player stat or gear property requires editing the networking crate. The networking layer
   acts as a domain encyclopedia instead of a dumb pipe.

2. **Closed gear catalogue.** `GearSkeletonState` enumerates every gear type by name. Adding a gear item means adding an
   enum arm in the networking crate. This is a domain registry disguised as a protocol type.

---

## The Solution: `ExportClientComponent<T>`

A single generic message type replaces all hand-rolled export structs:

```rust
pub struct ExportClientComponent<T: Component + Serialize + DeserializeOwned + ...> {
    pub entity: Entity,
    pub data: T,
}
```

Each concrete instantiation (`ExportClientComponent<Position>`, `ExportClientComponent<PlayerVitals>`,
`ExportClientComponent<FlashlightStatus>`, etc.) is a separate replicon message type. Registration and the send/receive
systems live in the **owning domain's plugin**, not in `unreplicon-plugin`.

### How it works

The data flow is the same three-leg relay that exists today — only the message shape and ownership change:

```
Client simulates locally
    ↓  ExportClientComponent<T> (manual message, per frame, unreliable UDP)
Server receives, writes T into its copy of the entity
    ↓  app.replicate::<T>() (replicon automatic, server→all clients)
Other clients receive updated T
```

The owning client's echo is suppressed by the existing `LocallyOwned` + `noop_write` marker system, which stays as-is.

### What each domain provides

For every component `T` that a client locally simulates and needs to upload:

1. **Message registration** in the domain plugin's `app_setup`:
   `app.add_mapped_client_message::<ExportClientComponent<T>>(Channel::Unreliable);`

2. **Export system** (runs on the client): reads `T` from locally-owned entities, writes
   `ExportClientComponent<T> { entity, data: component.clone() }`.

3. **Import system** (runs on the server): reads `FromClient<ExportClientComponent<T>>`, validates ownership, writes `T`
   on the server's copy of the entity.

### Partial coherence is acceptable

Splitting the old monolithic `ExportStateMessage` into per-component messages means independent UDP packets. Some may
arrive while others drop. The server may briefly have an updated `Position` with a stale `PlayerVitals`. This is
acceptable by design — all game systems already assume eventual consistency with 0–2 frames of delay. No code path
depends on atomic cross-component snapshots from a single frame.

### Phase 2: boilerplate reduction

The export and import systems will be near-identical across domains. A future phase can introduce a trait or generic
helper to generate them, so each domain only needs a one-line registration call. For now, systems are written manually
with a comment noting the Phase 2 intent.

---

## How This Eliminates `GearSkeletonState`

The closed enum exists because there was one message (`ExportGearStateMessage`) carrying one payload for all gear types.
The enum was the discriminator.

With `ExportClientComponent<T>`, each gear type exports its own component directly:
`ExportClientComponent<FlashlightStatus>`, `ExportClientComponent<SaltData>`, `ExportClientComponent<SageBundleData>`,
etc. No discriminator needed. No central enum. No file in the networking crate that lists every gear type.

### Gear component audit — COMPLETE (2026-03-22)

Every arm of `GearSkeletonState` maps to an existing, serializable, already-replicated component. No new types need to
be created. The gear side is unblocked.

| `GearSkeletonState` arm                  | Component to use             | Crate                | Replicated |
| ---------------------------------------- | ---------------------------- | -------------------- | ---------- |
| `Flashlight(FlashlightStatus)`           | `Flashlight { status }`      | `ungearitems-core`   | ✅         |
| `UVTorch(bool)`                          | `UVTorch { enabled }`        | `ungearitems-core`   | ✅         |
| `RedTorch(bool)`                         | `RedTorch { enabled }`       | `ungearitems-core`   | ✅         |
| `RepellentFlask { qty, liquid_content }` | `RepellentFlask` (full)      | `ungearitems-core`   | ✅         |
| `Salt(u8)`                               | `SaltData { charges }`       | `ungearitems-core`   | ✅         |
| `Sage { is_active, consumed }`           | `SageBundleData`             | `ungearitems-core`   | ✅         |
| `Quartz(u8)`                             | `QuartzStoneData { cracks }` | `ungearitems-core`   | ✅         |
| `Toggleable(bool)` (catch-all)           | `Toggleable { is_on }`       | `uninteraction-core` | ✅         |

**Implementation notes for the import systems:**

- `Flashlight`, `UVTorch`, `RedTorch` — the current import handler also writes `Toggleable.is_on` as a mirror. This
  secondary write must be preserved in each component's import system. It is domain logic, not a networking concern.
- `RepellentFlask.active` — not sent from the client. The server derives it as `qty > 0 && liquid_content.is_some()`.
  The import system must retain this derivation logic.
- `Toggleable` catch-all — covers any gear kind not matched by the above (currently none at runtime, but matches
  anything that only has `Toggleable` and no specialist component).

---

## What Gets Removed from `unreplicon-core`

Once all domains register their own `ExportClientComponent<T>` messages:

| Removed from `unreplicon-core` | Replaced by                                                                            |
| ------------------------------ | -------------------------------------------------------------------------------------- |
| `ExportStateMessage`           | Per-component messages in `unspatial-core`, `unvitals-core`, `unlocomotion-core`, etc. |
| `GearSkeletonState`            | Eliminated — each gear type uses its own component directly                            |
| `ExportGearStateMessage`       | Per-gear-type `ExportClientComponent<T>` in `ungearitems-plugin`                       |
| `PlayerMoveMessage`            | Appears to be dead code — verify and remove                                            |

`ExportPlayerGearMessage` (hand/inventory entity assignments) is a separate concern — it carries `Option<Entity>` slot
assignments, not component state. It may remain as a dedicated message or be converted to
`ExportClientComponent<PlayerGear>`. To be evaluated separately.

---

## Execution Order

1. **Define `ExportClientComponent<T>`** in `unreplicon-core` (it is a networking primitive — generic, no domain
   knowledge).

2. **Player-state components first** (unblocked today):
   - `Position`, `Direction` → registered in `unspatial-core`'s owning plugin or `unlocomotion-plugin`
   - `PlayerVitals`, `Stamina` → registered in `unvitals-plugin`
   - `PlayerLocomotionState` → registered in `unlocomotion-plugin`
   - Boolean markers (`Hiding`, `InTruck`, `PlayerSpectating`) → evaluate: are these insert/remove markers or
     component-value exports? Different pattern may apply.

3. **Gear-state components** (unblocked — see audit above):
   - Register `ExportClientComponent<T>` per gear type in `ungearitems-plugin`.
   - Eight types: `Flashlight`, `UVTorch`, `RedTorch`, `RepellentFlask`, `SaltData`, `SageBundleData`,
     `QuartzStoneData`, `Toggleable`.

4. **Remove old types** from `unreplicon-core` once all consumers are migrated.

5. **Phase 2 (future):** Extract boilerplate into a trait or generic helper so each domain's export/import is a one-line
   registration.

---

## Open Questions

- **`ExportPlayerGearMessage`** — carries entity slot assignments (`left_hand`, `right_hand`, `inventory`), not
  component data. Does it convert to `ExportClientComponent<PlayerGear>` or stay as a dedicated message?

- **Boolean marker components** (`Hiding`, `InTruck`, `PlayerSpectating`) — these are inserted/removed as tag
  components, not mutated in place. `ExportClientComponent<T>` assumes a component that is always present with changing
  data. Tag insert/remove may need a different message shape (e.g., `ExportClientMarker<T>` with a bool for
  present/absent) or can be handled by making the component always-present with an `active: bool` field.

- **Gear component audit results** — ✅ COMPLETE 2026-03-22. All 8 gear types have existing serializable components. See
  audit table above. Step 3 is unblocked.
