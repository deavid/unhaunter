use bevy::prelude::*;

use undifficulty_core::current_difficulty::CurrentDifficulty;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::GearKind;
use uninteraction_core::interaction::Toggleable;
use unplayer_core::components::PlayerSprite;
use untypes_core::states::{AppState, GameState};
use unwalkie_core::events::WalkieEvent;
use unwalkie_core::resources::WalkiePlay;

pub(crate) struct TutorialGearExplanationsTriggerPlugin;

impl Plugin for TutorialGearExplanationsTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            (
                trigger_evidence_gear_explanations,
                trigger_support_item_explanations,
            )
                .run_if(in_state(AppState::InGame).and(in_state(GameState::None))),
        );
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins(TutorialGearExplanationsTriggerPlugin);
}

fn trigger_evidence_gear_explanations(
    mut walkie_play: ResMut<WalkiePlay>,
    current_difficulty_res: Res<CurrentDifficulty>,
    player_gear_query: Query<&PlayerGear, With<PlayerSprite>>,
    time: Res<Time>,
    q_gear: Query<&GearKind>,
    q_toggle: Query<&Toggleable>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    if let Ok(player_gear) = player_gear_query.single() {
        let mut check_gear = |entity: Entity| {
            if let Ok(kind) = q_gear.get(entity) {
                match kind {
                    GearKind::Flashlight
                    | GearKind::Thermometer
                    | GearKind::EMFMeter
                    | GearKind::UVTorch
                    | GearKind::Videocam
                    | GearKind::Recorder
                    | GearKind::GeigerCounter
                    | GearKind::SpiritBox
                    | GearKind::RedTorch => {
                        let is_enabled = if let Ok(toggle) = q_toggle.get(entity) {
                            toggle.is_on
                        } else {
                            false
                        };

                        if is_enabled
                            && walkie_play
                                .set(WalkieEvent::GearExplanation(*kind), time.elapsed_secs_f64())
                        {
                            info!(
                                "Evidence gear explanation triggered for {:?} because it's enabled.",
                                kind
                            );
                        }
                    }
                    _ => {} // Not an evidence tool of interest for this system
                }
            }
        };

        if let Some(e) = player_gear.left_hand {
            check_gear(e);
        }
        if let Some(e) = player_gear.right_hand {
            check_gear(e);
        }
    }
}

fn trigger_support_item_explanations(
    mut walkie_play: ResMut<WalkiePlay>,
    current_difficulty_res: Res<CurrentDifficulty>,
    player_gear_query: Query<&PlayerGear, With<PlayerSprite>>,
    time: Res<Time>,
    q_gear: Query<&GearKind>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    if let Ok(player_gear) = player_gear_query.single()
        && let Some(entity) = player_gear.right_hand
        && let Ok(kind) = q_gear.get(entity)
        && matches!(
            kind,
            GearKind::Salt | GearKind::QuartzStone | GearKind::SageBundle
        )
        && walkie_play.set(WalkieEvent::GearExplanation(*kind), time.elapsed_secs_f64())
    {
        info!(
            "Support item explanation triggered for {:?} because it's in an active hand.",
            kind
        );
    }
}
