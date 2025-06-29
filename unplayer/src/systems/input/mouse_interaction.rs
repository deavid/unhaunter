use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::events::{Out, Over, Pointer},
    prelude::*,
};
use uncore::{
    behavior::{Behavior, component::Interactive},
    components::{board::position::Position, game_config::GameConfig, player_sprite::PlayerSprite},
    resources::{looking_gear::LookingGear, visibility_data::VisibilityData},
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
/// Only allows hover/click on interactive entities that are on the same floor as the player
/// and that are visible to the player (visibility > 0.1).
pub(crate) fn mouse_hover_interactive_system(
    mut q_interactives: Query<(Entity, &Position, &mut Interactive, &Behavior)>,
    q_player: Query<(&Position, &PlayerSprite)>,
    game_config: Res<GameConfig>,
    visibility_data: Res<VisibilityData>,
    mut hover_events: EventReader<Pointer<Over>>,
    mut exit_events: EventReader<Pointer<Out>>,
) {
    // Find the active player's position
    let player_pos = q_player.iter().find_map(|(pos, player)| {
        if player.id == game_config.player_id {
            Some(pos)
        } else {
            None
        }
    });

    let Some(player_pos) = player_pos else {
        return; // No active player found
    };

    let player_floor = player_pos.z.round() as i32;

    for over_event in hover_events.read() {
        if let Ok((entity, position, mut interactive, _behavior)) =
            q_interactives.get_mut(over_event.target)
        {
            let interactive_floor = position.z.round() as i32;

            // Only allow hover if the interactive is on the same floor as the player
            if interactive_floor == player_floor {
                // Check if the interactive is visible to the player
                let interactive_bpos = position.to_board_position();
                let visibility = if let Some(idx) =
                    interactive_bpos.ndidx_checked(visibility_data.visibility_field.dim())
                {
                    visibility_data
                        .visibility_field
                        .get(idx)
                        .copied()
                        .unwrap_or(0.0)
                } else {
                    0.0
                };

                // Only allow hover if visibility is above threshold (0.1)
                if visibility > 0.1 {
                    // Mouse entered an interactive entity on the same floor and visible
                    debug!(
                        "mouse_hover_interactive_system: Mouse entered interactive entity {:?} on floor {} with visibility {:.3}",
                        entity, interactive_floor, visibility
                    );
                    interactive.hovered = true;
                } else {
                    // Interactive is not visible enough, ignore the hover
                    debug!(
                        "mouse_hover_interactive_system: Ignoring hover on entity {:?} - visibility {:.3} < 0.1",
                        entity, visibility
                    );
                }
            } else {
                // Interactive is on a different floor, ignore the hover
                debug!(
                    "mouse_hover_interactive_system: Ignoring hover on entity {:?} - player on floor {}, interactive on floor {}",
                    entity, player_floor, interactive_floor
                );
            }
        }
    }

    for exit_event in exit_events.read() {
        if let Ok((entity, position, mut interactive, _behavior)) =
            q_interactives.get_mut(exit_event.target)
        {
            let interactive_floor = position.z.round() as i32;

            // Only process exit events for interactives on the same floor as the player
            if interactive_floor == player_floor {
                // Mouse exited an interactive entity on the same floor
                debug!(
                    "mouse_hover_interactive_system: Mouse exited interactive entity {:?} on floor {}",
                    entity, interactive_floor
                );

                interactive.hovered = false;
            }
        }
    }
}

/// System that continuously checks visibility of hovered interactive entities
/// and removes hover state if they become invisible (visibility < 0.1).
///
/// This system complements the mouse hover system by ensuring that if an
/// entity's visibility drops below the threshold while being hovered, the
/// hover state is removed without requiring a mouse movement.
pub(crate) fn visibility_hover_cleanup_system(
    mut q_interactives: Query<(Entity, &Position, &mut Interactive)>,
    q_player: Query<(&Position, &PlayerSprite)>,
    game_config: Res<GameConfig>,
    visibility_data: Res<VisibilityData>,
) {
    // Find the active player's position
    let player_pos = q_player.iter().find_map(|(pos, player)| {
        if player.id == game_config.player_id {
            Some(pos)
        } else {
            None
        }
    });

    let Some(player_pos) = player_pos else {
        return; // No active player found
    };

    let player_floor = player_pos.z.round() as i32;

    // Check all currently hovered interactives
    for (entity, position, mut interactive) in q_interactives.iter_mut() {
        if !interactive.hovered {
            continue; // Skip non-hovered entities
        }

        let interactive_floor = position.z.round() as i32;

        // Only check interactives on the same floor as the player
        if interactive_floor != player_floor {
            // If the interactive is no longer on the same floor, remove hover
            debug!(
                "visibility_hover_cleanup_system: Removing hover from entity {:?} - floor mismatch (player: {}, interactive: {})",
                entity, player_floor, interactive_floor
            );
            interactive.hovered = false;
            continue;
        }

        // Check visibility
        let interactive_bpos = position.to_board_position();
        let visibility = if let Some(idx) =
            interactive_bpos.ndidx_checked(visibility_data.visibility_field.dim())
        {
            visibility_data
                .visibility_field
                .get(idx)
                .copied()
                .unwrap_or(0.0)
        } else {
            0.0
        };

        // Remove hover if visibility drops below threshold
        if visibility <= 0.1 {
            debug!(
                "visibility_hover_cleanup_system: Removing hover from entity {:?} - low visibility {:.3}",
                entity, visibility
            );
            interactive.hovered = false;
        }
    }
}
