use std::f32::consts::PI;

use bevy::prelude::*;
use uncore::components::focus_ring::FocusRing;
use uncore::components::ghost_breach::GhostBreach;
use uncore::components::ghost_sprite::GhostSprite;
use unwalkiecore::WalkieTalkingEvent;
use unwalkiecore::events::WalkieEvent;

/// System that listens for WalkieTalkingEvent and activates the focus ring for ghosts and breaches
/// when the GhostShowcase or BreachShowcase events are triggered.
pub fn focus_ring_showcase_system(
    mut ev_walkie_talking: EventReader<WalkieTalkingEvent>,
    mut query_ghost_focus_rings: Query<&mut FocusRing, With<Parent>>,
    query_ghosts: Query<(Entity, &Children), With<GhostSprite>>,
    query_breaches: Query<(Entity, &Children), With<GhostBreach>>,
) {
    for event in ev_walkie_talking.read() {
        match event.event {
            WalkieEvent::GhostShowcase => {
                // Set pulse timer on ghost focus ring
                for (_ghost_entity, children) in query_ghosts.iter() {
                    for child in children.iter() {
                        if let Ok(mut focus_ring) = query_ghost_focus_rings.get_mut(*child) {
                            focus_ring.pulse_timer = 30.0;
                        }
                    }
                }
            }
            WalkieEvent::BreachShowcase => {
                // Set pulse timer on breach focus ring
                for (_breach_entity, children) in query_breaches.iter() {
                    for child in children.iter() {
                        if let Ok(mut focus_ring) = query_ghost_focus_rings.get_mut(*child) {
                            focus_ring.pulse_timer = 30.0;
                        }
                    }
                }
            }
            _ => {}
        }
    }
}

/// System that updates focus rings, decreasing the pulse timer over time
/// and updating the visual appearance based on the pulse value.
pub fn update_focus_rings(
    time: Res<Time>,
    mut query_focus_rings: Query<(&mut FocusRing, &mut Sprite)>,
) {
    for (mut focus_ring, mut sprite) in query_focus_rings.iter_mut() {
        if focus_ring.pulse_timer > 0.0 {
            // Decrease the pulse timer
            focus_ring.pulse_timer -= time.delta_secs();
            focus_ring.pulse_timer = focus_ring.pulse_timer.max(0.0);

            // Calculate the alpha value based on the pulse timer and a sin wave
            // This will create a pulsing effect
            let base_alpha = ((focus_ring.pulse_timer / 30.0).powi(3) * PI)
                .sin()
                .clamp(0.0, 1.0);
            let pulse = (time.elapsed_secs() * 2.0).sin() * 0.1 + 0.9;
            let alpha = base_alpha * pulse;

            // Update the sprite alpha
            sprite.color.set_alpha(alpha.clamp(0.0, 0.5));
        } else {
            // If the pulse timer is 0, make the focus ring invisible
            sprite.color.set_alpha(0.0);
        }
    }
}

/// Registers the focus ring systems to the Bevy app.
pub fn app_setup(app: &mut App) {
    app.add_systems(Update, (focus_ring_showcase_system, update_focus_rings));
}
