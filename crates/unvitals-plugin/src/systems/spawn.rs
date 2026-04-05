use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::resources::AuthorityRole;
use unvitals_core::components::{PlayerVitals, Stamina};

/// Authority: inserts PlayerVitals and Stamina on any player entity that is missing them.
/// Fires on the first Update frame after a PlayerSprite entity is spawned by the network layer,
/// covering both initial mission start and late-joining players.
fn hydrate_player_vitals(
    mut commands: Commands,
    q_new: Query<Entity, (With<PlayerSprite>, Without<PlayerVitals>)>,
) {
    for entity in q_new.iter() {
        commands
            .entity(entity)
            .insert((PlayerVitals::default(), Stamina::default()));
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        hydrate_player_vitals
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(UIContextState::InGame)),
    );
}
