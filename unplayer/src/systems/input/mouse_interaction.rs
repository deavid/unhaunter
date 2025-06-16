use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use uncore::{
    behavior::{Behavior, component::Interactive},
    components::{board::position::Position, player_sprite::PlayerSprite},
    resources::looking_gear::LookingGear,
};
use ungear::components::playergear::PlayerGear;
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;

/// System that handles right-click to activate right-hand gear.
///
/// This system listens for right-click events and triggers the right-hand gear
/// similar to the R-key functionality.
pub(crate) fn mouse_right_click_gear_system(
    mouse: Res<ButtonInput<MouseButton>>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut gs: GearStuff,
) {
    // Check for right-click (just pressed to match keyboard behavior)
    if mouse.just_pressed(MouseButton::Right) {
        for (_, mut playergear) in q_gear.iter_mut() {
            // Trigger the right-hand gear, same as R-key
            playergear.right_hand.set_trigger(&mut gs);
        }
    }
}

/// System that handles mouse scrolling to cycle through gear.
///
/// This system allows the player to use the mouse scroll wheel to cycle
/// through their gear items, similar to the Q key functionality.
/// Scrolling up (positive Y) cycles forward (same as Q key).
/// Scrolling down (negative Y) cycles backward (reverse direction).
pub(crate) fn mouse_scroll_gear_system(
    mut scroll_events: EventReader<MouseWheel>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    looking_gear: Res<LookingGear>,
) {
    for event in scroll_events.read() {
        // Determine scroll direction and whether to cycle
        let (should_cycle, cycle_forward) = match event.unit {
            MouseScrollUnit::Line => {
                if event.y > 0.0 {
                    (true, true) // Scroll up = forward
                } else if event.y < 0.0 {
                    (true, false) // Scroll down = backward
                } else {
                    (false, true) // No scroll
                }
            }
            MouseScrollUnit::Pixel => {
                if event.y > 2.0 {
                    (true, true) // Scroll up = forward
                } else if event.y < -2.0 {
                    (true, false) // Scroll down = backward
                } else {
                    (false, true) // Insufficient scroll
                }
            }
        };

        if should_cycle {
            for (_, mut playergear) in q_gear.iter_mut() {
                if cycle_forward {
                    // Cycle forward (same as Q key)
                    playergear.cycle(&looking_gear.hand());
                } else {
                    // Cycle backward (reverse direction)
                    playergear.cycle_reverse(&looking_gear.hand());
                }
            }
        }
    }
}

/// System that tracks mouse hover over interactive entities.
///
/// This system updates the `HoverState` component of interactive entities
/// when the mouse hovers over them, and resets the state when the mouse leaves.
pub(crate) fn mouse_hover_interactive_system(
    mut q_interactives: Query<(Entity, &Position, &mut Interactive, &Behavior)>,
    mut hover_events: EventReader<Pointer<Over>>,
    mut exit_events: EventReader<Pointer<Out>>,
) {
    for over_event in hover_events.read() {
        if let Ok((entity, _position, mut interactive, _behavior)) =
            q_interactives.get_mut(over_event.target)
        {
            // Mouse entered an interactive entity
            info!(
                "mouse_hover_interactive_system: Mouse entered interactive entity {:?}",
                entity
            );
            interactive.hovered = true;
        }
    }

    for exit_event in exit_events.read() {
        if let Ok((entity, _position, mut interactive, _behavior)) =
            q_interactives.get_mut(exit_event.target)
        {
            // Mouse exited an interactive entity
            info!(
                "mouse_hover_interactive_system: Mouse exited interactive entity {:?}",
                entity
            );

            interactive.hovered = false;
        }
    }
}
