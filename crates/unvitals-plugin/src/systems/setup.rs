use super::other;
use super::sanity;
use bevy::prelude::*;
use bevy_replicon::prelude::*;
use unmission_core::types::SimulationState;
use unreplicon_core::resources::LocalPlayerRole;
use unvitals_core::components::{PlayerVitals, Stamina};
use unvitals_core::events::PlayerDiedEvent;

pub(crate) fn app_setup(app: &mut App) {
    app.add_message::<PlayerDiedEvent>()
        .replicate::<PlayerVitals>()
        .replicate::<Stamina>()
        .add_systems(
            Update,
            (
                other::update_asphyxia_from_miasma.run_if(resource_exists::<LocalPlayerRole>),
                other::debug_log_asphyxia.run_if(resource_exists::<LocalPlayerRole>),
                sanity::drain_sanity_from_environment.run_if(resource_exists::<LocalPlayerRole>),
                other::recover_sanity_in_truck,
                other::regenerate_health_over_time,
                other::scale_stamina_rates_by_health,
                other::apply_ghost_proximity_damage.run_if(resource_exists::<LocalPlayerRole>),
                other::transition_to_spectator_on_death,
                other::debug_kill_spectator,
            )
                .run_if(in_state(SimulationState::Ready)),
        );
}
