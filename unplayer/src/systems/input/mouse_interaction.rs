use bevy::{
    input::mouse::{MouseScrollUnit, MouseWheel},
    picking::events::{Click, Out, Over, Pointer},
    prelude::*,
};
use uncore::{
    behavior::{Behavior, component::Interactive},
    components::{board::position::Position, move_to::MoveToTarget, player_sprite::PlayerSprite},
    events::roomchanged::{InteractionExecutionType, RoomChangedEvent},
    resources::{looking_gear::LookingGear, mouse_visibility::MouseVisibility},
};
use ungear::components::playergear::PlayerGear;
use ungear::gear_stuff::GearStuff;
use ungear::gear_usable::GearUsable;
use unstd::systemparam::interactivestuff::InteractiveStuff;

/// System that handles mouse picking for interactive objects.
///
/// When the player left-clicks on an interactive object (doors, switches, etc.),
/// this system determines whether to activate it immediately if the player is close enough,
/// or to move the player to the object first and then activate it.
pub(crate) fn mouse_interaction_system(
    mut commands: Commands,
    q_player: Query<(Entity, &Position), With<PlayerSprite>>,
    q_interactives: Query<(
        Entity,
        &Position,
        &Interactive,
        &Behavior,
        Option<&uncore::behavior::component::RoomState>,
    )>,
    mut interactive_stuff: InteractiveStuff,
    mut click_events: EventReader<Pointer<Click>>,
    mouse_visibility: Res<MouseVisibility>,
    mut ev_room: EventWriter<RoomChangedEvent>,
) {
    // Only process clicks when mouse is visible
    if !mouse_visibility.is_visible {
        return;
    }

    let Ok((player_entity, player_pos)) = q_player.single() else {
        return;
    };

    let interactive_count = q_interactives.iter().count();
    debug!(
        "mouse_interaction_system: Found {} interactive objects with InteractivePickable",
        interactive_count
    );

    for click_event in click_events.read() {
        info!(
            "mouse_interaction_system: Received click event on entity {:?} with button {:?}",
            click_event.target, click_event.button
        );

        // Only handle left clicks
        if click_event.button != PointerButton::Primary {
            debug!("mouse_interaction_system: Not a left click, ignoring");
            continue;
        }

        // Check if the clicked entity is an interactive object
        if let Ok((interactive_entity, interactive_pos, interactive, behavior, room_state)) =
            q_interactives.get(click_event.target)
        {
            info!(
                "mouse_interaction_system: Clicked entity {:?} is interactive at position {:?}",
                interactive_entity, interactive_pos
            );

            let distance = player_pos.distance(interactive_pos);
            const INTERACTION_DISTANCE: f32 = 1.4;

            debug!(
                "mouse_interaction_system: Distance to interactive: {:.2}, threshold: {:.2}",
                distance, INTERACTION_DISTANCE
            );

            if distance <= INTERACTION_DISTANCE {
                info!("mouse_interaction_system: Player close enough, activating immediately");

                // Player is close enough, activate immediately
                if behavior.is_npc() {
                    debug!("mouse_interaction_system: Target is NPC");
                    // Handle NPC interactions if needed
                    // ev_npc.write(NpcHelpEvent::new(interactive_entity));
                }

                if interactive_stuff.execute_interaction(
                    interactive_entity,
                    interactive_pos,
                    Some(interactive),
                    behavior,
                    room_state,
                    InteractionExecutionType::ChangeState,
                ) {
                    ev_room.write(RoomChangedEvent::default());

                    info!("mouse_interaction_system: Interaction executed.");
                }
            } else {
                info!("mouse_interaction_system: Player too far, setting up movement");

                // Player is too far, move to the interactive object first
                // Calculate a position near the interactive object for the player to move to
                let direction_to_player = player_pos.delta(*interactive_pos).normalized();
                let target_distance = INTERACTION_DISTANCE * 0.8; // Get a bit closer than the interaction threshold

                let target_pos = Position {
                    x: interactive_pos.x + direction_to_player.dx * target_distance,
                    y: interactive_pos.y + direction_to_player.dy * target_distance,
                    z: player_pos.z, // Maintain player's current Z level
                    global_z: 0.0,
                };

                debug!(
                    "mouse_interaction_system: Moving player to {:?}",
                    target_pos
                );

                // Add movement target with interaction
                commands
                    .entity(player_entity)
                    .insert(MoveToTarget::with_interaction(
                        target_pos,
                        interactive_entity,
                    ));
            }
        } else {
            warn!(
                "mouse_interaction_system: Clicked entity {:?} is not interactive or not found in query",
                click_event.target
            );
            // NOTE: This does not work and the sprites are not picked because the sprites on the map do not have the Sprite component - but
            // this is intentional since we're customizing a lot how they're drawn. Therefore we will need a custom SpritePickingPlugin.
        }
    }
}

/// System that handles completing interactions after the player reaches their target.
///
/// This system checks if the player has reached their movement target and if there's
/// an interaction to perform, it executes it and cleans up the MoveToTarget component.
pub fn complete_pending_interaction_system(
    mut commands: Commands,
    q_player: Query<(Entity, &Position, &MoveToTarget), With<PlayerSprite>>,
    q_interactives: Query<(
        Entity,
        &Position,
        &Interactive,
        &Behavior,
        Option<&uncore::behavior::component::RoomState>,
    )>,
    mut interactive_stuff: InteractiveStuff,
    mut ev_room: EventWriter<RoomChangedEvent>,
) {
    for (player_entity, player_pos, move_target) in q_player.iter() {
        // Check if there's an interaction target
        if let Some(interaction_target) = move_target.interaction_target {
            debug!(
                "complete_pending_interaction_system: Player {:?} has move target with interaction {:?}",
                player_entity, interaction_target
            );

            // Check if player has reached the target position
            let current = Vec2::new(player_pos.x, player_pos.y);
            let target_pos = Vec2::new(move_target.position.x, move_target.position.y);
            let to_target = target_pos - current;

            const ARRIVAL_THRESHOLD: f32 = 0.1;
            if to_target.length_squared() <= ARRIVAL_THRESHOLD * ARRIVAL_THRESHOLD {
                debug!("complete_pending_interaction_system: Player reached target position");

                // Player has reached the target, now check if we can interact
                if let Ok((_, interactive_pos, interactive, behavior, room_state)) =
                    q_interactives.get(interaction_target)
                {
                    let distance = player_pos.distance(interactive_pos);
                    const INTERACTION_DISTANCE: f32 = 1.4;

                    debug!(
                        "complete_pending_interaction_system: Distance to target: {:.2}, threshold: {:.2}",
                        distance, INTERACTION_DISTANCE
                    );

                    if distance <= INTERACTION_DISTANCE {
                        info!("complete_pending_interaction_system: Executing interaction");

                        // Execute the interaction
                        if interactive_stuff.execute_interaction(
                            interaction_target,
                            interactive_pos,
                            Some(interactive),
                            behavior,
                            room_state,
                            InteractionExecutionType::ChangeState,
                        ) {
                            ev_room.write(RoomChangedEvent::default());
                            info!("complete_pending_interaction_system: Interaction executed.");
                        }

                        // Remove the MoveToTarget component (interaction is complete)
                        commands.entity(player_entity).remove::<MoveToTarget>();
                    } else {
                        debug!(
                            "complete_pending_interaction_system: Still too far from target after reaching position"
                        );
                        // This shouldn't normally happen, but clean up if it does
                        commands.entity(player_entity).remove::<MoveToTarget>();
                    }
                } else {
                    warn!(
                        "complete_pending_interaction_system: Target entity no longer exists or is invalid, cleaning up"
                    );
                    // Target entity no longer exists or is invalid, clean up
                    commands.entity(player_entity).remove::<MoveToTarget>();
                }
            }
        }
    }
}

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
