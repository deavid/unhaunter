use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::behavior::component::{Interactive, RoomState, Stairs};
use uncore::behavior::{Behavior, Orientation};
use uncore::components::animation::{AnimationTimer, CharacterAnimation};
use uncore::components::board::direction::Direction;
use uncore::components::board::position::Position;
use uncore::components::game_config::GameConfig;
use uncore::components::player::{Hiding, Stamina};
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::npc_help::NpcHelpEvent;
use uncore::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uncore::resources::board_data::BoardData;
use uncore::systemparam::collision_handler::CollisionHandler;
use ungear::components::playergear::PlayerGear;
use unsettings::game::{GameplaySettings, MovementStyle};
use unstd::systemparam::interactivestuff::InteractiveStuff;

fn keyboard_player(
    time: Res<Time>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        &mut Position,
        &mut Direction,
        &mut PlayerSprite,
        &mut AnimationTimer,
        &PlayerGear,
        Option<&Hiding>,
        &mut Stamina,
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
    mut interactive_stuff: InteractiveStuff,
    mut ev_room: EventWriter<RoomChangedEvent>,
    mut ev_npc: EventWriter<NpcHelpEvent>,
    difficulty: Res<CurrentDifficulty>,
    game_settings: Res<Persistent<GameplaySettings>>,
    board_data: Res<BoardData>,
    mut avg_running: Local<f32>,
) {
    const PLAYER_SPEED: f32 = 0.04;
    const RUN_ADD_MULTIPLIER: f32 = 1.3; // Add Multiplier for running speed
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 40.0;
    const DIR_STEPS: f32 = 15.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    // DIR_MAG3 controls the camera to get it even further ahead when the player is running.
    const DIR_MAG3: f32 = DIR_MAG2 * 40.0;
    const DIR_RED: f32 = 1.001;
    let dt = time.delta_secs() * 60.0;
    for (mut pos, mut dir, player, mut anim, player_gear, hiding, mut stamina) in players.iter_mut()
    {
        if !dir.is_finite() {
            warn!("Player direction is not finite: {dir:?}");
            *dir = Direction::zero();
        }
        if !pos.is_finite() {
            warn!("Player position is not finite: {pos:?}");
            if let Some((_, int_pos, _, _, _)) = interactables.iter().next() {
                // Emergency: try to put a valid position from any interactable object, so the game remains playable.
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
        let mut d = Direction {
            dx: 0.0,
            dy: 0.0,
            dz: 0.0,
        };

        if keyboard_input.pressed(player.controls.up) {
            d.dy += 1.0;
        }
        if keyboard_input.pressed(player.controls.down) {
            d.dy -= 1.0;
        }
        if keyboard_input.pressed(player.controls.left) {
            d.dx -= 1.0;
        }
        if keyboard_input.pressed(player.controls.right) {
            d.dx += 1.0;
        }
        if matches!(
            game_settings.movement_style,
            MovementStyle::ScreenSpaceOrthogonal
        ) {
            pub const PERSPECTIVE_X: [f32; 2] = [1.0, 1.0];
            pub const PERSPECTIVE_Y: [f32; 2] = [-1.0, 1.0];
            let od = d;
            d.dx = od.dx * PERSPECTIVE_X[0] + od.dy * PERSPECTIVE_Y[0];
            d.dy = od.dx * PERSPECTIVE_X[1] + od.dy * PERSPECTIVE_Y[1];
        }

        d = d.normalized();
        let col_delta_n = (col_delta * 100.0).clamp_length_max(1.0);
        let col_dotp = (d.dx * col_delta_n.x + d.dy * col_delta_n.y).clamp(0.0, 1.0);
        d.dx -= col_delta_n.x * col_dotp;
        d.dy -= col_delta_n.y * col_dotp;
        let delta = d / 0.1 + dir.normalized() / DIR_MAG2 / 1000.0;

        // --- Speed Penalty Based on Held Object Weight ---
        let speed_penalty = if player_gear.held_item.is_some() {
            0.5
        } else {
            1.0
        };

        // --- Check for Running with Stamina System ---
        let wants_to_run = keyboard_input.pressed(player.controls.run);

        // --- Miasma Logic ---
        let bpos = pos.to_board_position();
        let Some(pressure) = board_data.miasma.pressure_field.get(bpos.ndidx()) else {
            continue;
        };
        // NOTE: miasma pressure can be negative.
        let miasma_factor = (*pressure / 100.0).max(0.0).cbrt().clamp(0.0, 0.7);

        // --- Stamina Modification ---
        // Increase depletion rate based on miasma.
        stamina.depletion_rate = miasma_factor;

        let is_running = stamina.update(dt, wants_to_run).cbrt();

        let run_multiplier = 1.0 + RUN_ADD_MULTIPLIER * is_running;

        dir.dx += DIR_MAG2 * d.dx;
        dir.dy += DIR_MAG2 * d.dy;
        let dir_dist = (dir.dx.powi(2) + dir.dy.powi(2)).sqrt();
        if dir_dist > DIR_MAX {
            dir.dx *= DIR_MAX / dir_dist;
            dir.dy *= DIR_MAX / dir_dist;
        } else if dir_dist > DIR_MIN {
            dir.dx /= DIR_RED;
            dir.dy /= DIR_RED;
        }

        // --- Check if Player is Hiding ---
        if hiding.is_some() {
            // Update player animation
            let dscreen = delta.to_screen_coord();
            anim.set_range(
                CharacterAnimation::from_dir(dscreen.x / 2000.0, dscreen.y / 1000.0).to_vec(),
            );

            // Check if the Hiding component is present Skip movement input handling if hiding
            continue;
        }

        // Apply speed penalty and run multiplier
        let pdx =
            PLAYER_SPEED * d.dx * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;
        let pdy =
            PLAYER_SPEED * d.dy * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;

        *avg_running = (*avg_running + is_running * dt) / (1.0 + dt);

        dir.dx += DIR_MAG3 * d.dx * (*avg_running + 0.5);
        dir.dy += DIR_MAG3 * d.dy * (*avg_running + 0.5);

        pos.x += pdx;
        pos.y += pdy;

        // Update player animation - make animations faster when running
        let animation_speed_factor = if run_multiplier > 1.0 { 1.5 } else { 1.0 };
        let dscreen = delta.to_screen_coord();
        anim.set_range(
            CharacterAnimation::from_dir(
                dscreen.x * animation_speed_factor,
                dscreen.y * 2.0 * animation_speed_factor,
            )
            .to_vec(),
        );

        // ---
        if keyboard_input.just_pressed(player.controls.activate) {
            // let d = dir.normalized();
            let mut max_dist = 1.4;
            let mut selected_entity = None;
            for (entity, item_pos, interactive, behavior, _) in interactables.iter() {
                let cp_delta = interactive.control_point_delta(behavior);

                // let old_dist = pos.delta(*item_pos);
                let item_pos = Position {
                    x: item_pos.x + cp_delta.x,
                    y: item_pos.y + cp_delta.y,
                    z: item_pos.z + cp_delta.z,
                    global_z: item_pos.global_z,
                };
                let new_dist = pos.delta(item_pos);

                // let new_dist_norm = new_dist.normalized(); let dot_p = (new_dist_norm.dx *
                // -d.dx + new_dist_norm.dy * -d.dy).clamp(0.0, 1.0); let dref = new_dist + (&d *
                // (new_dist.distance().min(1.0) * dot_p));
                let dref = new_dist;
                let dist = dref.distance();
                if dist < max_dist {
                    max_dist = dist + 0.00001;
                    selected_entity = Some(entity);
                }
            }
            if let Some(entity) = selected_entity {
                for (entity, item_pos, interactive, behavior, rs) in
                    interactables.iter().filter(|(e, _, _, _, _)| *e == entity)
                {
                    if behavior.is_npc() {
                        ev_npc.write(NpcHelpEvent::new(entity));
                    }
                    if interactive_stuff.execute_interaction(
                        entity,
                        item_pos,
                        Some(interactive),
                        behavior,
                        rs,
                        InteractionExecutionType::ChangeState,
                    ) {
                        ev_room.write(RoomChangedEvent::default());
                    }
                }
            }
        }
    }
}

fn stairs_player(
    mut players: Query<(&mut Position, &PlayerSprite)>,
    stairs: Query<(&Position, &Stairs, &Behavior), Without<PlayerSprite>>,
    gc: Res<GameConfig>,
) {
    let Some((mut player_pos, _player_sprite)) =
        players.iter_mut().find(|(_, ps)| ps.id == gc.player_id)
    else {
        return;
    };
    let player_bpos = player_pos.to_board_position();
    let mut in_stairs = false;

    for (stair_pos, stair, b) in &stairs {
        let stair_bpos = stair_pos.to_board_position();
        match b.orientation() {
            Orientation::XAxis => {
                if (stair_bpos.x == player_bpos.x || stair_bpos.x + 1 == player_bpos.x)
                    && (player_bpos.y - stair_bpos.y).abs() <= 1
                    && stair_bpos.z == player_bpos.z
                {
                    let dy = player_pos.y - stair_pos.y;
                    player_pos.z = stair_pos.z + (stair.z as f32) / 4.1 + dy / 4.1;
                    if stair.z > 0 {
                        player_pos.z = player_pos.z.clamp(stair_pos.z, stair_pos.z + 1.0);
                    } else {
                        player_pos.z = player_pos.z.clamp(stair_pos.z - 1.0, stair_pos.z);
                    }
                    in_stairs = true;
                }
            }
            Orientation::YAxis => {
                if (stair_bpos.y == player_bpos.y || stair_bpos.y - 1 == player_bpos.y)
                    && (player_bpos.x - stair_bpos.x).abs() <= 1
                    && stair_bpos.z == player_bpos.z
                {
                    let dx = stair_pos.x - player_pos.x;
                    player_pos.z = stair_pos.z + (stair.z as f32) / 4.1 + dx / 4.1;
                    // FIXME: We need to support mirroring the sprite in X direction, meaning that the player would move in the Y direction instead of X.
                    if stair.z > 0 {
                        player_pos.z = player_pos.z.clamp(stair_pos.z, stair_pos.z + 1.0);
                    } else {
                        player_pos.z = player_pos.z.clamp(stair_pos.z - 1.0, stair_pos.z);
                    }
                    in_stairs = true;
                }
            }
            _ => {}
        }
    }

    if !in_stairs && player_pos.z.fract().abs() > 0.001 {
        player_pos.z = player_bpos.z as f32;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (keyboard_player, stairs_player).run_if(in_state(uncore::states::GameState::None)),
    );
}
