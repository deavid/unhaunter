use bevy::prelude::*;
use bevy_asset_loader::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use uninvestigation_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use uninvestigation_core::resources::ghost_guess::GhostGuess;
use unghost_core::components::ghost_breach::GhostBreach;
use unghost_core::components::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::tags::GhostTag;
use unorchestrator_core::UIContextState;
use unsensing_core::components::SpectralClarity;

use unghost_core::events::{
    EvidenceClarityThresholdCrossed, GhostActualTypeChanged, GhostInteractionEvent,
    JournalEvidenceToggled, JournalGhostToggled,
};
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;

use crate::{ghost_events, ghost_orb, metrics};
use unghost_core::assets::GhostAssets;

pub struct UnhaunterGhostCorePlugin;

impl Plugin for UnhaunterGhostCorePlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GhostInteractionEvent>();
        app.add_message::<JournalEvidenceToggled>();
        app.add_message::<JournalGhostToggled>();
        app.add_message::<EvidenceClarityThresholdCrossed>();
        app.add_message::<GhostActualTypeChanged>();
        app.replicate::<GhostTag>();
        app.replicate::<GhostBreach>();
        app.replicate::<GhostSprite>();
        app.replicate::<GhostBehaviorDynamics>();
        app.replicate::<GhostGuess>();
        app.replicate::<SpectralClarity>();
        crate::systems::hydration::app_setup(app);
        crate::systems::evidence_decay::app_setup(app);
        crate::systems::ghost_ai::app_setup(app);
        crate::systems::journal::app_setup(app);
        crate::systems::hint_events::app_setup(app);
        ghost_events::app_setup(app);
        metrics::register_all(app);
        app.init_resource::<ObjectInteractionConfig>()
            .init_resource::<HauntState>()
            .init_resource::<CurrentEvidenceReadings>();

        app.add_systems(Startup, setup_noise);
    }
}

pub struct UnhaunterGhostPlugin;

impl Plugin for UnhaunterGhostPlugin {
    fn build(&self, app: &mut App) {
        app.add_loading_state(
            LoadingState::new(UIContextState::EngineBoot).load_collection::<GhostAssets>(),
        );
        ghost_orb::app_setup(app);
    }
}

fn setup_noise(
    local: Option<Res<unreplicon_core::resources::LocalPlayerRole>>,
    mut commands: Commands,
) {
    if local.is_none() {
        commands.insert_resource(unnoise_core::perlin::PerlinNoise::new_low_mem(1));
    } else {
        commands.init_resource::<unnoise_core::perlin::PerlinNoise>();
    }
}
