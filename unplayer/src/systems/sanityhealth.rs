use uncore::DEBUG_PLAYER;
use uncore::components::board::position::Position;
use uncore::components::game_config::GameConfig;
use uncore::components::game_ui::DamageBackground;
use uncore::components::player::Stamina;
use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::resources::board_data::BoardData;
use uncore::resources::roomdb::RoomDB;
use uncore::utils::PrintingTimer;
use uncore::utils::light::lerp_color;

use bevy::prelude::*;

#[derive(Default)]
pub struct MeanSound(f32);

pub fn lose_sanity(
    time: Res<Time>,
    mut timer: Local<PrintingTimer>,
    mut mean_sound: Local<MeanSound>,
    mut qp: Query<(&mut PlayerSprite, &Position)>,
    bf: Res<BoardData>,
    roomdb: Res<RoomDB>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    timer.tick(time.delta());
    let dt = time.delta_secs();
    for (mut ps, pos) in &mut qp {
        let bpos = pos.to_board_position();
        let p = bpos.ndidx();
        let lux = bf.light_field[p].lux.sqrt() + 0.001;
        let temp = bf.temperature_field[p];
        let f_temp = (temp - bf.ambient_temp / 2.0).clamp(0.0, 10.0) + 1.0;
        let f_temp2 = (bf.ambient_temp / 2.0 - temp).clamp(0.0, 10.0) + 1.0;
        let mut sound = 0.0;
        for bpos in bpos.iter_xy_neighbors_nosize(3) {
            sound += bf
                .sound_field
                .get(&bpos)
                .map(|x| x.iter().map(|y| y.length()).sum::<f32>())
                .unwrap_or_default()
                * 10.0;
        }
        const MASS: f32 = 10.0;
        if roomdb.room_tiles.contains_key(&bpos) {
            mean_sound.0 =
                ((sound * dt + mean_sound.0 * MASS) / (MASS + dt)).clamp(0.00000001, 100000.0);
        } else {
            // prevent sanity from being lost outside of the location.
            mean_sound.0 /= 1.8_f32.powf(dt);
        }
        let crazy =
            lux.recip() / f_temp * f_temp2 * mean_sound.0 * 10.0 + mean_sound.0 / f_temp * f_temp2;
        let sanity_recover: f32 = if ps.sanity() < difficulty.0.max_recoverable_sanity {
            4.0 / 100.0 / difficulty.0.sanity_drain_rate
        } else {
            0.0
        };
        ps.crazyness +=
            (crazy.clamp(0.000000001, 10000000.0).sqrt() * 0.2 * difficulty.0.sanity_drain_rate
                - sanity_recover * ps.crazyness / (1.0 + mean_sound.0 * 10.0))
                * dt;
        if ps.crazyness < 0.0 {
            ps.crazyness = 0.0;
        }
        ps.mean_sound = mean_sound.0;
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate;
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
        if timer.just_finished() && DEBUG_PLAYER {
            dbg!(ps.sanity(), mean_sound.0, ps.health);
        }
    }
}

pub fn recover_sanity(
    time: Res<Time>,
    mut qp: Query<&mut PlayerSprite>,
    gc: Res<GameConfig>,
    mut timer: Local<PrintingTimer>,
    // Access the difficulty settings
    difficulty: Res<CurrentDifficulty>,
) {
    // Current player recovers sanity while in the truck.
    let dt = time.delta_secs();
    timer.tick(time.delta());
    for mut ps in &mut qp {
        if ps.id == gc.player_id {
            // --- Gradual Health Recovery --- Health points recovered per second
            const HEALTH_RECOVERY_RATE: f32 = 2.0;
            if ps.health < 100.0 {
                ps.health += HEALTH_RECOVERY_RATE * dt;

                // Clamp health to a maximum of 100%
                ps.health = ps.health.min(100.0);
            }
            if ps.sanity() < difficulty.0.max_recoverable_sanity {
                ps.crazyness /= 1.07_f32.powf(dt);
            } else {
                ps.crazyness /= 1.005_f32.powf(dt);
            }
            if timer.just_finished() {
                dbg!(ps.sanity());
            }
        }
    }
}

pub fn visual_health(
    qp: Query<&PlayerSprite>,
    gc: Res<GameConfig>,
    mut qb: Query<(
        Option<&mut ImageNode>,
        &mut BackgroundColor,
        &DamageBackground,
    )>,
) {
    for player in &qp {
        if player.id != gc.player_id {
            continue;
        }
        let health = (player.health.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
        let crazyness = (1.0 - player.sanity() / 100.0).clamp(0.0, 1.0);
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

pub fn update_player_stamina(
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
