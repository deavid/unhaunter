use bevy::prelude::*;
use bevy_replicon::prelude::AppMarkerExt;
use bevy_replicon::prelude::AppRuleExt;
use unlocomotion_core::components::PlayerLocomotionState;
use unorchestrator_core::UIContextState;
use unplayer_core::components::{Hiding, PlayerSpectating, PlayerSprite};
use unreplicon_core::noop::{noop_remove, noop_write};
use unreplicon_core::ownership::LocallyOwned;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::direction::Direction;
use untruck_core::components::in_truck::InTruck;

use crate::systems::hide;
use crate::systems::hydration;
use crate::systems::input;
use crate::systems::net_state;
use crate::systems::spawn;
use crate::systems::styling;
use crate::systems::walk_target_indicator;

pub(crate) fn app_setup_core(app: &mut App) {
    spawn::app_setup(app);
    net_state::app_setup(app);

    app.replicate::<Direction>();
    app.replicate::<PlayerSprite>();
    app.set_marker_fns::<LocallyOwned, PlayerSprite>(noop_write::<PlayerSprite>, noop_remove);
    app.replicate::<PlayerLocomotionState>();
    app.replicate::<Hiding>();
    app.replicate::<InTruck>();
    app.replicate::<PlayerSpectating>();

    hydration::app_setup(app);

    // Configure the authoritative logic set
    app.configure_sets(
        Update,
        unplayer_core::authoritative::PlayerAuthoritativeLogicSet
            .run_if(resource_exists::<unreplicon_core::resources::AuthorityRole>)
            .after(uninput_core::PlayerInputSet),
    );
}

pub(crate) fn app_setup_client(app: &mut App) {
    hide::app_setup(app);

    app.add_systems(
        Update,
        styling::apply_player_tint_color.run_if(in_state(UIContextState::InGame)),
    );

    // Walk target indicator system: shows visual feedback for click-to-move target
    app.add_systems(
        Update,
        walk_target_indicator::update_move_target_indicator
            .run_if(in_state(UIContextState::InGame)),
    );

    app.add_systems(
        Update,
        input::mouse_interaction::toggle_gear_from_use_intent
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>)
            .after(uninput_core::PlayerInputSet),
    );
}
