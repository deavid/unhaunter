# Plan 22: Dynamic Player Spawning & Client Identity

## Goal

Make multiplayer actually work: the host spawns only **its own** player at mission start. When remote clients complete
handshake, their player entities are spawned **dynamically** on the host. On the client side, the local player is
identified by matching the snapshot's `NetworkId` against `LocalPlayer`, not by hardcoded assumptions. Dead code around
`PlayerInputMapping::new()` is removed.

After this work is done:

- Host starts the mission with 1 player entity (itself, `NetworkId(1)`).
- When a client completes handshake, the host spawns a new player entity with gear at an available spawn point.
- The client receives the new player entity via the existing snapshot system.
- The client tags its own entity with `MainPlayer` + `Viewer` + `SpatialListener` + `PlayerInputMapping` based on
  `LocalPlayer.0`, NOT based on a hardcoded ID.
- Remote players on the host have `PlayerInputMapping { controls: ControlKeys::NONE }` (no keybindings — their input
  arrives via network). Note: `ControlKeys::default()` is WASD, so we must use `ControlKeys::NONE` explicitly.

## Motivation

Plan 21 built the network plumbing for N clients. But the game layer still hardcodes 2 players: the orchestrator
pre-spawns player 2 before anyone joins, and the client assumes it is always `NetworkId(2)`. This means:

1. A player-2 entity exists on the host even if nobody ever connects.
2. Both host and client control the same `MainPlayer` entity (whoever has `id == 1` or `id == 2`).
3. A third client would have no entity at all — the orchestrator only spawns IDs 1 and 2.

This plan bridges the gap between "N connections accepted" and "N players actually playing."

## Context

### Relevant files

| File                                                       | Role                                                                                                                                                                                                                   |
| ---------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` | `classic_mode_orchestrator` — spawns all game entities at mission start. Lines 128–131 hardcode `player_ids_to_spawn = vec![1]; if networked { push(2); }`.                                                            |
| `crates/unnet-plugin/src/systems/connection.rs`            | `handshake_handler_system` (line ~395) — completes handshake, sets `needs_full_sync`. Does NOT emit any "player joined" event or trigger player entity spawning.                                                       |
| `crates/unnet-plugin/src/systems/client_sync.rs`           | `spawn_remote_player()` (line ~258) — creates a bare player entity from snapshot data. Missing: `MainPlayer`, `Viewer`, `SpatialListener`, `PlayerInputMapping`, `FocusRing` child, `board_entity_field` registration. |
| `crates/unplayer-core/src/components.rs`                   | `PlayerInputMapping` — has dead code: `new(id: NetworkId)` and `default_controls(id)` with 0 callers.                                                                                                                  |
| `crates/unnet-core/src/messages.rs`                        | `NetworkDisconnectEvent` exists. No `PlayerJoinedEvent` equivalent.                                                                                                                                                    |
| `crates/unnet-core/src/resources.rs`                       | `LocalPlayer(pub Option<NetworkId>)` — set on client when Welcome is received (line ~471 in connection.rs).                                                                                                            |
| `crates/unnet-plugin/src/resources.rs`                     | `PlayerRegistry` — maps UUID → NetworkId, `next_id` starts at 2. `NetworkConn` with `host_send_to()`.                                                                                                                  |
| `crates/unnet-plugin/src/systems/host_sync.rs`             | `host_send_snapshots_system` — builds `Vec<PlayerState>` from all entities with `PlayerSprite`. If a player entity exists on the host, it appears in the snapshot and therefore on the client.                         |
| `crates/unplayer-plugin/src/systems/input/keyboard.rs`     | `keyboard_input_system` — queries `With<MainPlayer>`, reads `input_mapping.controls`. Only processes input for `MainPlayer`.                                                                                           |

### How player spawning works today (orchestrator.rs, lines 128–260)

```rust
// Line 128-131: Hardcoded player list
let mut player_ids_to_spawn = vec![1_usize];
if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Offline) {
    player_ids_to_spawn.push(2);
}

// Line 133-138: MainPlayer assignment (hardcoded)
for (idx, id) in player_ids_to_spawn.into_iter().enumerate() {
    let is_main_player = match p.cli.net_mode {
        untypes_core::cli::NetMode::Offline => true,
        untypes_core::cli::NetMode::Host { .. } => id == 1,
        untypes_core::cli::NetMode::Join { .. } => id == 2,
    };
    // ...

    // Line 140-172: Gear spawning (skipped on Join clients)
    if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Join { .. }) {
        // Spawns left_hand, right_hand, inventory gear entities
        // Uses gear_id_counter = id as u64 * 1000
    }

    // Line 206-210: Components
    ec.insert(PlayerSprite::new(id_net, spawn_pos))
        .insert(id_net)  // NetworkId
        .insert(PlayerInputMapping { controls: **p.control_settings })
        // ...

    // Line 234-243: MainPlayer/Viewer/SpatialListener only for is_main_player
    if is_main_player {
        ec.insert(MainPlayer)
            .insert(Viewer { id: id_net, ..default() })
            .insert(SpatialListener::new(-p.audio_settings.sound_output.to_ear_offset()));
    }

    // Line 248-260: FocusRing child entity + board_entity_field registration
    ec.with_children(|parent| { /* FocusRing */ });
    p.board_entity_field.0[...].push(player_ent_id);
}
```

### How `spawn_remote_player` works today (client_sync.rs, lines 258–320)

Creates a player entity with visual/physics components: `PlayerSprite`, `NetworkId`, `PlayerInput`, `PlayerTag`,
`ShadowCaster`, `Position(0,0,0)`, `Stamina`, `PlayerGear::default()`, `MapColor`, `Movable`, `LightSensitive`,
`Direction`, `AnimationTimer`, `WaypointQueue`, `VisibilityData`, `MapEntityFieldBPos`.

**Modification in this plan:** The function signature will be changed to accept an `initial_pos: Position` parameter so
that newly spawned players appear at their correct snapshot position from the first frame, avoiding a 1-frame "pop" from
the map origin `(0,0,0)`. Callers must pass `Position::from_array(p_state.position)` (or equivalent construction from
the snapshot's `[f32; 3]`).

Does **NOT** insert:

- `MainPlayer` — never, for any player
- `Viewer` — never
- `SpatialListener` — never
- `PlayerInputMapping` — never
- `FocusRing` child entity — never
- `board_entity_field` registration — never (uses `MapEntityFieldBPos` component only)

### How `LocalPlayer` is set

- **Offline/Host:** `startup_network_system` sets `local_id.0 = Some(NetworkId(1))` (connection.rs line 30/36).
- **Client (Join):** `handshake_handler_system` sets `local_id.0 = Some(*id)` when Welcome is received (connection.rs
  line 471). The `id` comes from the Welcome message, which is the dynamically-assigned `NetworkId` from
  `PlayerRegistry` (starts at 2, increments).

### How `PlayerInputMapping` is used (7 files, 23 sites)

Every usage reads `input_mapping.controls` from a query that filters on `With<MainPlayer>`. The controls are used for
keyboard/mouse input on the local machine. Remote players receive their input via the network (`PlayerInput` component
is synced from snapshots), so they do **not** need keybindings.

`PlayerInputMapping::new(id: NetworkId)` and `default_controls(id: NetworkId)` have **zero callers** in the entire
codebase — confirmed dead code. The orchestrator uses `PlayerInputMapping { controls: **p.control_settings }` directly.

### NetworkId assignment: orchestrator vs PlayerRegistry

- **Orchestrator** (current): Uses `NetworkId(id as u64)` where `id` is a hardcoded `usize` (1 or 2).
- **PlayerRegistry** (from Plan 21): Assigns IDs dynamically starting at 2 for remote players. Host is always
  `NetworkId(1)`.
- **Reconciliation needed**: When the host spawns a player entity in response to a join event, it must use the
  `NetworkId` from `PlayerRegistry` (the same ID that was sent to the client in the Welcome message), not a hardcoded
  integer.

## Risk Analysis

| Risk                                                                                                       | Severity | Mitigation                                                                                                                                                                                                                                                                                                                                                                          |
| ---------------------------------------------------------------------------------------------------------- | -------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Gear spawning for dynamically-joined players replicates complex orchestrator logic                         | High     | Acceptable duplication for now. The gear block is ~25 lines and unlikely to drift. Extracting a shared helper is a larger refactor best done in a follow-up plan.                                                                                                                                                                                                                   |
| NetworkId mismatch between host entity and client's LocalPlayer                                            | High     | The join event carries the exact NetworkId from PlayerRegistry. The spawner uses that ID directly.                                                                                                                                                                                                                                                                                  |
| Race condition: client receives snapshot before MainPlayer is tagged                                       | Medium   | The snapshot system already handles unknown player IDs by calling `spawn_remote_player`, which skips position updates on the first frame (`continue` after spawn). MainPlayer tagging happens in the same system, same frame, before position updates.                                                                                                                              |
| `board_entity_field` registration missing for dynamically spawned players                                  | Medium   | The join-handler on the host adds `board_entity_field` registration. On the client, `MapEntityFieldBPos` component is inserted by `spawn_remote_player`; the `sync_map_entity_field` system in `unrender-plugin` automatically registers/moves entities in `BoardEntityField` whenever `Position` changes (`Changed<Position>` query). No manual registration needed on the client. |
| 1-frame position "pop" for newly spawned players on the client                                             | Low      | `spawn_remote_player` is modified to accept an initial `Position` from the snapshot `PlayerState`, so players appear at their correct location from the first frame.                                                                                                                                                                                                                |
| Client-side player sprites not upscaled                                                                    | Low      | Pre-existing limitation. `spawn_remote_player` uses base 32×32 sprites. Adding upscaling support is a polish task for a future plan, not a regression.                                                                                                                                                                                                                              |
| FocusRing child entity missing on client-spawned players                                                   | Low      | Add FocusRing child in `spawn_remote_player`. Requires access to ghost_assets for the vignette image.                                                                                                                                                                                                                                                                               |
| Removing `PlayerInputMapping::new()` breaks something unexpected                                           | Low      | Confirmed 0 callers. Safe to remove.                                                                                                                                                                                                                                                                                                                                                |
| Orchestrator still gives `PlayerInputMapping { controls: user_settings }` to ALL players, including remote | Medium   | After this plan: orchestrator only spawns the host's own player (always `is_main_player = true`), so it correctly uses user settings. Dynamically-spawned remote players get `PlayerInputMapping { controls: ControlKeys::NONE }` explicitly. **Important:** `ControlKeys::default()` is WASD, not NONE — never use `PlayerInputMapping::default()` for remote players.             |

## Conventions & Constraints for the Implementing Agent

1. **No terminal commands.** You can only search files, read files, edit files, and run workspace checks (`get_errors`).
   You cannot run `cargo clippy`, `cargo build`, or any shell command.

2. **Workspace check protocol.** After every phase (and after every `Cargo.toml` edit):
   - Run `get_errors` with no file filter to get ALL errors.
   - If ANY error is in a `Cargo.toml` file, **STOP all other work**. Fix the `Cargo.toml` first, then re-run
     `get_errors` until `Cargo.toml` files are clean.
   - Only then proceed to fix `.rs` errors.
   - Re-run `get_errors` after fixing `.rs` errors. Repeat until zero errors and zero warnings.

3. **No re-exports.** Never use `pub use`. Every symbol has one canonical path.

4. **`mod.rs` and `lib.rs` must contain only `mod` statements.** No code, no imports.

5. **Imports.** Prefer explicit imports. Wildcard `use` is allowed only for `bevy::prelude::*`.

6. **Do not modify files outside the scope of the current phase.** If you're working on Phase 1, do not touch files that
   belong to Phase 2, 3, or 4.

7. **Do not run the game.** You cannot test by running. Validate using workspace checks.

8. **Review the full workspace for warnings** after each phase. Address unused imports, dead code warnings, etc.

9. **No new crates.** All changes fit within existing crates. The new event type goes in `unnet-core` (data type, no
   logic). The new spawn system goes in `unclassic-mode-plugin` (game logic).

10. **Preserve offline mode.** Offline mode must continue to work exactly as before. Single player spawns with
    `NetworkId(1)`, `MainPlayer`, `Viewer`, `SpatialListener`, and user keybindings.

---

## Phase 1: Host spawns only its own player

### Phase 1 Goal

Modify `classic_mode_orchestrator` so that in networked mode (Host or Join), it spawns only **one** player — the local
player. The host spawns `NetworkId(1)`. The client (Join) spawns **zero** players (it will discover its entity from the
snapshot in Phase 3). Offline mode is unchanged.

### Phase 1, Step 1: Change player_ids_to_spawn in orchestrator.rs

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

**Current code (lines 128–131):**

```rust
    let mut player_ids_to_spawn = vec![1_usize];
    if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Offline) {
        player_ids_to_spawn.push(2);
    }
```

**Replace with:**

```rust
    let player_ids_to_spawn: Vec<usize> = match p.cli.net_mode {
        untypes_core::cli::NetMode::Offline => vec![1],
        untypes_core::cli::NetMode::Host { .. } => vec![1],
        untypes_core::cli::NetMode::Join { .. } => vec![],
    };
```

**Rationale:**

- **Offline:** Spawn player 1 (the only player) — unchanged behavior.
- **Host:** Spawn player 1 (the host) only. Remote players will be spawned dynamically in Phase 2.
- **Join (client):** Spawn zero players. The client will discover its entity from the snapshot (Phase 3).

### Phase 1, Step 2: Simplify is_main_player logic

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

**Current code (lines 133–138):**

```rust
    for (idx, id) in player_ids_to_spawn.into_iter().enumerate() {
        let is_main_player = match p.cli.net_mode {
            untypes_core::cli::NetMode::Offline => true,
            untypes_core::cli::NetMode::Host { .. } => id == 1,
            untypes_core::cli::NetMode::Join { .. } => id == 2,
        };
```

**Replace with:**

```rust
    for (idx, id) in player_ids_to_spawn.into_iter().enumerate() {
        // In the new model:
        // - Offline: always the main player (only player).
        // - Host: spawns only id=1, which is always the main player.
        // - Join: spawns nothing (vec is empty), so this loop body never runs.
        let is_main_player = true;
```

**Rationale:** Since the orchestrator now only spawns the local player (or nothing for Join), every player it spawns is
by definition the main player. The match arms for `id == 1` and `id == 2` are dead logic because:

- Host only spawns `id == 1`, so `id == 1` is always true.
- Join spawns nothing, so the `id == 2` arm is never reached.

### Phase 1, Step 3: Gear spawning guard is now redundant — verify and simplify

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

**Current code (lines 140–141):**

```rust
        if !matches!(p.cli.net_mode, untypes_core::cli::NetMode::Join { .. }) {
```

This guard skips gear spawning on the client. Since the client no longer enters this loop (empty `player_ids_to_spawn`),
the guard is tautologically true for all remaining cases (Offline and Host both spawn gear).

**Action:** Keep the guard as-is for safety. It is harmless and provides defense-in-depth. Do NOT remove it.

### Phase 1 Validation

After making the above changes:

1. Run `get_errors` with no file filter.
2. Verify no compile errors in `unclassic-mode-plugin`.
3. Check for warnings about unused variables (e.g., if `is_main_player` is now flagged as unnecessary — it should NOT
   be, because it is still used in the `if is_main_player { ... }` block on line 234).
4. Verify that `p.cli.net_mode` import is still used (it is, for the gear guard on line 140).

### Phase 1 Behavioral Impact

| Mode          | Before                                                | After                                                                    |
| ------------- | ----------------------------------------------------- | ------------------------------------------------------------------------ |
| Offline       | 1 player spawned, MainPlayer                          | 1 player spawned, MainPlayer — **unchanged**                             |
| Host          | 2 players spawned (ids 1, 2), only id=1 is MainPlayer | 1 player spawned (id 1), MainPlayer — **player 2 no longer pre-spawned** |
| Join (client) | 2 players spawned (ids 1, 2), only id=2 is MainPlayer | 0 players spawned — **client relies on snapshot**                        |

---

## Phase 2: Host dynamically spawns players on join

### Phase 2 Goal

When a remote client completes its handshake, the host spawns a new player entity with full gear at an available spawn
point. This entity then appears in snapshots and is automatically discovered by all clients.

### Phase 2, Step 1: Define `PlayerJoinedEvent` in unnet-core

**File:** `crates/unnet-core/src/messages.rs`

**Action:** Add the following at the end of the file, after the existing `SendNetworkMessage` struct:

```rust
/// Emitted on the host when a client completes handshake and is ready to play.
#[derive(Debug, Clone, Message)]
pub struct PlayerJoinedEvent {
    pub id: NetworkId,
}
```

**Note:** This follows the exact same pattern as `NetworkDisconnectEvent` (same derive, same field). The `Message`
derive comes from the `unevents-core` crate and is already in scope (it is used by `NetworkDataEvent`,
`NetworkDisconnectEvent`, and `SendNetworkMessage` in the same file).

### Phase 2, Step 2: Register the new event in the net plugin

**File:** `crates/unnet-plugin/src/plugin.rs`

**Current code (around lines 17–20):**

```rust
        app.add_message::<NetworkDataEvent>();
        app.add_message::<unnet_core::messages::NetworkDisconnectEvent>();
        app.add_message::<unnet_core::messages::SendNetworkMessage>();
        app.add_message::<unnet_core::messages::TransientEvent>();
```

**Action:** Add a new line after the `NetworkDisconnectEvent` registration:

```rust
        app.add_message::<NetworkDataEvent>();
        app.add_message::<unnet_core::messages::NetworkDisconnectEvent>();
        app.add_message::<unnet_core::messages::PlayerJoinedEvent>();
        app.add_message::<unnet_core::messages::SendNetworkMessage>();
        app.add_message::<unnet_core::messages::TransientEvent>();
```

### Phase 2, Step 3: Emit `PlayerJoinedEvent` from `handshake_handler_system`

**File:** `crates/unnet-plugin/src/systems/connection.rs`

**Current state:** `handshake_handler_system` function signature (around line 375) does NOT have a
`MessageWriter<PlayerJoinedEvent>` parameter.

**Action (3a):** Add a new parameter to `handshake_handler_system`:

```rust
    mut ev_player_joined: MessageWriter<unnet_core::messages::PlayerJoinedEvent>,
```

Add it after the existing `mut ev_reader: MessageReader<NetworkDataEvent>` parameter.

**Action (3b):** After the line `client.needs_full_sync = true;` (around line 424), add:

```rust
                            ev_player_joined.write(unnet_core::messages::PlayerJoinedEvent {
                                id: source_id,
                            });
```

**Important:** This must be inside the `for client in clients.iter_mut()` loop, inside the
`if client.associated_id == Some(source_id)` block, right after `client.needs_full_sync = true;`. It should fire for
every NEW handshake. It should NOT fire for re-associations (when a disconnected player reconnects) — but currently the
code does not distinguish between first-connect and reconnect. This is acceptable for now because:

- On reconnect, the player entity already exists (it was marked `PlayerDisconnected` but not despawned).
- The spawn handler in Step 4 will check if an entity with that `NetworkId` already exists and skip spawning if so.

### Phase 2, Step 4: Create a system to spawn players on join

**File:** `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`

**Action:** Add a new public(crate) function `spawn_joined_player` at the end of the file (before the final `}`).

This function runs on the **host only** and listens for `PlayerJoinedEvent`. When it fires, it spawns a player entity
with gear, using the same visual setup as the orchestrator. It reuses the same assets and difficulty settings.

```rust
/// Spawns a player entity when a remote client completes handshake (host only).
pub(crate) fn spawn_joined_player(
    mut p: ClassicModeSystemParam,
    mut commands: Commands,
    mut ev_joined: MessageReader<unnet_core::messages::PlayerJoinedEvent>,
    existing_players: Query<&NetworkId, With<PlayerTag>>,
    q_player_spawns: Query<&Position, With<PlayerSpawnPoint>>,
    q_player_positions: Query<&Position, With<PlayerSprite>>,
) {
    for ev in ev_joined.read() {
        let new_id = ev.id;

        // Skip if entity already exists (reconnect case)
        if existing_players.iter().any(|id| *id == new_id) {
            info!("Player {:?} already has an entity, skipping spawn", new_id);
            continue;
        }

        // Pick a spawn point. Try to find one not too close to existing players.
        let player_spawn_points: Vec<Position> = q_player_spawns.iter().copied().collect();
        if player_spawn_points.is_empty() {
            error!("No player spawn points found, cannot spawn joined player {:?}", new_id);
            continue;
        }

        // Use NetworkId to pick a deterministic spawn point. Host is NetworkId(1) → index 0,
        // first client is NetworkId(2) → index 1, etc. This is stable regardless of join/leave order.
        let spawn_idx = (new_id.0 as usize - 1) % player_spawn_points.len();
        let spawn_pos = player_spawn_points[spawn_idx];

        info!(
            "Spawning player {:?} at {:?} (spawn point index {})",
            new_id, spawn_pos, spawn_idx
        );

        // --- Gear ---
        let mut player_gear = PlayerGear::default();
        let mut gear_id_counter = new_id.0 * 1000;
        if p.difficulty.0.player_gear.left_hand.is_some() {
            let gear_entity = p
                .gear_registry
                .spawn(&mut commands, p.difficulty.0.player_gear.left_hand);
            player_gear.left_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert(NetworkId(gear_id_counter));
            gear_id_counter += 1;
        }
        if p.difficulty.0.player_gear.right_hand.is_some() {
            let gear_entity = p
                .gear_registry
                .spawn(&mut commands, p.difficulty.0.player_gear.right_hand);
            player_gear.right_hand = Some(gear_entity);
            commands
                .entity(gear_entity)
                .insert(NetworkId(gear_id_counter));
            gear_id_counter += 1;
        }
        for kind in &p.difficulty.0.player_gear.inventory {
            if kind.is_some() {
                let gear_entity = p.gear_registry.spawn(&mut commands, *kind);
                player_gear.inventory.push(gear_entity);
                commands
                    .entity(gear_entity)
                    .insert(NetworkId(gear_id_counter));
                gear_id_counter += 1;
            }
        }

        // --- Visual setup ---
        let mut player_image = p.player_assets.character.clone();
        let mut player_rf = 1.0;
        if let Some(resolved) = p.upscale_idx.resolve(
            "img/characters-model1-demo.png",
            p.video_settings.max_upscale_factor.factor(),
        ) {
            player_image = p.asset_server.load(resolved.path);
            player_rf = resolved.factor;
        }

        let sprite_size = Vec2::new(32.0 * player_rf, 32.0 * player_rf);
        let anchor = unplayer_core::assets::PLAYER_ANCHOR;
        let sprite_anchor = Vec2::new(
            sprite_size.x * (anchor.x + 0.5),
            sprite_size.y * (0.5 - anchor.y),
        );
        let src_mesh_handle = p
            .meshes
            .add(Mesh::from(QuadCC::new(sprite_size, sprite_anchor)));

        let spawn_scoord = perspective::to_screen_coord(spawn_pos);

        let mut material = CustomMaterial1::from_texture(player_image);
        material.data.sheet_cols = 16;
        material.data.sheet_rows = 4;
        material.data.sprite_width = 32.0 * player_rf;
        material.data.sprite_height = 32.0 * player_rf;
        material.data.upscale_factor = player_rf;
        material.data.y_anchor = anchor.y;

        let material_handle = p.materials1.add(material);

        let mut ec = commands.spawn(Mesh2d(src_mesh_handle));
        ec.insert(MeshMaterial2d(material_handle))
            .insert(
                Transform::from_xyz(spawn_scoord[0], spawn_scoord[1], spawn_scoord[2])
                    .with_scale(Vec3::new(1.0 / player_rf, 1.0 / player_rf, 1.0 / player_rf)),
            )
            .insert(ResolutionFactor(player_rf))
            .insert(GameSprite)
            .insert(MapTileSprite)
            .insert(SpriteLayer(0.00001));

        // Remote player: no MainPlayer, no Viewer, no SpatialListener.
        // Use explicit ControlKeys::NONE — ControlKeys::default() is WASD, not NONE!
        ec.insert(PlayerSprite::new(new_id, spawn_pos))
            .insert(new_id)
            .insert(MapColor {
                color: Color::WHITE,
            })
            .insert(PlayerInputMapping {
                controls: unsettings_core::controls::ControlKeys::NONE,
            })
            .insert(PlayerInput::default())
            .insert(VisibilityData::default())
            .insert(PlayerTag)
            .insert(ShadowCaster::default())
            .insert(spawn_pos)
            .insert(MapEntityFieldBPos(spawn_pos.to_board_position()))
            .insert(Movable)
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.01,
            })
            .insert(Direction::new_right())
            .insert(AnimationTimer::from_range(
                Timer::from_seconds(0.20, TimerMode::Repeating),
                CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
            ))
            .insert(Stamina::default())
            .insert(unnavigation_core::components::waypoint::WaypointQueue::default());

        ec.insert(player_gear);

        let player_ent_id = ec
            .with_children(|parent| {
                parent
                    .spawn(Sprite {
                        image: p.ghost_assets.focus_ring_vignette.clone(),
                        color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                        ..default()
                    })
                    .insert(
                        Transform::from_scale(Vec3::splat(1.1 * player_rf))
                            .with_translation(Vec3::new(0.0, 0.1, 0.01)),
                    )
                    .insert(FocusRing::default());
            })
            .id();

        p.board_entity_field.0[spawn_pos.to_board_position().ndidx()].push(player_ent_id);
    }
}
```

**Key design choices:**

- **No `MainPlayer`/`Viewer`/`SpatialListener`:** This is a remote player on the host. Only the host's own player
  (spawned in the orchestrator) has these.
- **`PlayerInputMapping { controls: ControlKeys::NONE }`:** Explicit NONE controls. `ControlKeys::default()` is actually
  WASD (not NONE), so we must use the `NONE` constant explicitly. Remote players don't use keyboard input on the host —
  their input arrives via `NetworkMessage::PlayerInput`.
- **Gear spawning:** Uses `new_id.0 * 1000` as the base for gear `NetworkId`s, same formula as the orchestrator.
- **FocusRing child:** Included for visual consistency.
- **`board_entity_field` registration:** Included to match the orchestrator.
- **Reconnect guard:** The `existing_players.iter().any(|id| *id == new_id)` check prevents duplicate spawning when a
  player reconnects (their entity was marked `PlayerDisconnected` but not despawned).

### Phase 2, Step 5: Register `spawn_joined_player` in the plugin

**File:** `crates/unclassic-mode-plugin/src/systems/plugin.rs`

**Current content (around lines 12–15):**

Look at the current plugin build function. The orchestrator is registered like this:

```rust
        app.add_systems(
            Update,
            crate::systems::orchestrator::classic_mode_orchestrator
                .run_if(on_message::<MapEntitiesReadyEvent>),
        );
```

**Action:** Add a new system registration after the orchestrator:

```rust
        app.add_systems(
            Update,
            crate::systems::orchestrator::spawn_joined_player
                .run_if(in_state(AppState::InGame))
                .run_if(on_message::<unnet_core::messages::PlayerJoinedEvent>),
        );
```

This system:

- Runs only when `AppState::InGame` (game is loaded and running).
- Runs only when a `PlayerJoinedEvent` has been emitted (event-driven, not polling).
- Runs on the host (only the host emits `PlayerJoinedEvent`).

**Note:** Add the necessary imports at the top of `plugin.rs`:

```rust
use untypes_core::states::AppState;
```

The `on_message` import should already be in scope from the orchestrator registration. If not, add:

```rust
use unevents_core::prelude::on_message;
```

### Phase 2 Validation

After making the above changes:

1. Run `get_errors` with no file filter.
2. Verify no compile errors across the workspace.
3. Grep for `PlayerJoinedEvent` to confirm:
   - Defined in `crates/unnet-core/src/messages.rs`
   - Registered in `crates/unnet-plugin/src/plugin.rs`
   - Written in `crates/unnet-plugin/src/systems/connection.rs`
   - Read in `crates/unclassic-mode-plugin/src/systems/orchestrator.rs`
4. Check for unused import warnings.

### Phase 2 Behavioral Impact

| Scenario                       | Before                                   | After                                                                                                                       |
| ------------------------------ | ---------------------------------------- | --------------------------------------------------------------------------------------------------------------------------- |
| Host starts game, no clients   | 2 players spawned                        | 1 player spawned (host only)                                                                                                |
| Client connects and handshakes | No new entity (player 2 was pre-spawned) | `PlayerJoinedEvent` emitted → `spawn_joined_player` creates a new entity                                                    |
| Client reconnects              | `PlayerDisconnected` removed             | `PlayerJoinedEvent` emitted → guard detects existing entity → skip spawn, `PlayerDisconnected` removed by handshake handler |
| 3rd client connects            | No entity (only ids 1, 2 existed)        | `PlayerJoinedEvent` → new entity with `NetworkId(3)` from `PlayerRegistry`                                                  |

---

## Phase 3: Client identifies its own player from snapshot

### Phase 3 Goal

On the client (Join mode), the local player entity is discovered dynamically from snapshot data, not pre-spawned by the
orchestrator. When `spawn_remote_player` (in `client_sync.rs`) creates an entity whose `NetworkId` matches
`LocalPlayer.0`, that entity is tagged with `MainPlayer`, `Viewer`, `SpatialListener`, and `PlayerInputMapping` with the
user's control settings.

### Phase 3, Step 1: Tag MainPlayer in `client_apply_snapshots_system`

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

**Current code (around lines 712–725):**

```rust
            // Update players
            for p_state in players {
                let p_entity = if let Some(e) = net_to_entity.get(&p_state.id) {
                    *e
                } else {
                    debug!("Spawning remote player {:?}", p_state.id);
                    let initial_pos = Position {
                        x: p_state.position[0],
                        y: p_state.position[1],
                        z: p_state.position[2],
                        visual_priority: 0.0,
                    };
                    let ent = spawn_remote_player(&mut params, p_state.id, initial_pos);
                    // Insert into net_to_entity immediately to prevent duplicate
                    // spawns when multiple snapshots are processed in the same
                    // frame. The entity was created via Commands (deferred) so it
                    // won't be queryable until next frame, but
                    // spawn_remote_player already applied initial state.
                    net_to_entity.insert(p_state.id, ent);
                    continue;
                };
```

**Action:** After `net_to_entity.insert(p_state.id, ent);`, add a block that tags the entity as MainPlayer if it matches
`LocalPlayer`:

```rust
                    net_to_entity.insert(p_state.id, ent);

                    // If this is OUR player, tag it as MainPlayer
                    if local_player.0 == Some(p_state.id) {
                        info!("Tagging spawned player {:?} as MainPlayer", p_state.id);
                        params.commands.entity(ent)
                            .insert(unplayer_core::components::MainPlayer);
                    }

                    continue;
```

**Why here?** This is the moment a new player entity is created from a snapshot. If the new entity's `NetworkId` matches
`LocalPlayer.0`, this is our player and must be tagged immediately.

### Phase 3, Step 2: Ensure `local_player` resource is accessible in the system

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

Check whether `client_apply_snapshots_system` already has access to `LocalPlayer`. Search the function signature and the
`ClientSnapshotParams` system param.

**Current state:** The system function `client_apply_snapshots_system` likely accesses `LocalPlayer` directly or through
a system parameter. Look at lines around 440 to see how `main_player_net_id` is determined:

```rust
            for (_, id, _, _, _, _, _, gear, main_player, _, _, _, _) in params.query_players.iter()
            {
                if main_player.is_some() {
                    main_player_net_id = Some(*id);
```

This block finds the MainPlayer by querying for the `MainPlayer` component. But on the first snapshot after join, no
entity has `MainPlayer` yet (we're about to add it in Step 1).

**Action (2a):** Add `local_player: Res<unnet_core::resources::LocalPlayer>` as a parameter to
`client_apply_snapshots_system` if it's not already there. Check the existing function signature first.

Let me check the actual signature of the function:

The function uses `ClientSnapshotParams` as its primary system parameter. Check if `LocalPlayer` is already a field in
`ClientSnapshotParams`. If not, either:

- **(Option A)** Add `local_player: Res<'w, unnet_core::resources::LocalPlayer>` to `ClientSnapshotParams`, OR
- **(Option B)** Add it as a separate parameter to the function.

**Choose Option B** — adding to the function directly is simpler and avoids touching the SystemParam struct.

### Phase 3, Step 3: Add `Viewer` and `SpatialListener` to the MainPlayer entity

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

**Problem:** `spawn_remote_player` does not insert `Viewer` or `SpatialListener`. These are needed for the local player
to see lights and hear sounds. On the host, the orchestrator adds them. On the client, we need to add them when we
discover our player.

**Action:** Expand the MainPlayer tagging block from Step 1:

```rust
                    // If this is OUR player, tag it as MainPlayer
                    if local_player.0 == Some(p_state.id) {
                        info!("Tagging spawned player {:?} as MainPlayer", p_state.id);
                        params.commands.entity(ent)
                            .insert(unplayer_core::components::MainPlayer)
                            .insert(unrender_std::components::visuals::Viewer {
                                id: p_state.id,
                                ..default()
                            })
                            .insert(SpatialListener::new(
                                -params.audio_settings.sound_output.to_ear_offset(),
                            ));
                    }
```

**Dependency:** This requires `audio_settings` to be accessible. Check if `ClientSnapshotParams` has it. If not, add:

```rust
pub audio_settings: Res<'w, bevy_persistent::Persistent<unsettings_core::audio::AudioSettings>>,
```

to the `ClientSnapshotParams` struct.

**Also required:** `SpatialListener` must be imported. Check if it's already in scope. If not, add:

```rust
use bevy::audio::SpatialListener;
```

### Phase 3, Step 4: Add `PlayerInputMapping` with user settings

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

The MainPlayer on the client needs `PlayerInputMapping` with the user's actual control settings, not the default NONE
controls.

**Action:** Extend the tagging block to also insert `PlayerInputMapping`:

```rust
                    // If this is OUR player, tag it as MainPlayer
                    if local_player.0 == Some(p_state.id) {
                        info!("Tagging spawned player {:?} as MainPlayer", p_state.id);
                        params.commands.entity(ent)
                            .insert(unplayer_core::components::MainPlayer)
                            .insert(unrender_std::components::visuals::Viewer {
                                id: p_state.id,
                                ..default()
                            })
                            .insert(SpatialListener::new(
                                -params.audio_settings.sound_output.to_ear_offset(),
                            ))
                            .insert(unplayer_core::components::PlayerInputMapping {
                                controls: **params.control_settings,
                            });
                    }
```

**Dependency:** This requires `control_settings` to be accessible. Check if `ClientSnapshotParams` has it. If not, add:

```rust
pub control_settings: Res<'w, bevy_persistent::Persistent<unsettings_core::controls::ControlKeys>>,
```

to the `ClientSnapshotParams` struct.

### Phase 3, Step 5: Modify `spawn_remote_player` signature and fix position initialization

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

**Problem:** `spawn_remote_player` currently takes only `(params, id)` and initializes position to `(0, 0, 0)`. This
causes a 1-frame visual "pop" where the player appears at the map origin before snapping to their real location.

**Action (5a):** Change the function signature from:

```rust
pub(crate) fn spawn_remote_player(params: &mut ClientSnapshotParams, id: NetworkId) -> Entity {
```

to:

```rust
pub(crate) fn spawn_remote_player(params: &mut ClientSnapshotParams, id: NetworkId, initial_pos: Position) -> Entity {
```

**Action (5b):** Inside the function, replace the two places where `Position::new_i64(0, 0, 0)` is used:

- `PlayerSprite::new(id, Position::new_i64(0, 0, 0))` → `PlayerSprite::new(id, initial_pos)`
- `.insert(Position::new_i64(0, 0, 0))` → `.insert(initial_pos)`
- `.insert(MapEntityFieldBPos(Position::new_i64(0, 0, 0).to_board_position()))` →
  `.insert(MapEntityFieldBPos(initial_pos.to_board_position()))`

### Phase 3, Step 6: Add FocusRing child to `spawn_remote_player`

**File:** `crates/unnet-plugin/src/systems/client_sync.rs`

The orchestrator spawns a `FocusRing` child entity on every player. `spawn_remote_player` does not. This causes a visual
gap — the focus ring vignette is missing on client-side player entities.

**Action:** Before `ec.id()` at the end of `spawn_remote_player`, add:

```rust
    ec.with_children(|parent| {
        parent
            .spawn(Sprite {
                image: params.ghost_assets.focus_ring_vignette.clone(),
                color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                ..default()
            })
            .insert(
                Transform::from_scale(Vec3::splat(1.1 * player_rf))
                    .with_translation(Vec3::new(0.0, 0.1, 0.01)),
            )
            .insert(unrender_std::components::focus_ring::FocusRing::default());
    });
```

**Dependency:** `params.ghost_assets` must be accessible in `spawn_remote_player`. Check if `ClientSnapshotParams` has a
`ghost_assets` field. If not, add:

```rust
pub ghost_assets: Res<'w, unghost_core::assets::GhostAssets>,
```

### Phase 3, Step 7: Verify `ClientSnapshotParams` has all required fields

Before implementing Steps 1–6, read the current `ClientSnapshotParams` definition (around line 40–120 in
`client_sync.rs`) to see what fields already exist. Add only the missing fields:

- `audio_settings: Res<'w, bevy_persistent::Persistent<unsettings_core::audio::AudioSettings>>`
- `control_settings: Res<'w, bevy_persistent::Persistent<unsettings_core::controls::ControlKeys>>`
- `ghost_assets: Res<'w, unghost_core::assets::GhostAssets>`

Add new dependencies to `crates/unnet-plugin/Cargo.toml` if needed. Check if `unsettings-core`, `unghost-core`, and
`bevy_persistent` are already dependencies. If any are missing, add them (this is the only reason to edit Cargo.toml).

**Workspace check protocol applies:** After editing `Cargo.toml`, run `get_errors` IMMEDIATELY before touching any `.rs`
file.

### Phase 3 Validation

After making all changes:

1. Run `get_errors` with no file filter.
2. Verify no compile errors.
3. Verify that the `LocalPlayer` resource is correctly passed to the snapshot system.
4. Grep for `MainPlayer` in `client_sync.rs` to verify:
   - Import exists
   - It is inserted conditionally when `local_player.0 == Some(p_state.id)`
5. Grep for `Viewer` in `client_sync.rs` to verify it is inserted with the player's `NetworkId`.
6. Grep for `SpatialListener` in `client_sync.rs` to verify it is inserted.
7. Grep for `PlayerInputMapping` in `client_sync.rs` to verify it uses `**params.control_settings`.

### Phase 3 Behavioral Impact

| Scenario                                         | Before                                                                                              | After                                                                                                                        |
| ------------------------------------------------ | --------------------------------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------- |
| Client receives first snapshot with its player   | `spawn_remote_player` creates bare entity; MainPlayer was set by orchestrator on a different entity | `spawn_remote_player` creates bare entity → MainPlayer/Viewer/SpatialListener/PlayerInputMapping inserted on matching entity |
| Client receives snapshot with other players      | `spawn_remote_player` creates bare entity                                                           | `spawn_remote_player` creates bare entity — **unchanged**                                                                    |
| MainPlayer identification in snapshot processing | Finds entity with `MainPlayer` component (works if orchestrator set it)                             | Finds entity with `MainPlayer` component (works because we set it in Step 1)                                                 |

---

## Phase 4: Clean up dead code

### Phase 4 Goal

Remove dead code related to the old hardcoded player spawning model. This makes the codebase clearer and prevents future
confusion.

### Phase 4, Step 1: Remove `PlayerInputMapping::new()` and `default_controls()`

**File:** `crates/unplayer-core/src/components.rs`

**Current code (lines 166–183):**

```rust
impl PlayerInputMapping {
    pub fn new(id: NetworkId) -> Self {
        Self {
            controls: Self::default_controls(id),
        }
    }

    /// Returns the default `ControlKeys` for the given player ID.
    fn default_controls(id: NetworkId) -> ControlKeys {
        match id.0 {
            1 => ControlKeys::WASD,
            2 => ControlKeys::IJKL,
            _ => ControlKeys::NONE,
        }
    }
}
```

**Action:** Remove the entire `impl PlayerInputMapping` block. The `#[derive(Default)]` on the struct provides a
`default()` implementation which gives `ControlKeys::default()` = **WASD** controls. This is fine because every site
that creates a `PlayerInputMapping` either uses the user's saved settings (orchestrator, client tagging) or explicit
`ControlKeys::NONE` (spawn_joined_player). No code calls `new()` or `default_controls()`.

After removal, check if `NetworkId` is still imported in this file. If the only reason for importing `NetworkId` was
`PlayerInputMapping::new()`, remove the import too. However, `NetworkId` is also used by `PlayerSprite`, so it should
still be needed.

### Phase 4, Step 2: Verify no new callers

After removal:

1. Grep for `PlayerInputMapping::new` across the workspace — should return 0 results.
2. Grep for `default_controls` across the workspace — should return 0 results.
3. Run `get_errors` to confirm clean compile.

### Phase 4 Validation

1. Run `get_errors` with no file filter.
2. Verify zero compile errors.
3. Verify zero warnings about dead code in `PlayerInputMapping`.

---

## Summary: Files modified per phase

### Phase 1

| File                                                       | Change                                                        |
| ---------------------------------------------------------- | ------------------------------------------------------------- |
| `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` | Change `player_ids_to_spawn` logic, simplify `is_main_player` |

### Phase 2

| File                                                       | Change                                                 |
| ---------------------------------------------------------- | ------------------------------------------------------ |
| `crates/unnet-core/src/messages.rs`                        | Add `PlayerJoinedEvent` struct                         |
| `crates/unnet-plugin/src/plugin.rs`                        | Register `PlayerJoinedEvent` with `add_message`        |
| `crates/unnet-plugin/src/systems/connection.rs`            | Emit `PlayerJoinedEvent` in `handshake_handler_system` |
| `crates/unclassic-mode-plugin/src/systems/orchestrator.rs` | Add `spawn_joined_player` system                       |
| `crates/unclassic-mode-plugin/src/systems/plugin.rs`       | Register `spawn_joined_player` system                  |

### Phase 3

| File                                             | Change                                                                                                                                                             |
| ------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `crates/unnet-plugin/src/systems/client_sync.rs` | Tag MainPlayer in snapshot processing, add Viewer/SpatialListener/PlayerInputMapping, change `spawn_remote_player` to accept initial position, add FocusRing to `spawn_remote_player`, add fields to `ClientSnapshotParams` |
| `crates/unnet-plugin/Cargo.toml`                 | Add missing dependencies (if any)                                                                                                                                  |

### Phase 4

| File                                     | Change                                             |
| ---------------------------------------- | -------------------------------------------------- |
| `crates/unplayer-core/src/components.rs` | Remove `impl PlayerInputMapping` block (dead code) |

## Post-Implementation Verification

After all 4 phases are complete, perform a final workspace-wide verification:

1. **`get_errors`** — zero errors, zero warnings.
2. **Grep `player_ids_to_spawn`** — should only appear in the modified orchestrator, with the new match expression.
3. **Grep `PlayerJoinedEvent`** — should appear in 4 files (messages.rs, plugin.rs, connection.rs, orchestrator.rs).
4. **Grep `MainPlayer` in client_sync.rs** — should show the new tagging logic.
5. **Grep `PlayerInputMapping::new`** — should return 0 results.
6. **Grep `default_controls`** — should return 0 results (was only in the deleted impl block).
7. **Grep `is_main_player`** — should still appear in orchestrator.rs (set to `true`), used in the `if is_main_player`
   block.
