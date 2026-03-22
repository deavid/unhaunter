use bevy::prelude::*;
use unplayer_core::components::{PlayerVitals, PlayerSprite, MainPlayer, PlayerInput, PlayerSpectating, Stamina};
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unsoundfield_core::resources::SoundGrid;
use unlight_core::resources::light_grid::LightGrid;
use unboard_core::resources::roomdb::RoomTopology;
use unthermal_core::resources::ThermalGrid;
use unsummary_core::summary::SummaryData;
use unsummary_core::grade::Grade;
use unghost_core::components::ghost_sprite::GhostSprite;
use untags_core::tags::GhostTag;
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::messages::PlayerDiedEvent;
use unprofile_core::profile::PlayerProfileData;
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use unui_core::components::game_ui::DamageBackground;
use unrender_std::utils::light::lerp_color;

pub fn calculate_sanity(crazyness: f32) -> f32 {
    const LINEAR: f32 = 30.0;
    const SCALE: f32 = 100.0;
    (SCALE * LINEAR) / ((crazyness + LINEAR * LINEAR).max(0.01).sqrt())
}

pub(crate) fn drain_sanity_from_environment(
    time: Res<Time>,
    mut qp: Query<
        (&mut PlayerVitals, &Position),
        (
            With<MainPlayer>,
            Without<InTruck>,
            Without<PlayerSpectating>,
        ),
    >,
    thermal_grid: Option<Res<ThermalGrid>>,
    sound_grid: Option<Res<SoundGrid>>,
    lg: Option<Res<LightGrid>>,
    room_topology: Res<RoomTopology>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    let (Some(thermal_grid), Some(sound_grid), Some(lg)) = (thermal_grid, sound_grid, lg) else {
        return;
    };
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
        if room_topology.room_tiles.contains_key(&bpos) {
            ps.mean_sound =
                ((sound * dt + ps.mean_sound * MASS) / (MASS + dt)).clamp(0.00000001, 100000.0);
        } else {
            // prevent sanity from being lost outside of the location.
            ps.mean_sound /= 1.8_f32.powf(dt);
        }
        let crazy = lux.max(0.00001).recip() / f_temp * f_temp2 * ps.mean_sound * 10.0
            + ps.mean_sound / f_temp * f_temp2;
        let sanity_recover: f32 = if ps.sanity < difficulty.0.max_recoverable_sanity() {
            4.0 / 100.0 / difficulty.0.sanity_drain_rate()
        } else {
            0.0
        };
        ps.crazyness +=
            (crazy.clamp(0.000000001, 10000000.0).sqrt() * 0.2 * difficulty.0.sanity_drain_rate()
                - sanity_recover * ps.crazyness / (1.0 + ps.mean_sound * 10.0))
                * dt;
        if ps.crazyness < 0.0 {
            ps.crazyness = 0.0;
        }
        ps.sanity = calculate_sanity(ps.crazyness);
    }
}

pub(crate) fn regenerate_health_over_time(
    time: Res<Time>,
    mut qp: Query<&mut PlayerVitals, (Without<InTruck>, Without<PlayerSpectating>)>,
    difficulty: Res<CurrentDifficulty>,
) {
    let dt = time.delta_secs();
    for mut ps in &mut qp {
        if ps.health < 100.0 && ps.health > 0.0 {
            ps.health += (0.1 * dt + (1.0 - ps.health / 100.0) * dt * 10.0)
                * difficulty.0.health_recovery_rate();
        }
        if ps.health > 100.0 {
            ps.health = 100.0;
        }
    }
}

pub(crate) fn recover_sanity_in_truck(
    time: Res<Time>,
    mut qp: Query<&mut PlayerVitals, (With<MainPlayer>, With<InTruck>, Without<PlayerSpectating>)>,
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
        if ps.sanity < difficulty.0.max_recoverable_sanity() {
            ps.crazyness /= 1.07_f32.powf(dt);
        } else {
            ps.crazyness /= 1.005_f32.powf(dt);
        }
        ps.sanity = calculate_sanity(ps.crazyness);
    }
}

pub(crate) fn update_damage_vignette_color(
    qp: Query<(&PlayerVitals, Has<PlayerSpectating>), With<MainPlayer>>,
    mut qb: Query<(
        Option<&mut ImageNode>,
        &mut BackgroundColor,
        &DamageBackground,
    )>,
) {
    for (player_vitals, is_spectating) in &qp {
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
            let health = (player_vitals.health.clamp(0.0, 100.0) / 100.0).clamp(0.0, 1.0);
            let crazyness = (1.0 - player_vitals.sanity / 100.0).clamp(0.0, 1.0);
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

pub(crate) fn scale_stamina_rates_by_health(
    mut players: Query<(&PlayerVitals, &mut Stamina)>,
    difficulty: Res<CurrentDifficulty>,
) {
    for (player_vitals, mut stamina) in players.iter_mut() {
        // Adjust stamina parameters based on health
        let health_percentage = player_vitals.health / 100.0;

        // When health is low, stamina depletes faster and recovers slower
        if health_percentage < 0.3 {
            stamina.depletion_rate = 1.2 * difficulty.0.health_recovery_rate(); // Depletes 50% faster when health is critical
            stamina.recovery_rate = 0.15 * difficulty.0.health_recovery_rate(); // Recovers 50% slower when health is critical
        } else if health_percentage < 0.6 {
            stamina.depletion_rate = 1.0 * difficulty.0.health_recovery_rate(); // Depletes 25% faster when health is low
            stamina.recovery_rate = 0.2 * difficulty.0.health_recovery_rate(); // Recovers 33% slower when health is low
        } else {
            // Reset to default rates based on difficulty
            stamina.depletion_rate = 0.8 * difficulty.0.health_recovery_rate();
            stamina.recovery_rate = 0.3 * difficulty.0.health_recovery_rate();
        }
    }
}

pub(crate) fn transition_to_spectator_on_death(
    mut commands: Commands,
    mut player_query: Query<
        (
            Entity,
            &PlayerSprite,
            &PlayerVitals,
        ),
        Without<PlayerSpectating>,
    >,
    mut ev_death: MessageWriter<PlayerDiedEvent>,
) {
    for (entity, player, vitals) in player_query.iter_mut() {
        if vitals.health <= 0.0 {
            info!(
                "Player {:?} ({:?}) died! Entering spectate mode.",
                entity, player.id
            );
            commands.entity(entity).insert(PlayerSpectating);

            ev_death.write(PlayerDiedEvent {
                id: player.network_id,
            });
        }
    }
}

pub(crate) fn record_death_to_profile(
    mut ev_death: MessageReader<PlayerDiedEvent>,
    mut player_profile: ResMut<Persistent<PlayerProfileData>>,
    local_player: Res<unreplicon_core::resources::LocalPlayer>,
    q_players: Query<&PlayerSprite>,
    mut summary_data: ResMut<SummaryData>,
    board_topology: Res<BoardTopology>,
    difficulty_res: Res<CurrentDifficulty>,
) {
    for ev in ev_death.read() {
        let player_uuid = q_players
            .iter()
            .find(|p| p.network_id == ev.id)
            .map(|p| p.id);

        if local_player.0 == player_uuid && player_uuid.is_some() {
            // It's us!
            let initial_deposit_held = player_profile.progression.insurance_deposit;

            player_profile.progression.insurance_deposit = 0;
            player_profile.statistics.total_deaths += 1;

            let map_path_str = board_topology.map_path.clone();
            let current_difficulty_variant = difficulty_res.0;

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
    keyboard_input: Option<Res<ButtonInput<KeyCode>>>,
    mut player_query: Query<&mut PlayerVitals, With<MainPlayer>>,
) {
    let Some(keyboard_input) = keyboard_input else {
        return;
    };
    let shift =
        keyboard_input.pressed(KeyCode::ShiftLeft) || keyboard_input.pressed(KeyCode::ShiftRight);
    let ctrl = keyboard_input.pressed(KeyCode::ControlLeft)
        || keyboard_input.pressed(KeyCode::ControlRight);

    if shift && ctrl && keyboard_input.just_pressed(KeyCode::KeyK) {
        for mut vitals in player_query.iter_mut() {
            vitals.health = -10.0;
        }
    }
}

pub(crate) fn sync_client_reported_sanity(
    mut q_player: Query<(&PlayerInput, &mut PlayerVitals), Without<MainPlayer>>,
) {
    for (input, mut vitals) in &mut q_player {
        vitals.sanity = input.sanity;
        vitals.mean_sound = input.mean_sound;
    }
}

pub(crate) fn apply_ghost_proximity_damage(
    mut q_local_player: Query<
        (&Position, &mut PlayerVitals),
        (
            With<LocallyOwned>,
            Without<PlayerSpectating>,
            Without<InTruck>,
        ),
    >,
    q_ghost: Query<(&Position, &GhostSprite), With<GhostTag>>,
    time: Res<Time>,
    difficulty: Res<CurrentDifficulty>,
    mut hunt_start: Local<f32>,
) {
    let dt = time.delta_secs();

    let Ok((player_pos, mut vitals)) = q_local_player.single_mut() else {
        return;
    };

    let any_hunting = q_ghost.iter().any(|(_, g)| g.hunt_target);

    if !any_hunting {
        *hunt_start = 0.0;
    }

    for (ghost_pos, ghost) in q_ghost.iter() {
        if !ghost.hunt_target {
            continue;
        }

        if *hunt_start == 0.0 {
            *hunt_start = time.elapsed_secs();
        }
        let ghost_strength = (time.elapsed_secs() - *hunt_start).clamp(0.0, 2.0);

        let dist2 = player_pos.weighted_distance_squared(ghost_pos) + 2.0;

        let dmg = dist2.recip() * difficulty.0.health_drain_rate();
        let damage_to_apply = dmg * dt * 30.0 * ghost_strength / (1.0 + ghost.calm_time_secs / 5.0);
        vitals.health -= damage_to_apply;
    }
}

pub(crate) fn app_setup(app: &mut App) {
    use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
    use untypes_core::states::SimulationState;
    use unreplicon_core::messages::PlayerDiedEvent;

    app.add_message::<PlayerDiedEvent>().add_systems(
        Update,
        (
            drain_sanity_from_environment.run_if(resource_exists::<LocalPlayerRole>),
            recover_sanity_in_truck,
            regenerate_health_over_time.run_if(resource_exists::<AuthorityRole>),
            sync_client_reported_sanity.run_if(resource_exists::<AuthorityRole>),
            update_damage_vignette_color.run_if(resource_exists::<LocalPlayerRole>),
            scale_stamina_rates_by_health,
            apply_ghost_proximity_damage.run_if(resource_exists::<LocalPlayerRole>),
            transition_to_spectator_on_death,
            record_death_to_profile.run_if(resource_exists::<LocalPlayerRole>),
            debug_kill_spectator,
        )
            .run_if(in_state(SimulationState::Ready)),
    );
}
