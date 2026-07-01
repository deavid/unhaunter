use bevy::prelude::*;
use bevy_replicon::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::types::SimulationState;

use crate::metrics;
use unfog_core::resources::MiasmaConfig;

pub struct UnhaunterFogCorePlugin;

impl Plugin for UnhaunterFogCorePlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<MiasmaConfig>();

        app.add_systems(Update, crate::systems::init_miasma_grid);
        app.add_systems(
            Update,
            crate::systems::initialize_miasma
                .run_if(bevy::prelude::on_message::<unmission_core::events::LevelReadyEvent>)
                .after(crate::systems::init_miasma_grid),
        );
        app.add_systems(
            Update,
            crate::systems::update_miasma.run_if(in_state(SimulationState::Ready)),
        );
        app.add_systems(
            Update,
            crate::systems::diffuse_smoke_field
                .run_if(in_state(SimulationState::Ready))
                .after(crate::systems::update_miasma),
        );
        app.add_systems(
            Update,
            crate::systems::apply_flashlight_miasma_effects
                .run_if(in_state(SimulationState::Ready))
                .after(crate::systems::diffuse_smoke_field),
        );
        app.replicate::<unfog_core::components::MiasmaHazardParticle>();
        app.add_systems(
            Update,
            (
                crate::systems::server_spawn_miasma_hazards,
                crate::systems::update_miasma_hazards,
                crate::systems::miasma_hazard_damage,
            )
                .run_if(in_state(SimulationState::Ready))
                .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>),
        );
        app.add_message::<unfog_core::messages::MiasmaTakeDamageMessage>();
        app.add_client_message::<unfog_core::messages::RequestSpawnHazardParticle>(
            bevy_replicon::prelude::Channel::Ordered,
        );
        app.add_systems(
            OnExit(UIContextState::InGame),
            crate::systems::reset_miasma_grid,
        );

        metrics::register_all(app);
    }
}

pub struct UnhaunterFogPlugin;

impl Plugin for UnhaunterFogPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                crate::systems::spawn_miasma,
                crate::systems::animate_miasma_sprites,
                crate::systems::hydrate_miasma_hazards,
                crate::systems::spawn_static_sparks,
                crate::systems::update_static_sparks,
                crate::systems::client_request_miasma_hazards,
                crate::systems::miasma_player_attraction.after(crate::systems::update_miasma),
            )
                .run_if(in_state(UIContextState::InGame)),
        );
    }
}
