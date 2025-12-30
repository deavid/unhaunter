use bevy::prelude::*;
use rand::Rng;
use uncore_components::{Battery, Electronic, Toggleable};
use uncore_foundation::random_seed;
use ungear_core::gear_stuff::GearStuff;
use unspatial_core::Position;

pub fn system_electronic_interference(
    gs: GearStuff,
    mut q_electronic: Query<(&Position, &mut Electronic, &Toggleable)>,
) {
    let mut rng = random_seed::rng();
    let dt = gs.time.delta_secs();

    for (pos, mut electronic, toggle) in q_electronic.iter_mut() {
        // Decrement glitch timer if active
        if electronic.glitch_timer > 0.0 {
            electronic.glitch_timer -= dt;
        }

        // Apply EMI if warning is active and item is on
        if let Some(ghost_pos) = &gs.haunt_state.ghost_warning_position {
            let distance2 = pos.distance2(ghost_pos);
            if gs.haunt_state.ghost_warning_intensity > 0.0001 && toggle.is_on {
                // Scale effect by distance and warning level
                let effect_strength = gs.haunt_state.ghost_warning_intensity
                    * (100.0 / distance2).min(1.0)
                    * electronic.sensitivity;

                electronic.glitch_intensity = effect_strength;

                // Random glitches
                if rng.random_range(0.0..1.0) < effect_strength.powi(2) {
                    electronic.glitch_timer = rng.random_range(0.2..0.6);
                }
            } else {
                electronic.glitch_intensity = 0.0;
            }
        } else {
            electronic.glitch_intensity = 0.0;
        }
    }
}

pub fn system_battery_drain(
    time: Res<Time>,
    mut q_battery: Query<(&mut Battery, &mut Toggleable)>,
) {
    let dt = time.delta_secs();

    for (mut battery, mut toggle) in q_battery.iter_mut() {
        if toggle.is_on {
            battery.level -= battery.drain_rate * dt;
            if battery.level <= 0.0 {
                battery.level = 0.0;
                toggle.is_on = false; // Auto-shutdown
            }
        }
    }
}
