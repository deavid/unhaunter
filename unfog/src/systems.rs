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

    let mut rng = rand::rng();
    let collision_field = board_data.collision_field.clone();

    for (board_position, cfield) in &collision_field {
        let opt_room_id = roomdb.room_tiles.get(board_position);

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
            config.initial_room_pressure * modifier,
        );
    }
}

pub fn spawn_miasma(
    vf: Res<VisibilityData>,
    gc: Res<GameConfig>,
    q_miasma: Query<(Entity, &MiasmaSprite)>,
    qp: Query<(&Position, &PlayerSprite)>,
    handles: Res<GameAssets>,
    mut commands: Commands,
) {
    const THRESHOLD: f32 = 0.00001;
    const DIST_FACTOR: f32 = 0.0001;
    let mut rng = rand::rng();

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
    for (entity, miasma_sprite) in q_miasma.iter() {
        let bpos = miasma_sprite.base_position.to_board_position();
        let player_dst2 = player_pos.distance2(&miasma_sprite.base_position);

        let vis =
            vf.visibility_field.get(&bpos).copied().unwrap_or_default() + DIST_FACTOR / player_dst2;
        let pos_count = count.entry(bpos).or_default();
        if vis < THRESHOLD || *pos_count > 15 {
            commands.entity(entity).despawn_recursive();
        } else {
            *pos_count += 1;
        }
    }

    for (bpos, vis) in vf.visibility_field.iter() {
        let player_dst2 = player_pos.distance2(&bpos.to_position_center());
        let vis = vis + DIST_FACTOR / player_dst2;
        if vis < THRESHOLD * 2.0 {
            continue;
        }
        let pos_count = count.entry(bpos.clone()).or_default();
        if *pos_count < 6 {
            // Spawn miasma if too low
            let scale = rng.random_range(0.5..1.3_f32).cbrt();
            let pos = bpos
                .to_position_center()
                .with_global_z(0.00045)
                .with_random(0.4);

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
                    angular_speed: rng.random_range(0.1..1.0), // Slow speed
                    phase: rng.random_range(0.0..std::f32::consts::TAU), // Random initial angle. TAU is 2*PI
                    noise_offset_x: rng.random_range(0.0..1000.0),       // Large, distinct offsets
                    noise_offset_y: rng.random_range(0.0..1000.0),
                    visibility: rng.random_range(0.3..1.0_f32).powi(3),
                    life: 0.0,
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
    mut query: Query<(&mut Position, &mut MiasmaSprite)>,
) {
    let dt = time.delta_secs();
    let perlin = Perlin::new(1); // 1 is just the seed.

    for (mut pos, mut miasma_sprite) in query.iter_mut() {
        miasma_sprite.life += dt;
        // 1. Circular Motion:
        let angle = miasma_sprite.angular_speed * miasma_sprite.life + miasma_sprite.phase;
        let circular_x = miasma_sprite.radius * angle.cos();
        let circular_y = miasma_sprite.radius * angle.sin();

        // 2. Perlin Noise Offset:
        let noise_x = perlin.get([
            (miasma_sprite.noise_offset_x + miasma_sprite.life * 0.3) as f64, // Slow change over time
            miasma_sprite.noise_offset_y as f64,
        ]) as f32;
        let noise_y = perlin.get([
            miasma_sprite.noise_offset_x as f64,
            (miasma_sprite.noise_offset_y + miasma_sprite.life * 0.3) as f64, // Different offset for Y
        ]) as f32;

        // 3. Combine and Update Position:
        pos.x = miasma_sprite.base_position.x + circular_x + noise_x * 0.4; // Scale noise influence
        pos.y = miasma_sprite.base_position.y + circular_y + noise_y * 0.4;

        // We do *not* modify pos.z or pos.global_z here.  The Z position is set
        // during initialization and should remain constant.
    }
}
