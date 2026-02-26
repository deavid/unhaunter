use crate::components::player::Stamina;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use unbehavior::roomdb::RoomDB;
use unboard_core::resources::board_topology::BoardTopology;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unfoundation_core::types::grade::Grade;
use ungear_core::components::playergear::PlayerGear;
use unlight_core::resources::light_grid::LightGrid;
use unplayer_core::components::MainPlayer;
use unplayer_core::components::PlayerInput;
use unplayer_core::components::PlayerSpectating;
use unplayer_core::components::PlayerSprite;
use unprofile_core::profile::PlayerProfileData;
use unrender_std::utils::light::lerp_color;
use unsound_core::resources::SoundGrid;
use unspatial_core::position::Position;
use unsummary_core::summary::SummaryData;
use unthermal_core::resources::ThermalGrid;
use untruck_core::components::in_truck::InTruck;
use unui_core::components::game_ui::DamageBackground;

pub(crate) fn calculate_sanity(crazyness: f32) -> f32 {
    const LINEAR: f32 = 30.0;
    const SCALE: f32 = 100.0;
    (SCALE * LINEAR) / ((crazyness + LINEAR * LINEAR).max(0.01).sqrt())
}

fn lose_sanity(
    time: Res<Time>,
    mut qp: Query<
        (&mut PlayerSprite, &Position),
        (
            With<MainPlayer>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    thermal_grid: Res<ThermalGrid>,
    sound_grid: Res<SoundGrid>,
    lg: Res<LightGrid>,
    roomdb: Res<RoomDB>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    for (mut ps, pos) in &mut qp {
        let bpos = pos.to_board_position();
        let p = bpos.ndidx();
        if p.0 >= lg.light_field.shape()[0]
            || p.1 >= lg.light_field.shape()[1]
            || p.2 >= lg.light_field.shape()[2]
        {
            continue;
        }
        let lux = lg.light_field[p].lux.sqrt() + 0.001;
        let temp = thermal_grid.temperature_field[p];
        let f_temp = (temp - thermal_grid.ambient_temp / 2.0).clamp(0.0, 10.0) + 1.0;
        let f_temp2 = (thermal_grid.ambient_temp / 2.0 - temp).clamp(0.0, 10.0) + 1.0;
        let mut sound = 0.0;
        for bpos in bpos.iter_xy_neighbors_nosize(3) {
            sound += sound_grid
                .sound_field
                .get(&bpos)
                .map(|x: &Vec<Vec2>| x.iter().map(|y: &Vec2| y.length()).sum::<f32>())
                .unwrap_or_default()
                * 10.0;
        }
        const MASS: f32 = 10.0;
        if roomdb.room_tiles.contains_key(&bpos) {
            ps.mean_sound =
                ((sound * dt + ps.mean_sound * MASS) / (MASS + dt)).clamp(0.00000001, 100000.0);
        } else {
            // prevent sanity from being lost outside of the location.
            ps.mean_sound /= 1.8_f32.powf(dt);
        }
        let crazy = lux.max(0.00001).recip() / f_temp * f_temp2 * ps.mean_sound * 10.0
            + ps.mean_sound / f_temp * f_temp2;
        let sanity_recover: f32 = if ps.sanity < difficulty.0.max_recoverable_sanity {
            4.0 / 100.0 / difficulty.0.sanity_drain_rate
        } else {
            0.0
        };
        ps.crazyness +=
            (crazy.clamp(0.000000001, 10000000.0).sqrt() * 0.2 * difficulty.0.sanity_drain_rate
                - sanity_recover * ps.crazyness / (1.0 + ps.mean_sound * 10.0))
                * dt;
        if ps.crazyness < 0.0 {
            ps.crazyness = 0.0;
        }
        ps.sanity = calculate_sanity(ps.crazyness);
    }
}

fn health_regen(
    time: Res<Time>,
    mut qp: Query<&mut PlayerSprite, (Without<InTruck>, Without<PlayerSpectating>)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate;
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
    }
}

fn recover_sanity(
    time: Res<Time>,
    mut qp: Query<&mut PlayerSprite, (With<MainPlayer>, With<InTruck>, Without<PlayerSpectating>)>,
    difficulty: Res<CurrentDifficulty>,
) {
    // Players recover sanity while in the truck.
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        // --- Gradual Health Recovery --- Health points recovered per second
        const HEALTH_RECOVERY_RATE: f32 = 2.0;
        if ps.health < 100.0 {
            ps.health += HEALTH_RECOVERY_RATE * dt;

            // Clamp health to a maximum of 100%
            ps.health = ps.health.min(100.0);
        }
        if ps.sanity < difficulty.0.max_recoverable_sanity {
            ps.crazyness /= 1.07_f32.powf(dt);
        } else {
            ps.crazyness /= 1.005_f32.powf(dt);
        }
        ps.sanity = calculate_sanity(ps.crazyness);
    }
}

fn visual_health(
    qp: Query<(&PlayerSprite, Has<PlayerSpectating>), With<MainPlayer>>,
    mut qb: Query<(
        Option<&mut ImageNode>,
        &mut BackgroundColor,
        &DamageBackground,
    )>,
) {
    for (player_sprite, is_spectating) in &qp {
        if is_spectating {
            // Spectator visual effect (desaturated/blue tint)
            for (mut o_uiimage, mut bgcolor, _dmg) in &mut qb {
                // Ignore dmg.exp for spectator, use fixed visual
                let dst_color = Color::srgba(0.0, 0.0, 0.2, 0.4);
                let old_color = o_uiimage.as_ref().map(|x| x.color).unwrap_or(bgcolor.0);
                let new_color = lerp_color(old_color, dst_color, 0.1);
                if old_color != new_color {
                    if let Some(uiimage) = o_uiimage.as_mut() {
                        uiimage.color = new_color;
                    } else {
                        bgcolor.0 = new_color;
                    }
                }
            }
        } else {
            let health = (player_sprite.health.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
            let crazyness = (1.0 - player_sprite.sanity / 100.0).clamp(0.0, 1.0);
            for (mut o_uiimage, mut bgcolor, dmg) in &mut qb {
                let rhealth = (1.0 - health).powf(dmg.exp);
                let crazyness = crazyness.powf(dmg.exp);
                let alpha = ((rhealth * 10.0).clamp(0.0, 0.3) + rhealth.powi(2) * 0.7 + crazyness)
                    .clamp(0.0, 1.0);
                let rhealth2 = (1.0 - alpha * 0.9).clamp(0.0001, 1.0);
                let red = f32::tanh(rhealth * 2.0).clamp(0.0, 1.0) * rhealth2;
                let dst_color = Color::srgba(red, 0.0, 0.0, alpha);
                let old_color = o_uiimage.as_ref().map(|x| x.color).unwrap_or(bgcolor.0);
                let new_color = lerp_color(old_color, dst_color, 0.2);
                if old_color != new_color {
                    if let Some(uiimage) = o_uiimage.as_mut() {
                        uiimage.color = new_color;
                    } else {
                        bgcolor.0 = new_color;
                    }
                }
            }
        }
    }
}

fn update_player_stamina(
    mut players: Query<(&PlayerSprite, &mut Stamina)>,
    difficulty: Res<CurrentDifficulty>,
) {
    for (player_sprite, mut stamina) in players.iter_mut() {
        // Adjust stamina parameters based on health
        let health_percentage = player_sprite.health / 100.0;

        // When health is low, stamina depletes faster and recovers slower
        if health_percentage < 0.3 {
            stamina.depletion_rate = 1.2 * difficulty.0.health_recovery_rate; // Depletes 50% faster when health is critical
            stamina.recovery_rate = 0.15 * difficulty.0.health_recovery_rate; // Recovers 50% slower when health is critical
        } else if health_percentage < 0.6 {
            stamina.depletion_rate = 1.0 * difficulty.0.health_recovery_rate; // Depletes 25% faster when health is low
            stamina.recovery_rate = 0.2 * difficulty.0.health_recovery_rate; // Recovers 33% slower when health is low
        } else {
            // Reset to default rates based on difficulty
            stamina.depletion_rate = 0.8 * difficulty.0.health_recovery_rate;
            stamina.recovery_rate = 0.3 * difficulty.0.health_recovery_rate;
        }
    }
}

use unreplicon_core::messages::PlayerDiedEvent;

fn detect_and_apply_death(
    mut commands: Commands,
    mut player_query: Query<
        (Entity, &mut PlayerSprite, Option<&mut PlayerGear>),
        Without<PlayerSpectating>,
    >,
    mut ev_death: MessageWriter<PlayerDiedEvent>,
) {
    for (entity, player, mut gear) in player_query.iter_mut() {
        if player.health <= 0.0 {
            info!(
                "Player {:?} ({:?}) died! Entering spectate mode.",
                entity, player.id
            );
            commands.entity(entity).insert(PlayerSpectating);

            // Despawn all gear
            if let Some(ref mut gear) = gear {
                if let Some(e) = gear.left_hand {
                    commands.entity(e).despawn();
                }
                if let Some(e) = gear.right_hand {
                    commands.entity(e).despawn();
                }
                for e in gear.inventory.iter() {
                    commands.entity(*e).despawn();
                }
                if let Some(h) = &gear.held_item {
                    commands.entity(h.entity).despawn();
                }
                // Empty the inventory
                **gear = PlayerGear::default();
            }
            ev_death.write(PlayerDiedEvent { id: player.id });
        }
    }
}

fn update_profile_death_stats(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    local_player: Res<unreplicon_core::resources::LocalPlayer>,
    mut summary_data: ResMut<SummaryData>,
    board_topology: Res<BoardTopology>,
    difficulty_res: Res<CurrentDifficulty>,
) {
    for ev in ev_death.read() {
        if local_player.0 == Some(ev.id) {
            // It's us!
            let initial_deposit_held = player_profile.progression.insurance_deposit;

            player_profile.progression.insurance_deposit = 0;
            player_profile.statistics.total_deaths += 1;

            let map_path_str = board_topology.map_path.clone();
            let current_difficulty_variant = difficulty_res.0.difficulty;

            let map_specific_stats = player_profile
                .map_statistics
                .entry(map_path_str.clone())
                .or_default()
                .entry(current_difficulty_variant)
                .or_default();
            map_specific_stats.total_deaths += 1;

            if let Err(e) = player_profile.persist() {
                error!("Failed to persist PlayerProfileData after death: {:?}", e);
            }

            summary_data.map_path = map_path_str;
            summary_data.deposit_originally_held = initial_deposit_held;
            summary_data.deposit_returned_to_bank = 0;
            summary_data.costs_deducted_from_deposit = initial_deposit_held;
            summary_data.money_earned = 0;
            summary_data.grade_achieved = Grade::NA;
        }
    }
}

pub(crate) fn debug_kill_spectator(
    keyboard_input: If<Res<ButtonInput<KeyCode>>>,
    mut player_query: Query<&mut PlayerSprite, With<MainPlayer>>,
) {
    let shift =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    if shift && ctrl && keyboard_input.just_pressed(KeyCode::KeyK) {
        for mut player in player_query.iter_mut() {
            player.health = -10.0;
        }
    }
}

pub(crate) fn server_apply_client_sanity(
    mut q_player: Query<(&PlayerInput, &mut PlayerSprite), Without<MainPlayer>>,
) {
    for (input, mut sprite) in &mut q_player {
        sprite.sanity = input.sanity;
        sprite.mean_sound = input.mean_sound;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use untypes_core::cli::{is_authority, is_headless};
    use untypes_core::states::SimulationState;

    app.add_message::<PlayerDiedEvent>().add_systems(
        Update,
        (
            lose_sanity.run_if(not(is_headless)),
            recover_sanity,
            health_regen.run_if(is_authority),
            server_apply_client_sanity.run_if(is_authority),
            visual_health.run_if(not(is_headless)),
            update_player_stamina,
            detect_and_apply_death,
            update_profile_death_stats.run_if(not(is_headless)),
            debug_kill_spectator,
        )
            .run_if(in_state(SimulationState::Running)),
    );
}
