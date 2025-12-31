use crate::resources::ambient_mute::AmbientMuteController;
use bevy::prelude::*;
use bevy_persistent::Persistent;
use ndarray::s;
use uncore_board::resources::roomdb::RoomDB;
use uncore_events::events::ambient_sound_mute::AmbientSoundMuteEvent;
use uncore_foundation::types::sound::SoundType;
use uncore_types::states::AppState;
use unplayer_core::components::PlayerSprite;
use unrender_std::VisibilityData;
use unrender_std::components::game::GameSound;
use unsettings_core::audio::AudioSettings;
use unspatial_core::{BoardPosition, Position};

/// Calculates the ambient sound volumes based on player visibility.
///
/// # Arguments
///
/// * `vf` - A reference to the `VisibilityData` resource.
/// * `roomdb` - A reference to the `RoomDB` resource.
/// * `player_bpos` - The player's board position.
///
/// # Returns
///
/// A tuple containing the calculated `house_volume` and `street_volume`.
fn calculate_ambient_sound_volumes(
    vf: &VisibilityData,
    roomdb: &RoomDB,
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
            v * match roomdb.room_tiles.get(&k).is_some() {
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
/// 4. Applies logarithmic smoothing (IIR filter in log space) for perceptual volume transitions
/// 5. Applies audio settings (volume_ambient, volume_master)
/// 6. Applies mute effects from the ambient mute controller
/// 7. Updates the actual AudioSink volumes for GameSound entities
fn update_ambient_sound_volumes(
    mut game_sound_query: Query<(&GameSound, &mut AudioSink)>,
    player_query: Query<(&Position, &PlayerSprite), With<PlayerSprite>>,
    visibility_data: Res<VisibilityData>,
    roomdb: Res<RoomDB>,
    audio_settings: Res<Persistent<AudioSettings>>,
    ambient_mute_controller: Res<AmbientMuteController>,
) {
    // Get player position and sprite data
    let Some((player_pos, player_sprite)) = player_query.iter().next() else {
        return;
    };
    let player_bpos = player_pos.to_board_position();

    // Calculate the base ambient volumes
    let (house_volume, street_volume) =
        calculate_ambient_sound_volumes(&visibility_data, &roomdb, &player_bpos);

    // Calculate HeartBeat volume based on player health (analog/fuzzy logic)
    // HeartBeat should get louder as health gets lower
    let health_ratio = (player_sprite.health / 100.0).clamp(0.0, 1.0);
    let heartbeat_volume = if health_ratio < 0.5 {
        // Health is below 50%, calculate heartbeat intensity
        let health_deficit = 1.0 - health_ratio; // 0.5 to 1.0
        let intensity: f32 = ((health_deficit - 0.5) * 2.0).clamp(0.0, 1.0); // 0.0 to 1.0 when health 50% to 0%
        intensity.powf(0.8) // Smooth curve, gets louder faster as health drops
    } else {
        0.0 // No heartbeat when health is above 50%
    };

    // Calculate Insane volume based on player sanity (analog/fuzzy logic)
    // Insane sounds should get louder as sanity gets lower
    let sanity_ratio = (player_sprite.sanity() / 100.0).clamp(0.0, 1.0);
    let insane_volume = if sanity_ratio < 0.7 {
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

    // Original IIR smoothing constant (simple and robust)
    const SMOOTH: f32 = 60.0;
    let volume_factor = 2.0 * master_volume_setting * ambient_volume_setting;

    // Update each ambient sound entity
    for (game_sound, mut audio_sink) in &mut game_sound_query {
        let base_volume = match game_sound.class {
            SoundType::BackgroundHouse => house_volume,
            SoundType::BackgroundStreet => street_volume,
            SoundType::HeartBeat => heartbeat_volume,
            SoundType::Insane => insane_volume,
        };

        // Calculate target volume: base * mute (settings are applied in volume_factor)
        let calculated_volume = base_volume * mute_multiplier;

        // Apply original logarithmic smoothing logic (reads current volume from AudioSink)
        let ln_volume = (audio_sink.volume().to_linear() / (volume_factor + 0.0000001) + 0.000001)
            .max(0.000001)
            .ln();
        let v = (ln_volume * SMOOTH + calculated_volume.ln()) / (SMOOTH + 1.0);
        let new_volume = v.exp() * volume_factor;

        // Apply to audio sink
        audio_sink.set_volume(bevy::audio::Volume::Linear(new_volume.clamp(0.00001, 10.0)));
    }
}

/// Sets up the ambient sound systems for the application.
/// Registers the mute controller resource, mute events, and ambient sound volume systems.
pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<AmbientMuteController>();
    app.add_message::<AmbientSoundMuteEvent>();
    app.add_systems(
        Update,
        (
            crate::systems::ambient_sound_mute::process_ambient_mute_events
                .run_if(in_state(AppState::InGame)),
            update_ambient_sound_volumes.run_if(in_state(AppState::InGame)),
        ),
    );
}
