use bevy::prelude::*;
use bevy_persistent::Persistent;
use uncore::behavior::Behavior;
use uncore::behavior::component::{Interactive, RoomState};
use uncore::components::animation::{AnimationTimer, CharacterAnimation};
use uncore::components::board::direction::Direction;
use uncore::components::board::position::Position;
use uncore::components::player::{Hiding, Stamina};
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::events::npc_help::NpcHelpEvent;
use uncore::events::roomchanged::{InteractionExecutionType, RoomChangedEvent};
use uncore::systemparam::collision_handler::CollisionHandler;
use ungear::components::playergear::PlayerGear;
use unsettings::game::{GameplaySettings, MovementStyle};
use unstd::systemparam::interactivestuff::InteractiveStuff;

#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn keyboard_player(
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
) {
    const PLAYER_SPEED: f32 = 0.04;
    const RUN_MULTIPLIER: f32 = 1.6; // Multiplier for running speed
    const DIR_MIN: f32 = 5.0;
    const DIR_MAX: f32 = 80.0;
    const DIR_STEPS: f32 = 15.0;
    const DIR_MAG2: f32 = DIR_MAX / DIR_STEPS;
    const DIR_RED: f32 = 1.001;
    let dt = time.delta_secs() * 60.0;
    for (mut pos, mut dir, player, mut anim, player_gear, hiding, mut stamina) in players.iter_mut()
    {
        let col_delta = colhand.delta(&pos);
        pos.x -= col_delta.x;
        pos.y -= col_delta.y;
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
        let is_running = stamina.update(dt, wants_to_run);

        let run_multiplier = if is_running { RUN_MULTIPLIER } else { 1.0 };

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
        pos.x +=
            PLAYER_SPEED * d.dx * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;
        pos.y +=
            PLAYER_SPEED * d.dy * dt * speed_penalty * difficulty.0.player_speed * run_multiplier;

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
                        ev_npc.send(NpcHelpEvent::new(entity));
                    }
                    if interactive_stuff.execute_interaction(
                        entity,
                        item_pos,
                        Some(interactive),
                        behavior,
                        rs,
                        InteractionExecutionType::ChangeState,
                    ) {
                        ev_room.send(RoomChangedEvent::default());
                    }
                }
            }
        }
    }
}
