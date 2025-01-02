use crate::behavior::Behavior;
use crate::board::Position;
use crate::gear::playergear::PlayerGear;
use crate::gear::GearStuff;
use crate::maplight::MapColor;
use crate::player::{AnimationTimer, Hiding, PlayerSprite};

use bevy::color::palettes::css;
use bevy::prelude::*;
use bevy::utils::HashMap;

/// Allows the player to hide in a designated hiding spot.
///
/// This system checks if the player is pressing the 'activate' key and is near a
/// valid hiding spot. If so, the player character enters the hiding spot, becoming
/// partially hidden. A visual overlay is added to the hiding spot to indicate the
/// player's presence. Note that the player's transparency while hiding is not yet
/// fully implemented.
#[allow(clippy::type_complexity, clippy::too_many_arguments)]
pub fn hide_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<
        (Entity, &mut PlayerSprite, &mut Position, &PlayerGear),
        (Without<Hiding>, Without<Behavior>),
    >,
    hiding_spots: Query<(Entity, &Position, &Behavior), Without<PlayerSprite>>,
    asset_server: Res<AssetServer>,
    mut gs: GearStuff,
    time: Res<Time>,
    mut hold_timers: Local<HashMap<Entity, Timer>>,
) {
    for (player_entity, player, mut player_pos, player_gear) in players.iter_mut() {
        // Get the player's hold timer or create a new one
        let timer = hold_timers
            .entry(player_entity)
            .or_insert_with(|| Timer::from_seconds(0.3, TimerMode::Once));
        if keyboard_input.pressed(player.controls.activate) {
            if player_gear.held_item.is_some() {
                // Player cannot hide while carrying furniture.
                continue;
            }

            // Using 'activate' for hiding Find a hiding spot near the player
            if let Some((hiding_spot_entity, hiding_spot_pos, _)) = hiding_spots
                .iter()
                // Manually filter for hiding spots
                .filter(|(_, _, behavior)| behavior.p.object.hidingspot)
                .find(|(_, hiding_spot_pos, _)| player_pos.distance(hiding_spot_pos) < 1.0)
            {
                // Key is held down, tick the timer
                timer.tick(time.delta());
                if !timer.finished() {
                    continue;
                }
                timer.reset();

                // Add the Hiding component to the player
                commands
                    .entity(player_entity)
                    .insert(Hiding {
                        hiding_spot: hiding_spot_entity,
                    })
                    .insert(MapColor {
                        color: css::DARK_GRAY.with_alpha(0.5).into(),
                    });
                player_pos.x = (player_pos.x + hiding_spot_pos.x) / 2.0;
                player_pos.y = (player_pos.y + hiding_spot_pos.y) / 2.0;

                // Play "Hide" sound effect
                gs.play_audio("sounds/hide-rustle.ogg".into(), 1.0, &player_pos);

                // Add Visual Overlay
                commands.entity(hiding_spot_entity).with_children(|parent| {
                    parent
                        .spawn(Sprite {
                            image: asset_server.load("img/hiding_overlay.png"),
                            color: css::WHITE.with_alpha(0.4).into(),
                            ..default()
                        })
                        .insert(
                            // Position relative to parent
                            Transform::from_xyz(0.0, 0.0, 0.02)
                                .with_scale(Vec3::new(0.20, 0.20, 0.20)),
                        );
                });
            }
        } else {
            timer.reset();
        }
    }
}

/// Allows the player to leave a hiding spot.
///
/// This system checks if the player is pressing the 'activate' key and is
/// currently hiding. If so, the player character exits the hiding spot, their
/// visibility is restored, and the visual overlay is removed from the hiding spot.
pub fn unhide_player(
    mut commands: Commands,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut players: Query<(
        Entity,
        &mut PlayerSprite,
        &mut Transform,
        &mut Visibility,
        &Hiding,
    )>,
) {
    for (player_entity, player, _, _visibility, hiding) in players.iter_mut() {
        if keyboard_input.just_pressed(player.controls.activate) {
            // Using 'activate' for unhiding Remove the Hiding component
            commands.entity(player_entity).remove::<Hiding>();

            // Reset player sprite animation TODO: Define default animation For now, let's
            // just set it back to the standing animation (index 32)
            commands
                .entity(player_entity)
                .insert(AnimationTimer::from_range(
                    Timer::from_seconds(0.20, TimerMode::Repeating),
                    vec![32],
                ))
                .insert(MapColor {
                    color: Color::WHITE.with_alpha(1.0),
                });

            // Reset player position TODO: Consider using the hiding spot's position For now,
            // let's just leave the position as is. Reset player visibility *visibility =
            // Visibility::Visible; --- Remove Visual Overlay ---
            commands.entity(hiding.hiding_spot).despawn_descendants();
        }
    }
}
