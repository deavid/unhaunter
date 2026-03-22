use crate::components::player::Hiding;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use unaudiospatial_core::emitter::AudioEmitter;
use unbehavior::behavior::Behavior;
use ungear_core::components::playergear::PlayerGear;
use unplayer_core::components::{MainPlayer, PlayerInput, PlayerSprite};
use unrender_std::components::animation::AnimationTimer;
use unrender_std::components::visuals::ResolutionFactor;
use unspatial_core::position::Position;

/// Component to tag the hiding overlay visual, linking it to the player.
#[derive(Component)]
struct HidingOverlay {
    player: Entity,
}

/// Allows the player to hide in a designated hiding spot.
///
/// This system checks if the player is pressing the 'activate' key and is near a
/// valid hiding spot. If so, the player character enters the hiding spot, becoming
/// partially hidden. A visual overlay is added to the hiding spot to indicate the
/// player's presence.
fn enter_hidespot(
    mut commands: Commands,
    mut players: Query<
        (Entity, &PlayerInput, &mut Position, &PlayerGear),
        (With<MainPlayer>, Without<Hiding>, Without<Behavior>),
    >,
    hiding_spots: Query<
        (Entity, &Position, &Behavior, Option<&ResolutionFactor>),
        Without<PlayerSprite>,
    >,
    mut ga: AudioEmitter,
    mut hold_timers: Local<HashMap<Entity, Timer>>,
) {
    for (player_entity, player_input, mut player_pos, player_gear) in players.iter_mut() {
        // Get the player's hold timer or create a new one
        let timer = hold_timers
            .entry(player_entity)
            .or_insert_with(|| Timer::from_seconds(0.3, TimerMode::Once));
        if player_input.hide_requested {
            if player_gear.held_item.is_some() {
                // Player cannot hide while carrying furniture.
                continue;
            }

            // Using 'activate' for hiding Find a hiding spot near the player
            if let Some((hiding_spot_entity, hiding_spot_pos, _, rf)) = hiding_spots
                .iter()
                // Manually filter for hiding spots
                .filter(|(_, _, behavior, _)| behavior.p.object.hidingspot)
                .find(
                    |(_, hiding_spot_pos, _, _): &(
                        Entity,
                        &Position,
                        &Behavior,
                        Option<&ResolutionFactor>,
                    )| player_pos.distance(hiding_spot_pos) < 1.3,
                )
            {
                // Key is held down, tick the timer
                timer.tick(ga.time.delta());
                if !timer.is_finished() {
                    continue;
                }
                timer.reset();

                // Add the Hiding component to the player
                commands.entity(player_entity).insert(Hiding {
                    hiding_spot: Some(hiding_spot_entity),
                });
                player_pos.x = (player_pos.x + hiding_spot_pos.x) / 2.0;
                player_pos.y = (player_pos.y + hiding_spot_pos.y) / 2.0;

                // Play "Hide" sound effect
                ga.play_audio("sounds/hide-rustle.ogg".into(), 1.0, &player_pos);

                let upscale_f = rf.map(|r| r.0).unwrap_or(1.0);

                // Add Visual Overlay
                commands.entity(hiding_spot_entity).with_children(|parent| {
                    parent
                        .spawn(Sprite {
                            image: ga.asset_server.load("img/hiding_overlay.png"),
                            color: Color::WHITE.with_alpha(0.4),
                            ..default()
                        })
                        .insert(
                            // Position relative to parent
                            Transform::from_xyz(0.0, 0.0, 0.02)
                                .with_scale(Vec3::splat(0.20 * upscale_f)),
                        )
                        .insert(HidingOverlay {
                            player: player_entity,
                        });
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
fn exit_hidespot(
    mut commands: Commands,
    mut players: Query<(Entity, &PlayerInput, &Hiding), (With<MainPlayer>, With<PlayerSprite>)>,
) {
    for (player_entity, player_input, _) in players.iter_mut() {
        if player_input.unhide_requested {
            // Using 'activate' for unhiding Remove the Hiding component
            commands.entity(player_entity).remove::<Hiding>();

            commands
                .entity(player_entity)
                .insert(AnimationTimer::from_range(
                    Timer::from_seconds(0.20, TimerMode::Repeating),
                    vec![32],
                ));
        }
    }
}

/// System to cleanup hiding overlays when the player stops hiding.
///
/// This handles cleaning up visuals regardless of how the player stopped hiding
/// (input or network sync).
fn despawn_hidespot_overlays(
    mut commands: Commands,
    overlays: Query<(Entity, &HidingOverlay)>,
    players: Query<&Hiding>,
) {
    for (overlay_entity, overlay) in overlays.iter() {
        if !players.contains(overlay.player) {
            commands.entity(overlay_entity).despawn();
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (enter_hidespot, exit_hidespot, despawn_hidespot_overlays)
            .run_if(in_state(untypes_core::states::GameState::Running)),
    );
}
