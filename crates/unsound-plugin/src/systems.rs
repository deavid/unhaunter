use crate::metrics;
use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use rand::prelude::*;
use std::mem::swap;
use unboard_core::components::physics::SoundEmitter;
use unboard_core::resources::roomdb::RoomTopology;
use unfoundation_core::random_seed;
use unmapload_core::events::loadlevel::MapGeometryInitializedEvent;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::messages::GhostSoundFieldBroadcast;
use unsound_core::resources::SoundGrid;
use unspatial_core::position::Position;
use untypes_core::roles::AuthorityRole;

pub fn reset_sound_grid(mut commands: Commands) {
    commands.remove_resource::<SoundGrid>();
}

pub fn sound_update(
    mut sound_grid: If<ResMut<SoundGrid>>,
    room_topology: Res<RoomTopology>,
    qe: Query<(&SoundEmitter, &Position)>,
    is_authority: Option<Res<AuthorityRole>>,
    mut ev_broadcast: MessageWriter<ToClients<GhostSoundFieldBroadcast>>,
) {
    let measure = metrics::SOUND_UPDATE.time_measure();

    let mut rng = random_seed::rng();
    let gn = rng.random_range(0..30_u32);
    if gn == 0 && is_authority.is_some() {
        // Ghost talk once in a while — Authority only.
        // Applies locally (so the host/offline instance hears it),
        // then broadcasts to pure clients.
        for (_, pos) in qe.iter() {
            let pos: &Position = pos;
            let bpos = pos.to_board_position();

            // Apply to local SoundGrid (unchanged inner loop)
            for _ in 0..16 {
                let mut v = Vec2::new(rng.random_range(-2.0..2.0), rng.random_range(-2.0..2.0));
                let l = v.length();
                if l < 0.02 {
                    continue;
                }
                let loudness = rng.random_range(0.3_f32..2.0).powi(2);
                v *= loudness * 4.5 / l;
                let vn = v.normalize() * 1.5;
                let newbpos = Position {
                    x: (bpos.x as f32 + vn.x),
                    y: (bpos.y as f32 + vn.y),
                    z: bpos.z as f32,
                    visual_priority: 0.0,
                }
                .to_board_position();
                sound_grid.sound_field.entry(newbpos).or_default().push(v);
            }

            // Broadcast to pure clients.
            // ev_broadcast is always present (registered globally in plugin setup),
            // safe to call unconditionally here — the is_authority guard above
            // ensures this block is never entered on a pure client.
            ev_broadcast.write(ToClients {
                mode: SendMode::Broadcast,
                message: GhostSoundFieldBroadcast {
                    position: [pos.x, pos.y, pos.z],
                },
            });
        }
    }
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

/// Client-side: receive `GhostSoundFieldBroadcast` from the server and inject
/// the sound-field pulse into the local `SoundGrid`.
///
/// Must NOT run on the Authority (offline or host): `sound_update` already
/// applies the pulse locally on those instances.
/// Registered with `.run_if(is_pure_client)` in `UnhaunterSoundPlugin`.
///
/// Note: The 16-iteration random-vector loop is re-run with local RNG, so the
/// exact vector magnitudes differ between server and client. The trigger timing
/// (when the pulse occurs) is synchronized; the per-vector randomness is
/// intentionally not synchronized. This is acceptable for sanity simulation.
pub fn handle_ghost_sound_field_broadcast(
    mut reader: MessageReader<GhostSoundFieldBroadcast>,
    mut sound_grid: If<ResMut<SoundGrid>>,
) {
    for msg in reader.read() {
        let pos = Position {
            x: msg.position[0],
            y: msg.position[1],
            z: msg.position[2],
            visual_priority: 0.0,
        };
        let bpos = pos.to_board_position();
        let mut rng = random_seed::rng();
        for _ in 0..16 {
            let mut v = Vec2::new(rng.random_range(-2.0..2.0), rng.random_range(-2.0..2.0));
            let l = v.length();
            if l < 0.02 {
                continue;
            }
            let loudness = rng.random_range(0.3_f32..2.0).powi(2);
            v *= loudness * 4.5 / l;
            let vn = v.normalize() * 1.5;
            let newbpos = Position {
                x: (bpos.x as f32 + vn.x),
                y: (bpos.y as f32 + vn.y),
                z: bpos.z as f32,
                visual_priority: 0.0,
            }
            .to_board_position();
            sound_grid.sound_field.entry(newbpos).or_default().push(v);
        }
    }
}
