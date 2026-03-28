# PR 4 Specification: Map Loading & Stitching (Workstream 1)

> **Context for AI Agents (Copilot/Jules):** This is the final architectural pillar of the _Unhaunter_ multiplayer
> refactor. **The Problem:** The game currently has a "Mirror World" map. The server and client both load the exact same
> `.tmx` map from disk, generating two completely separate sets of entities. The server cannot natively replicate
> dynamic state (like an open door) because the client has its own disconnected version of that door. **The Solution:**
> We implement a **"Deterministic Spawn & Stitch"** pipeline.
>
> 1. We define a unique `TmxEntityId` for every interactive object.
> 2. The Server adds `Replicated` to these objects. The Client does not.
> 3. A Client-side "Stitcher" observer watches for incoming `Replicated` map entities, finds the local matching
>    `TmxEntityId`, and transfers the graphical "Flesh" from the local entity to the Server's authoritative entity.

---

## 1. Scope Boundaries

| In scope                                              | Out of scope                                                   |
| :---------------------------------------------------- | :------------------------------------------------------------- |
| `unbehavior` (TmxEntityId, RoomDB refactor)           | Replicating static walls or floors                             |
| Map Spawning loops (where TMX becomes Entities)       | Map file asset parsing logic (`untmxmap-plugin` loading logic) |
| `unreplicon-plugin` (The Stitcher system)             |                                                                |
| `uninteraction-plugin` (Separating State from Flavor) |                                                                |

---

## 2. Context & Goal

The game currently loads the same `.tmx` map on both server and client, producing two fully independent sets of
entities. When the server opens a door, it mutates its own `Behavior` component but has no way to replicate that
specific entity's state to the client, because the client has a completely different entity for the same door.

This PR establishes the foundation:

1. **Anchor** every interactive map entity with a `TmxEntityId` (stable, deterministic key based on Tiled layer index +
   tile position).
2. **Tag** server-side interactive entities with `Replicated` so Replicon tracks them.
3. **Stitch** incoming replicated entities on the client: transfer the client's locally-spawned visual components onto
   the replicated entity and despawn the placeholder.
4. **Split** `RoomDB` into `RoomTopology` (topology, all nodes) and `RoomStateMap` (mutable state, all nodes but only
   written by Authority).

> **Critical constraint:** `Behavior` replication is **deferred** — `Behavior` is not currently serializable and must
> NOT be made serializable in this PR. The existing `RemoteInteractionBroadcast` → `apply_remote_interaction` →
> `ExecuteInteractionEvent` path is the only mechanism syncing door/switch state to clients and must remain intact and
> unchanged. Do not add `Serialize`/`Deserialize` to `Behavior`, do not call `app.replicate::<Behavior>()`, do not add
> `silent`/`flavor_only` flags to `execute_interaction`.

---

## 3. Implementation Steps

### Step 1: Define `TmxEntityId` in `unbehavior`

**File:** `crates/unbehavior/src/components.rs`

Add the following at the end of the file (after the existing `RoomState` component):

```rust
/// Stable Tiled-space identifier for dynamic map entities.
///
/// Uniquely identifies an entity by the Tiled layer it came from and the tile's
/// raw Tiled coordinates. This is the anchor used by the client-side "Stitcher"
/// to find the local placeholder entity that corresponds to a server-replicated
/// dynamic entity.
///
/// `layer_idx` is the 0-based index of the Tiled layer in the enumerated
/// `tile_layers_iter()` in `unmapload-plugin/src/level_setup.rs`.
/// `x` and `y` are the raw `tile.pos.x` and `tile.pos.y` from the Tiled map,
/// **before** the coordinate transformation applied in `process_and_spawn_tile`.
#[derive(Component, Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
pub struct TmxEntityId {
    pub layer_idx: usize,
    pub x: i32,
    pub y: i32,
}
```

`Serialize` and `Deserialize` are already available in `unbehavior` via:

```toml
serde = { workspace = true }
```

`Reflect` is available via `bevy = { workspace = true }`.

Do NOT add `TmxEntityId` to `unbehavior/src/lib.rs` as a re-export (the project forbids `pub use`). The canonical path
is `unbehavior::components::TmxEntityId`.

---

### Step 2: Split `RoomDB` into `RoomTopology` and `RoomStateMap`

This step touches many files. Follow this order to keep the project compilable between sub-steps.

#### 2a. Rewrite `crates/unbehavior/src/roomdb.rs`

Replace the entire file:

```rust
use bevy::prelude::*;
use bevy_platform::collections::HashMap;

use crate::state::TileState;
use unspatial_core::boardposition::BoardPosition;

/// Maps each board position to the room name it belongs to.
///
/// All nodes (server and client) maintain this. It is populated during
/// `HydrationStage<2>` in `unrender-plugin` and reset on `OnExit(AppState::InGame)`.
#[derive(Clone, Default, Resource)]
pub struct RoomTopology {
    pub room_tiles: HashMap<BoardPosition, String>,
}

impl RoomTopology {
    pub fn reset(&mut self) {
        self.room_tiles.clear();
    }
}

/// Tracks the current TileState of each named room (e.g. On/Off for lights).
///
/// Written exclusively on the Authority node (server or offline host).
/// All nodes hold this resource (initialised to TileState::Off per room during
/// `HydrationStage<2>`), but only the Authority mutates it after initialisation.
/// Pure clients read the initial default values; state changes reach clients via
/// `RemoteInteractionBroadcast` (existing mechanism, unchanged in this PR).
#[derive(Clone, Default, Resource)]
pub struct RoomStateMap {
    pub room_state: HashMap<String, TileState>,
}

impl RoomStateMap {
    pub fn reset(&mut self) {
        self.room_state.clear();
    }
}
```

#### 2b. Update all `use unbehavior::roomdb::RoomDB` import sites

Every file in the following list currently imports `RoomDB`. Change them to import `RoomTopology` (and `RoomStateMap`
where the file also accesses `room_state`):

**Files that only use `room_tiles` (import `RoomTopology` only):**

- `crates/unlight-plugin/src/audio.rs`
- `crates/unlight-plugin/src/maplight/visibility.rs`
- `crates/unlight-plugin/src/maplight/systems/gathering.rs`
- `crates/unsound-plugin/src/systems.rs`
- `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs`
- `crates/unfog-plugin/src/systems.rs`
- `crates/unwalkie-plugin/src/triggers/locomotion_interaction.rs`
- `crates/unwalkie-plugin/src/triggers/repellent_expulsion.rs`
- `crates/unwalkie-plugin/src/triggers/player_wellbeing.rs`
- `crates/unwalkie-plugin/src/triggers/ghost_behavior_and_hunting.rs`
- `crates/unwalkie-plugin/src/triggers/environmental_awareness.rs`
- `crates/unwalkie-plugin/src/triggers/consumables_and_defense.rs`

For each: replace `use unbehavior::roomdb::RoomDB;` with `use unbehavior::roomdb::RoomTopology;`, and replace all
occurrences of `Res<RoomDB>` / `ResMut<RoomDB>` / `&RoomDB` / `&mut RoomDB` with the `RoomTopology` equivalent. All
actual field accesses (`roomdb.room_tiles.get(...)`) stay the same; just the type name changes.

**Files that use both `room_tiles` and `room_state`:**

- `crates/unrender-plugin/src/systems/hydration.rs` — see 2c
- `crates/uninteraction-plugin/src/systems/interactivestuff.rs` — see 2d
- `crates/unmapload-plugin/src/level_setup.rs` — see 2e

#### 2c. Update `unrender-plugin/src/systems/hydration.rs`

`hydration_simulation_system` currently takes `mut roomdb: ResMut<RoomDB>`. Change to:

```rust
fn hydration_simulation_system(
    mut q: Query<(Entity, &Behavior, &unspatial_core::position::Position), With<HydrationStage<2>>>,
    mut roomtopo: ResMut<RoomTopology>,
    mut roomstate: ResMut<RoomStateMap>,
    mut commands: Commands,
) {
    // ...
    if let Util::RoomDef(name) = &behavior.p.util {
        roomtopo.room_tiles.insert(pos.to_board_position(), name.to_owned());
        roomstate.room_state.insert(name.clone(), TileState::Off);
    }
}
```

`RoomStateMap` is available on all nodes (it is `init_resource`'d in Step 2f below), so no `run_if` guard is needed here
for the initialisation write.

#### 2d. Update `uninteraction-plugin/src/systems/interactivestuff.rs`

The `InteractiveStuff` SystemParam and all methods use `RoomDB`. Changes:

1. Import: replace `use unbehavior::roomdb::RoomDB;` with:

   ```rust
   use unbehavior::roomdb::{RoomTopology, RoomStateMap};
   ```

2. In the `InteractiveStuff` struct, replace `pub roomdb: ResMut<'w, RoomDB>,` with:

   ```rust
   pub roomtopo: ResMut<'w, RoomTopology>,
   pub roomstate: ResMut<'w, RoomStateMap>,
   ```

3. In `synchronize_entity`: replace `self.roomdb.room_tiles` with `self.roomtopo.room_tiles` and
   `self.roomdb.room_state` with `self.roomstate.room_state`.
4. In `execute_interaction`: same replacements. The line:

   ```rust
   self.roomdb.room_state.get_mut(&room_name)
   ```

   becomes:

   ```rust
   self.roomstate.room_state.get_mut(&room_name)
   ```

   and:

   ```rust
   self.roomdb.room_state.get(&room_name)
   ```

   becomes:

   ```rust
   self.roomstate.room_state.get(&room_name)
   ```

Do NOT change any logic. The `authority == Authority::Host` guard inside `execute_interaction` that gates
`room_state.get_mut()` already ensures only the server mutates it.

#### 2e. Update `unmapload-plugin/src/level_setup.rs`

`LoadLevelSystemParam` imports and uses `RoomDB`. Changes:

1. Import: replace `use unbehavior::roomdb::RoomDB;` with:

   ```rust
   use unbehavior::roomdb::{RoomTopology, RoomStateMap};
   ```

2. In `LoadLevelSystemParam`, replace `pub roomdb: ResMut<'w, RoomDB>,` with:

   ```rust
   pub roomtopo: ResMut<'w, RoomTopology>,
   pub roomstate: ResMut<'w, RoomStateMap>,
   ```

3. In `load_level_handler`, replace:

   ```rust
   p.roomdb.room_state.clear();
   p.roomdb.room_tiles.clear();
   ```

   with:

   ```rust
   p.roomtopo.room_tiles.clear();
   p.roomstate.room_state.clear();
   ```

4. In `reset_level_resources`, replace:

   ```rust
   pub(crate) fn reset_level_resources(mut roomdb: ResMut<RoomDB>, mut sdb: ResMut<SpriteDB>) {
       roomdb.reset();
   ```

   with:

   ```rust
   pub(crate) fn reset_level_resources(
       mut roomtopo: ResMut<RoomTopology>,
       mut roomstate: ResMut<RoomStateMap>,
       mut sdb: ResMut<SpriteDB>,
   ) {
       roomtopo.reset();
       roomstate.reset();
   ```

#### 2f. Register the new resources in `unmapload-plugin` (or wherever `RoomDB` was registered)

Search for where `app.init_resource::<RoomDB>()` is called. Replace with:

```rust
app.init_resource::<RoomTopology>();
app.init_resource::<RoomStateMap>();
```

Both resources must be registered on **all** nodes. `RoomStateMap` is only _written_ by the Authority after
initialisation, but it is present on clients as well (holding the initial `TileState::Off` defaults from hydration).

---

### Step 3: Tag dynamic map entities with `TmxEntityId` during spawning

**Files:** `crates/unmapload-plugin/src/level_setup.rs` and `crates/unmapload-plugin/src/tile_spawning.rs`

#### 3a. Enumerate the layer loop in `level_setup.rs`

In `load_level_handler`, the tile spawning loop is:

```rust
for (maptiles, layer) in tile_layers_iter() {
    let floor_z = ...;
    for tile in &maptiles.v {
        tile_spawning::process_and_spawn_tile(
            tile, layer, origin.0, origin.1, map_size, floor_z, &mut p, &mut commands, &mut depth_counter,
        );
    }
}
```

Change the outer loop to track `layer_idx`:

```rust
for (layer_idx, (maptiles, layer)) in tile_layers_iter().enumerate() {
    let floor_z = ...;
    for tile in &maptiles.v {
        tile_spawning::process_and_spawn_tile(
            tile, layer, layer_idx, origin.0, origin.1, map_size, floor_z, &mut p, &mut commands, &mut depth_counter,
        );
    }
}
```

#### 3b. Add `layer_idx` parameter to `process_and_spawn_tile` in `tile_spawning.rs`

Add `layer_idx: usize` as a new parameter (after `layer: &MapLayer`). Update the call site in `level_setup.rs` to pass
it.

At the top of `tile_spawning.rs`, add the import:

```rust
use unbehavior::components::TmxEntityId;
use bevy_replicon::prelude::Replicated;
use uncommon-app_core::roles::AuthorityRole;
```

#### 3c. Insert `TmxEntityId` (and conditionally `Replicated`) at the end of `process_and_spawn_tile`

After the existing component insertions (after `.insert(Visibility::Hidden)`), add:

```rust
// Determine if this entity is "dynamic" — needs network identity for replication.
let is_dynamic = beh.p.is_door
    || beh.p.is_switch
    || beh.p.is_room_switch
    || beh.p.is_breaker
    || beh.p.is_floor_light
    || beh.p.is_table_light
    || beh.p.object.movable;

if is_dynamic {
    let tmx_id = TmxEntityId {
        layer_idx,
        x: tile.pos.x,
        y: tile.pos.y,
    };
    entity.insert(tmx_id);

    // Only the Authority (server / offline host) adds Replicated.
    // The resource_exists check is evaluated at call time inside
    // process_and_spawn_tile; pass it in as a bool from the caller.
    if p.is_authority {
        entity.insert(Replicated);
    }
}
```

**Note on `is_authority`:** `process_and_spawn_tile` receives `p: &mut LoadLevelSystemParam`. Add
`pub is_authority: bool` to `LoadLevelSystemParam` by adding a derived field:

```rust
pub authority: Option<Res<'w, AuthorityRole>>,
```

Then compute `let is_authority = p.authority.is_some();` at the call site (top of the outer loop in
`load_level_handler`), or pass it through `p`. Do NOT poll `resource_exists` inside `process_and_spawn_tile` directly;
`LoadLevelSystemParam` is the right place for it.

**Why `x: tile.pos.x, y: tile.pos.y` (raw Tiled coords)?** The raw Tiled coordinates, before the
`(tile.pos.x - map_min_x)` offset, are stable across different map loads (they come from the `.tmx` file directly).
Using the post-offset board coordinates would make `TmxEntityId` dependent on `map_min_x`/`map_min_y`, which should be
avoided.

**Warning:** Do NOT add `TmxEntityId` to static tiles (floors, walls, opaque walls). Only dynamic interactive objects
need it. The `is_dynamic` predicate above is the gate.

---

### Step 4: Register `TmxEntityId` for replication and build the Stitcher

#### 4a. Register `TmxEntityId` in `unreplicon-plugin/src/systems/players.rs`

In `players::app_setup`, add:

```rust
app.replicate::<TmxEntityId>();
```

with import:

```rust
use unbehavior::components::TmxEntityId;
```

The `unreplicon-plugin/Cargo.toml` already contains `unbehavior = { path = "../unbehavior" }`; no dependency change is
needed.

#### 4b. Create `crates/unreplicon-plugin/src/systems/map_sync.rs`

> **Important:** Because `Behavior` replication is deferred to a future PR (Option A), the server entity arrives on the
> client _without_ `Behavior`, `Interactive`, `RoomState`, or `MapEntityFieldBPos`. If the stitcher only transferred
> visual components and then despawned the local placeholder, _all logic components would be permanently lost_. The
> `RemoteInteractionBroadcast` handler queries for `MapEntityFieldBPos` to find entities; without it the door becomes
> invisible to interaction lookup. Similarly, `Mesh2d` and `MeshMaterial2d` are not present on the local placeholder
> until later hydration stages, so they must be optional.
>
> The stitcher must therefore transfer **both** visual and logic components, with visuals as optional to handle the
> hydration race.

```rust
use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::components::{RoomState, TmxEntityId};
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unrender_std::components::game::GameSprite;
use unrender_std::materials::CustomMaterial1;
use unspatial_core::boardposition::MapEntityFieldBPos;
use unspatial_core::position::Position;
use uncommon-app_core::roles::is_pure_client;

pub(super) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        stitch_map_entities.run_if(is_pure_client),
    );
}

/// Client-side stitcher: when a server-replicated dynamic entity arrives (carrying
/// `TmxEntityId` + `Replicated`), find the matching locally-spawned placeholder,
/// transfer ALL components (visual and logic) to the replicated entity, then despawn
/// the placeholder.
///
/// Run condition: `is_pure_client` only. On a host node the server and client share
/// the same entities, so no stitching is required or safe.
///
/// ## Why logic components must be transferred
///
/// Because `Behavior` is not yet replicated, the server entity arrives on the client
/// without `Behavior`, `Interactive`, `RoomState`, or `MapEntityFieldBPos`. The client's
/// `apply_remote_interaction` system looks up interactive entities via `MapEntityFieldBPos`.
/// If these components are discarded when the placeholder is despawned, all door/switch
/// interactions on the client become permanently broken.
///
/// ## Why visual components are optional
///
/// Local placeholders begin at `HydrationStage<1>`. `Mesh2d` and `MeshMaterial2d` are not
/// added until later hydration stages. If the server entity arrives before hydration
/// completes, the mesh query fails and `Added<TmxEntityId>` expires — the stitch window
/// is lost forever. Making mesh and material optional allows the stitch to proceed with
/// logic components regardless of hydration progress; the visuals will be absent until the
/// local placeholder finishes hydrating, at which point the stitcher will not re-fire
/// (because `Added<TmxEntityId>` has already been consumed). This is an acceptable
/// trade-off: a briefly invisible door is better than a permanently non-interactive one.
/// In practice the server entity arrives after the client has had time to hydrate, so
/// this race should be rare.
fn stitch_map_entities(
    mut commands: Commands,
    q_server: Query<(Entity, &TmxEntityId), (Added<TmxEntityId>, With<Replicated>)>,
    q_local: Query<
        (
            Entity,
            &TmxEntityId,
            &Transform,
            &Position,
            &Visibility,
            &Behavior,
            Option<&Interactive>,
            Option<&RoomState>,
            Option<&MapEntityFieldBPos>,
            Option<&MeshMaterial2d<CustomMaterial1>>,
            Option<&Mesh2d>,
        ),
        Without<Replicated>,
    >,
) {
    for (server_ent, server_id) in q_server.iter() {
        // Find the local placeholder that matches this replicated entity.
        let found = q_local
            .iter()
            .find(|(_, local_id, ..)| *local_id == server_id);

        let Some((
            local_ent,
            _,
            transform,
            position,
            visibility,
            behavior,
            interactive,
            room_state,
            map_bpos,
            material,
            mesh,
        )) = found
        else {
            warn!(
                "stitch_map_entities: replicated entity {:?} has TmxEntityId {:?} but no \
                 matching local placeholder was found. Skipping.",
                server_ent, server_id
            );
            continue;
        };

        // Build the insert bundle for the server entity. Start with components
        // that are always present on a hydrated placeholder.
        let mut ec = commands.entity(server_ent);
        ec.insert((
            *transform,
            *position,
            *visibility,
            behavior.clone(),
            GameSprite, // ensure cleanup marker is present on mission exit
        ));

        // Transfer optional logic components.
        if let Some(i) = interactive {
            ec.insert(i.clone());
        }
        if let Some(rs) = room_state {
            ec.insert(rs.clone());
        }
        if let Some(bpos) = map_bpos {
            ec.insert(*bpos);
        }

        // Transfer optional visual components (may be absent if hydration is not yet done).
        if let Some(mat) = material {
            ec.insert(mat.clone());
        }
        if let Some(m) = mesh {
            ec.insert(m.clone());
        }

        // Despawn the now-redundant placeholder.
        commands.entity(local_ent).despawn();

        debug!(
            "stitch_map_entities: stitched local {:?} → server {:?} (id={:?})",
            local_ent, server_ent, server_id
        );
    }
}
```

**Key design notes for Jules:**

- The query on `q_server` uses `(Added<TmxEntityId>, With<Replicated>)` — NOT just `Added<TmxEntityId>`. This ensures
  only replicated-from-server entities trigger the stitch. Local placeholders also have `TmxEntityId` but never have
  `Replicated`, so they will never appear in `q_server`. This is the fix for Critical Issue 3.

- The `q_local` query uses `Without<Replicated>`. Local placeholders are never marked `Replicated`, so they are always
  in this query until despawned.

- `Behavior` is always present on a map entity that has finished spawning. If it is somehow absent (pre-hydration Stage
  1 race), the query will find no match and the warn log fires. This is acceptable — it is a far better outcome than a
  silent logic lobotomy.

- `Behavior` transferred here is the **locally-computed** copy (from the `.tmx`/`SpriteDB`). It is authoritative for its
  _initial_ state. When `RemoteInteractionBroadcast` fires for a subsequent interaction, `apply_remote_interaction`
  calls `execute_interaction` on the stitched entity, which updates the `Behavior` component via `apply_visual_update`
  as it does today. Nothing in this PR changes that flow.

- `MapEntityFieldBPos` is critical. The `apply_remote_interaction` handler in `unreplicon-plugin/src/systems/players.rs`
  finds its target entity by `Query<(Entity, &MapEntityFieldBPos)>`. Without this component on the stitched entity, the
  client can never respond to door/switch broadcasts from the server.

#### 4c. Register the new module in `crates/unreplicon-plugin/src/systems/mod.rs`

Add `mod map_sync;` and call `map_sync::app_setup(app);` inside `app_setup`.

**Strictly follow the project rule: `mod.rs` contains ONLY `mod` declarations and `app_setup` function calls. No logic.
No `use` statements beyond what is needed to call sub-module functions.**

---

---

## 4. Explicit Out-of-Scope Constraints

The following must NOT be implemented in this PR:

- Making `Behavior` serializable / adding `Serialize + Deserialize` derives to `Behavior`, `SpriteConfig`, or
  `Properties`.
- Calling `app.replicate::<Behavior>()`.
- Adding a `silent: bool` or `flavor_only: bool` flag to `execute_interaction`.
- Modifying `apply_remote_interaction` to suppress state mutations.
- Removing or bypassing the `RemoteInteractionBroadcast` → `ExecuteInteractionEvent` path on clients.

The existing interaction broadcast mechanism is sound for the current "Mirror World" setup and must remain intact. The
goal of this PR is to establish the infrastructure (`TmxEntityId`, stitching, and the `RoomDB` split) that future PRs
will build on.

---

## 5. Verification Checklist

> All items must pass before submitting.

- [ ] `cargo clippy` reports zero errors and zero warnings on a global run.
- [ ] `RoomDB` no longer exists anywhere in `crates/`. Every import is either `RoomTopology` or `RoomStateMap`.
- [ ] `RoomStateMap` (the new resource) does NOT have the same name as the `RoomState` component in
      `unbehavior::components`. Confirm by grepping: `grep -r "struct RoomState" crates/`.
- [ ] `TmxEntityId` is defined in `unbehavior::components` with the correct derives.
- [ ] `app.replicate::<TmxEntityId>()` is called exactly once (in `unreplicon-plugin`).
- [ ] `TmxEntityId` is inserted on dynamic entities during `process_and_spawn_tile` for the correct predicate
      (`is_door || is_switch || is_room_switch || is_breaker || is_floor_light ||     is_table_light || object.movable`).
- [ ] `Replicated` is added to dynamic entities only when `AuthorityRole` is present.
- [ ] `stitch_map_entities` uses `(Added<TmxEntityId>, With<Replicated>)` — NOT bare `Added<TmxEntityId>`.
- [ ] `stitch_map_entities` runs only under `.run_if(is_pure_client)`.
- [ ] `apply_remote_interaction` in `unreplicon-plugin/src/systems/players.rs` is unchanged.
- [ ] `execute_interaction` in `interactivestuff.rs` has no new `silent` or `flavor_only` parameter.
- [ ] `hydration_simulation_system` writes to both `RoomTopology` and `RoomStateMap` (the initialisation write for each
      room must still set `TileState::Off`).
- [ ] `reset_level_resources` in `unmapload-plugin` resets both `RoomTopology` and `RoomStateMap`.
- [ ] No `pub use` re-exports have been added anywhere.
- [ ] `lib.rs` and `mod.rs` files contain only `mod` statements (and minimal `use`/call overhead in `app_setup`). No
      actual logic.

---

## 6. File Change Summary

| File                                                                | Change                                                     |
| ------------------------------------------------------------------- | ---------------------------------------------------------- |
| `crates/unbehavior/src/components.rs`                               | Add `TmxEntityId` struct                                   |
| `crates/unbehavior/src/roomdb.rs`                                   | Replace `RoomDB` with `RoomTopology` + `RoomStateMap`      |
| `crates/unmapload-plugin/src/level_setup.rs`                        | Enumerate layers, update SystemParam, split reset          |
| `crates/unmapload-plugin/src/tile_spawning.rs`                      | Add `layer_idx` param, insert `TmxEntityId` + `Replicated` |
| `crates/unrender-plugin/src/systems/hydration.rs`                   | Use `RoomTopology` + `RoomStateMap`                        |
| `crates/uninteraction-plugin/src/systems/interactivestuff.rs`       | Use `RoomTopology` + `RoomStateMap`                        |
| `crates/unlight-plugin/src/audio.rs`                                | `RoomDB` → `RoomTopology`                                  |
| `crates/unlight-plugin/src/maplight/visibility.rs`                  | `RoomDB` → `RoomTopology`                                  |
| `crates/unlight-plugin/src/maplight/systems/gathering.rs`           | `RoomDB` → `RoomTopology`                                  |
| `crates/unsound-plugin/src/systems.rs`                              | `RoomDB` → `RoomTopology`                                  |
| `crates/unghost-plugin/src/systems/ghost_ai/enrage.rs`              | `RoomDB` → `RoomTopology`                                  |
| `crates/unfog-plugin/src/systems.rs`                                | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/locomotion_interaction.rs`     | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/repellent_expulsion.rs`        | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/player_wellbeing.rs`           | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/ghost_behavior_and_hunting.rs` | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/environmental_awareness.rs`    | `RoomDB` → `RoomTopology`                                  |
| `crates/unwalkie-plugin/src/triggers/consumables_and_defense.rs`    | `RoomDB` → `RoomTopology`                                  |
| `crates/unreplicon-plugin/src/systems/map_sync.rs`                  | **NEW FILE** — stitcher system                             |
| `crates/unreplicon-plugin/src/systems/mod.rs`                       | Add `mod map_sync` + `map_sync::app_setup(app)`            |
| `crates/unreplicon-plugin/src/systems/players.rs`                   | Add `app.replicate::<TmxEntityId>()`                       |
