use super::other;
use super::sanity;
use bevy::prelude::*;
use unreplicon_core::messages::PlayerDiedEvent;
use untypes_core::roles::{AuthorityRole, LocalPlayerRole};
use untypes_core::states::SimulationState;

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<PlayerDiedEvent>().add_systems(
        Update,
        (
            sanity::drain_sanity_from_environment.run_if(resource_exists::<LocalPlayerRole>),
            other::recover_sanity_in_truck,
            other::regenerate_health_over_time.run_if(resource_exists::<AuthorityRole>),
            other::sync_client_reported_sanity.run_if(resource_exists::<AuthorityRole>),
            other::update_damage_vignette_color.run_if(resource_exists::<LocalPlayerRole>),
            other::scale_stamina_rates_by_health,
            other::apply_ghost_proximity_damage.run_if(resource_exists::<LocalPlayerRole>),
            other::transition_to_spectator_on_death,
            other::record_death_to_profile.run_if(resource_exists::<LocalPlayerRole>),
            other::debug_kill_spectator,
        )
            .run_if(in_state(SimulationState::Ready)),
    );
}
