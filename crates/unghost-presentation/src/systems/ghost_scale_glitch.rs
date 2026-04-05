use bevy::prelude::*;
use rand::RngExt;
use unboard_core::entity::ResolutionFactor;
use uncommon_app_core::random_seed;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unmission_core::types::SimulationState;

/// Apply a visual "glitch" effect to ghosts by modifying their Transform scale.
/// This creates a visual indication of ghost instability without affecting position or movement.
/// Only runs on ghosts that are not currently dying.
fn ghost_scale_glitch_system(
    time: Res<Time>,
    mut q_ghost: Query<
        (&GhostSprite, &mut Transform, Option<&ResolutionFactor>),
        Without<GhostDeathSignal>,
    >,
) {
    let mut rng = random_seed::rng();
    let dt = time.delta_secs();

    for (ghost, mut transform, rf) in q_ghost.iter_mut() {
        let base_scale = rf.map(|r| r.ratio()).unwrap_or(1.0);
        let base_vec = Vec3::new(base_scale, base_scale, base_scale);
        if ghost.repellent_hits_delta > 0.0 {
            let glitch_intensity = ghost.repellent_hits_delta.clamp(0.0, 1.0);
            let scale_x = base_scale + rng.random_range(-glitch_intensity..glitch_intensity) * 0.4
                - glitch_intensity * 0.1;
            let scale_y = base_scale + rng.random_range(-glitch_intensity..glitch_intensity) * 0.4;
            let scale_z = base_scale;
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.5);
        } else if ghost.repellent_misses_delta > 0.0 {
            let glitch_intensity = ghost.repellent_misses_delta.clamp(0.0, 1.0);
            let scale_x = base_scale + glitch_intensity * 0.075;
            let scale_y = base_scale + glitch_intensity * 0.05;
            let scale_z = base_scale;
            transform.scale = transform
                .scale
                .lerp(Vec3::new(scale_x, scale_y, scale_z), dt * 0.2);
        } else if transform.scale != base_vec {
            transform.scale = transform.scale.lerp(base_vec, dt * 0.5);
            if (transform.scale - base_vec).length() < 0.01 {
                transform.scale = base_vec;
            }
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        ghost_scale_glitch_system
            .run_if(in_state(SimulationState::Ready))
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>),
    );
}
