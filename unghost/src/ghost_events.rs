use bevy::prelude::*;
use rand::Rng;
use uncore::behavior;
use uncore::components::board::position::Position;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::board_data_rebuild::BoardDataToRebuild;
use uncore::events::roomchanged::InteractionExecutionType;
use uncore::events::sound::SoundEvent;
use uncore::random_seed;
use unstd::systemparam::interactivestuff::InteractiveStuff;

#[derive(Debug, Clone)]
pub enum GhostEvent {
    DoorSlam,
    LightFlicker,
}

#[derive(Component)]
struct FlickerTimer(Timer);

#[allow(clippy::too_many_arguments, clippy::type_complexity)]
pub fn trigger_ghost_events(
    mut commands: Commands,
    q_player: Query<(&Position, &PlayerSprite)>,
    q_ghost: Query<(&GhostSprite, &Position)>,
    // Query for doors, excluding lights
    q_doors: Query<
        (Entity, &Position, &behavior::Behavior),
        (
            With<behavior::component::Door>,
            Without<behavior::component::Light>,
        ),
    >,
    // Query for lights, excluding doors
    mut q_lights: Query<
        (Entity, &Position, &mut behavior::Behavior),
        (
            With<behavior::component::Light>,
            Without<behavior::component::Interactive>,
        ),
    >,
    mut interactive_stuff: InteractiveStuff,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
    difficulty: Res<CurrentDifficulty>,
) {
    let mut rng = random_seed::rng();
    let roomdb = interactive_stuff.roomdb.clone();

    // Iterate through players inside the house
    for (player_pos, _player) in q_player.iter().filter(|(pos, _)| {
        let bpos = pos.to_board_position();
        roomdb.room_tiles.contains_key(&bpos)
    }) {
        // Find the ghost
        let Ok((_ghost, ghost_pos)) = q_ghost.get_single() else {
            return;
        };

        // Calculate distance and event probability
        let distance = player_pos.distance2(ghost_pos);
        let event_probability =
            (10.0 / (distance + 2.0)).sqrt() / 200.0 * difficulty.0.ghost_interaction_frequency;

        // Roll for an event
        if rng.random_range(0.0..1.0) < event_probability {
            // Choose a random event
            let event = match rng.random_range(0..10) {
                0 => GhostEvent::DoorSlam,
                _ => GhostEvent::LightFlicker,
            };
            // warn!("Event: {:?}", event);
            match event {
                GhostEvent::DoorSlam => {
                    // Find doors in the player's room
                    let player_room = roomdb
                        .room_tiles
                        .get(&player_pos.to_board_position())
                        .cloned();
                    let mut doors_in_room = Vec::new();
                    if let Some(player_room) = player_room {
                        for (entity, door_pos, behavior) in q_doors.iter() {
                            if roomdb.room_tiles.get(&door_pos.to_board_position())
                                == Some(&player_room)
                                && !behavior.p.movement.player_collision
                            {
                                // Just put here the open doors as candidates.
                                doors_in_room.push(entity);
                            }
                        }
                    }

                    // If there are doors, slam a random one
                    if !doors_in_room.is_empty() {
                        let door_to_slam = doors_in_room[rng.random_range(0..doors_in_room.len())];

                        // Retrieve the door's Behavior component
                        if let Ok((_, door_position, behavior)) = q_doors.get(door_to_slam) {
                            // FIXME: This is not correct! We're using a player interaction function for a
                            // ghost event, which leads to awkward workarounds and potential bugs. We should
                            // create a separate mechanism for handling ghost events.
                            dbg!(interactive_stuff.execute_interaction(
                                door_to_slam,
                                // Pass the door's position
                                door_position,
                                // No interactive component needed
                                None,
                                behavior,
                                None,
                                InteractionExecutionType::ChangeState,
                            ));

                            // Play door slam sound effect
                            interactive_stuff.sound_events.send(SoundEvent {
                                sound_file: "sounds/door-close.ogg".to_string(),
                                volume: 1.0,
                                position: Some(*door_position),
                            });

                            ev_bdr.send(BoardDataToRebuild {
                                lighting: true,
                                collision: true,
                            });
                        }
                        // warn!("Slamming door: {:?}", door_to_slam);
                    }
                }
                GhostEvent::LightFlicker => {
                    // Find lights in the player's room
                    let player_room = roomdb
                        .room_tiles
                        .get(&player_pos.to_board_position())
                        .cloned();
                    if let Some(player_room) = player_room {
                        let mut flicker = false;
                        for (entity, light_pos, mut behavior) in q_lights.iter_mut() {
                            if behavior.can_emit_light()
                                && roomdb.room_tiles.get(&light_pos.to_board_position())
                                    == Some(&player_room)
                            {
                                // Toggle the light's state using the public method
                                behavior.p.light.flickering = true;

                                // Add a timer to reset the light after a short duration
                                commands
                                    .entity(entity)
                                    .insert(FlickerTimer(Timer::from_seconds(
                                        0.5,
                                        TimerMode::Once,
                                    )));
                                // warn!("Flickering light: {:?}", entity);
                                flicker = true;
                            }
                        }
                        if flicker {
                            ev_bdr.send(BoardDataToRebuild {
                                lighting: true,
                                collision: true,
                            });
                        }
                    }
                }
            }
        }
    }
}

fn update_flicker_timers(
    mut commands: Commands,
    time: Res<Time>,
    mut q_lights: Query<(Entity, &mut FlickerTimer, &mut behavior::Behavior)>,
    mut ev_bdr: EventWriter<BoardDataToRebuild>,
) {
    for (entity, mut flicker_timer, mut behavior) in q_lights.iter_mut() {
        flicker_timer.0.tick(time.delta());
        if flicker_timer.0.finished() {
            // Reset the light to its original state using the public method
            behavior.p.light.flickering = false;
            commands.entity(entity).remove::<FlickerTimer>();
            ev_bdr.send(BoardDataToRebuild {
                lighting: true,
                collision: true,
            });
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.add_systems(Update, (trigger_ghost_events, update_flicker_timers));
}
