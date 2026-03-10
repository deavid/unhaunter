# 18 — Client Ownership & Replicon Visibility Research

- **Date:** March 2026
- **Branch:** `dev-deavid`
- **Status:** Architecture decided. Strategy E (Receive Markers) is the implementation plan. Ready to code.
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

The key invariant: **a client should never receive replicated updates for its own entities after ownership is
established.** The client is the sole authority over all components it writes on owned entities.

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

The `lib.rs` crate-level documentation states:

> "Replication happens only from server to clients. It's necessary to prevent cheating."

`bevy_replicon` is server-authoritative by design and has no first-class "client authority component". However, this
does not make our use case unsupported or fragile.

**Client-Side Prediction (CSP) makes our use case fully supported.** The `Receive Markers` API (`AppMarkerExt`) was
designed for CSP and interpolation. CSP fundamentally requires the ability to discard server updates — the client runs
ahead of the server, and when the delayed server state arrives, it either reconciles or discards. Permanently discarding
server updates for `LocallyOwned` entities is the "always trust client" end of that spectrum. The library cannot break
this without breaking its own CSP infrastructure.

There is no stability risk. Using `noop_write` + `noop_remove` on `LocallyOwned` entities is a first-class supported
idiom within the receive markers API. No GitHub issue is needed. See §12.

**Visibility filters are a separate tool for a separate purpose.** Their confirmed behavior — despawn on entity-scope
visibility loss, component removal on component-scope loss (see §4.7, Q1, Q2) — makes them a trap for client ownership
control. They are the correct tool for lobby-to-game entity gating. See §5.4.

---

## 5. Strategy Options

### 5.1 Summary Table

All viable options at a glance. Mechanism, sides affected, gear pickup safety, and score.

| Option | Name                                           | Server changes? | Client changes?             | Gear pickup safe?                         | Score |
| ------ | ---------------------------------------------- | --------------- | --------------------------- | ----------------------------------------- | ----- |
| A      | Entity blacklist post-spawn                    | Yes             | No                          | ✗ (despawn)                               | ★☆☆☆☆ |
| B      | Component filter post-spawn + re-add observer  | Yes             | Yes (observer)              | ✗ (1-frame flicker)                       | ★★☆☆☆ |
| C1     | Entity scope at spawn (full entity blacklist)  | Yes             | Needs client spawn rethink  | ✗ (client never receives initial state)   | ✗     |
| C2     | Component scope at spawn (only mutes Position) | Yes             | None                        | ✗ (client has no initial state either)    | ✗     |
| E      | Receive markers only (client-side no-op)       | None            | Yes (register marker + fns) | ✓ if LocallyOwned inserted before removal | ★★★★★ |
| F      | C2 + E combined ("belt and suspenders")        | Yes             | Yes                         | ✗ (BROKEN — Despawn bypasses markers)     | 0★    |
| D      | Hybrid: replicate once then block              | Yes             | Yes (complex)               | ✗                                         | ★☆☆☆☆ |
| G      | Save/restore buffer around replication         | None            | Yes (fragile workaround)    | ✓                                         | ★☆☆☆☆ |

**Strategy E is the only viable option.** All others are either traps or redundant. See detailed sections below.

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

**Why mitigations don't work:**

- **Removing `Replicated` on the client:** Ineffective. Removing `Replicated` client-side has no effect on what the
  server sends — this is the original broken workaround documented in §1. The server's despawn decision is driven by the
  server-side visibility filter result, not by any component on the client entity.
- **Parent anchor:** Parent-child despawn behavior is version-dependent and not guaranteed to protect children.
- **Observer interception:** Observe `RemovedComponents::<Replicated>` to re-spawn. Fights the engine and may leave
  `ServerEntityMap` in an inconsistent state.

**Best use of this pattern:** At spawn time (Strategy C1) before the entity is ever replicated.

---

### Strategy B — Component-Level Filter Post-Spawn + Re-add Observer ★☆☆☆☆

**Score rationale:** Does not fix despawn (just switches from entity loss to component loss). Requires per-component
observers, a re-add dance, and still loses the component for 1 frame. More ad-hoc work than A for a worse result.

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

### Strategy C1 — Entity Scope at Spawn ✗

**Why it doesn't work:** The client must receive the full initial entity state (position, appearance, all components) to
know where the entity is and what it looks like before taking ownership. "Never send it" defeats the purpose: the player
entity spawns on the owning client with no data.

---

### Strategy C2 — Component Scope at Spawn ✗

**Why it doesn't work:** Fixing only `Position` misunderstands the problem. Every replicated component on a
locally-owned entity needs shielding, not just one. More fundamentally: if `Position` is filtered at spawn, the owning
client's player entity arrives without knowing where it spawned. The client needs the complete initial state before it
can take ownership.

---

### Strategy E — Receive Markers with No-Op Functions (Client-Side Only) ★★★★★

**This is the chosen implementation.** Zero server changes required. The server continues broadcasting everything; the
client uses `LocallyOwned` as an impenetrable shield for all owned components. This is a supported use of the CSP
infrastructure — see §4.9.

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
    rule_fns.consume(ctx, message) // advance the deserializer, discard the bytes
}

pub fn noop_remove(_ctx: &mut RemoveCtx, _entity: &mut DeferredEntity) {
    // intentionally empty — ignore server removal
}
```

`rule_fns.consume(ctx, message)` advances the deserializer past this component's bytes without applying them. This is
the standard discard path — every receive function either writes data or discards it. Always call it in `noop_write`.

**How it works for gear pickup mid-session:**

1. Client picks up gear. Server sends `OwnershipGranted` for the gear entity (ordered-reliable channel).
2. Client receives `OwnershipGranted`, inserts `LocallyOwned` on the gear entity.
3. All subsequent server writes for registered components on that entity are silently discarded via the noop functions.
   No flicker, no component removal, no despawn.

**Boundary:** `noop_remove` only intercepts **Component Removal** packets. If the server despawns the entity,
`bevy_replicon` sends an explicit **Despawn** packet which is processed at the packet-header level, before the component
registry is consulted. The entity is destroyed regardless of `LocallyOwned`. The server retains full authority to
destroy entities; the client controls only the data inside living entities. This is correct by design.

---

### Strategy F — C2 + E Combined ✗ (0 Stars — Fundamentally Broken)

**Why it's broken:** Adding server-side visibility filters (the C2 part) is a trap. When a visibility filter loses
visibility for an already-replicated entity, `bevy_replicon` processes the result based on the filter's `Scope` type:

- `Scope = Entity`: the server sends a **Despawn** packet. This is processed at the packet-header level, before the
  component registry is consulted. `noop_remove` is never invoked. The entity is destroyed on the client regardless of
  `LocallyOwned`.
- `Scope = SingleComponent<C>`: the server sends a **Component Removal** packet, which `noop_remove` would intercept.
  But now we have two systems (server filter + client no-op) doing the same job redundantly, with Despawn risk if anyone
  ever uses the wrong Scope.

Do not combine visibility filters with receive markers for ownership. Trust Strategy E alone.

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

### 5.6 The Legitimate Use of Visibility Filters (Not Client Ownership)

Visibility filters are the right tool for a different problem we will need to solve: **clients that connect to the
server should not immediately receive the entire replicated game world while still in the lobby.**

A filter on game-world entities (map, ghost, gear) that returns `false` for lobby-state clients keeps the initial
connection from flooding clients with data they can't act on. When a client transitions to `InGame`, the filter is
removed and Replicon sends the initial game state.

This is the canonical fog-of-war / lobby-gating use case these filters were designed for. It is entirely separate from
the client ownership problem.

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

## 7. Component Ownership: What Gets Shielded

**The rule is simple: every replicated component on a `LocallyOwned` entity is shielded.** When an entity is locally
owned, the client is the authority on its state. Register `noop_write` and `noop_remove` for every component type that
is replicated. Do not maintain a curated list — a list implies completeness and any new replicated component would
silently bypass the shield.

### Component Taxonomy

Three categories of components exist. Communicate them via **module scoping** rather than naming prefixes or suffixes:

| Category                  | Description                                                                                                                    | Network behavior                                                                                                         | Example module path         |
| ------------------------- | ------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------ | --------------------------- |
| **Skeleton Ownable**      | Components the owning client writes; server tracks and rebroadcasts to others. Shield with `LocallyOwned`.                     | Replicated server→clients; client sends authoritative value back via `ExportStateMessage`. Incoming discarded by client. | `crate::skeleton::Position` |
| **Skeleton Unshieldable** | Components the server controls even on owned entities (e.g., a server-applied status effect). Exempt from `noop` registration. | Replicated server→clients; client must receive.                                                                          | `crate::skeleton::Stunned`  |
| **Skin**                  | Local/visual components. Never replicated.                                                                                     | Never on the network.                                                                                                    | `crate::skin::PlayerSprite` |

**Module scoping as enforcement:** When reviewing a client-side system that mutates a `skeleton::` component without
`With<LocallyOwned>`, that is an immediate code-review flag. The import path carries the intent. No suffixes needed.

### Edge Case: Unshieldable Components

At time of writing, no confirmed Skeleton Unshieldable components exist. When the need arises (e.g., server applies a
stun to a locally-owned player), the two options are:

1. Use a separate server-authoritative component exempt from the shield. Client observes it and applies consequences
   locally.
2. Route the effect through `ExportStateMessage` (client receives a server message, applies locally, includes result in
   next export).

Option 1 is simpler for most effects. Decide per-component when the need arises.

---

## 8. What the "Release Ownership" Flow Looks Like

When the owning client releases ownership (e.g., drops gear, disconnects, or the server revokes it):

**With Strategy E (receive markers only — no server-side visibility filters):**

1. Client sends a final `ExportStateMessage` with its current authoritative state. The server applies it immediately,
   ensuring it has the correct value before resuming authority.
2. Client removes `LocallyOwned` from the entity.
3. Receive markers no longer apply — server updates now land as normal on that entity.
4. On the next replication tick the client receives the server's current value, which should be in sync with step 1.

No filter to remove. No `ServerEntityMap` manipulation. The server was always broadcasting; the client simply stops
discarding.

### Jitter/Pop Warning on Release

There is a potential **1-frame position pop** when ownership is released. If the server's last-known position (from
`ExportStateMessage`) differs significantly from the client's actual position when it sends `ReleaseOwnership`, the
client will see a snap. Mitigation: send the final `ExportStateMessage` in the same frame as removing `LocallyOwned`,
before the marker is stripped, so the server has the latest value immediately.

### `ReleaseOwnership` Event Pattern

The release event must include the final state so the server can update its authoritative copy before resuming
replication:

```rust
#[derive(Event, Serialize, Deserialize, Reflect)]
pub struct ReleaseOwnership {
    pub entity: Entity,  // Client-local ID; will be mapped to server ID automatically
    // Final state is carried by ExportStateMessage, not this event
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

## 10. Open Questions — Status

All questions are now resolved or moot given Strategy E.

| #   | Question                                                                                                    | Status                                                                                                         |
| --- | ----------------------------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------- |
| Q5  | Does `bevy_replicon` 0.39 expose any API to remove a specific entry from the client-side `ServerEntityMap`? | ✅ **Moot** — Strategy E does not touch `ServerEntityMap`. `remove_by_server` is a dead end; do not pursue it. |
| Q7  | At the moment `setup_mission_players` runs, is the owning client's `RepliconId` guaranteed to be available? | ✅ **Moot** — Strategy E requires no `RepliconId` at spawn time. No server-side filter is applied.             |

All other questions (Q1–Q4, Q6, Q8) are confirmed from source. See §6 for the full table.

---

## 11. Note on Current Workaround vs. "Not Broken Enough"

The current code (`remove::<Replicated>()` on client) partially reduces the frequency of overwrites — the server may or
may not re-insert `Replicated` (behavior is unclear). This produces the "slow" movement rather than "frozen" movement.
It is not safe to leave as-is: it is data-race behavior that depends on Replicon internals and may change in future
versions.

The fix described in this document should be treated as **correctness work**, not optimization.

---

## 12. Should We File a GitHub Issue?

**No. No issue is needed.**

The `Receive Markers` API (`AppMarkerExt`) covers our use case as a supported, stable feature. Permanently discarding
server updates via `noop_write` / `noop_remove` is the "always trust client" end of the Client-Side Prediction spectrum
— a first-class idiom in the library. The library authors cannot deprecate or break this without also breaking CSP
itself.

There is no feature gap. The existing API is sufficient and correctly captures our intent.
