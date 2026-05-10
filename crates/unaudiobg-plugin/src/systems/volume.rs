use bevy::prelude::*;
use bevy_persistent::Persistent;
use bevy_seedling::prelude::*;
use ndarray::s;
use unboard_core::resources::roomdb::RoomTopology;
use unboard_core::resources::visibility_data::VisibilityData;
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unsettings_core::audio::AudioSettings;
use unspatial_core::boardposition::BoardPosition;
use unspatial_core::position::Position;
use unvitals_core::components::PlayerVitals;

use unaudiobg_core::components::{GameSound, SoundType};
use unaudiobg_core::mute::AmbientMuteController;
use unaudiobg_core::smooth::smooth_volume;

/// Calculates the ambient sound volumes based on player visibility.
///
/// # Arguments
///
/// * `vf` - A reference to the `VisibilityData` resource.
/// * `room_topology` - A reference to the `RoomTopology` resource.
/// * `player_bpos` - The player's board position.
///
/// # Returns
///
/// A tuple containing the calculated `house_volume` and `street_volume`.
fn calculate_ambient_sound_volumes(
    vf: &VisibilityData,
    room_topology: &RoomTopology,
    player_bpos: &BoardPosition,
) -> (f32, f32) {
    // Check if visibility field is properly initialized
    let (map_width, map_height, map_depth) = vf.visibility_field.dim();
    if map_width == 0 || map_height == 0 || map_depth == 0 {
        // Return default volumes if visibility field is not ready
        return (0.1, 0.1);
    }

    // Define a radius around the player
    const RADIUS: usize = 32;

    // Calculate bounds for our slice
    let player_ndidx = player_bpos.ndidx();

    // Ensure we don't cause integer underflow
    let min_x = player_ndidx.0.saturating_sub(RADIUS);
    let max_x = (player_ndidx.0 + RADIUS).min(map_width.saturating_sub(1));
    let min_y = player_ndidx.1.saturating_sub(RADIUS);
    let max_y = (player_ndidx.1 + RADIUS).min(map_height.saturating_sub(1));
    let z = player_ndidx.2.clamp(0, map_depth.saturating_sub(1));

    // Ensure valid bounds before slicing
    if min_x > max_x || min_y > max_y {
        return (0.1, 0.1);
    }

    // Calculate total_vis only for the subslice
    let total_vis: f32 = vf
        .visibility_field
        .slice(s![min_x..=max_x, min_y..=max_y, z..=z])
        .indexed_iter()
        .map(|(rel_idx, v)| {
            // Convert relative indices back to absolute indices correctly
            let abs_x = rel_idx.0 + min_x;
            let abs_y = rel_idx.1 + min_y;
            let abs_z = z; // Z is constant, use the original z value
            let k = BoardPosition::from_ndidx((abs_x, abs_y, abs_z));
            v * match room_topology.room_tiles.get(&k).is_some() {
                true => 0.2,
                false => 1.0,
            }
        })
        .sum();

    let house_volume = (20.0 / total_vis.max(1.0))
        .powi(3)
        .tanh()
        .clamp(0.00001, 0.9999)
        * 6.0;
    let street_volume = (total_vis / 20.0).powi(3).tanh().clamp(0.00001, 0.9999) * 6.0;

    (house_volume, street_volume)
}

/// System that updates ambient sound volumes based on player visibility,
/// audio settings, and mute effects.
///
/// This system:
/// 1. Calculates ambient sound volumes based on player visibility
/// 2. Calculates HeartBeat volume based on player health (analog/fuzzy logic)
/// 3. Calculates Insane volume based on player sanity (analog/fuzzy logic)
/// 4. Applies dB-based smoothing for perceptual volume transitions
/// 5. Applies audio settings (volume_ambient, volume_master)
/// 6. Applies mute effects from the ambient mute controller
/// 7. Updates the actual AudioSink volumes for GameSound entities
pub(crate) fn update_ambient_sound_volumes(
    game_sound_query: Query<(&GameSound, &SampleEffects)>,
    mut q_volume: Query<&mut VolumeNode>,
    player_query: Query<
        (
            &Position,
            &PlayerVitals,
            &VisibilityData,
            Has<PlayerSpectating>,
        ),
        With<MainPlayer>,
    >,
    room_topology: Res<RoomTopology>,
    audio_settings: Res<Persistent<AudioSettings>>,
    ambient_mute_controller: Res<AmbientMuteController>,
    time: Res<Time>,
) {
    // Get player position and viewer data
    let Ok((player_pos, vitals, visibility_data, is_spectating)) = player_query.single() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    // Calculate the base ambient volumes
    let (house_volume, street_volume) =
        calculate_ambient_sound_volumes(visibility_data, &room_topology, &player_bpos);

    // Calculate HeartBeat volume based on health (analog/fuzzy logic)
    // HeartBeat should get louder as health gets lower
    let health_ratio = (vitals.health / 100.0).clamp(0.0, 1.0);
    let heartbeat_volume = if !is_spectating && health_ratio < 0.5 {
        // Health is below 50%, calculate heartbeat intensity
        let health_deficit = 1.0 - health_ratio; // 0.5 to 1.0
        let intensity: f32 = ((health_deficit - 0.5) * 2.0).clamp(0.0, 1.0); // 0.0 to 1.0 when health 50% to 0%
        intensity.powf(0.8) // Smooth curve, gets louder faster as health drops
    } else {
        0.0 // No heartbeat when health is above 50%
    };

    // Calculate Insane volume based on sanity (analog/fuzzy logic)
    // Insane sounds should get louder as sanity gets lower
    let sanity_ratio = (vitals.sanity / 100.0).clamp(0.0, 1.0);
    let insane_volume = if !is_spectating && sanity_ratio < 0.7 {
        // Sanity is below 70%, calculate insane sound intensity
        let sanity_deficit = 1.0 - sanity_ratio; // 0.3 to 1.0
        let intensity: f32 = ((sanity_deficit - 0.3) / 0.7).clamp(0.0, 1.0); // 0.0 to 1.0 when sanity 70% to 0%
        intensity.powf(0.6) // Smooth curve, gets louder as sanity drops
    } else {
        0.0 // No insane sounds when sanity is above 70%
    };

    // Apply audio settings
    let ambient_volume_setting = audio_settings.volume_ambient.as_f32();
    let master_volume_setting = audio_settings.volume_master.as_f32();

    // Apply mute effects (multiplicative)
    let mute_multiplier = ambient_mute_controller.current_multiplier();

    // Volume scaling factors
    let volume_factor = 2.0 * master_volume_setting * ambient_volume_setting;

    // Unified perceptual smoothing: 2.0 perceptual units per second (0 to 1 in 500ms) for most sounds
    let dt_secs = time.delta_secs();

    // Update each ambient sound entity
    for (game_sound, effects) in &game_sound_query {
        if let Ok(mut volume_node) = q_volume.get_effect_mut(effects) {
            let (base_volume, speed) = match game_sound.class {
                SoundType::BackgroundHouse => (house_volume, 0.4), // 5x slower (2.5s)
                SoundType::BackgroundStreet => (street_volume, 0.4), // 5x slower (2.5s)
                SoundType::HeartBeat => (heartbeat_volume, 2.0),
                SoundType::Insane => (insane_volume, 2.0),
            };

            // Calculate target volume: base * mute (settings are applied in volume_factor)
            let calculated_volume = base_volume * mute_multiplier;

            // Apply cubic-based smoothing for all tracks
            let current_linear = volume_node.volume.linear();
            let target_linear = calculated_volume * volume_factor;
            let new_volume = smooth_volume(current_linear, target_linear, speed, dt_secs);

            // Apply to volume node
            volume_node.volume = Volume::Linear(new_volume.clamp(0.00001, 10.0));
        }
    }
}
