# PR 2 Specification: Universal Spawning & Hydration (Workstream 2)

> **Document status:** Authored against the live codebase (March 2026). All file paths, struct names, field names,
> system names, function signatures, and component lists have been verified by direct inspection of the source.
> Line-number references are indicative and may drift; the symbol names are authoritative.

---

## 1. Objective

Refactor the core player and ghost spawning architecture to fix the **"Mirror World / Orange Clones" desync bug**, in
which pure Join clients incorrectly run legacy single-player spawning code instead of receiving entities from the
server.

The new architecture rests on two rules:

1. **The Authority is the sole spawner of Skeletons.** `setup_mission_players` (in `unreplicon-plugin`) spawns complete
   player skeleton entities for every player recorded in `LobbyInfo`. `classic_mode_orchestrator` (in
   `unclassic-mode-plugin`) spawns the ghost skeleton only, stripped of all visual components.

2. **Every process with a local player Hydrates.** Two new reactive systems — `hydrate_players_system` and
   `hydrate_ghosts_system` — run on every node that has `LocalPlayerRole` (Host and Join clients). They detect skeleton
   entities the moment they appear (either freshly spawned locally or just replicated from the server) and attach
   visuals, input, and camera components.

The result is that on Offline and PeerHost, the host creates skeletons and immediately hydrates them in the next frame.
On a pure Join client, the server's skeleton entities arrive via replication and are hydrated on arrival. Both paths
converge on identical logic.

---

## 2. Scope Boundaries

| In scope                                                   | Out of scope                                        |
| :--------------------------------------------------------- | :-------------------------------------------------- |
| `crates/unreplicon-plugin/src/systems/players.rs`          | Gear / Inventory spawning (deferred to WS3)         |
| `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` | Map tile / prop generation                          |
| `crates/unclassic-mode-plugin/src/plugin.rs`               | Ghost Breach entity spawning (deferred)             |
| Player skeleton for every lobby player                     | Modifying `PlayerSprite` component fields or layout |
| Ghost skeleton stripped of visual components               | Modifying the camera-follow algorithm               |
| `hydrate_players_system` and `hydrate_ghosts_system`       | Any `Cargo.toml` changes                            |

No new external crate dependencies are required. All types used here belong to crates already in scope.

---

## 3. Background: Key Types and Current State

Read this section completely before touching any file.

### 3.1 `ClassicModeSystemParam` (verified)

Defined at `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` lines 51–71:

```rust
#[derive(SystemParam)]
pub(crate) struct ClassicModeSystemParam<'w> {
    pub local_player_role: Option<Res<'w, untypes_core::roles::LocalPlayerRole>>,
    pub authority_role: Option<Res<'w, untypes_core::roles::AuthorityRole>>,
    pub asset_server: Res<'w, AssetServer>,
    pub haunt_state: ResMut<'w, HauntState>,
    pub player_assets: Option<Res<'w, unplayer_core::assets::PlayerAssets>>,
    pub ghost_assets: Option<Res<'w, unghost_core::assets::GhostAssets>>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub gear_registry: Res<'w, GearSpawnerRegistry>,
    pub upscale_idx: Res<'w, UpscaleIndex>,
    pub video_settings: Option<Res<'w, Persistent<VideoSettings>>>,
    pub materials1: Option<ResMut<'w, Assets<unrender_std::materials::CustomMaterial1>>>,
    pub meshes: Option<ResMut<'w, Assets<Mesh>>>,
    pub images: Option<Res<'w, Assets<Image>>>,
    pub audio_settings: Option<Res<'w, Persistent<unsettings_core::audio::AudioSettings>>>,
    pub control_settings: Option<Res<'w, Persistent<unsettings_core::controls::ControlKeys>>>,
    pub board_topology: Res<'w, BoardTopology>,
    pub board_entity_field: ResMut<'w, unboard_core::resources::board_topology::BoardEntityField>,
    pub roomdb: Res<'w, RoomDB>,
}
```

**Critical:** Every `Option<Res<...>>` and `Option<ResMut<...>>` field must be unwrapped with
`if let Some(x) = &p.field { ... }` before use. Never call `.unwrap()` directly — these resources may be absent on
headless dedicated servers. This pattern appears throughout the existing orchestrator code; follow it exactly.

#### Deref rules for `Persistent<T>` fields

`p.audio_settings` is `Option<Res<'w, Persistent<AudioSettings>>>`. Once you bind it with
`let Some(audio_settings) = &p.audio_settings`, `audio_settings` is `&Res<Persistent<AudioSettings>>`. Dereferencing via
`**audio_settings` gives `Persistent<AudioSettings>`, and `Deref` on `Persistent<T>` gives `T`. The audio field access
`audio_settings.sound_output.to_ear_offset()` works because `Deref` chains automatically. Similarly `p.control_settings`
requires triple-deref `***control_settings` when passing a `ControlKeys` value by copy (see Step 3 for exact usage).

### 3.2 `PlayerSprite::new` (verified)

Defined at `crates/unplayer-core/src/components.rs`:

```rust
impl PlayerSprite {
    pub fn new(id: Uuid, network_id: NetworkId, spawn_position: Position) -> Self {
        Self {
            id,                  // Uuid — stable player identity
            network_id,          // NetworkId — replicon network id
            crazyness: 0.0,
            sanity: 100.0,
            mean_sound: 0.0,
            health: 100.0,
            spawn_position,
            movement: Direction::zero(),
        }
    }
}
```

The `NetworkId` for a player is derived from their UUID via the `From<Uuid>` impl in
`crates/unreplicon-core/src/network_id.rs`:

```rust
impl From<Uuid> for NetworkId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid.to_u128_le() as u64)
    }
}
```

Use `NetworkId::from(player.player_uuid)` in `setup_mission_players`.

### 3.3 `LobbyPlayerInfo` fields (the `player` loop variable)

`LobbyInfo.players` is `Vec<LobbyPlayerInfo>`. Each entry has the following relevant fields:

- `player_uuid: Uuid` — stable identity. This is the canonical source of truth.
- `current_socket: Option<OwnerId>` — `None` when the player is the Host (offline or PeerHost),
  `Some(OwnerId::Client(e))` when they are a remote client.
- `tint_color_index: u8` — colour slot assigned in the lobby.
- `connected: bool`

**Host detection rule:** `player.current_socket.is_none()` means this is the host's player record.

### 3.4 `ControlKeys::NONE` (verified)

Defined at `crates/unsettings-core/src/controls.rs`:

```rust
pub const NONE: Self = ControlKeys {
    up: KeyCode::NonConvert,
    down: KeyCode::NonConvert,
    left: KeyCode::NonConvert,
    right: KeyCode::NonConvert,
    // ... all keys set to NonConvert
};
```

Use `unsettings_core::controls::ControlKeys::NONE` for remote players who should not accept input.

### 3.5 Current state of `setup_mission_players` (what you are rewriting)

File: `crates/unreplicon-plugin/src/systems/players.rs`

Current function signature:

```rust
fn setup_mission_players(
    q_host_player: Query<(Entity, &Position, &PlayerSprite)>,  // ← DELETE this param
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    mut commands: Commands,
    filter_bit: Res<crate::plugin::GlobalFilterBit>,           // ← KEEP
    mut q_clients: Query<&mut ClientVisibility>,               // ← KEEP
)
```

Current run condition (already correct, do not change):

```rust
app.add_systems(
    OnEnter(AppState::InGame),
    setup_mission_players.run_if(resource_exists::<AuthorityRole>),
);
```

The stale comment above this registration reads "gated by LocalPlayerRole" — fix it to read "gated by AuthorityRole".

Current logic (what you are replacing):

1. Iterates `q_host_player` (the pre-existing host entity spawned by `classic_mode_orchestrator`) and inserts
   `Replicated`, `Owner(OwnerId::Server)`, `LocallyOwned` onto it.
2. Iterates `lobby.players`, skips entries with `current_socket == None` (host), and spawns bare network-only entities
   for remote clients with `PlayerSprite::default()`.

After this PR, neither of those paths exists. The function spawns complete skeletons for every lobby player including
the host. The `q_host_player` query parameter is deleted. `filter_bit` and `q_clients` are kept because they are needed
for the `ClientVisibility` call and `OwnershipGranted` message dispatch (see Step 2).

### 3.6 Current state of `classic_mode_orchestrator` (what you are modifying)

File: `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

The function starts at the `// --- Spawn Player ---` comment (roughly line 104) and ends with
`ev_level_ready.write(LevelReadyEvent { open_van })`.

The player-spawning block is roughly the section from `let mut player_image = …` through the end of the
`for (idx, id) in player_ids_to_spawn.into_iter().enumerate()` loop, including all nested `ec.insert(...)` calls and the
`ec.with_children(...)` call for the `FocusRing`.

The ghost-spawning block starts at `if p.authority_role.is_some() {` and contains two sub-blocks: the `GhostBreach`
entity spawn and the ghost entity spawn.

`dist_to_van` and `open_van` are computed in the player section but used in the final
`ev_level_ready.write(LevelReadyEvent { open_van })`. These variables must be preserved. Move their computation to
before the (now-deleted) player spawning loop if needed.

### 3.7 `setup_ghost_entities` in `unreplicon-plugin` (verified)

Defined at `crates/unreplicon-plugin/src/systems/ghost.rs`:

```rust
fn setup_ghost_entities(
    q_ghost: Query<(Entity, &Position), With<GhostTag>>,
    mut commands: Commands,
) {
    commands.insert_resource(RepliconGhostSpawningActive);
    for (entity, pos) in q_ghost.iter() {
        commands.entity(entity).insert((Replicated, LerpPosition::new(*pos)));
    }
}
```

This system is responsible for inserting `Replicated` and `LerpPosition` onto ghost entities. **Do not insert
`Replicated` or `LerpPosition` in either `classic_mode_orchestrator` or `hydrate_ghosts_system`.**

### 3.8 What `Added<GhostSprite>` means for the ghost hydration trigger

`GhostSprite` is registered for replication: `app.replicate::<GhostSprite>()` in `unreplicon-plugin`.

- **On the Host:** `classic_mode_orchestrator` spawns the entity with `GhostSprite`. `Added<GhostSprite>` fires in the
  next `Update` frame. `hydrate_ghosts_system` adds visuals.
- **On a pure client:** The server sends the replicated entity. When `GhostSprite` arrives via replication,
  `Added<GhostSprite>` fires. `hydrate_ghosts_system` adds visuals.

This is the same pattern that `setup_replicated_ghost_visuals` already uses. The new system is a direct replacement with
corrected semantics (it does not re-create the `GhostSprite` — see §4.4.1 below).

### 3.9 Systems being deleted

| System                            | File              | Current run condition                                                       |
| :-------------------------------- | :---------------- | :-------------------------------------------------------------------------- |
| `spawn_joined_player`             | `orchestrator.rs` | `in_state(InGame)` + `not(resource_exists::<RepliconPlayerSpawningActive>)` |
| `setup_replicated_player_visuals` | `orchestrator.rs` | `in_state(InGame)`                                                          |
| `setup_replicated_ghost_visuals`  | `orchestrator.rs` | `in_state(InGame)`                                                          |

All three are registered inside `UnhaunterClassicModeCorePlugin::build` in `plugin.rs`.

---

## 4. Implementation Steps

### Step 1: Delete the Player-Spawning Block from `classic_mode_orchestrator`

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

Locate the player-spawning code inside `classic_mode_orchestrator`. It begins with:

```rust
    // --- Spawn Player ---
    let mut rng = random_seed::rng();
    let player_position = player_spawn_points.choose(&mut rng).copied().unwrap();

    let mut player_image = p
        .player_assets
        .as_ref()
        .map(|a| a.character.clone())
        .unwrap_or_default();
```

And ends just before the comment `// Join clients do not spawn the ghost locally…` or the
`if p.authority_role.is_some() {` block (whichever comes first), including the final line:

```rust
        p.board_entity_field.0[player_position.to_board_position().ndidx()].push(player_ent_id);
    }
```

**Delete the entire player spawning section**, including:

- The `player_image` / `player_rf` variables (they are not used by the ghost block).
- The `sprite_size`, `anchor`, `sprite_anchor`, `src_mesh_handle` variables.
- The `player_ids_to_spawn: Vec<Uuid>` variable and its entire `for` loop.
- The call to `spawn_initial_gear` inside that loop.
- The `FocusRing` child spawn inside that loop.

**Preserve the following variables** that are computed near the start of the player block but consumed later:

```rust
    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

    let open_van = dist_to_van < 8.0 && p.difficulty.0.van_auto_open;
```

These two lines feed `ev_level_ready.write(LevelReadyEvent { open_van })` at the bottom of the function. They reference
`player_position` (chosen from `player_spawn_points`). Keep `player_spawn_points`, `rng`, and `player_position` in scope
for this purpose.

After Step 1, `classic_mode_orchestrator` still spawns the ghost skeleton and the breach entity, still calls
`spawn_ambient_sounds`, still calls `assign_ghost_influence`, and still writes `LevelReadyEvent`. It no longer touches
players at all.

Also delete the function `spawn_initial_gear` — it is defined later in `orchestrator.rs` (around line 541) and is no
longer called by any code after Steps 1–3. Confirm it has no other callers before deleting (use
`git grep 'spawn_initial_gear'`).

#### Step 1 checklist

- [ ] `player_ids_to_spawn` and its `for` loop are gone.
- [ ] `spawn_initial_gear` call site inside the loop is gone.
- [ ] `dist_to_van` and `open_van` are preserved and still feed `ev_level_ready.write(...)`.
- [ ] `classic_mode_orchestrator` still compiles with no dead-variable warnings.

---

### Step 2: Rewrite `setup_mission_players` to Spawn Complete Skeletons

**File:** `crates/unreplicon-plugin/src/systems/players.rs`

Delete the entire body of `setup_mission_players` and replace it with the implementation below. The function signature
must also change (remove `q_host_player`, keep `filter_bit` and `q_clients`).

#### 2.1 New function signature

```rust
fn setup_mission_players(
    q_lobby: Query<&LobbyInfo>,
    q_spawn_points: Query<&Position, (With<PlayerSpawnPoint>, Without<PlayerSprite>)>,
    mut commands: Commands,
    filter_bit: Res<crate::plugin::GlobalFilterBit>,
    mut q_clients: Query<&mut ClientVisibility>,
) {
```

Removed compared to current: `q_host_player: Query<(Entity, &Position, &PlayerSprite)>`. Kept: `filter_bit` and
`q_clients` — they are required by the `ClientVisibility.set()` call and the `OwnershipGranted` message dispatch, both
of which must be preserved (see §2.3 below).

#### 2.2 New function body

```rust
    // Signal that replicon-based player spawning is now active.
    // This marker causes spawn_joined_player (being deleted in Step 3) to be a no-op
    // on any node that still has the old system registered during the transition period.
    commands.insert_resource(RepliconPlayerSpawningActive);

    let spawn_points: Vec<Position> = q_spawn_points.iter().copied().collect();
    let default_pos = Position {
        x: 0.0,
        y: 0.0,
        z: 0.0,
        visual_priority: 0.0,
    };

    let Ok(lobby) = q_lobby.single() else {
        warn!("setup_mission_players: no LobbyInfo entity found — skipping player spawn");
        return;
    };

    for (idx, player) in lobby.players.iter().enumerate() {
        // Pick a deterministic spawn point based on lobby order.
        let spawn_pos = spawn_points
            .get(idx % spawn_points.len().max(1))
            .copied()
            .unwrap_or(default_pos);

        let net_id = NetworkId::from(player.player_uuid);

        // Spawn the player skeleton. Every node (host, join client, dedicated server)
        // that replicates will receive this entity. Visual components are NOT added
        // here — hydrate_players_system handles that.
        let entity = commands
            .spawn((
                spawn_pos,
                unspatial_core::lerp_position::LerpPosition::new(spawn_pos),
                PlayerSprite::new(player.player_uuid, net_id, spawn_pos),
                net_id,
                Stamina::default(),
                PlayerGear::default(),
                Direction::new_right(),
                unbehavior::components::Movable,
                unnavigation_core::components::waypoint::WaypointQueue::default(),
                unspatial_core::boardposition::MapEntityFieldBPos(
                    spawn_pos.to_board_position(),
                ),
                untags_core::tags::PlayerTag,
                unrender_std::resources::visibility_data::VisibilityData::default(),
                unplayer_core::components::PlayerInput::default(),
                Replicated,
            ))
            .id();

        let is_host = player.current_socket.is_none();

        if is_host {
            // Host player: authority owns it locally. Insert LocallyOwned directly —
            // the OwnershipGranted message path is not used because the host is not
            // a pure client.
            commands
                .entity(entity)
                .insert((Owner(OwnerId::Server), LocallyOwned));
            info!(
                "setup_mission_players: spawned host skeleton {:?} for player {}",
                entity, player.player_uuid
            );
        } else {
            let socket_owner_id = player.current_socket.unwrap();
            commands.entity(entity).insert(Owner(socket_owner_id));

            // Configure Replicon's spatial-interest filter for this remote client so
            // that the server sends this entity's state to the owning client.
            if let OwnerId::Client(client_entity) = socket_owner_id
                && let Ok(mut visibility) = q_clients.get_mut(client_entity)
            {
                visibility.set(entity, filter_bit.0, false);
            }

            // Notify the client of which entity it owns. On the client side,
            // handle_ownership_granted receives this message and inserts LocallyOwned
            // onto the entity — without this, the client never knows which entity
            // belongs to it, and send_export_state (which queries With<LocallyOwned>)
            // will never fire for that client.
            let client_id = from_owner_id(socket_owner_id);
            commands.write_message(ToClients {
                mode: SendMode::Direct(client_id),
                message: OwnershipGranted { entity },
            });
            info!(
                "setup_mission_players: spawned remote skeleton {:?} for player {} (owner={:?})",
                entity, player.player_uuid, socket_owner_id
            );
        }
    }
```

#### 2.3 Why `filter_bit`, `q_clients`, and `OwnershipGranted` must be kept

These are not optional. They are part of the Replicon ownership protocol:

- `ClientVisibility::set(entity, filter_bit.0, false)` tells Replicon's spatial filter that the owning client should
  receive updates for this entity. Skip this and the client may never see the entity's state changes.

- `commands.write_message(ToClients { mode: SendMode::Direct(client_id), message: OwnershipGranted { entity } })`
  triggers `handle_ownership_granted` on the pure client, which inserts `LocallyOwned` onto the client's own player
  entity. Without `LocallyOwned`, the system `send_export_state` (which queries `With<LocallyOwned>`) never fires for
  that client, breaking state synchronisation.

#### 2.4 Step 2 checklist

- [ ] `q_host_player` parameter is removed from the function signature.
- [ ] The old "Add network components to the existing host player entity" loop is gone.
- [ ] The old "Spawn bare network entities for remote clients" loop is replaced by the unified loop.
- [ ] Every player in `lobby.players` gets a spawned entity (host and remote alike).
- [ ] `ClientVisibility::set()` is called for each remote client.
- [ ] `OwnershipGranted` is sent for each remote client.
- [ ] `LocallyOwned` is inserted directly for the host (not via message).
- [ ] Stale comment above the `app.add_systems(OnEnter(...), setup_mission_players...)` call is fixed.

---

### Step 3: Delete Legacy Player Systems and Replace with `hydrate_players_system`

**Files:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` and `plugin.rs`

#### 3.1 Delete three functions entirely

Delete the entire function bodies and `pub(crate) fn` declarations for:

1. **`spawn_joined_player`** — starts around line 546.
2. **`setup_replicated_player_visuals`** — starts around line 816.
3. **`spawn_initial_gear`** — already scheduled for deletion in Step 1; confirm it is gone.

After deletion, any `use` imports at the top of `orchestrator.rs` that are only used by these functions may become
unused. Clippy will catch them — remove any that cause warnings.

#### 3.2 Create `hydrate_players_system`

Add the following new function to `orchestrator.rs`, in place of the deleted `setup_replicated_player_visuals`:

```rust
/// Reactive system: fires whenever a PlayerSprite component appears on an entity,
/// either freshly spawned locally (host/offline) or replicated from the server
/// (pure client). Attaches all visual, audio, and input components.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_players_system(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    local_player: Res<LocalPlayer>,
    q_added: Query<(Entity, &PlayerSprite, &Position), Added<PlayerSprite>>,
) {
    // Headless guard: dedicated servers have no local player and must not run this.
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, player_sprite, pos) in q_added.iter() {
        let spawn_pos = *pos;
        let is_local = local_player
            .0
            .map(|uuid| uuid == player_sprite.id)
            .unwrap_or(false);

        // --- Resolve asset handles and resolution factor ---
        let mut player_image = p
            .player_assets
            .as_ref()
            .map(|a| a.character.clone())
            .unwrap_or_default();
        let mut player_rf = 1.0f32;

        if let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p.upscale_idx.resolve(
                "img/characters-model1-demo.png",
                video_settings.max_upscale_factor.factor(),
            )
        {
            player_image = p.asset_server.load(resolved.path);
            player_rf = resolved.factor;
        }

        // --- Compute mesh geometry ---
        let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
        let anchor = unplayer_core::assets::PLAYER_ANCHOR;
        let sprite_anchor = Vec2::new(
            sprite_size.x * (anchor.x + 0.5),
            sprite_size.y * (0.5 - anchor.y),
        );

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut ec = commands.entity(entity);

        // --- Attach shared components (all local-player nodes) ---
        ec.insert(GameSprite)
            .insert(MapColor { color: Color::WHITE })
            .insert(ShadowCaster::default())
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ));

        // --- Attach visual mesh (requires meshes and materials1 to be present) ---
        if let Some(meshes) = &mut p.meshes {
            let src_mesh_handle = meshes.add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

            let mut material = CustomMaterial1::from_texture(player_image);
            material.data.sheet_cols = 16;
            material.data.sheet_rows = 4;
            material.data.sprite_width = 32.0 * player_rf;
            material.data.sprite_height = 32.0 * player_rf;
            material.data.upscale_factor = player_rf;
            material.data.y_anchor = anchor.y;

            if let Some(materials1) = &mut p.materials1 {
                let material_handle = materials1.add(material);

                ec.insert(Mesh2d(src_mesh_handle))
                    .insert(MeshMaterial2d(material_handle))
                    .insert(
                        Transform::from_xyz(
                            spawn_scoord[0],
                            spawn_scoord[1],
                            spawn_scoord[2],
                        )
                        .with_scale(Vec3::new(
                            1.0 / player_rf,
                            1.0 / player_rf,
                            1.0 / player_rf,
                        )),
                    )
                    .insert(ResolutionFactor(player_rf))
                    .insert(MapTileSprite)
                    .insert(SpriteLayer(0.00001));
            }
        }

        // --- Attach controls and camera: local vs. remote branch ---
        if is_local {
            // This is the player that sits at this screen. Give it input, camera
            // targeting, and spatial audio.
            if let Some(control_settings) = &p.control_settings {
                // Triple-deref: &Res<Persistent<ControlKeys>> → Persistent<ControlKeys> → ControlKeys
                ec.insert(PlayerInputMapping {
                    controls: ***control_settings,
                });
            }
            ec.insert(MainPlayer).insert(Viewer {
                id: player_sprite.network_id,
                ..default()
            });
            if let Some(audio_settings) = &p.audio_settings {
                ec.insert(SpatialListener::new(
                    -audio_settings.sound_output.to_ear_offset(),
                ));
            }
        } else {
            // Remote player visible on this screen — no input mapping, use null keys.
            ec.insert(PlayerInputMapping {
                controls: unsettings_core::controls::ControlKeys::NONE,
            });
        }

        // --- Attach FocusRing child entity ---
        if let Some(ghost_assets) = &p.ghost_assets {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(1.1 * player_rf))
                            .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }

        // NOTE: Do NOT push the entity into board_entity_field here.
        // The skeleton already has MapEntityFieldBPos inserted by setup_mission_players.
        // A separate spatial-sync system reacts to Added<MapEntityFieldBPos> and populates
        // the grid for all peers uniformly. Duplicating that push here would:
        //   (a) cause a double-push on the Host (which is both Authority and LocalPlayer), and
        //   (b) leave the grid empty on the Dedicated Server, which skips this system entirely.

        info!(
            "hydrate_players_system: hydrated entity {:?} player_uuid={} is_local={}",
            entity, player_sprite.id, is_local
        );
    }
}
```

#### 3.3 Important: do NOT call `spawn_initial_gear` here

The legacy systems called `spawn_initial_gear` during hydration. That function is being deleted. Gear initialisation is
deferred to Workstream 3 entirely. Leave a `// TODO(WS3): gear init` comment if you want a marker.

#### 3.4 Important: `Added<PlayerSprite>` fires on both paths

- **Host/Offline path:** `setup_mission_players` (now universal) inserts `PlayerSprite` on `OnEnter(InGame)`. In the
  next `Update` frame, `Added<PlayerSprite>` fires for all freshly spawned player entities and `hydrate_players_system`
  runs.
- **Pure client path:** The server replicates the entity. When `PlayerSprite` arrives via replication the Bevy change
  detection registers `Added<PlayerSprite>` and `hydrate_players_system` runs.

Both paths are handled by the same function with no special casing. This is the whole point.

#### 3.5 Step 3 checklist

- [ ] `spawn_joined_player` function is gone.
- [ ] `setup_replicated_player_visuals` function is gone.
- [ ] `hydrate_players_system` is added and compiles cleanly.
- [ ] `MainPlayer`, `Viewer`, `SpatialListener` are only inserted inside the `is_local == true` branch.
- [ ] Remote players get `PlayerInputMapping { controls: ControlKeys::NONE }`.
- [ ] No call to `spawn_initial_gear` anywhere in `orchestrator.rs`.

---

### Step 4: Strip Ghost Visuals from `classic_mode_orchestrator` and Add `hydrate_ghosts_system`

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

#### 4.1 Strip visual components from the ghost skeleton in `classic_mode_orchestrator`

Inside `classic_mode_orchestrator`, find the ghost entity spawn block (inside `if p.authority_role.is_some() { ... }`).
There are two entity spawns: the **GhostBreach** and the **Ghost** itself.

**Do NOT touch the GhostBreach entity spawn at all.** Breach hydration is deferred to a future PR. Leave the entire
breach block (`let breach_id = { ... }`) unchanged.

For the **Ghost entity spawn**, delete all visual component insertions and keep only the skeleton:

**Delete these lines from the ghost entity's `ec.insert(...)` chain:**

```rust
// DELETE the entire `if p.local_player_role.is_some() && let (Some(meshes), Some(materials1))` block
// that inserts: Mesh2d, MeshMaterial2d, Transform, MapTileSprite, ResolutionFactor, SpriteLayer,
// Ethereal, Emissive, SpectralClarity, AlphaModulator, EctoplasmVisuals

// DELETE the FocusRing child spawn at the bottom of the ghost block:
// if p.local_player_role.is_some() && let Some(ghost_assets) = &p.ghost_assets {
//     ec.with_children(|parent| { ... FocusRing ... });
// }
```

**Keep these lines** — this is the ghost skeleton:

```rust
            ec.insert(ghost_sprite.with_breachid(breach_id))
                .insert(p.haunt_state.ghost_dynamics)
                .insert(GhostTag)
                .insert(ghost_id_net)
                .insert(GameSprite)
                .insert(MapEntityFieldBPos(ghost_spawn.to_board_position()))
                .insert(Movable)
                .insert(LightSensitive {
                    exposure_factor: 0.5,
                    bias: 0.01,
                })
                .insert(UltravioletSensitive {
                    intensity: 1.0,
                    ..default()
                })
                .insert(InfraredSensitive {
                    intensity: 1.0,
                    ..default()
                })
                .insert(ThermalEmitter {
                    room_restricted: true,
                    ..default()
                })
                .insert(FluidEmitter::default())
                .insert(SoundEmitter::default());
```

**Do NOT add `Replicated` or `LerpPosition` here.** The system `setup_ghost_entities` in
`unreplicon-plugin/src/systems/ghost.rs` is responsible for inserting those on ghost entities. The ghost entity has
`GhostTag`, which is what `setup_ghost_entities` queries.

Also keep the `ghost_spawn` position (it is still inserted via `ec.insert(ghost_sprite.with_breachid(…))` which uses it
internally), and keep `ghost_id_net`, `ghost_sprite`, `breach_id`, and all values computed before the visual block.

After stripping, the ghost variables `ghost_image`, `ghost_rf`, `ghost_img_size`, `anchor`, `sprite_anchor`, and
`src_mesh_handle` are no longer referenced. Delete them or Clippy will warn.

#### 4.2 Create `hydrate_ghosts_system`

Add the following new function to `orchestrator.rs`:

```rust
/// Reactive system: fires whenever a GhostSprite component appears on an entity,
/// either just spawned by classic_mode_orchestrator (host/offline) or just replicated
/// from the server (pure client). Attaches all ghost visual components.
/// Runs only on nodes with a local player (LocalPlayerRole present).
pub(crate) fn hydrate_ghosts_system(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    q_added: Query<(Entity, &Position, &GhostSprite), Added<GhostSprite>>,
) {
    // Headless guard: dedicated servers have no local player and no visuals needed.
    if p.local_player_role.is_none() {
        return;
    }

    for (entity, pos, _ghost) in q_added.iter() {
        let ghost_spawn = *pos;

        // --- Resolve asset handles and resolution factor ---
        let mut ghost_image = p
            .ghost_assets
            .as_ref()
            .map(|a| a.ghost.clone())
            .unwrap_or_default();
        let mut ghost_rf = 1.0f32;

        if let Some(video_settings) = &p.video_settings
            && let Some(resolved) = p
                .upscale_idx
                .resolve("img/ghost.png", video_settings.max_upscale_factor.factor())
        {
            ghost_image = p.asset_server.load(resolved.path);
            ghost_rf = resolved.factor;
        }

        // --- Compute mesh geometry from image dimensions ---
        let mut ghost_img_size = Vec2::new(128.0, 128.0);
        if let Some(images) = &p.images
            && let Some(img) = images.get(ghost_image.id())
        {
            ghost_img_size = Vec2::new(
                img.texture_descriptor.size.width as f32,
                img.texture_descriptor.size.height as f32,
            );
        }

        let anchor = unmapload_core::assets::GRID_1X1X4_ANCHOR;
        let sprite_anchor = Vec2::new(
            ghost_img_size.x * (anchor.x + 0.5),
            ghost_img_size.y * (0.5 - anchor.y),
        );

        let mut ec = commands.entity(entity);

        // --- Attach visual mesh ---
        if let (Some(meshes), Some(materials1)) = (&mut p.meshes, &mut p.materials1) {
            let mesh_handle =
                meshes.add(Mesh::from(QuadCC::new(ghost_img_size, sprite_anchor)));
            let mut material = CustomMaterial1::from_texture(ghost_image);
            material.data.color = Color::NONE.into();
            material.data.y_anchor = anchor.y;
            let material_handle = materials1.add(material);

            ec.insert(Mesh2d(mesh_handle))
                .insert(MeshMaterial2d(material_handle))
                .insert(
                    Transform::from_xyz(-1000.0, -1000.0, -1000.0)
                        .with_scale(Vec3::splat(1.0 / ghost_rf)),
                )
                .insert(MapTileSprite)
                .insert(ResolutionFactor(ghost_rf))
                .insert(SpriteLayer(10.0))
                .insert(Ethereal::default())
                .insert(Emissive::default())
                .insert(SpectralClarity::default())
                .insert(AlphaModulator {
                    frequency: 1.0,
                    amplitude: 0.5,
                })
                .insert(EctoplasmVisuals {
                    use_breach_curve: false,
                });
        }

        // --- Attach FocusRing child entity ---
        if let Some(ghost_assets) = &p.ghost_assets {
            ec.with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(0.5 * ghost_rf))
                            .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                    )
                    .insert(FocusRing::default());
            });
        }

        // NOTE: Do NOT push the entity into board_entity_field here.
        // The skeleton already has MapEntityFieldBPos inserted by classic_mode_orchestrator.
        // A separate spatial-sync system reacts to Added<MapEntityFieldBPos> and populates
        // the grid for all peers uniformly. Duplicating that push here would:
        //   (a) cause a double-push on the Host (which is both Authority and LocalPlayer), and
        //   (b) leave the grid empty on the Dedicated Server, which skips this system entirely.

        info!(
            "hydrate_ghosts_system: hydrated ghost entity {:?} at {:?}",
            entity, ghost_spawn
        );
    }
}
```

#### 4.3 Critical: do NOT re-insert `GhostSprite` in `hydrate_ghosts_system`

The legacy `setup_replicated_ghost_visuals` contained this line near the end:

```rust
ec.insert(ghost_sprite)   // ghost_sprite was locally re-created from scratch
```

This was a bug: it overwrote the replicated `GhostSprite` (which contains live server state such as `rage`, `hunting`,
`repellent_hits`, etc.) with a freshly-constructed empty one. **`hydrate_ghosts_system` must NOT call `ec.insert(...)`
with any `GhostSprite` value.** The entity already has `GhostSprite` on it — that is precisely what caused
`Added<GhostSprite>` to fire.

Similarly, do not re-insert `GhostBehaviorDynamics`, `GhostTag`, `NetworkId(0)`, `GameSprite`, `MapEntityFieldBPos`,
`Movable`, or any of the sensor components. Those are all part of the skeleton provided by `classic_mode_orchestrator`.
Only insert the visual components listed in §4.2.

#### 4.4 Step 4 checklist

- [ ] The visual-mesh block inside `classic_mode_orchestrator`'s ghost spawn is deleted.
- [ ] The `FocusRing` child spawn inside `classic_mode_orchestrator`'s ghost spawn is deleted.
- [ ] `ghost_image`, `ghost_rf`, `ghost_img_size` variables deleted from `classic_mode_orchestrator` to avoid
      dead-variable warnings.
- [ ] `GhostBreach` entity spawn is untouched.
- [ ] `hydrate_ghosts_system` is added.
- [ ] `hydrate_ghosts_system` does NOT call `ec.insert(ghost_sprite)` or re-insert any skeleton component.
- [ ] Neither `classic_mode_orchestrator` nor `hydrate_ghosts_system` inserts `Replicated` or `LerpPosition` on ghost
      entities.

---

### Step 5: Update `plugin.rs` Registrations

**File:** `crates/unclassic-mode-plugin/src/plugin.rs`

#### 5.1 Remove three system registrations from `UnhaunterClassicModeCorePlugin::build`

Locate this block:

```rust
        app.add_systems(
            Update,
            (
                crate::systems::orchestrator::classic_mode_orchestrator.run_if(
                    bevy::prelude::on_message::<
                        unmapload_core::events::loadlevel::MapEntitiesReadyEvent,
                    >,
                ),
                crate::systems::orchestrator::spawn_joined_player
                    .run_if(in_state(untypes_core::states::AppState::InGame))
                    .run_if(not(resource_exists::<RepliconPlayerSpawningActive>)),
                crate::systems::orchestrator::setup_replicated_player_visuals
                    .run_if(in_state(untypes_core::states::AppState::InGame)),
                crate::systems::orchestrator::setup_replicated_ghost_visuals
                    .run_if(in_state(untypes_core::states::AppState::InGame)),
            ),
        );
```

Replace it with:

```rust
        app.add_systems(
            Update,
            (
                crate::systems::orchestrator::classic_mode_orchestrator.run_if(
                    bevy::prelude::on_message::<
                        unmapload_core::events::loadlevel::MapEntitiesReadyEvent,
                    >,
                ),
                crate::systems::orchestrator::hydrate_players_system
                    .run_if(in_state(untypes_core::states::AppState::InGame)),
                crate::systems::orchestrator::hydrate_ghosts_system
                    .run_if(in_state(untypes_core::states::AppState::InGame)),
            ),
        );
```

Three systems are removed (`spawn_joined_player`, `setup_replicated_player_visuals`, `setup_replicated_ghost_visuals`)
and two new ones are added (`hydrate_players_system`, `hydrate_ghosts_system`).

The `classic_mode_orchestrator` registration is unchanged.

After removing the `spawn_joined_player` registration, the import of `RepliconPlayerSpawningActive` in `plugin.rs` (the
`use unreplicon_core::components::RepliconPlayerSpawningActive;` line) may become unused. Remove it if Clippy complains
— the resource is still inserted by `setup_mission_players` in `unreplicon-plugin`, so the type itself is alive; it just
isn't referenced by name in `plugin.rs` anymore.

#### 5.2 Step 5 checklist

- [ ] `spawn_joined_player` registration removed.
- [ ] `setup_replicated_player_visuals` registration removed.
- [ ] `setup_replicated_ghost_visuals` registration removed.
- [ ] `hydrate_players_system` registered under `in_state(InGame)`.
- [ ] `hydrate_ghosts_system` registered under `in_state(InGame)`.
- [ ] `classic_mode_orchestrator` registration unchanged.
- [ ] No unused imports remain.

---

## 5. New or Modified Cargo Dependencies

No `Cargo.toml` files need to change. Every type used in this PR already belongs to a crate that is listed as a
dependency of the relevant plugin. Verify with `cargo clippy` — any missing crate will produce an `unresolved import`
error at compile time.

---

## 6. Success Criteria

Run `cargo clippy` after completing all steps. Zero new warnings are acceptable.

The agent must verify the following constraints before submitting:

- [ ] **`spawn_joined_player`** is deleted from `orchestrator.rs` completely.
- [ ] **`setup_replicated_player_visuals`** is deleted from `orchestrator.rs` completely.
- [ ] **`setup_replicated_ghost_visuals`** is deleted from `orchestrator.rs` completely.
- [ ] **`spawn_initial_gear`** is deleted from `orchestrator.rs` completely (no callers remain).
- [ ] **`q_host_player`** parameter has been removed from `setup_mission_players`.
- [ ] **`filter_bit`** and **`q_clients`** parameters are still in `setup_mission_players`.
- [ ] **`OwnershipGranted`** message is sent for every remote (non-host) player in `setup_mission_players`.
- [ ] **`ClientVisibility::set()`** is called for every remote client in `setup_mission_players`.
- [ ] **`LocallyOwned`** is inserted directly in `setup_mission_players` for the host player (not via message).
- [ ] **`hydrate_players_system`** and **`hydrate_ghosts_system`** are registered in `UnhaunterClassicModeCorePlugin`.
- [ ] Both hydration systems return early when `p.local_player_role.is_none()`.
- [ ] **`MainPlayer`**, **`Viewer`**, and **`SpatialListener`** are inserted only inside `hydrate_players_system`, and
      only when `is_local == true`.
- [ ] **`hydrate_ghosts_system`** does NOT call `ec.insert(ghost_sprite)` or any `GhostSprite` / `GhostBehaviorDynamics`
      insert.
- [ ] Neither spawner nor hydrator inserts `Replicated` or `LerpPosition` on ghost entities.
- [ ] `cargo clippy` produces no new warnings on `unclassic-mode-plugin` or `unreplicon-plugin`.
- [ ] Use `despawn()` not `despawn_recursive()` wherever entity cleanup is involved.
- [ ] All `query.single()` calls use `let Ok(val) = query.single() else { return; }` pattern.
- [ ] `hydrate_players_system` and `hydrate_ghosts_system` do NOT call `p.board_entity_field.0[...].push(entity)`. Grid
      population is handled by the existing `Added<MapEntityFieldBPos>` spatial-sync system.
