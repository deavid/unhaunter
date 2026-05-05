use bevy::prelude::*;
use uninput_core::components::PlayerInput;
use unplayer_core::components::{MainPlayer, PlayerNameLabel, PlayerSprite};
use unreplicon_core::identity::generate_deterministic_name;
use unspatial_core::position::Position;

/// Marker for the hydrated player name label.
#[derive(Component)]
struct PlayerNameLabelHydrated;

fn hydrate_player_name_system(
    mut commands: Commands,
    q_players: Query<(Entity, &PlayerSprite), Without<PlayerNameLabelHydrated>>,
) {
    for (entity, player_sprite) in q_players.iter() {
        let name = generate_deterministic_name(player_sprite.id);
        commands.entity(entity).with_children(|parent| {
            parent
                .spawn((
                    Text2d::new(name),
                    TextFont {
                        font_size: 16.0,
                        ..default()
                    },
                    TextColor(Color::WHITE),
                    PlayerNameLabel,
                    // Position it below the character's feet.
                    // The character anchor is roughly (0.0, -0.4) in normalized sprite coords.
                    // In screen pixels it depends on rf, but (0, -18, 0.01) is a decent start.
                    Transform::from_xyz(0.0, -18.0, 0.01),
                    Visibility::Hidden,
                ))
                .insert(bevy::sprite::Anchor(Vec2::new(0.0, 0.5))); // TopCenter
        });
        commands.entity(entity).insert(PlayerNameLabelHydrated);
    }
}

fn update_player_name_visibility_system(
    q_main_player: Query<(Entity, &Position, &PlayerInput), With<MainPlayer>>,
    q_other_players: Query<(Entity, &Position), (With<PlayerSprite>, Without<MainPlayer>)>,
    mut q_labels: Query<(&ChildOf, &mut Visibility), With<PlayerNameLabel>>,
) {
    let Ok((main_entity, main_pos, main_input)) = q_main_player.single() else {
        return;
    };

    let aim_dir = main_input.aim_direction.normalize_or_zero();

    let mut best_target: Option<(Entity, f32)> = None;

    for (entity, pos) in q_other_players.iter() {
        let delta = pos.delta(*main_pos);
        let dist = delta.distance();
        if dist > 8.0 {
            continue;
        }
        let dir = Vec2::new(delta.dx, delta.dy).normalize_or_zero();
        let dot = aim_dir.dot(dir);

        // Score based on aiming (dot product) and proximity.
        // Higher dot and lower distance result in a higher score.
        if dot > 0.8 {
            let score = dot / (dist + 1.0);
            if let Some((_, best_score)) = best_target {
                if score > best_score {
                    best_target = Some((entity, score));
                }
            } else {
                best_target = Some((entity, score));
            }
        }
    }

    for (parent, mut visibility) in q_labels.iter_mut() {
        if parent.parent() == main_entity {
            *visibility = Visibility::Inherited;
            continue;
        }

        if let Some((target_entity, _)) = best_target {
            if parent.parent() == target_entity {
                *visibility = Visibility::Inherited;
            } else {
                *visibility = Visibility::Hidden;
            }
        } else {
            *visibility = Visibility::Hidden;
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (hydrate_player_name_system, update_player_name_visibility_system)
            .run_if(in_state(uncommon_states_core::UIContextState::InGame)),
    );
}
