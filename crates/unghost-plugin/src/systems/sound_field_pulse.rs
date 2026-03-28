use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use rand::prelude::*;
use uncommon_app_core::random_seed;
use unreplicon_core::messages::GhostSoundFieldBroadcast;
use unreplicon_core::resources::AuthorityRole;
use unsoundfield_core::components::SoundFieldSource;
use unsoundfield_core::resources::SoundGrid;
use unspatial_core::position::Position;

/// Authority-side system: once in a while (~1/30 frames), each ghost/breach entity
/// with a `SoundFieldSource` pulses its position into the local `SoundGrid` and
/// broadcasts the event to pure clients.
pub(crate) fn ghost_sound_field_pulse(
    mut sound_grid: If<ResMut<SoundGrid>>,
    qe: Query<(&SoundFieldSource, &Position)>,
    is_authority: Option<Res<AuthorityRole>>,
    mut ev_broadcast: MessageWriter<ToClients<GhostSoundFieldBroadcast>>,
) {
    if is_authority.is_none() {
        return;
    }
    let mut rng = random_seed::rng();
    let gn = rng.random_range(0..30_u32);
    if gn != 0 {
        return;
    }
    for (_, pos) in qe.iter() {
        let bpos = pos.to_board_position();

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

        ev_broadcast.write(ToClients {
            mode: SendMode::Broadcast,
            message: GhostSoundFieldBroadcast {
                position: [pos.x, pos.y, pos.z],
            },
        });
    }
}

/// Pure-client system: receives `GhostSoundFieldBroadcast` from the server and
/// injects the sound-field pulse into the local `SoundGrid`.
///
/// Must NOT run on the Authority (offline or host): `ghost_sound_field_pulse`
/// already applies the pulse locally on those instances.
/// Registered with `.run_if(is_pure_client)`.
///
/// Note: The 16-iteration random-vector loop is re-run with local RNG, so the
/// exact vector magnitudes differ between server and client. The trigger timing
/// is synchronized; the per-vector randomness is intentionally not synchronized.
pub(crate) fn handle_ghost_sound_field_broadcast(
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

pub(crate) fn app_setup(app: &mut bevy::prelude::App) {
    use unreplicon_core::resources::is_pure_client;
    app.add_systems(bevy::prelude::Update, ghost_sound_field_pulse);
    app.add_systems(
        bevy::prelude::Update,
        handle_ghost_sound_field_broadcast.run_if(is_pure_client),
    );
}
