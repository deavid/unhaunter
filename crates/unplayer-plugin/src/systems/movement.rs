use crate::components::player::{Hiding, Stamina};
use bevy::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::behavior::Interactive;
use unbehavior::components::RoomState;
use unboard_core::resources::board_topology::{BoardCollisionField, BoardTopology};
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unevents_core::events::npc_help::NpcHelpEvent;
use unevents_core::events::roomchanged::InteractionExecutionType;
use unfog_core::miasma::MiasmaGrid;
use ungear_core::components::playergear::PlayerGear;
use uninteraction_core::interaction::ExecuteInteractionEvent;
use unnavigation_core::collision_handler::CollisionHandler;
use unplayer_core::components::MainPlayer;
use unplayer_core::components::PlayerInput;
use unplayer_core::components::PlayerSprite;
use unrender_std::components::animation::{AnimationTimer, CharacterAnimation};
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use unui_core::resources::MouseVisibility;

const PLAYER_SPEED: f32 = 0.04;
const RUN_ADD_MULTIPLIER: f32 = 1.3;
const DIR_MIN: f32 = 5.0;
const DIR_MAX: f32 = 40.0;
const DIR_STEPS: f32 = 15.0;
const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
const DIR_MAG3: f32 = DIR_MAG2 * 40.0;
const DIR_RED: f32 = 1.001;

/// System responsible for applying movement to the player based on the PlayerInput component.
///
/// This system handles all player movement logic including:
/// - Reading movement input from the PlayerInput component (populated by input systems)
/// - Applying movement speed, running, and stamina calculations
/// - Collision detection and handling
/// - Direction updates and animation
/// - Interaction with objects (E key)
/// - Running state management
///
/// This system decouples movement logic from input sources, allowing both keyboard
/// and click-to-move input to use the same movement implementation.
pub(crate) fn player_movement_system(
    time: Res<Time>,
    mut players: Query<(
        &mut Position,
        &mut Direction,
        &mut PlayerSprite,
        &mut AnimationTimer,
        &PlayerGear,
        &PlayerInput,
        Option<&Hiding>,
        &mut Stamina,
        Option<&MainPlayer>,
    )>,
    colhand: CollisionHandler,
    interactables: Query<
        (
            Entity,
            &Position,
            &Interactive,
            &Behavior,
            Option<&RoomState>,
        ),
        Without<PlayerSprite>,
    >,
    mut ev_interaction: MessageWriter<ExecuteInteractionEvent>,
    mut ev_npc: MessageWriter<NpcHelpEvent>,
    difficulty: Res<CurrentDifficulty>,
    _board_topology: Res<BoardTopology>,
    _board_collision: Res<BoardCollisionField>,
    miasma: Res<MiasmaGrid>,
    mut avg_running: Local<f32>,
    mouse_visibility: Res<MouseVisibility>,
) {
    let dt = time.delta_secs() * 60.0;

    for (
        mut pos,
        mut dir,
        mut player,
        mut anim,
        player_gear,
        player_input,
        hiding,
        mut stamina,
        main_player,
    ) in players.iter_mut()
    {
        let is_main_player = main_player.is_some();
        if !dir.is_finite() {
            warn!("Player direction is not finite: {dir:?}");
            *dir = Direction::zero();
        }
        if !pos.is_finite() {
            warn!("Player position is not finite: {pos:?}");
            if let Some((_, int_pos, _, _, _)) = interactables.iter().next() {
                *pos = *int_pos;
            }
        }

        let mut col_delta;
        if hiding.is_none() {
            col_delta = colhand.delta(&pos);
            if col_delta.is_finite() {
                pos.x -= col_delta.x;
                pos.y -= col_delta.y;
            } else {
                warn!("Player collision delta is not finite: {col_delta:?}");
                col_delta = Vec3::ZERO;
            }
        } else {
            col_delta = Vec3::ZERO;
        }

        // Get movement direction from PlayerInput component
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
        let delta = d / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;

        // Speed Penalty Based on Held Object Weight
        let speed_penalty = if player_gear.held_item.is_some() {
            0.5
        } else {
            1.0
        };

        // Check for Running with Stamina System
        let wants_to_run = player_input.run;

        // Miasma Logic
        let bpos = pos.to_board_position();
        let Some(pressure) = miasma.pressure_field.get(bpos.ndidx()) else {
            continue;
        };
        let miasma_factor = (*pressure / 100.0).max(0.0).cbrt().clamp(0.0, 0.7);

        // Stamina Modification
        stamina.depletion_rate = miasma_factor;
        let is_running = stamina.update(dt, wants_to_run).cbrt();
        let run_multiplier = 1.0 + RUN_ADD_MULTIPLIER * is_running;

        player.movement.dx += DIR_MAG2 * d.dx;
        player.movement.dy += DIR_MAG2 * d.dy;
        let dir_dist = (player.movement.dx.powi(2) + player.movement.dy.powi(2)).sqrt();
        if dir_dist > DIR_MAX {
            player.movement.dx *= DIR_MAX / dir_dist;
            player.movement.dy *= DIR_MAX / dir_dist;
        } else if dir_dist > DIR_MIN {
            player.movement.dx /= DIR_RED;
            player.movement.dy /= DIR_RED;
        }

        // Check if Player is Hiding
        if hiding.is_some() {
            let dscreen = perspective::direction_to_screen_coord(delta);
            anim.set_range(
                CharacterAnimation::from_dir(dscreen.x / 2000.0, dscreen.y / 1000.0).to_vec(),
            );
            continue;
        }

        // Apply speed penalty and run multiplier
        let pdx =
            PLAYER_SPEED * d.dx * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;
        let pdy =
            PLAYER_SPEED * d.dy * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;

        *avg_running = (*avg_running + is_running * dt) / (1.0 + dt);

        player.movement.dx += DIR_MAG3 * d.dx * (*avg_running + 0.5);
        player.movement.dy += DIR_MAG3 * d.dy * (*avg_running + 0.5);

        pos.x += pdx;
        pos.y += pdy;

        // Update player animation - make animations faster when running
        let animation_speed_factor = if run_multiplier > 1.0 { 1.5 } else { 1.0 };
        let dscreen = perspective::direction_to_screen_coord(delta);
        anim.set_range(
            CharacterAnimation::from_dir(
                dscreen.x * animation_speed_factor,
                dscreen.y * 2.0 * animation_speed_factor,
            )
            .to_vec(),
        );

        // Handle Interaction
        if player_input.interact {
            let mut max_dist = 1.4;
            let mut selected_entity = None;
            for (entity, item_pos, interactive, behavior, _) in interactables.iter() {
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
                for (entity, _, _, behavior, _) in
                    interactables.iter().filter(|(e, _, _, _, _)| *e == entity)
                {
                    if behavior.is_npc() {
                        ev_npc.write(NpcHelpEvent::new(entity));
                    }
                    ev_interaction.write(ExecuteInteractionEvent {
                        entity,
                        ietype: InteractionExecutionType::ChangeState,
                        force_tuid: None,
                    });
                }
            }
        }

        if is_main_player && mouse_visibility.is_visible {
            // Let mouse_aim_system handle Direction for MainPlayer
        } else if player_input.aim_direction.length_squared() > 0.001 {
            dir.dx = player_input.aim_direction.x;
            dir.dy = player_input.aim_direction.y;
            dir.dz = 0.0;
        } else if d.distance() > 0.1 {
            *dir = player.movement;
        } else {
            let dir_dist = (dir.dx.powi(2) + dir.dy.powi(2)).sqrt();
            if dir_dist > DIR_MIN {
                dir.dx /= DIR_RED;
                dir.dy /= DIR_RED;
            }
        }
    }
}
