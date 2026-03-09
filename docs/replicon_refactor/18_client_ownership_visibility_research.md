# 18 — Client Ownership & Replicon Visibility Research

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Status:** Research & design only. No implementation yet. Many open questions flagged.
- **Purpose:** Reference document for implementing proper player-entity client ownership using `bevy_replicon` 0.39
  visibility filters. Captures everything known, everything unknown, and what needs to be tested before code is written.

---

## 1. The Bug We Are Solving

### Symptom

Player movement appears slow and "laggy" on the owning client. On other clients the player moves at the correct speed.
The ghost's position is unaffected.

### Root Cause

`handle_ownership_granted` (client-side, `unreplicon-plugin/src/systems/players.rs`) currently:

1. Inserts `LocallyOwned` onto the client-local entity.
2. **Removes `Replicated` from the client-local entity.**

Step 2 is the workaround. It does not work. Removing `Replicated` on the **client** has no effect on what the **server**
sends. The server is unaware of the removal. It continues to serialize and broadcast the `Position` component every
replication tick. On the client side, `bevy_replicon`'s internal "apply updates" system reads the incoming packet, finds
the local entity via `ServerEntityMap`, and **overwrites `Position`** — regardless of whether `Replicated` is present on
the client entity. The `ServerEntityMap` entry never disappears.

The result is that the local player's position is constantly being snapped back toward the server's last-known value.
The client's own input writes a new position; one frame later the replication system partially overwrites it. This is
the "slow movement" symptom.

There is a `// FIXME` comment at that line in the code acknowledging the issue. The private API `remove_by_server` that
would have allowed removing the `ServerEntityMap` entry was not available in 0.38.2. It has not been revisited since
upgrading to 0.39.

### Why Other Players Move Fine

Other clients do **not** have `LocallyOwned` on that entity. Their position comes entirely from the replicated
`Position` component, which the server updates every frame via `handle_export_state`. That path is working correctly —
the owning client sends `ExportStateMessage` every frame, the server writes it to the entity's `Position`, and Replicon
broadcasts it to everyone else.

---

## 2. The Target Ownership Model

The desired architecture is clear and has been agreed on:

| Entity category                        | Owned by                        | Position authority                                           | How others see it      |
| -------------------------------------- | ------------------------------- | ------------------------------------------------------------ | ---------------------- |
| Local player entity                    | Owning client                   | Client updates locally; sends `ExportStateMessage` to server | Replicated from server |
| Remote player entities                 | Their respective owning clients | Server applies `ExportStateMessage`                          | Replicated from server |
| Ghost, breach, doors, switches, lights | Server always                   | Server simulation                                            | Replicated from server |
| Gear held by local player              | Owning client                   | Follows player locally                                       | Replicated from server |
| Gear on the floor / world items        | Server                          | Server simulation                                            | Replicated from server |

**Ownership transfer** (gear pickup/drop) requires a state transition: the client requests the action, the server
authorizes it and re-assigns ownership.

The key invariant: **a client should never receive replicated updates for its own player entity's movement components
after ownership is established.** The client is the sole writer of its own `Position`.

---

## 3. Current Data Flow (What Already Works)

```text
[Owning Client]                            [Server]                   [Other Clients]
  send_export_state                   handle_export_state
  writes ExportStateMessage  ──────►  writes pos/stamina/health      bevy_replicon
  (With<LocallyOwned>)                onto server-side entity   ───► replicates Position
                                      (Without<LocallyOwned>)         to remote clients
```

`send_export_state` reads `Position` from the local entity (which the client's own movement systems update) and sends it
as an unreliable `ClientMessage`. The server applies it. Replicon then broadcasts the updated `Position` to all other
clients.

This path is **confirmed working** — other players see correct movement. The problem is the return path silently
overwrites the owning client's own `Position`.

---

## 4. The Replicon 0.39 Visibility API (Verified from Source)

> ✅ All information in this section was read directly from
> `/home/deavid/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_replicon-0.39.0/src/server/visibility.rs` and
> its submodules. It supersedes everything the external AI said.

### 4.1 The `VisibilityFilter` Trait (Actual Signature)

```rust
pub trait VisibilityFilter: Component<Mutability = Immutable> {
    type ClientComponent: Component<Mutability = Immutable>;
    type Scope: FilterScope;
    fn is_visible(&self, client: Entity, component: Option<&Self::ClientComponent>) -> bool;
}
```

Key facts:

- **`client: Entity`** — the ECS `Entity` that represents the connected client on the server. This is the entity that
  has `ClientVisibility`. It is **NOT** a `RepliconId`. To identify the client by `RepliconId`, you must use the
  `ClientComponent` pattern (see §4.3).
- **`component: Option<&Self::ClientComponent>`** — an optional component living on the _client entity_. Designed for
  per-client context data. `None` if the `ClientComponent` is not present on that client entity.
- **`Component<Mutability = Immutable>`** — both the filter component itself **and** `ClientComponent` must be declared
  immutable with `#[component(immutable)]`. Mutable components are not allowed.

### 4.2 The `FilterScope` Implementations (Actual Types)

The `type Scope` in your `VisibilityFilter` impl determines what is hidden when `is_visible` returns `false`:

| `type Scope = ...`               | What gets hidden to the client            | Wire message sent     |
| -------------------------------- | ----------------------------------------- | --------------------- |
| `Entity`                         | The entire entity (entity despawn)        | `add_despawn()`       |
| `SingleComponent<C>`             | Only component `C` (explicit removal)     | `add_removal(fns_id)` |
| `(C1, C2, ...)` (up to 10 types) | All listed components (explicit removals) | `add_removal(fns_id)` |

> Note: `ComponentScope<A>` is a deprecated alias for `SingleComponent<A>` as of 0.39.0.

### 4.3 The `ClientComponent` Pattern

`ClientComponent` is an optional associated type that lets the filter receive per-client context. When Replicon
evaluates visibility for a client entity, it calls `is_visible(client_entity, client.get::<Self::ClientComponent>())`.

If you need to identify the client by `RepliconId` rather than by ECS `Entity` (which changes between runs), insert a
identifier component onto each client entity at connect time:

```rust
// Placed on each client entity when it connects
#[derive(Component, Clone)]
#[component(immutable)]
pub struct ClientOwnerId(pub RepliconId);
```

Then the filter can compare by `RepliconId`:

```rust
#[derive(Component, Clone)]
#[component(immutable)]
pub struct BlacklistForOwner(pub RepliconId);

impl VisibilityFilter for BlacklistForOwner {
    type ClientComponent = ClientOwnerId;
    type Scope = Entity; // entity-level OR SingleComponent<Position> for component-level

    fn is_visible(&self, _client: Entity, component: Option<&ClientOwnerId>) -> bool {
        // If the client has a ClientOwnerId and it matches → hide (return false)
        // If no ClientOwnerId (e.g. dedicated-server internal client) → visible
        component.map(|c| c.0 != self.0).unwrap_or(true)
    }
}
```

Alternatively, if you resolve the client `Entity` at spawn time (querying for the client with a matching `RepliconId`),
you can skip `ClientComponent` entirely and store the client `Entity` directly in the filter — but this is less robust
because the client entity is an ECS runtime value.

### 4.4 The `VisibilityScope` Enum (Actual Definition)

```rust
pub enum VisibilityScope {
    Entity,                     // hides the entire entity → despawn
    Components(ComponentMask),  // hides a set of components → removal messages
}
```

The scope is derived from the `FilterScope` impl of your `Scope` type at registration time. You do not set it at
runtime. Scope examples:

- `Entity::visibility_scope(...)` → `VisibilityScope::Entity`
- `SingleComponent::<Position>::visibility_scope(...)` → `VisibilityScope::Components(mask with Position)`

### 4.5 Supporting Types

| Name                                             | Kind      | Purpose                                                                                   |
| ------------------------------------------------ | --------- | ----------------------------------------------------------------------------------------- |
| `ClientVisibility`                               | Component | Per-client visibility state. Lives on client entities. Stores a `u32` bitmask per entity. |
| `FilterRegistry`                                 | Resource  | Maps filter types to `FilterBit` (u8 index). Max **32** filters globally.                 |
| `FilterBit`                                      | Struct    | Newtype `u8` index into the filter registry.                                              |
| `FiltersMask`                                    | Struct    | `u32` bitset; one bit per registered filter.                                              |
| `AppVisibilityExt::add_visibility_filter::<T>()` | Method    | Registers a filter type and wires up its 4 observers. Call once in `app_setup`.           |

There is **no global `VisibilityPolicy`** or whitelist/blacklist mode in 0.39. All entities are visible to all clients
by default. Filters only ever restrict visibility.

### 4.6 How Visibility Changes Propagate (Observer-Based, Not System-Based)

Visibility is maintained by four observers registered by `add_visibility_filter::<F>()`:

- **`on_insert<F>`** — fires when `F` is inserted on any entity (client or replicated). Evaluates `is_visible` for the
  affected entity+client pair and updates `ClientVisibility`.
- **`on_remove<F>`** — fires when `F` is removed from a replicated entity. Sets visibility to `true` for **ALL** clients
  unconditionally (filter is gone, no more restriction).
- **`update_for_new_clients<F>`** — fires when `ClientVisibility` is inserted on a new client entity. Evaluates
  `is_visible` for all existing entities carrying `F`.
- **`on_client_remove<F>`** — fires when `F::ClientComponent` is removed from a client entity. Re-evaluates visibility
  for all replicated entities carrying `F`, for that one client.

### 4.7 What "Lost Visibility" Produces on the Wire (Confirmed from `server.rs`)

When `is_visible` returns `false` for a previously-visible entity/component, the result goes into
`ClientVisibility.lost`. Two server systems then drain it:

- **`collect_despawns`** — reads `iter_lost()`. For every entry where the scope is `VisibilityScope::Entity`, sends an
  **entity despawn message** to the client. The client's `ServerEntityMap` entry is removed. The client entity is
  despawned.
- **`collect_removals`** — reads `drain_lost()`. For every entry where the scope is `VisibilityScope::Components`, sends
  **per-component removal messages**. The entity stays alive on the client but the listed components are explicitly
  removed.

---

### 4.8 Receive Markers API (Confirmed from `receive_markers.rs` + `receive_fns.rs`)

> ✅ All information in this section was read directly from `src/shared/replication/receive_markers.rs` and
> `src/shared/replication/registry/receive_fns.rs`.

`bevy_replicon` ships a **receive markers** API that lets the **client** override what happens when the server writes or
removes a component for entities that have a specific marker component. This is the client-side counterpart to
visibility filters.

**`WriteFn<C>` type alias:**

```rust
// src/shared/replication/registry/receive_fns.rs
pub type WriteFn<C> = fn(&mut WriteCtx, &RuleFns<C>, &mut DeferredEntity, &mut Bytes) -> Result<()>;
```

**`RemoveFn` type alias (NOT generic):**

```rust
pub type RemoveFn = fn(&mut RemoveCtx, &mut DeferredEntity);
```

Note: `RemoveFn` has **no type parameter**. When calling `set_marker_fns`, you must supply a concrete monomorphization
like `default_remove::<MyComponent>` or a named non-generic function.

**Built-in receive functions (from `receive_fns.rs`):**

| Function               | What it does                                                        |
| ---------------------- | ------------------------------------------------------------------- |
| `default_write`        | Deserialize + insert (in-place if mutable component already exists) |
| `default_insert_write` | For immutable components — always deserialize + insert              |
| `write_if_neq`         | Only writes if value changed (mutable + PartialEq)                  |
| `default_remove`       | Calls `entity.remove::<C>()`                                        |

**Registration API:**

```rust
// Register the marker type first:
app.register_marker::<LocallyOwned>();

// Then override write/remove for specific components:
app.set_marker_fns::<LocallyOwned, Position>(write_fn, remove_fn);
```

**No-op functions (not in library — must be written):**

```rust
use bevy_replicon::shared::replication::registry::receive_fns::{WriteCtx, RemoveCtx};
use bevy_replicon::shared::replication::registry::rule_fns::RuleFns;
use bevy::ecs::entity::DeferredEntity;
use bevy::prelude::Component;

/// Discards the server's value without writing it to the component.
fn noop_write<C: Component>(
    ctx: &mut WriteCtx,
    rule_fns: &RuleFns<C>,
    _entity: &mut DeferredEntity,
    message: &mut Bytes,
) -> Result<()> {
    rule_fns.consume(ctx, message) // consume bytes, do nothing with the entity
}

/// Suppresses the server's component removal completely.
fn noop_remove(_ctx: &mut RemoveCtx, _entity: &mut DeferredEntity) {
    // intentionally empty — server removal is ignored on the client
}
```

**How it interacts with visibility filters:** The two mechanisms are orthogonal.

- Visibility filters run on the **server** and control what the server sends.
- Receive markers run on the **client** and control what the client does with what it receives.
- If only receive markers are used (no server-side filter), the server still sends the update, but the client ignores
  it. Slightly wasteful of bandwidth but fully functional.
- If both are used ("belt and suspenders"), the server never sends the update AND the client ignores any that slip
  through.

---

### 4.9 Design Intent and Library Posture

> ⚠️ This section describes how `bevy_replicon` was designed to be used, and how our approach differs.

The `lib.rs` crate-level documentation (confirmed from source) states:

> "Replication happens only from server to clients. It's necessary to prevent cheating."

There is no mention of "client authority", "client-owned", "client-authoritative component", or any equivalent concept
anywhere in the `bevy_replicon` 0.39 source. The library is **explicitly server-authoritative by design**.

The receive markers API (`AppMarkerExt`) was designed for **client-side prediction (CSP)** and interpolation — scenarios
where the client _tentatively_ applies its own version of reality and the server _reconciles_ it. It was not designed
for the scenario where the client is a permanent authority over a component and the server must never overwrite it.

The visibility filter API (`add_visibility_filter`) was designed for **spatial or per-group visibility** — e.g., fog of
war, per-team information hiding. The docstring explicitly warns that losing visibility causes despawn (entity scope) or
component removal (component scope). These are the documented, intended behaviors.

**We are repurposing both systems** to implement a "client authority" model:

- Visibility filters → used to prevent the server from sending components the client owns
- Receive markers → used to prevent the client from applying server-sent components it owns

Neither usage aligns with the library's documented intent. This creates two risks:

1. **Future version breakage** — if `bevy_replicon` changes how visibility or receive markers work (especially in edge
   cases like reconnection, late client join, or entity re-spawn), our usage could break silently.
2. **Unexpected interaction** — the library may not test the combination of visibility filters + receive markers on the
   same entity/component. Edge cases may exist.

**Recommendation:** File a GitHub issue requesting an official `ClientAuthority<C>` or `NoReplicateIncoming<C>`
primitive. See §12.

---

## 5. Strategy Options

### 5.1 Summary Table

All viable options at a glance. Mechanism, sides affected, gear pickup safety, and score.

| Option | Name                                           | Server changes? | Client changes?             | Gear pickup safe?                         | Score |
| ------ | ---------------------------------------------- | --------------- | --------------------------- | ----------------------------------------- | ----- |
| A      | Entity blacklist post-spawn                    | Yes             | No                          | ✗ (despawn)                               | ★☆☆☆☆ |
| B      | Component filter post-spawn + re-add observer  | Yes             | Yes (observer)              | ✗ (1-frame flicker)                       | ★★☆☆☆ |
| C1     | Entity scope at spawn (full entity blacklist)  | Yes             | Needs client spawn rethink  | ✓ if gear also handled at spawn           | ★★★☆☆ |
| C2     | Component scope at spawn (only mutes Position) | Yes             | None                        | ✗ for mid-session gear pickup             | ★★★★☆ |
| E      | Receive markers only (client-side no-op)       | None            | Yes (register marker + fns) | ✓ if LocallyOwned inserted before removal | ★★★★★ |
| F      | C2 + E combined ("belt and suspenders")        | Yes             | Yes                         | ✓                                         | ★★★★★ |
| D      | Hybrid: replicate once then block              | Yes             | Yes (complex)               | ✗                                         | ★☆☆☆☆ |
| G      | Save/restore buffer around replication         | None            | Yes (fragile workaround)    | ✓                                         | ★☆☆☆☆ |

**Recommended option: E for the minimum viable fix, or F for long-term robustness.** See detailed sections below.

---

### Strategy A — Entity-Level Blacklist Post-Spawn ★☆☆☆☆

**Score rationale:** Confirmed broken — causes despawn. Documented for completeness only.

Add a "blacklist" component to the player entity on the **server** after ownership is claimed. The component stores the
`RepliconId` of the owning client.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::server::visibility::{VisibilityFilter, SingleComponent};

// Must be immutable
#[derive(Component, Clone)]
#[component(immutable)]
struct ReplicationBlacklist(RepliconId);

// Also immutable
#[derive(Component, Clone)]
#[component(immutable)]
struct ClientOwnerId(RepliconId); // inserted on client entities at connect

impl VisibilityFilter for ReplicationBlacklist {
    type ClientComponent = ClientOwnerId;
    type Scope = Entity; // entity-level → despawn

    fn is_visible(&self, _client: Entity, component: Option<&ClientOwnerId>) -> bool {
        component.map(|c| c.0 != self.0).unwrap_or(true)
    }
}
```

**✅ CONFIRMED from source code** (`server.rs` → `collect_despawns`): A despawn message IS sent when an entity-scope
visibility filter is applied to an already-replicated entity. `iter_lost()` inspects entries where
`filter_mask.is_hidden()` is true (entity-level scope), calls `message.add_despawn()`, and removes the entity's
`ClientTicks` entry. The client despawns the entity and purges the `ServerEntityMap` entry. **Strategy A is broken for
post-spawn use.**

**Mitigations (all unreliable):**

- **Race condition approach:** Remove `Replicated` on the client before the despawn arrives. The despawn packet may
  arrive first. Do not rely on this.
- **Parent anchor approach:** Parent the entity to a local anchor. Parent-child despawn behavior is version-dependent
  and not guaranteed.
- **Observer interception:** Observe `RemovedComponents::<Replicated>` to detect despawn and re-spawn. Fights the engine
  and may corrupt `ServerEntityMap`.

**Best use of this pattern:** At spawn time (Strategy C1) before the entity is ever replicated.

---

### Strategy B — Component-Level Filter Post-Spawn + Re-add Observer ★★☆☆☆

**Score rationale:** Avoids despawn but causes 1-frame component removal. Re-add observer is fragile. Does not handle
gear pickup cleanly without combining with receive markers.

Instead of hiding the entire entity, hide only `Position` (and optionally `Stamina`) using `SingleComponent<C>` scope.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use bevy_replicon::server::visibility::SingleComponent;
use unspatial_core::position::Position;

#[derive(Component, Clone)]
#[component(immutable)]
pub struct MutePositionForOwner(pub RepliconId);

#[derive(Component, Clone)]
#[component(immutable)]
pub struct ClientOwnerId(pub RepliconId);

impl VisibilityFilter for MutePositionForOwner {
    type ClientComponent = ClientOwnerId;
    type Scope = SingleComponent<Position>;

    fn is_visible(&self, _client: Entity, component: Option<&ClientOwnerId>) -> bool {
        component.map(|c| c.0 != self.0).unwrap_or(true)
    }
}
```

**✅ CONFIRMED from source code** (`server.rs` → `collect_removals`): When a component-scope filter hides a
previously-visible component, `drain_lost()` yields entries with `VisibilityScope::Components`. For each hidden
component, the server sends an explicit `add_removal(fns_id)` message. The client receives it and removes the component.
The entity stays alive but `Position` is explicitly removed.

**Mitigation sequence:**

1. Server applies `MutePositionForOwner` filter (after ownership granted).
2. Client receives removal message → `Position` removed from client entity.
3. Client observer on `RemovedComponents<Position>` (filtered to `With<LocallyOwned>`) detects removal.
4. Client re-inserts `Position` with last known value.
5. Filter now prevents any further server sends → client owns it exclusively.

**Problem:** The removal message and the `OwnershipGranted`/`LocallyOwned` insertion travel on different channels. If
the removal arrives before `LocallyOwned` is on the entity, the observer may not fire. Also, there is a 1-frame window
where `Position` is absent — visually noticeable.

**For gear pickup mid-session:** See §5.7 (The Component Removal Dance Problem). This strategy hits that problem
directly when gear changes ownership.

---

### Strategy C1 — Entity Scope at Spawn (Full Blacklist) ★★★☆☆

**Score rationale:** Clean on paper but requires rethinking client-side spawn path. Gear must also be excluded at spawn
time. Only viable if all owned entities are known at spawn.

Apply the entity-level filter at spawn time, before the entity ever replicates to the owning client:

```rust
commands.spawn((
    Replicated,
    Position::default(),
    PlayerSprite::new(/* ... */),
    ReplicationBlacklist(owner_replicon_id), // type Scope = Entity
));
```

The entity NEVER appears in the owning client's `ServerEntityMap`. `OwnershipGranted` must also carry enough initial
state for the client to initialize its local copy (position, etc.), because the entity will NOT arrive via replication.

**Pros:**

- Zero despawn, zero component removal dance.
- Ownership release is clean: remove the filter → entity appears as a fresh spawn.

**Cons:**

- Requires rethinking the "OwnershipGranted" path: the client must initialize the player entity from the message
  payload, not from replication.
- The current code relies on Replicon spawning the entity first, then `OwnershipGranted` identifying it. That changes
  fundamentally under C1.
- Gear entities picked up mid-session need their own entity-level filters inserted at the moment of pickup — the server
  must know to exclude them when sending to the owning client.

---

### Strategy C2 — Component Scope at Spawn (Mute Only Position) ★★★★☆

**Score rationale:** Clean for the primary movement bug. Does not handle gear pickup mid-session without additional
work. Combine with E for full solution.

Apply the component-scope filter at spawn time:

```rust
commands.spawn((
    Replicated,
    Position::default(),
    PlayerSprite::new(/* ... */),
    MutePositionForOwner(owner_replicon_id), // type Scope = SingleComponent<Position>
));
```

The entity replicates normally to the owning client (they get all components), but `Position` is NEVER included in any
packet to them. The entity arrives on the client WITHOUT `Position`. The client's own movement systems initialize and
own `Position` locally.

**Pros:**

- No removal dance (component was never replicated to the owning client, so no removal message ever fires).
- Entity IS in `ServerEntityMap` — `OwnershipGranted` still works exactly as today.
- On ownership release: remove the filter → server sends `Position` as an INSERT (new component) → clean.

**Cons:**

- Does not handle gear entities picked up mid-session (the gear entity already has `Position` replicated to the client;
  applying the filter post-pickup causes a removal message). See §5.7.
- Requires knowing the owner's `RepliconId` at spawn time. See Q7 in §6.

---

### Strategy E — Receive Markers with No-Op Functions (Client-Side Only) ★★★★★

**Score rationale:** Requires zero server changes. Works for gear pickup mid-session if `LocallyOwned` is inserted on
the client before the removal message arrives (very likely since `OwnershipGranted` uses ordered-reliable channel).
Risk: repurposing a CSP-designed API for a permanent authority model (see §4.9 design intent warning).

`LocallyOwned` is already a marker component. Registering it with receive markers lets the client override what happens
when the server writes or removes `Position` (or any other component) on entities marked as locally owned.

```rust
// In app setup — client side only (also safe to call on server, is a no-op there)
app.register_marker::<LocallyOwned>().priority(0); // or call .set_marker_priority::<LocallyOwned>(0)

app.set_marker_fns::<LocallyOwned, Position>(
    noop_write::<Position>,
    noop_remove,
);
```

**No-op functions:**

```rust
use bevy_replicon::shared::replication::registry::receive_fns::{WriteCtx, RemoveCtx};
use bevy_replicon::shared::replication::registry::rule_fns::RuleFns;
use bevy::ecs::entity::DeferredEntity;
use bevy::prelude::Component;
use bevy_replicon::bytes::Bytes;

pub fn noop_write<C: Component>(
    ctx: &mut WriteCtx,
    rule_fns: &RuleFns<C>,
    _entity: &mut DeferredEntity,
    message: &mut Bytes,
) -> Result<()> {
    rule_fns.consume(ctx, message) // MUST consume bytes or stream becomes corrupted
}

pub fn noop_remove(_ctx: &mut RemoveCtx, _entity: &mut DeferredEntity) {
    // intentionally empty — ignore server removal
}
```

**⚠️ MUST call `rule_fns.consume(ctx, message)` in `noop_write`** — if the bytes are not consumed, the deserializer will
be misaligned and all subsequent component updates in the same packet will be corrupted.

**How it works for gear pickup mid-session:**

1. Client picks up gear. Server sends `OwnershipGranted` for the gear entity (ordered-reliable channel).
2. Client receives `OwnershipGranted`, inserts `LocallyOwned` on the gear entity.
3. Server applies a component-scope visibility filter `MutePositionForOwner` on the gear entity (optional, saves
   bandwidth).
4. If the server sends a removal message for `Position` on that gear entity (from step 3), the client receives it, but
   because `LocallyOwned` is now on the entity, `noop_remove` fires instead of `default_remove::<Position>`. Component
   is NOT removed.

**Ordering guarantee:** `OwnershipGranted` is sent on an ordered-reliable channel. The removal message (from visibility
filter change) is sent as a Replicon update. Since both are reliable, order is preserved: `OwnershipGranted` arrives and
is processed before the removal message, assuming they were sent in the same frame or the ownership message was sent
first. This is the most likely scenario. If there is a multi-frame gap between pickup authorization and visibility
filter change, the ordering is guaranteed.

**Risk:** If for any reason the removal message arrives before `LocallyOwned` is inserted (e.g., unreliable channel
race, out-of-order processing), the component IS removed. Client must then re-add it. This is the same 1-frame flicker
as Strategy B. In practice, using ordered-reliable channels for `OwnershipGranted` makes this very unlikely.

---

### Strategy F — C2 at Spawn + E Receive Markers (Belt and Suspenders) ★★★★★

**Score rationale:** Most robust. Server never sends the component. Client also never applies server writes. Handles
both the base case (spawn-time filter) and the edge case (gear pickup mid-session via receive markers). Complex to set
up but extremely reliable once in place.

Combines:

- **C2 at spawn**: When the server spawns the player entity, it applies `MutePositionForOwner` at spawn time. `Position`
  is never included in any replication packet to the owning client.
- **E receive markers**: The client registers `LocallyOwned` as a receive marker with no-op write/remove for `Position`.
  This handles: (a) any case where C2 isn't applied yet (race condition), (b) gear entities picked up mid-session (where
  C2 can't be applied at spawn), and (c) redundant protection.

```rust
// Server: at setup_mission_players spawn
commands.spawn((
    Replicated,
    Position::default(),
    /* ... */
    MutePositionForOwner(owner_replicon_id), // C2 applied at spawn
));

// Client: in app_setup (registration)
app.register_marker::<LocallyOwned>();
app.set_marker_fns::<LocallyOwned, Position>(noop_write::<Position>, noop_remove);
```

When gear is picked up mid-session:

- Server sends `OwnershipGranted` for the gear entity.
- Client inserts `LocallyOwned` on gear entity → receive markers kick in, no-op for Position.
- Server optionally applies `MutePositionForOwner` on the gear entity for bandwidth savings.
- No removal dance, no 1-frame flicker.

---

### Strategy D — Hybrid: Replicate Once, Then Block ★☆☆☆☆

**Score rationale:** Most complex. Relies on Replicon internals in an unintended way. Documented for completeness.

1. Spawn entity with `Replicated` (no filter) → entity arrives on owning client.
2. Immediately (same frame or next frame) apply the entity-level blacklist filter.
3. If Replicon sends a despawn: the owning client's entity map already has the entity. Since it was just spawned, it's
   very unlikely the client has applied any local writes yet. The despawn removes the entity, but we already know the
   entity's server ID from the `OwnershipGranted` message. We can re-spawn it locally and NOT add it back to
   `ServerEntityMap` so Replicon won't touch it again.

This is the most complex option and most reliant on Replicon internals. Not recommended unless all other strategies
fail.

---

### Strategy G — Save/Restore Buffer Around Replication ★☆☆☆☆

**Score rationale:** Workaround that fights the engine every frame. Not a real fix. Documented for completeness.

1. Add `ClientOwned` marker to the entity on the owning client.
2. Write a system that runs BEFORE Replicon's update-apply system: saves owned component values to a temporary resource.
3. Write a system that runs AFTER Replicon's update-apply: restores those values from the buffer.

**Problems:**

- Every replication tick the values are overwritten and then restored — wasteful and potentially visible as 1-frame
  flicker.
- System ordering relative to Replicon's internal systems is fragile and can change across versions.
- Not a real fix; a workaround that fights the replication engine.

**Not recommended.** Documented only to explain why it was rejected.

---

### 5.7 The Component Removal Dance Problem (Gear Pickup Mid-Session)

> This is a new issue that affects any strategy that applies a component-scope visibility filter AFTER the entity has
> already been replicated to the client.

**Scenario:**

1. A gear item (e.g., flashlight) is on the floor. Server replicates it to all clients, including the player who picks
   it up. The gear entity has `Position` replicated. Client A (the owner-to-be) has the full entity including
   `Position`.
2. Client A picks up the gear. The server authorizes the pickup, inserts `LocallyOwned` on the server-side gear entity
   (or sends `OwnershipGranted` to Client A).
3. The server now wants to stop sending `Position` for the gear entity to Client A. It applies a `MutePositionForOwner`
   filter (component scope, `SingleComponent<Position>`).
4. ✅ CONFIRMED (see §4.7): The server sends an explicit `add_removal(fns_id)` message for `Position` to Client A.
5. Client A receives the removal message. `default_remove::<Position>` is called. `Position` is removed from the gear
   entity.
6. The gear entity now has no `Position` on Client A's world. If anything renders based on `Position`, the entity may
   disappear or teleport for one frame.

**The "dance" (if not using receive markers):**

1. Server sends removal → `Position` removed.
2. Client detects removal (observer on `RemovedComponents<Position>` + `With<LocallyOwned>`).
3. Client re-inserts `Position` with last known value (from before removal).
4. Now client owns `Position` exclusively.

The "dance" has a 1-frame gap where `Position` is absent. For visual components, this causes a 1-frame disappearance.
For physics, it could cause a 1-frame reset. This is unacceptable for user-visible entities.

**Solutions:**

| Solution                        | Mechanism                               | 1-frame flicker? | Complexity |
| ------------------------------- | --------------------------------------- | ---------------- | ---------- |
| Apply filter at spawn time (C2) | Never existed → no removal              | ✗                | Low        |
| Receive markers (E)             | `noop_remove` suppresses removal        | ✗                | Medium     |
| Re-add observer (B)             | Detects removal, re-inserts immediately | ✓ (1 frame)      | Medium     |
| Backup/restore buffer (G)       | Saves and restores every frame          | ✓ (subtle)       | High       |

**Best solution: Receive markers (Strategy E).** `LocallyOwned` is already in the codebase. Registering it with a
`noop_remove` function for `Position` means the removal message is received but `Position` is never removed from the
entity. No dance, no flicker.

**Ordering concern:** For this to work, `LocallyOwned` must be on the entity BEFORE the removal message arrives and is
processed. The `OwnershipGranted` message uses an ordered-reliable channel. The visibility filter change (that triggers
the removal message) happens on the server in the same frame or the next frame after the ownership event. This means
`OwnershipGranted` → client inserts `LocallyOwned` → removal message arrives. The ordering is correct in the common
case. If the removal message arrives before `OwnershipGranted` (pathological case), the component is still removed and
the 1-frame flicker occurs. Combining C2 (for spawn-time entities) and E (for mid-session gear) covers this.

---

## 6. Open Questions

Questions confirmed from source are marked ✅. Remaining unknowns are marked ❓.

| #   | Question                                                                                                                                     | Why it matters                                                   | Answer                                                                                                                                                                                                              |
| --- | -------------------------------------------------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q1  | When an entity-level visibility filter is added after the entity is already replicated, does Replicon send a despawn to the filtered client? | Determines if Strategy A is usable in non-spawn-time form        | ✅ **YES** — `collect_despawns` sends `add_despawn()` for `VisibilityScope::Entity` entries in `iter_lost()`                                                                                                        |
| Q2  | When a component-level filter returns false for an already-present component, does the client remove that component?                         | Determines the severity of Strategy B's component-removal gotcha | ✅ **YES** — `collect_removals` sends explicit `add_removal(fns_id)` per component via `drain_lost()`                                                                                                               |
| Q3  | What are the exact type signatures of `VisibilityScope` and the `Scope` associated type?                                                     | Required to write any code at all                                | ✅ `type Scope = Entity` or `SingleComponent<C>` or tuples. NOT `ComponentId` or `RepliconId`. See §4.                                                                                                              |
| Q4  | After a visibility-triggered despawn, is the `ServerEntityMap` entry purged?                                                                 | Determines if releasing ownership causes a re-spawn or a resume  | ✅ **YES** — `collect_despawns` calls `ticks.entities.remove(&entity)` which removes the tracking. The server treats this client as "never saw this entity." On filter removal, the server re-sends as a new spawn. |
| Q5  | Does `bevy_replicon` 0.39 expose any API to remove or bypass the `ServerEntityMap` entry for specific entities?                              | The old FIXME mentioned `remove_by_server` was private in 0.38.2 | ❓ Not investigated in this session. The `ServerEntityMap` lives on the client side and may still be private.                                                                                                       |
| Q6  | Is there a `VisibilityPolicy` or equivalent global setting on `ServerPlugin`?                                                                | Affects default behavior for entities without explicit filters   | ✅ **NONE** — No global policy in 0.39. All entities are visible to all clients by default. Filters only restrict.                                                                                                  |
| Q7  | At the moment `setup_mission_players` runs, is the owning client's `RepliconId` guaranteed to be available?                                  | Required for Strategy C (filter at spawn)                        | ❓ Likely yes (clients are identified before `InGame` is reached), but not verified in code.                                                                                                                        |
| Q8  | Can multiple `VisibilityFilter` components be placed on the same entity?                                                                     | Needed if we want to mute multiple components separately         | ✅ **YES** — each filter gets a `FilterBit` (u8). Max **32** filters registered globally across the whole app.                                                                                                      |

---

## 7. Components That Need Ownership Filtering

Based on the replicated component registry (see `17_server_entity_replication_inventory.md`) and the ownership model,
these are the components where the owning client should NOT receive server updates:

| Component                       | Why owner shouldn't receive it                                                                                          |
| ------------------------------- | ----------------------------------------------------------------------------------------------------------------------- |
| `Position`                      | **Primary culprit.** Client writes this locally; server overwrites cause slow movement                                  |
| `Stamina`                       | Client manages running/stamina locally. **BUT** the server may also write it (ghost damage?). Needs thought.            |
| `PlayerSprite` (health, sanity) | Server writes health/sanity from ghost damage. Owner should probably still receive these. **NOT to be muted.**          |
| `HeldObject`                    | Client manages gear locally. But gear changes go via interaction messages, so likely fine to keep server-authoritative. |
| `PlayerGear`                    | Complex. Owner manages gear locally. Server replicates for pickup/drop decisions.                                       |
| `Hiding`                        | Client-side action. Server should know, but if owner is authoritative, blocking inbound is fine.                        |

**Minimum viable fix:** Mute only `Position` for the owning client. Everything else can stay server-replicated for now.
This alone should fix the slow movement bug.

---

## 8. What the "Release Ownership" Flow Looks Like

When the owning client releases ownership (e.g., disconnects, spectates, or we add a mechanism):

1. Client sends a `ReleaseOwnership { final_position, ... }` message to the server.
2. Server receives it, **immediately** writes the final position to the server-side entity **in the same system** that
   removes the filter. This is critical — it ensures the very first packet the client receives after re-subscribing
   contains the correct up-to-date position.
3. Server removes the visibility filter component from the entity.
4. On the next replication tick, the server includes that entity in the packet to the client.
5. If the entity is still in `ServerEntityMap`, the existing local entity receives the update. No re-spawn.
6. If the entity was despawned (despawn-on-filter scenario), the server treats it as a new spawn and the client gets a
   fresh entity. The `ServerEntityMap` gets a new entry.

This is only clean if Q1 is "no despawn." If despawn happens on filter application, release is also problematic.

### Jitter/Pop Warning on Release

There is a potential **1-frame position pop** when ownership is released. If the server's last-known position (from
`ExportStateMessage`) differs significantly from the client's actual position when it sends `ReleaseOwnership`, the
client will see a snap. Mitigation: include the final position in the `ReleaseOwnership` message and apply it
immediately on the server before the filter is removed (step 2 above).

### `ReleaseOwnership` Event Pattern

The release event must include the final state so the server can update its authoritative copy before resuming
replication:

```rust
#[derive(Event, Serialize, Deserialize, Reflect)]
pub struct ReleaseOwnership {
    pub entity: Entity,  // Client-local ID; will be mapped to server ID automatically
    pub final_position: Vec3,
    // Add other owned state fields as needed
}
```

**Critical:** Since this event contains an `Entity` field (which has different values on client vs. server), you MUST
register it with `add_mapped_client_event`, not `add_client_event`. This causes `bevy_replicon` to automatically
translate the client-local `Entity` into the server-side `Entity` before the server system receives it.

```rust
// In app_setup, on both client and server:
app.add_mapped_client_event::<ReleaseOwnership>(ChannelKind::Ordered);
```

The same rule applies to `RequestOwnership` or any other client→server event that references an `Entity` by ID. Failing
to use `add_mapped_client_event` causes the server to receive a garbage `Entity` value — the client's local ECS
generation/index has no meaning on the server.

---

## 9. Current Code Locations

| Concern                                                | File                                              | Lines               |
| ------------------------------------------------------ | ------------------------------------------------- | ------------------- |
| `handle_ownership_granted` (broken Replicated removal) | `crates/unreplicon-plugin/src/systems/players.rs` | ~844–868            |
| Server-side player spawn + Owner assignment            | `crates/unreplicon-plugin/src/systems/players.rs` | ~218–320            |
| `send_export_state` (client → server position)         | `crates/unreplicon-plugin/src/systems/players.rs` | ~751–780            |
| `handle_export_state` (server applies client state)    | `crates/unreplicon-plugin/src/systems/players.rs` | ~487–531            |
| Replicon plugin setup (no visibility policy set)       | `crates/unreplicon-plugin/src/systems/mod.rs`     | ~14–15              |
| `LocallyOwned` component definition                    | `crates/unreplicon-core/src/ownership.rs`         | (search for struct) |
| `OwnershipGranted` message                             | `crates/unreplicon-core/src/messages.rs`          | (search for struct) |

---

## 10. Remaining Open Questions

Most investigation items from earlier in this session are now resolved from source. Only two remain.

| #   | Question                                                                                                    | Why it matters                                                        | Status                                                                                                                                               |
| --- | ----------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------- |
| Q5  | Does `bevy_replicon` 0.39 expose any API to remove a specific entry from the client-side `ServerEntityMap`? | Old FIXME mentioned `remove_by_server` was private in 0.38.2          | ❓ Not investigated. Strategy E (receive markers) makes this moot for our use case.                                                                  |
| Q7  | At the moment `setup_mission_players` runs, is the owning client's `RepliconId` guaranteed to be available? | Required for Strategy C2/F — the filter needs the owner's ID at spawn | ❓ Likely yes (clients are identified before `InGame` is entered), but not verified in code. Check `process_newly_connected_clients` and lobby flow. |

All Q1–Q4, Q6, Q8 are confirmed from source. See §6 for the full table.

---

## 11. Note on Current Workaround vs. "Not Broken Enough"

The current code (`remove::<Replicated>()` on client) partially reduces the frequency of overwrites — the server may or
may not re-insert `Replicated` (behavior is unclear). This produces the "slow" movement rather than "frozen" movement.
It is not safe to leave as-is: it is data-race behavior that depends on Replicon internals and may change in future
versions.

The fix described in this document should be treated as **correctness work**, not optimization.

---

## 12. Should We File a GitHub Issue?

**Short answer: Yes, if bandwidth allows. It is not a blocker.**

**What to request:** An official "client-authoritative component" primitive. Proposed names:

- `ClientAuthority<C>` — a component that, when present on an entity, tells Replicon to never overwrite component `C`
  with server-sent values for the client that "owns" it.
- `NoReplicateIncoming<C>` — an entity marker directing the client-side replication system to silently discard updates
  for component `C`.

**Why it's needed:** The goal is to have a component that is _written by the client_ and _read by the server_ (via
`ExportStateMessage`), but never _sent back to_ the owning client by the server. `bevy_replicon` has no first-class
concept for this — replication is strictly one direction (server → all clients).

**Why our current workaround is risky:** We are repurposing:

- Visibility filters (designed for fog-of-war / spatial visibility) to suppress server sends.
- Receive markers (designed for client-side prediction / interpolation) to suppress client-side applies.

Both APIs are documented with different intent. Future versions of `bevy_replicon` may change edge case behaviors
(reconnection, mid-session join, entity respawn) in ways that break our repurposed usage without a deprecation warning.

**What to include in the issue:**

1. Use case: client-owned movement components in a hybrid authoritative model. Client sends position to server; server
   broadcasts to others; owning client should not receive its own position back.
2. Current workaround: visibility filters + receive markers. Functional but unintended use.
3. Proposed API: something like `app.set_client_authority::<C>()` that internally does the receive-marker no-op and
   optionally a visibility filter.
4. Reference: the `ExportStateMessage` / CSP pattern is similar — there may already be roadmap items for this.

**Upstream link:** `https://github.com/projectharmonia/bevy_replicon` — file under Issues as a Feature Request.

**Priority:** Low. Our repurposed workaround (Strategies E or F) is functional. File the issue when convenient, not as a
prerequisite to implementation.
