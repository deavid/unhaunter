use bevy::prelude::*;
use unbehavior_core::components::Movable;
use unlocomotion_core::components::PlayerLocomotionState;
use unplayer_core::components::PlayerSprite;
use unspatial_core::direction::Direction;
use untypes_core::roles::AuthorityRole;
use untypes_core::states::AppState;

/// Authority: inserts PlayerLocomotionState, Direction, and Movable on any player entity
/// that is missing them. Fires on the first Update frame after a PlayerSprite entity is
/// spawned by the network layer.
fn hydrate_player_locomotion(
    mut commands: Commands,
    q_new: Query<Entity, (With<PlayerSprite>, Without<PlayerLocomotionState>)>,
) {
    for entity in q_new.iter() {
        commands.entity(entity).insert((
            PlayerLocomotionState::default(),
            Direction::new_right(),
            Movable,
        ));
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_player_locomotion
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(AppState::InGame)),
    );
}
