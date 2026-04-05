use bevy::prelude::*;
use bevy_replicon::prelude::AppRuleExt;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::logic::ghost_influence::GhostInfluence;
use unghost_core::components::logic::ghost_sprite::{GhostBehaviorDynamics, GhostSprite};
use unghost_core::components::logic::interaction::{InteractionMotion, Locked};
use unghost_core::components::presentation::spectral::SpectralClarity;
use unghost_core::tags::GhostTag;
use uninvestigation_core::resources::current_evidence_readings::CurrentEvidenceReadings;
use uninvestigation_core::resources::ghost_guess::GhostGuess;

use unghost_core::events::{
    EvidenceClarityThresholdCrossed, GhostActualTypeChanged, GhostBreakerSparkRequest,
    GhostInteractionEvent, JournalEvidenceToggled, JournalGhostToggled,
};
use unghost_core::resources::haunt_state::HauntState;
use unghost_core::resources::object_interaction::ObjectInteractionConfig;
use unghost_core::resources::signals::GhostHuntSignals;

use crate::{ghost_events, metrics};

pub struct UnhaunterGhostLogicPlugin;

impl Plugin for UnhaunterGhostLogicPlugin {
    fn build(&self, app: &mut App) {
        app.add_message::<GhostInteractionEvent>();
        app.add_message::<JournalEvidenceToggled>();
        app.add_message::<JournalGhostToggled>();
        app.add_message::<EvidenceClarityThresholdCrossed>();
        app.add_message::<GhostActualTypeChanged>();
        app.add_message::<GhostBreakerSparkRequest>();
        app.replicate::<GhostTag>();
        app.replicate::<GhostBreach>();
        app.replicate::<GhostDeathSignal>();
        app.replicate::<GhostInfluence>();
        app.replicate::<GhostSprite>();
        app.replicate::<GhostBehaviorDynamics>();
        app.replicate::<GhostGuess>();
        app.replicate::<InteractionMotion>();
        app.replicate::<Locked>();
        app.replicate::<SpectralClarity>();
        crate::systems::hydration::app_setup(app);
        crate::systems::evidence_decay::app_setup(app);
        crate::systems::ghost_ai::app_setup(app);
        crate::systems::journal::app_setup(app);
        crate::systems::hint_events::app_setup(app);
        ghost_events::app_setup(app);
        metrics::register_logic(app);
        app.init_resource::<ObjectInteractionConfig>()
            .init_resource::<GhostHuntSignals>()
            .init_resource::<HauntState>()
            .init_resource::<CurrentEvidenceReadings>();

        app.add_systems(Startup, setup_noise);
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
