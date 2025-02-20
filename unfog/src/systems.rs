use bevy::prelude::*;
use bevy::sprite::Anchor;
use bevy::utils::HashMap;
use noise::{NoiseFn, Perlin};
use rand::Rng;
use uncore::behavior::Behavior;
use uncore::components::board::boardposition::BoardPosition;
use uncore::components::board::position::Position;
use uncore::components::game::GameSprite;
use uncore::components::game_config::GameConfig;
use uncore::components::player_sprite::PlayerSprite;
use uncore::components::sprite_type::SpriteType;
use uncore::events::loadlevel::LevelReadyEvent;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::resources::visibility_data::VisibilityData;
use uncore::types::root::game_assets::GameAssets;
use unstd::plugins::board::rebuild_collision_data;

use crate::components::MiasmaSprite;
use crate::resources::MiasmaConfig;

pub fn initialize_miasma(
    mut board_data: ResMut<BoardData>,
    roomdb: Res<RoomDB>,
    config: Res<MiasmaConfig>,
    mut level_ready: EventReader<LevelReadyEvent>,
    qt: Query<(&Position, &Behavior)>,
) {
    // Only run on LevelLoadedEvent
    if level_ready.read().next().is_none() {
        return;
    }
    rebuild_collision_data(&mut board_data, &qt);
    board_data.miasma.pressure_field.clear();
    board_data.miasma.room_modifiers.clear();
    board_data.miasma.velocity_field.clear();

    let mut rng = rand::rng();
    let collision_field = board_data.collision_field.clone();

    for (p, cfield) in collision_field.indexed_iter() {
        let board_position = BoardPosition::from_ndidx(p);
        let opt_room_id = roomdb.room_tiles.get(&board_position);

        // 1. Get or Insert Room Modifier:
        let mut modifier = if let Some(room_id) = opt_room_id {
            *board_data
                .miasma
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
        board_data.miasma.pressure_field.insert(
            board_position.clone(),
            config.initial_room_pressure * modifier * rng.random_range(0.9..=1.1),
        );
    }
}

pub fn spawn_miasma(
    time: Res<Time>,
    vf: Res<VisibilityData>,
    gc: Res<GameConfig>,
    mut q_miasma: Query<(Entity, &mut MiasmaSprite)>,
    qp: Query<(&Position, &PlayerSprite)>,
    handles: Res<GameAssets>,
    board_data: Res<BoardData>,
    mut commands: Commands,
) {
    const THRESHOLD: f32 = 0.01;
    const DIST_FACTOR: f32 = 0.00001;
    const MIASMA_TARGET_SPRITE_COUNT: usize = 6;
    let mut rng = rand::rng();
    let dt = time.delta_secs();

    // Find the active player's position
    let Some(player_pos) = qp.iter().find_map(|(pos, player)| {
        if player.id == gc.player_id {
            Some(*pos)
        } else {
            None
        }
    }) else {
        return;
    };

    let mut count: HashMap<BoardPosition, usize> = HashMap::new();
    for (entity, mut miasma_sprite) in q_miasma.iter_mut() {
        if miasma_sprite.despawn {
            miasma_sprite.life -= dt * 2.0;
        }
        if miasma_sprite.life < -2.0 {
            miasma_sprite.life = -2.0;
            commands.entity(entity).despawn_recursive();
        }
        if miasma_sprite.despawn {
            continue;
        }
        miasma_sprite.life -= dt / 4.0;
        if miasma_sprite.life < 0.02 {
            miasma_sprite.despawn = true;
            continue;
        }
        let bpos = miasma_sprite.base_position.to_board_position();
        let player_dst2 = player_pos.distance2(&miasma_sprite.base_position);

        let vis =
            vf.visibility_field.get(&bpos).copied().unwrap_or_default() + DIST_FACTOR / player_dst2;
        let target_count = ((board_data
            .miasma
            .pressure_field
            .get(&bpos)
            .copied()
            .unwrap_or_default()
            / 1.1
            + 0.1)
            .min(1.0)
            * MIASMA_TARGET_SPRITE_COUNT as f32) as usize;

        let pos_count = count.entry(bpos).or_default();

        if vis < THRESHOLD || *pos_count > target_count {
            miasma_sprite.despawn = true;
        } else {
            *pos_count += 1;
        }
    }

    for (bpos, vis) in vf.visibility_field.iter() {
        if !board_data.collision_field[bpos.ndidx()].player_free {
            continue;
        }

        let player_dst2 = player_pos.distance2(&bpos.to_position_center());
        let vis = vis + DIST_FACTOR / player_dst2;
        if vis < THRESHOLD * 2.0 {
            continue;
        }
        let target_count = ((board_data
            .miasma
            .pressure_field
            .get(bpos)
            .copied()
            .unwrap_or_default()
            / 1.1
            + 0.1)
            .min(1.0)
            * MIASMA_TARGET_SPRITE_COUNT as f32) as usize;

        let pos_count = count.entry(bpos.clone()).or_default();
        if *pos_count < target_count {
            // Spawn miasma if too low
            let scale = rng.random_range(0.1..0.9_f32).cbrt();
            let pos = bpos
                .to_position_center()
                .with_global_z(0.00037 * rng.random_range(0.99..1.01))
                .with_random(1.0);

            commands
                .spawn(Sprite {
                    image: handles.images.miasma.clone(),
                    color: Color::linear_rgba(1.0, 1.0, 1.0, 0.0),
                    anchor: Anchor::Custom(handles.anchors.grid1x1),
                    ..default()
                })
                .insert(MiasmaSprite {
                    base_position: pos,
                    radius: rng.random_range(0.15..0.45), // Small radius
                    angular_speed: rng.random_range(0.05..0.5), // Slow speed
                    phase: rng.random_range(0.0..std::f32::consts::TAU), // Random initial angle. TAU is 2*PI
                    noise_offset_x: rng.random_range(0.0..1000.0),       // Large, distinct offsets
                    noise_offset_y: rng.random_range(0.0..1000.0),
                    visibility: rng.random_range(0.3..1.0_f32).powi(3),
                    time_alive: 0.0,
                    despawn: false,
                    life: 1.0 + rng.random_range(0.0..0.5),
                    vel_speed: rng.random_range(0.2..1.0_f32).powi(2),
                    direction: board_data
                        .miasma
                        .velocity_field
                        .get(bpos)
                        .copied()
                        .unwrap_or_default(),
                })
                .insert(Transform::from_scale(Vec3::new(scale, scale, 1.0)))
                .insert(SpriteType::Miasma)
                .insert(pos)
                .insert(GameSprite);
            *pos_count += 1;
        }
    }
}

pub fn animate_miasma_sprites(
    time: Res<Time>,
    board_data: Res<BoardData>,
    mut query: Query<(&mut Position, &mut MiasmaSprite)>,
) {
    let dt = time.delta_secs();
    let perlin = Perlin::new(1); // 1 is just the seed.
    const MOVEMENT_FACTOR: f32 = 1.01;
    for (mut pos, mut miasma_sprite) in query.iter_mut() {
        miasma_sprite.time_alive += dt;
        // 1. Circular Motion:
        let angle = miasma_sprite.angular_speed * miasma_sprite.time_alive + miasma_sprite.phase;
        let circular_x = miasma_sprite.radius * angle.cos();
        let circular_y = miasma_sprite.radius * angle.sin();

        // 2. Perlin Noise Offset:
        let noise_x = perlin.get([
            (miasma_sprite.noise_offset_x + miasma_sprite.time_alive * 0.2) as f64, // Slow change over time
            miasma_sprite.noise_offset_y as f64,
        ]) as f32;
        let noise_y = perlin.get([
            miasma_sprite.noise_offset_x as f64,
            (miasma_sprite.noise_offset_y + miasma_sprite.time_alive * 0.2) as f64, // Different offset for Y
        ]) as f32;

        // 3. Combine and Update Position:
        pos.x = miasma_sprite.base_position.x + (circular_x + noise_x * 0.6) * MOVEMENT_FACTOR; // Scale noise influence
        pos.y = miasma_sprite.base_position.y + (circular_y + noise_y * 0.6) * MOVEMENT_FACTOR;

        // We do *not* modify pos.z or pos.global_z here.  The Z position is set
        // during initialization and should remain constant.
        let bpos = pos.to_board_position();
        let mut total_vel = Vec2::ZERO;
        let mut total_w = 0.0001;
        for bpos in bpos.xy_neighbors(1) {
            if !board_data.collision_field[bpos.ndidx()].player_free {
                continue;
            }
            let w = (bpos.to_position().distance2(&pos) + 0.1).recip();
            let vel = board_data
                .miasma
                .velocity_field
                .get(&bpos)
                .cloned()
                .unwrap_or_default();
            total_vel += vel * w;
            total_w += w;
        }
        let vel = total_vel / total_w;
        const F: f32 = 0.1;
        miasma_sprite.direction /= 1.01;
        miasma_sprite.direction = miasma_sprite.direction * (1.0 - F) + vel * F;

        let vel = miasma_sprite.direction;

        let vel_len = (vel.length() + 0.00001) * 5.0;
        let vel = vel * (vel_len.sqrt() / vel_len);
        const SPEED: f32 = 10.9;
        miasma_sprite.base_position.x += vel.x * dt * SPEED * miasma_sprite.vel_speed;
        miasma_sprite.base_position.y += vel.y * dt * SPEED * miasma_sprite.vel_speed;
    }
}

pub fn update_miasma(
    mut board_data: ResMut<BoardData>,
    miasma_config: Res<MiasmaConfig>,
    time: Res<Time>,
    roomdb: Res<RoomDB>,
) {
    let dt = time.delta_secs();
    let diffusion_rate = miasma_config.diffusion_rate;
    const EXCHANGE_VEL_SCALE: f32 = 1.0;
    let mut pressure_changes = HashMap::<BoardPosition, f32>::new();
    let mut velocity_changes = HashMap::<BoardPosition, Vec2>::new();

    // Create a copy of the keys, to allow concurrent read & write.
    let keys: Vec<BoardPosition> = board_data.miasma.pressure_field.keys().cloned().collect();

    // Iterate through all cells in the pressure field.
    for pos in keys {
        // Check for walls and closed doors (collision)
        // This part is to check we don't diffuse the walls themselves.
        if !board_data.collision_field[pos.ndidx()].player_free {
            continue; // Skip walls and out-of-bounds
        }
        let is_room = roomdb.room_tiles.contains_key(&pos);

        // Get the current cell's pressure.
        let p1 = board_data
            .miasma
            .pressure_field
            .get(&pos)
            .copied()
            .unwrap_or(0.0);

        // Process each neighbor (up, down, left, right):
        let neighbors = [pos.top(), pos.bottom(), pos.left(), pos.right()];
        let neighbors = neighbors
            .into_iter()
            .filter(|nb_pos| {
                board_data
                    .collision_field
                    .get(nb_pos.ndidx())
                    .map(|x| x.player_free)
                    .unwrap_or(true)
            })
            .collect::<Vec<_>>();
        let nb_len = neighbors.len() as f32 + 0.01;
        let mut total_v = Vec2::ZERO;
        for neighbor_pos in neighbors {
            if board_data
                .collision_field
                .get(neighbor_pos.ndidx())
                .is_none()
            {
                continue;
            }
            // Get the neighbor's pressure (treat out-of-bounds as 0.0)
            let mut p2 = board_data
                .miasma
                .pressure_field
                .get(&neighbor_pos)
                .copied()
                .unwrap_or(0.0);
            let mut v2 = board_data
                .miasma
                .velocity_field
                .get(&neighbor_pos)
                .copied()
                .unwrap_or(Vec2::ZERO);
            let is_room_nb = roomdb.room_tiles.contains_key(&neighbor_pos);
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

            // if delta_pressure.abs() > 100.0 {
            //     dbg!(p1, p2, max_exchange_outwards, max_exchange_inwards);
            // }
            let mut exchange = delta_pressure * diffusion_rate * dt / nb_len;
            if !is_room {
                // Diffuse slower outside of rooms
                exchange /= 100.0;
            }

            // --- Biased Diffusion ---
            let velocity = *board_data
                .miasma
                .velocity_field
                .get(&pos)
                .unwrap_or(&Vec2::ZERO);
            // Adjust exchange based on velocity components
            if neighbor_pos == pos.top() {
                exchange -= velocity.y * EXCHANGE_VEL_SCALE;
            } else if neighbor_pos == pos.bottom() {
                exchange += velocity.y * EXCHANGE_VEL_SCALE;
            } else if neighbor_pos == pos.left() {
                exchange -= velocity.x * EXCHANGE_VEL_SCALE;
            } else if neighbor_pos == pos.right() {
                exchange += velocity.x * EXCHANGE_VEL_SCALE;
            }

            exchange = exchange.clamp(max_exchange_inwards, max_exchange_outwards);

            *pressure_changes.entry(pos.clone()).or_insert(0.0) -= exchange;
            *pressure_changes.entry(neighbor_pos).or_insert(0.0) += exchange;
        }
        velocity_changes.insert(pos, total_v / nb_len);
    }

    // Average velocities over space
    for (pos, vel) in velocity_changes {
        let is_room = roomdb.room_tiles.contains_key(&pos)
            && board_data.collision_field[pos.ndidx()].player_free;
        let entry = board_data
            .miasma
            .velocity_field
            .entry(pos)
            .or_insert(Vec2::ZERO);
        let f = 0.9;
        *entry = *entry * (1.0 - f) + vel * f;

        if !is_room {
            // Slow particles that aren't in a room.
            *entry /= 1.0001;
        }
    }
    for (pos, delta) in pressure_changes {
        let is_room = roomdb.room_tiles.contains_key(&pos)
            && board_data.collision_field[pos.ndidx()].player_free;

        let entry = board_data.miasma.pressure_field.entry(pos).or_insert(0.0);
        *entry += delta;
        if !is_room {
            // Evaporate miasma fast when outside.
            *entry /= 1.001;
        }
        // *entry = entry.max(0.0);
    }

    // --- 2. Velocity Calculation and Inertia ---
    let mut new_velocities = HashMap::<BoardPosition, Vec2>::new();
    for (pos, &p_center) in &board_data.miasma.pressure_field {
        let is_room = roomdb.room_tiles.contains_key(pos);
        if !is_room {
            // Don't compute velocity outside of rooms.
            continue;
        }

        let get_pressure = |pos: &BoardPosition| -> f32 {
            let is_room = roomdb.room_tiles.contains_key(pos);
            if !is_room {
                // Consider outside to be zero pressure always.
                return 0.0;
            }
            if board_data.collision_field[pos.ndidx()].player_free {
                board_data
                    .miasma
                    .pressure_field
                    .get(pos)
                    .cloned()
                    .unwrap_or(0.0)
            } else {
                p_center
            }
        };

        let p_left = get_pressure(&pos.left());
        let p_right = get_pressure(&pos.right());
        let p_top = get_pressure(&pos.top());
        let p_bottom = get_pressure(&pos.bottom());

        let calculated_velocity = Vec2::new(
            (p_left - p_right) * miasma_config.velocity_scale,
            (p_top - p_bottom) * miasma_config.velocity_scale,
        );
        let calc_vel_len = calculated_velocity.length() + 0.000001;
        let adjusted_vel = calc_vel_len.cbrt().min(5.0);
        let calculated_velocity = calculated_velocity * (adjusted_vel / calc_vel_len); // .min(calculated_velocity);
        let previous_velocity = *board_data
            .miasma
            .velocity_field
            .get(pos)
            .unwrap_or(&Vec2::ZERO);

        // FIXME: This should be proportional change of `dt`
        let mut new_velocity = (previous_velocity * miasma_config.inertia_factor
            + calculated_velocity)
            / (1.0 + miasma_config.inertia_factor + miasma_config.friction);

        // Take walls into account.
        const WALL_REPEL_SPEED: f32 = 0.001;
        let old_speed = new_velocity.length();
        if new_velocity.x > -WALL_REPEL_SPEED
            && !board_data
                .collision_field
                .get(pos.right().ndidx())
                .map(|c| c.player_free)
                .unwrap_or(true)
        {
            new_velocity.x = -WALL_REPEL_SPEED;
        }
        if new_velocity.x < WALL_REPEL_SPEED
            && !board_data
                .collision_field
                .get(pos.left().ndidx())
                .map(|c| c.player_free)
                .unwrap_or(true)
        {
            new_velocity.x = WALL_REPEL_SPEED;
        }
        if new_velocity.y < WALL_REPEL_SPEED
            && !board_data
                .collision_field
                .get(pos.top().ndidx())
                .map(|c| c.player_free)
                .unwrap_or(true)
        {
            new_velocity.y = WALL_REPEL_SPEED;
        }
        if new_velocity.y > -WALL_REPEL_SPEED
            && !board_data
                .collision_field
                .get(pos.bottom().ndidx())
                .map(|c| c.player_free)
                .unwrap_or(true)
        {
            new_velocity.y = -WALL_REPEL_SPEED;
        }
        new_velocity = new_velocity.normalize_or_zero() * old_speed;
        new_velocities.insert(pos.clone(), new_velocity); // Store calculated velocity
    }

    // --- 3. Apply New Velocities ---
    board_data.miasma.velocity_field = new_velocities;
}
