use crate::metrics;
use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy_persistent::Persistent;
use bevy_platform::collections::HashMap;
use ndarray::{Array3, s};
use rand::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::roomdb::RoomTopology;
use unboard_core::components::chunk::{CellIterator, ChunkIterator};
use unboard_core::components::physics::FluidEmitter;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use unfog_core::components::MiasmaSprite;
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;
use unfoundation_core::random_seed;
use unmapload_core::events::loadlevel::{LevelReadyEvent, MapGeometryInitializedEvent};
use unmetrics_core::metrics::SendMetric;
use unnoise_core::perlin::PerlinNoise;
use unplayer_core::components::MainPlayer;
use unrender_std::components::game::GameSprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unrender_std::components::visuals::LightSensitive;
use unrender_std::resources::visibility_data::VisibilityData;
use unrender_std::utils::collision::rebuild_collision_data;
use unsettings_core::video::VideoSettings;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;

pub(crate) fn init_miasma_grid(
    mut commands: Commands,
    mut ev: MessageReader<MapGeometryInitializedEvent>,
) {
    for ev in ev.read() {
        commands.insert_resource(MiasmaGrid {
            pressure_field: Array3::from_elem(ev.map_size, 0.0),
            velocity_field: Array3::from_elem(ev.map_size, Vec2::ZERO),
            room_modifiers: Default::default(),
        });
    }
}

pub(crate) fn reset_miasma_grid(mut commands: Commands) {
    commands.remove_resource::<MiasmaGrid>();
}

pub(crate) fn initialize_miasma(
    board_data: Res<BoardTopology>,
    mut bcf: ResMut<BoardCollisionField>,
    mut miasma: If<ResMut<MiasmaGrid>>,
    room_topology: Res<RoomTopology>,
    config: Res<MiasmaConfig>,
    mut level_ready: MessageReader<LevelReadyEvent>,
    qt: Query<(Entity, &Position, &Behavior)>,
) {
    // Only run on LevelLoadedEvent
    if level_ready.read().next().is_none() {
        return;
    }
    trace!("Miasma Init");
    rebuild_collision_data(&board_data, &mut bcf, &qt);

    miasma.room_modifiers.clear();

    let mut rng = random_seed::rng();
    let collision_field = bcf.0.clone();

    for (p, cfield) in collision_field.indexed_iter() {
        let board_position = BoardPosition::from_ndidx(p);
        let opt_room_id = room_topology.room_tiles.get(&board_position);

        // 1. Get or Insert Room Modifier:
        let mut modifier = if let Some(room_id) = opt_room_id {
            *miasma
                .room_modifiers
                .entry(room_id.clone()) // Try to get the modifier for this room_id.
                .or_insert_with(|| rng.random_range(0.4..=2.9)) // If not found, create one.
        } else {
            0.0
        };
        if !cfield.player_free {
            modifier = 0.0;
        }

        // 2. Set Initial Pressure:
        miasma.pressure_field[board_position.ndidx()] =
            config.initial_room_pressure * modifier * rng.random_range(0.9..=1.1);
    }
    trace!("Done: Miasma Init");
}

pub(crate) fn spawn_miasma(
    time: Res<Time>,
    miasma: If<Res<MiasmaGrid>>,
    q_vf: Query<&VisibilityData, With<MainPlayer>>,
    mut q_miasma: Query<(Entity, &mut MiasmaSprite)>,
    q_player: Query<&Position, With<MainPlayer>>,
    ghost_assets: Res<unghost_core::assets::GhostAssets>,
    board_data: Res<BoardTopology>,
    bcf: Res<BoardCollisionField>,
    video_settings: Res<Persistent<VideoSettings>>,
    mut commands: Commands,
) {
    let measure = metrics::SPAWN_MIASMA.time_measure();

    let Ok(vf) = q_vf.single() else {
        return;
    };
    let count_quality = video_settings.quality.to_quality_factor2();
    let area_quality = video_settings.quality.to_quality_factor1();
    const THRESHOLD: f32 = 0.000001;
    const DIST_FACTOR: f32 = 0.00001;
    const MIASMA_TARGET_SPRITE_COUNT: usize = 3;
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    if board_data.map_size.0 == 0 {
        return;
    }
    // Find the active player's position
    let Ok(player_pos) = q_player.single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    if vf.visibility_field.dim() != bcf.0.dim() {
        // If the visibility field hasn't updated to the same size, skip processing.
        // This happens on map load and takes 1-2 frames to stabilize.
        return;
    }

    let mut count: HashMap<BoardPosition, usize> = HashMap::new();
    for (entity, mut miasma_sprite) in q_miasma.iter_mut() {
        if miasma_sprite.despawn {
            miasma_sprite.life -= dt * 2.0;
        }
        if miasma_sprite.life < -2.0 {
            miasma_sprite.life = -2.0;
            commands.entity(entity).despawn();
        }
        if miasma_sprite.despawn {
            continue;
        }
        let bpos = miasma_sprite.base_position.to_board_position();
        let Some(pressure) = miasma.pressure_field.get(bpos.ndidx()) else {
            miasma_sprite.despawn = true;
            continue;
        };
        miasma_sprite.life -= dt / 10.0;
        if miasma_sprite.life < 0.02 {
            miasma_sprite.despawn = true;
            continue;
        }
        let player_dst2 = player_pos.distance2(&miasma_sprite.base_position);

        let vis = vf.visibility_field[bpos.ndidx()] + DIST_FACTOR / player_dst2;
        let target_f =
            (f32::cbrt(*pressure) / 3.1 + 0.1).min(1.0) * MIASMA_TARGET_SPRITE_COUNT as f32;
        let target_count = (target_f * count_quality).ceil() as usize;

        let pos_count = count.entry(bpos).or_default();

        if vis < THRESHOLD || *pos_count > target_count * 9 {
            miasma_sprite.despawn = true;
        } else {
            *pos_count += 1;
        }
    }
    // Limit the number of cells to check to 15x15 around the player
    let max_radius = (15.0 * area_quality).max(4.0) as i64;
    let visibility_boost = (1.0 / count_quality).sqrt();
    let min_x = (player_bpos.x - max_radius).max(0) as usize;
    let max_x = (player_bpos.x + max_radius).min(board_data.map_size.0 as i64 - 1) as usize;
    let min_y = (player_bpos.y - max_radius).max(0) as usize;
    let max_y = (player_bpos.y + max_radius).min(board_data.map_size.1 as i64 - 1) as usize;
    let z = player_bpos.z as usize;

    for (bp, vis) in vf
        .visibility_field
        .slice(s![min_x..=max_x, min_y..=max_y, z..=z])
        .indexed_iter()
    {
        let bp = (bp.0 + min_x, bp.1 + min_y, bp.2 + z);
        let collision = &bcf.0[bp];
        if !collision.player_free && !collision.see_through {
            continue; // Skip only full walls, allow half-walls for miasma sprites
        }
        let bpos = BoardPosition::from_ndidx(bp);
        let player_dst2 = player_pos.distance2(&bpos.to_position_center());
        let vis = vis + DIST_FACTOR / player_dst2;
        if vis < THRESHOLD * 2.0 {
            continue;
        }
        let target9_f = (miasma.pressure_field[bpos.ndidx()] / 1.1 + 0.1).min(1.0)
            * 9.0
            * MIASMA_TARGET_SPRITE_COUNT as f32;
        let target9_count = (target9_f * count_quality).ceil() as usize;

        let pos9_count = bpos
            .iter_xy_neighbors(1, board_data.map_size)
            .map(|bpos| count.get(&bpos).copied().unwrap_or_default())
            .sum::<usize>();

        let pos_count = count.entry(bpos.clone()).or_default();

        if pos9_count < target9_count {
            // Spawn miasma if too low
            let scale = rng.random_range(0.15..1.0_f32).sqrt() * 1.8;
            let mut pos = bpos
                .to_position_center()
                .with_visual_priority(0.00037 * rng.random_range(0.99..1.01));
            pos.x += rng.random_range(-0.5..0.5);
            pos.y += rng.random_range(-0.5..0.5);

            commands
                .spawn(Sprite {
                    image: ghost_assets.miasma.clone(),
                    color: Color::linear_rgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                })
                .insert(Anchor(unmapload_core::assets::GRID_1X1_ANCHOR))
                .insert(MiasmaSprite {
                    base_position: pos,
                    radius: rng.random_range(0.15..0.45), // Small radius
                    angular_speed: rng.random_range(0.05..0.5), // Slow speed
                    phase: rng.random_range(0.0..std::f32::consts::TAU), // Random initial angle. TAU is 2*PI
                    noise_offset_x: rng.random_range(0.0..1000.0),       // Large, distinct offsets
                    noise_offset_y: rng.random_range(0.0..1000.0),
                    visibility: (rng.random_range(0.9..1.0_f32) / scale / 1.3)
                        .powi(2)
                        .clamp(0.3, 2.0)
                        * visibility_boost,
                    time_alive: 0.0,
                    despawn: false,
                    life: 1.0 + rng.random_range(0.0..0.5),
                    vel_speed: rng.random_range(0.2..1.0_f32).powi(2),
                    direction: miasma.velocity_field[bpos.ndidx()],
                })
                .insert(Transform::from_scale(Vec3::new(scale, scale, 1.0)))
                .insert(LightSensitive {
                    exposure_factor: 0.5,
                    bias: 0.6,
                })
                .insert(pos)
                .insert(GameSprite)
                .insert(SpriteLayer(-0.0002 + rng.random_range(-0.0001..0.0004)));
            *pos_count += 1;
        }
    }
    measure.end_ms();
}

pub(crate) fn animate_miasma_sprites(
    time: Res<Time>,
    board_data: Res<BoardTopology>,
    bcf: Res<BoardCollisionField>,
    miasma: If<Res<MiasmaGrid>>,
    noise_table: Res<PerlinNoise>,
    mut query: Query<(&mut Position, &mut MiasmaSprite)>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    let measure = metrics::ANIMATE_MIASMA.time_measure();

    let dt = time.delta_secs();
    let quality_factor = video_settings.quality.to_quality_factor2();
    const MOVEMENT_FACTOR: f32 = 1.01;
    for (mut pos, mut miasma_sprite) in query.iter_mut() {
        miasma_sprite.time_alive += dt;
        // 1. Circular Motion:
        let angle = miasma_sprite.angular_speed * miasma_sprite.time_alive + miasma_sprite.phase;
        let circular_x = miasma_sprite.radius * angle.cos();
        let circular_y = miasma_sprite.radius * angle.sin();

        // 2. Perlin Noise Offset using precomputed values:
        let noise_x = noise_table.get(
            miasma_sprite.noise_offset_x + miasma_sprite.time_alive * 0.2,
            miasma_sprite.noise_offset_y,
        );
        let noise_y = noise_table.get(
            miasma_sprite.noise_offset_x,
            miasma_sprite.noise_offset_y + miasma_sprite.time_alive * 0.2,
        );

        // 3. Combine and Update Position:
        pos.x = miasma_sprite.base_position.x + (circular_x + noise_x * 0.6) * MOVEMENT_FACTOR; // Scale noise influence
        pos.y = miasma_sprite.base_position.y + (circular_y + noise_y * 0.6) * MOVEMENT_FACTOR;

        // We do *not* modify pos.z or pos.visual_priority here.  The Z position is set
        // during initialization and should remain constant.
        let bpos = pos.to_board_position();
        let vel = if quality_factor > 0.4 {
            let mut total_vel = Vec2::ZERO;
            let mut total_w = 0.0001;
            for bpos in bpos.iter_xy_neighbors(1, board_data.map_size) {
                if !bcf.0[bpos.ndidx()].player_free {
                    continue;
                }
                let w = (bpos.to_position().distance2(&pos) + 0.1).recip();
                let vel = miasma.velocity_field[bpos.ndidx()];
                total_vel += vel * w;
                total_w += w;
            }
            total_vel / total_w
        } else {
            miasma
                .velocity_field
                .get(bpos.ndidx())
                .copied()
                .unwrap_or_default()
        };
        const F: f32 = 0.1;
        miasma_sprite.direction /= 1.01;
        miasma_sprite.direction = miasma_sprite.direction * (1.0 - F) + vel * F;

        let vel = miasma_sprite.direction;

        let vel_len = (vel.length() + 0.00001) * 5.0;
        let vel = vel * (vel_len.sqrt() / vel_len);
        const SPEED: f32 = 10.9;
        miasma_sprite.base_position.x += vel.x * dt * SPEED * miasma_sprite.vel_speed;
        miasma_sprite.base_position.y += vel.y * dt * SPEED * miasma_sprite.vel_speed;
        let bpos = miasma_sprite.base_position.to_board_position();
        if !bcf
            .0
            .get(bpos.ndidx())
            .map(|collision| collision.player_free)
            .unwrap_or_default()
        {
            let oc_pos = bpos.to_position_center();
            let delta = miasma_sprite.base_position.delta(oc_pos);
            let new_pos = delta.normalized().add_to_position(&oc_pos);
            miasma_sprite.base_position = new_pos;
        }
    }
    measure.end_ms();
}

pub(crate) fn update_miasma(
    board_data: Res<BoardTopology>,
    bcf: If<Res<BoardCollisionField>>,
    mut miasma: If<ResMut<MiasmaGrid>>,
    miasma_config: Res<MiasmaConfig>,
    time: Res<Time>,
    room_topology: Res<RoomTopology>,
    q_player: Query<&Position, With<MainPlayer>>,
    fluid_emitter_query: Query<&FluidEmitter>,
    mut room_present: Local<Array3<bool>>,
    video_settings: Res<Persistent<VideoSettings>>,
) {
    let measure = metrics::UPDATE_MIASMA.time_measure();

    let mut rng = random_seed::rng();
    let quality_factor = video_settings.quality.to_quality_factor2();
    let max_chunks = (8.0 * quality_factor).max(1.0) as usize;

    let dt = time.delta_secs();
    let emitters_remain = !fluid_emitter_query.is_empty();
    let diffusion_rate = if emitters_remain {
        miasma_config.diffusion_rate
    } else {
        miasma_config.diffusion_rate * 20.0
    };
    const EXCHANGE_VEL_SCALE: f32 = 2.0;
    let mut pressure_changes = Array3::from_elem(board_data.map_size, 0.0);
    let mut velocity_changes = Array3::from_elem(board_data.map_size, Vec2::ZERO);

    if room_present.dim() != board_data.map_size {
        // FIXME: This will introduce a bug, if the player loads a new map with exact same size, it will not update.
        *room_present = Array3::from_elem(board_data.map_size, false);
        for bpos in room_topology.room_tiles.keys() {
            let p = bpos.ndidx();
            room_present[p] = true;
        }
    }

    // Find the active player's position
    let Ok(player_pos) = q_player.single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    // Iterate through chunks
    let chunks = ChunkIterator::new(board_data.map_size)
        .filter(|chunk| {
            let dist = player_bpos.distance_to_chunk(chunk);
            let r = rng.random_range(1.0..3.0);
            dist < (r * r * r) as i64
        })
        .take(max_chunks)
        .collect::<Vec<_>>();

    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            // Check for walls and closed doors (collision)
            // Allow miasma to spread through half-walls (like repellent particles)
            let collision = &bcf.0.0[p];
            if !collision.player_free && !collision.see_through {
                continue; // Skip full walls that block both movement and sight
            }
            let p1 = miasma.pressure_field[p];
            let bpos = BoardPosition::from_ndidx(p);
            let is_room = room_present[p];
            // let player_presence =
            //     (256 / (1 + bpos.distance_taxicab(&player_bpos).clamp(0, 64))).clamp(0, 255) as u8;
            // arr_j += 1;
            // arr_j %= arr.len();
            // if arr[arr_j] > player_presence {
            //     continue;
            // }

            // Process each neighbor (up, down, left, right):
            let mut neighbors = vec![bpos.top(), bpos.bottom(), bpos.left(), bpos.right()];

            // Add stair connections for very strong miasma transmission
            let cp = &bcf.0.0[p];
            if cp.stair_offset != 0 {
                let stair_target_z = bpos.z + cp.stair_offset as i64;
                if stair_target_z >= 0 && stair_target_z < board_data.map_size.2 as i64 {
                    let stair_neighbor = BoardPosition {
                        x: bpos.x,
                        y: bpos.y,
                        z: stair_target_z,
                    };
                    // Add stair neighbor - we'll handle the extra strength in the exchange calculation
                    neighbors.push(stair_neighbor);
                }
            }

            let neighbors = neighbors
                .into_iter()
                .filter(|nb_pos| {
                    let n_idx = nb_pos.ndidx();
                    bcf.0
                        .0
                        .get(n_idx)
                        .map(|collision| {
                            // Allow miasma to spread through half-walls (like repellent particles)
                            collision.player_free || collision.see_through
                        })
                        .unwrap_or(true)
                })
                .collect::<Vec<_>>();
            let nb_len = neighbors.len() as f32 + 0.01;
            let mut total_v = Vec2::ZERO;
            for neighbor_pos in neighbors {
                if bcf.0.0.get(neighbor_pos.ndidx()).is_none() {
                    continue;
                }
                let np = neighbor_pos.ndidx();
                // Get the neighbor's pressure (treat out-of-bounds as 0.0)
                let mut p2 = miasma.pressure_field.get(np).copied().unwrap_or(0.0);
                let mut v2 = miasma.velocity_field.get(np).copied().unwrap_or(Vec2::ZERO);
                let is_room_nb = room_present[np];
                if !is_room_nb {
                    // Consider outside to be zero pressure and velocity.
                    p2 = 0.0;
                    v2 = Vec2::ZERO;
                }

                total_v += v2;
                // Calculate pressure difference and exchange amount.
                let delta_pressure = p1 - p2;
                let max_exchange_outwards = p1.max(-p2) / 4.0; // Max positive exchange
                let max_exchange_inwards = (-p2).min(p1) / 4.0; // Max negative exchange

                let mut exchange = delta_pressure * diffusion_rate * dt / nb_len;

                // Check if this is a stair connection for super strong miasma flow
                let is_stair_connection = neighbor_pos.z != bpos.z;
                if is_stair_connection {
                    // Miasma flows extremely strongly through stairs - like air
                    // Use 100x stronger diffusion for stairs (increased from 50x)
                    exchange = delta_pressure * diffusion_rate * dt * 100.0 / nb_len;
                } else if !is_room {
                    // Diffuse slower outside of rooms (only for non-stair connections)
                    exchange /= 10.0;
                }

                // --- Biased Diffusion ---
                let velocity = *miasma.velocity_field.get(p).unwrap_or(&Vec2::ZERO);
                // Skip velocity adjustments for stair connections since they're vertical
                if !is_stair_connection {
                    // Adjust exchange based on velocity components
                    if neighbor_pos == bpos.top() {
                        exchange -= velocity.y * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.bottom() {
                        exchange += velocity.y * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.left() {
                        exchange -= velocity.x * EXCHANGE_VEL_SCALE;
                    } else if neighbor_pos == bpos.right() {
                        exchange += velocity.x * EXCHANGE_VEL_SCALE;
                    }
                }

                exchange = exchange.clamp(max_exchange_inwards, max_exchange_outwards);

                pressure_changes[p] -= exchange;
                pressure_changes[np] += exchange;
            }
            velocity_changes[p] = total_v / nb_len;
        }
    }

    // Average velocities over space
    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            let vel = velocity_changes[p];
            let Some(entry) = miasma.velocity_field.get_mut(p) else {
                continue;
            };
            let f = 0.0001;
            *entry = *entry * (1.0 - f) + vel * f;

            // let is_room = room_present[p] && bcf.0[p].player_free;
            // if !is_room {
            //     // Slow particles that aren't in a room.
            //     *entry /= 1.00001;
            // }
        }
    }
    for chunk in &chunks {
        // Iterate through all cells in the pressure field within the chunk.
        for p in CellIterator::new(chunk) {
            let delta = pressure_changes[p];
            let collision = &bcf.0.0[p];
            let is_room = room_present[p] && (collision.player_free || collision.see_through);

            let Some(entry) = miasma.pressure_field.get_mut(p) else {
                continue;
            };
            *entry += delta;
            if !is_room {
                // Evaporate miasma fast when outside.
                *entry /= 1.00001;
            }
            if !emitters_remain {
                // Once every ghost is expelled, evaporate the miasma.
                *entry /= 1.001;
            }
        }
    }

    // --- 2. Velocity Calculation and Inertia ---
    let mut new_velocities = miasma.velocity_field.clone();
    for chunk in &chunks {
        for p in CellIterator::new(chunk) {
            let p_center = miasma.pressure_field[p];
            let bpos = BoardPosition::from_ndidx(p);
            let is_room = room_present[p];
            if !is_room {
                // Don't compute velocity outside of rooms.
                // Slow particles that aren't in a room.
                new_velocities[p] /= 1.00001;
                continue;
            }
            // let player_presence = (256
            //     / (1 + (bpos.distance_taxicab(&player_bpos) / 8).clamp(0, 64)))
            // .clamp(0, 255) as u8;
            // arr_j += 1;
            // arr_j %= arr.len();
            // if arr[arr_j] > player_presence {
            //     continue;
            // }

            let get_pressure = |pos: &BoardPosition| -> f32 {
                let gp = pos.ndidx();
                let is_room = room_present[gp];
                if !is_room {
                    // Consider outside to be zero pressure always.
                    return 0.0;
                }
                let collision = &bcf.0.0[gp];
                if collision.player_free || collision.see_through {
                    // Allow pressure reading from half-walls (like repellent particles)
                    miasma.pressure_field[gp]
                } else {
                    p_center
                }
            };

            let p_left = get_pressure(&bpos.left());
            let p_right = get_pressure(&bpos.right());
            let p_top = get_pressure(&bpos.top());
            let p_bottom = get_pressure(&bpos.bottom());

            let calculated_velocity = Vec2::new(
                (p_left - p_right) * miasma_config.velocity_scale,
                (p_top - p_bottom) * miasma_config.velocity_scale,
            );
            let calc_vel_len = calculated_velocity.length() + 0.000001;
            let adjusted_vel = f32::cbrt(calc_vel_len).min(1.0);
            let calculated_velocity = calculated_velocity * (adjusted_vel / calc_vel_len); // .min(calculated_velocity);
            let previous_velocity = miasma.velocity_field[p];

            // FIXME: This should be proportional change of dt
            let mut new_velocity = (previous_velocity * miasma_config.inertia_factor
                + calculated_velocity)
                / (1.0 + miasma_config.inertia_factor + miasma_config.friction);

            // Take walls into account.
            const WALL_REPEL_SPEED: f32 = 0.00;
            let old_speed = new_velocity.length();
            if new_velocity.x > -WALL_REPEL_SPEED
                && !bcf
                    .0
                    .0
                    .get(bpos.right().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.x = -WALL_REPEL_SPEED;
            }
            if new_velocity.x < WALL_REPEL_SPEED
                && !bcf
                    .0
                    .0
                    .get(bpos.left().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.x = WALL_REPEL_SPEED;
            }
            if new_velocity.y < WALL_REPEL_SPEED
                && !bcf
                    .0
                    .0
                    .get(bpos.top().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.y = WALL_REPEL_SPEED;
            }
            if new_velocity.y > -WALL_REPEL_SPEED
                && !bcf
                    .0
                    .0
                    .get(bpos.bottom().ndidx())
                    .map(|c| c.player_free)
                    .unwrap_or(true)
            {
                new_velocity.y = -WALL_REPEL_SPEED;
            }
            new_velocity = new_velocity.normalize_or_zero() * old_speed;
            new_velocities[p] = new_velocity; // Store calculated velocity
        }
    }

    // --- 3. Apply New Velocities ---
    miasma.velocity_field = new_velocities;

    measure.end_ms();
}
