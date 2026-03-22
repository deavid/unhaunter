use crate::metrics;
use bevy::prelude::*;
use rand::prelude::*;
use std::mem::swap;
use unboard_core::resources::roomdb::RoomTopology;
use unfoundation_core::random_seed;
use unmapload_core::events::loadlevel::MapGeometryInitializedEvent;
use unmetrics_core::metrics::SendMetric;
use unsoundfield_core::resources::SoundGrid;
use unspatial_core::position::Position;

pub fn reset_sound_grid(mut commands: Commands) {
    commands.remove_resource::<SoundGrid>();
}

/// Diffuses the sound field across the board by propagating vectors from their
/// current positions to adjacent tiles within the room topology.
/// Ghost-specific pulse generation is handled by `unghost-plugin`.
pub fn sound_field_propagate(
    mut sound_grid: If<ResMut<SoundGrid>>,
    room_topology: Res<RoomTopology>,
) {
    let measure = metrics::SOUND_UPDATE.time_measure();
    let mut rng = random_seed::rng();
    let map_pos = sound_grid.sound_field.keys().cloned().collect::<Vec<_>>();
    for mpos in map_pos.into_iter() {
        let Some(s_v) = sound_grid.sound_field.get_mut(&mpos) else {
            continue;
        };
        let mut data: Vec<Vec2> = vec![];
        swap(s_v, &mut data);
        if data.is_empty() {
            continue;
        }
        let v: Vec2 = data.into_iter().sum();
        let sz = (v.length() * 2.0).ceil() as usize;
        for _ in 0..sz {
            let mut v = v;
            v.x += rng.random_range(-0.05..0.05) + rng.random_range(-0.05..0.05);
            v.y += rng.random_range(-0.05..0.05) + rng.random_range(-0.05..0.05);
            v /= 1.05 * sz as f32;
            let mut v1 = v.normalize() * 1.1;
            v1 *= rng.random_range(0.1..1.0);
            v1.x += rng.random_range(-0.3..0.3) + rng.random_range(-0.3..0.3);
            v1.y += rng.random_range(-0.3..0.3) + rng.random_range(-0.3..0.3);
            let n_p = Position {
                x: mpos.x as f32 + v1.x,
                y: mpos.y as f32 + v1.y,
                z: mpos.z as f32,
                visual_priority: 0.0,
            };
            let bn_p = n_p.to_board_position();
            if room_topology.room_tiles.get(&bn_p).is_some() && v.length() > 0.00002 {
                sound_grid.sound_field.entry(bn_p).or_default().push(v);
            }
        }
    }

    measure.end_ms();
}

pub fn init_sound_grid(mut commands: Commands, mut ev: MessageReader<MapGeometryInitializedEvent>) {
    for _ in ev.read() {
        commands.insert_resource(SoundGrid::default());
    }
}
