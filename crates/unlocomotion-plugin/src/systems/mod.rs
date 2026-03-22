use bevy::prelude::*;
use unaudiospatial_core::events::SoundEvent;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::behavior::Interactive;
use unbehavior_core::components::RoomStateDelta;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unfog_core::miasma::MiasmaGrid;
use ungear_core::components::playergear::PlayerGear;
use uninput_core::components::PlayerInput;
use uninteraction_core::events::InteractionExecutionType;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unnavigation_core::collision_handler::CollisionHandler;
use unnpc_core::events::NpcHelpEvent;
use unplayer_core::components::{
    Hiding, MainPlayer, PlayerLocomotionState, PlayerSpectating, PlayerSprite,
};
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unreplicon_core::messages::InteractionRequestMessage;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use untypes_core::states::GameState;
use unui_core::resources::MouseVisibility;
use unvitals_core::components::Stamina;

const PLAYER_SPEED: f32 = 0.04;
const RUN_ADD_MULTIPLIER: f32 = 1.3;
const DIR_MIN: f32 = 5.0;
const DIR_MAX: f32 = 40.0;
const DIR_STEPS: f32 = 15.0;
const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
const DIR_MAG3: f32 = DIR_MAG2 * 40.0;
const DIR_RED: f32 = 1.001;

pub(crate) fn dispatch_interact_intent(
    players: Query<
        (
            &Position,
            &PlayerInput,
            Option<&Hiding>,
            Option<&InTruck>,
            Option<&PlayerSpectating>,
        ),
        With<MainPlayer>,
    >,
    interactables: Query<
        (
            Entity,
            &Position,
            Option<&Interactive>,
            &Behavior,
            Option<&RoomStateDelta>,
        ),
        Without<PlayerSprite>,
    >,
    mut ev_interaction: MessageWriter<ExecuteInteractionEvent>,
    mut ev_interaction_req: MessageWriter<InteractionRequestMessage>,
    mut game_next_state: ResMut<NextState<GameState>>,
    mut ev_sound: MessageWriter<SoundEvent>,
    mut ev_npc: Option<MessageWriter<NpcHelpEvent>>,
    authority: Option<Res<untypes_core::roles::AuthorityRole>>,
) {
    for (pos, player_input, hiding, in_truck, spectating) in players.iter() {
        if in_truck.is_some() || hiding.is_some() || spectating.is_some() {
            continue;
        }
        if player_input.interact {
            let mut max_dist = 1.4;
            let mut selected_entity = None;
            for (entity, item_pos, interactive, behavior, _) in interactables.iter() {
                let Some(interactive) = interactive else {
                    continue;
                };
                let cp_delta = interactive.control_point_delta(behavior);
                let item_pos = Position {
                    x: item_pos.x + cp_delta.x,
                    y: item_pos.y + cp_delta.y,
                    z: item_pos.z + cp_delta.z,
                    visual_priority: item_pos.visual_priority,
                };
                let new_dist = pos.delta(item_pos);
                let dref = new_dist;
                let dist = dref.distance();
                if dist < max_dist {
                    max_dist = dist + 0.00001;
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                for (entity, item_pos, interactive, behavior, _) in
                    interactables.iter().filter(|(e, _, _, _, _)| *e == entity)
                {
                    if behavior.is_npc()
                        && let Some(ev) = ev_npc.as_mut()
                    {
                        ev.write(NpcHelpEvent::new(entity));
                    }
                    if behavior.is_van_entry() {
                        game_next_state.set(GameState::Truck);
                        if let Some(interactive) = interactive {
                            ev_sound.write(SoundEvent {
                                sound_file: interactive.sound_for_moving_into_state(behavior),
                                volume: 1.0,
                                position: Some(*item_pos),
                                broadcast: false,
                            });
                        }
                    } else {
                        let bpos = item_pos.to_board_position();
                        let bpos_arr = [bpos.x as i32, bpos.y as i32, bpos.z as i32];
                        if authority.is_some() {
                            ev_interaction.write(ExecuteInteractionEvent {
                                entity,
                                ietype: InteractionExecutionType::ChangeState,
                                force_tuid: None,
                            });
                        } else {
                            ev_interaction_req.write(InteractionRequestMessage {
                                position: bpos_arr,
                                ietype: InteractionExecutionType::ChangeState,
                                force_tuid: None,
                            });
                        }
                    }
                }
            }
        }
    }
}

pub(crate) fn apply_movement_intent(
    time: Res<Time>,
    authority: Option<Res<untypes_core::roles::AuthorityRole>>,
    mut players: Query<(
        &mut Position,
        &mut Direction,
        &mut PlayerSprite,
        &mut PlayerLocomotionState,
        &PlayerGear,
        &PlayerInput,
        Option<&Hiding>,
        Option<&InTruck>,
        &mut Stamina,
        Option<&MainPlayer>,
        Has<PlayerSpectating>,
    )>,
    colhand: CollisionHandler,
    interactables: Query<
        (
            Entity,
            &Position,
            &Interactive,
            &Behavior,
            Option<&RoomStateDelta>,
        ),
        Without<PlayerSprite>,
    >,
    difficulty: Res<CurrentDifficulty>,
    miasma: Option<Res<MiasmaGrid>>,
    mut avg_running: Local<f32>,
    mut last_error_log: Local<f32>,
    mouse_visibility: Option<Res<MouseVisibility>>,
) {
    let is_authority = authority.is_some();
    let dt = time.delta_secs() * 60.0;
    let now = time.elapsed_secs();
    let mut can_log = false;
    if now - *last_error_log > 1.0 {
        can_log = true;
        *last_error_log = now;
    }

    for (
        mut pos,
        mut dir,
        _player_sprite,
        mut player_loco,
        player_gear,
        player_input,
        hiding,
        in_truck,
        mut stamina,
        main_player,
        is_spectating,
    ) in players.iter_mut()
    {
        let is_main_player = main_player.is_some();
        if !is_authority && !is_main_player {
            continue;
        }

        if in_truck.is_some() {
            continue;
        }

        if !dir.is_finite() {
            if can_log {
                error!("Player direction is not finite: {dir:?}");
                can_log = false;
            }
            *dir = Direction::zero();
        }
        if !pos.is_finite() {
            if can_log {
                error!("Player position is not finite: {pos:?}");
                can_log = false;
            }
            if let Some((_, int_pos, _, _, _)) = interactables.iter().next() {
                *pos = *int_pos;
            }
        }

        let mut col_delta;
        if hiding.is_none() && !is_spectating {
            col_delta = colhand.delta(&pos);
            if col_delta.is_finite() {
                pos.x -= col_delta.x;
                pos.y -= col_delta.y;
            } else {
                if can_log {
                    error!("Player collision delta is not finite: {col_delta:?}");
                    can_log = false;
                }
                col_delta = Vec3::ZERO;
            }
        } else {
            col_delta = Vec3::ZERO;
        }

        let input_vec = player_input.movement;
        let mut d = Direction {
            dx: input_vec.x,
            dy: input_vec.y,
            dz: 0.0,
        };

        d = d.normalized();
        let col_delta_n = (col_delta * 100.0).clamp_length_max(1.0);
        let col_dotp = (d.dx * col_delta_n.x + d.dy * col_delta_n.y).clamp(0.0, 1.0);
        d.dx -= col_delta_n.x * col_dotp;
        d.dy -= col_delta_n.y * col_dotp;

        player_loco.velocity = Vec2::new(d.dx, d.dy);

        if is_spectating {
            let spectate_speed = PLAYER_SPEED * difficulty.0.player_speed() * 2.0;
            pos.x += d.dx * spectate_speed * dt;
            pos.y += d.dy * spectate_speed * dt;

            col_delta = colhand.spectator_delta(&pos);
            if col_delta.is_finite() {
                pos.x -= col_delta.x;
                pos.y -= col_delta.y;
            }

            if d.distance() > 0.001 {
                dir.dx = d.dx;
                dir.dy = d.dy;
                dir.dz = 0.0;
            }

            continue;
        }

        let speed_penalty = if player_gear.held_item.is_some() {
            0.5
        } else {
            1.0
        };

        let wants_to_run = player_input.run;

        let miasma_factor = if let Some(miasma) = miasma.as_ref() {
            let bpos = pos.to_board_position();
            miasma
                .pressure_field
                .get(bpos.ndidx())
                .map(|pressure| (*pressure / 100.0).max(0.0).cbrt().clamp(0.0, 0.7))
                .unwrap_or(0.0)
        } else {
            0.0
        };

        stamina.depletion_rate = miasma_factor;
        let is_running = stamina.update(dt, wants_to_run).cbrt();
        let run_multiplier = 1.0 + RUN_ADD_MULTIPLIER * is_running;

        player_loco.movement.dx += DIR_MAG2 * d.dx;
        player_loco.movement.dy += DIR_MAG2 * d.dy;
        let dir_dist = (player_loco.movement.dx.powi(2) + player_loco.movement.dy.powi(2)).sqrt();
        if dir_dist > DIR_MAX {
            player_loco.movement.dx *= DIR_MAX / dir_dist;
            player_loco.movement.dy *= DIR_MAX / dir_dist;
        } else if dir_dist > DIR_MIN {
            player_loco.movement.dx /= DIR_RED;
            player_loco.movement.dy /= DIR_RED;
        }

        if hiding.is_some() {
            continue;
        }

        let pdx =
            PLAYER_SPEED * d.dx * dt * speed_penalty * difficulty.0.player_speed() * run_multiplier;
        let pdy =
            PLAYER_SPEED * d.dy * dt * speed_penalty * difficulty.0.player_speed() * run_multiplier;

        *avg_running = (*avg_running + is_running * dt) / (1.0 + dt);

        player_loco.movement.dx += DIR_MAG3 * d.dx * (*avg_running + 0.5);
        player_loco.movement.dy += DIR_MAG3 * d.dy * (*avg_running + 0.5);

        if is_main_player {
            pos.x += pdx;
            pos.y += pdy;
        }

        if is_main_player
            && mouse_visibility
                .as_ref()
                .map(|m| m.is_visible)
                .unwrap_or(false)
        {
        } else if player_input.aim_direction.length_squared() > 0.001 {
            dir.dx = player_input.aim_direction.x;
            dir.dy = player_input.aim_direction.y;
            dir.dz = 0.0;
        } else if d.distance() > 0.1 {
            *dir = player_loco.movement;
        } else {
            let dir_dist = (dir.dx.powi(2) + dir.dy.powi(2)).sqrt();
            if dir_dist > DIR_MIN {
                dir.dx /= DIR_RED;
                dir.dy /= DIR_RED;
            }
        }
    }
}

pub(crate) fn drive_character_animation(
    mut players: Query<(
        &PlayerLocomotionState,
        &Direction,
        &Stamina,
        &mut AnimationTimer,
        Option<&Hiding>,
        Option<&InTruck>,
    )>,
) {
    for (player_loco, dir, stamina, mut anim, hiding, in_truck) in players.iter_mut() {
        if in_truck.is_some() {
            continue;
        }

        if hiding.is_some() {
            anim.set_range(CharacterAnimation::from_dir(0.0, 0.0).to_vec());
            continue;
        }

        let delta =
            Direction::from(player_loco.velocity) / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;
        let animation_speed_factor = if stamina.running { 1.5 } else { 1.0 };
        let dscreen = perspective::direction_to_screen_coord(delta);
        anim.set_range(
            CharacterAnimation::from_dir(
                dscreen.x * animation_speed_factor,
                dscreen.y * 2.0 * animation_speed_factor,
            )
            .to_vec(),
        );
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            dispatch_interact_intent,
            apply_movement_intent,
            drive_character_animation,
        )
            .run_if(in_state(GameState::Running)),
    );
}
