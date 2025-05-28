use bevy::prelude::*;

use uncore::components::player_sprite::PlayerSprite;
use uncore::difficulty::CurrentDifficulty;
use uncore::states::{AppState, GameState};
use uncore::types::gear_kind::GearKind;
use ungear::components::playergear::PlayerGear;
use ungear::gear_usable::GearUsable;
use unwalkiecore::{WalkieEvent, WalkiePlay};

pub struct TutorialGearExplanationsTriggerPlugin;

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

fn trigger_evidence_gear_explanations(
    mut walkie_play: ResMut<WalkiePlay>,
    current_difficulty_res: Res<CurrentDifficulty>,
    player_gear_query: Query<&PlayerGear, With<PlayerSprite>>,
    time: Res<Time>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    if let Ok(player_gear) = player_gear_query.get_single() {
        for gear_item in [&player_gear.left_hand, &player_gear.right_hand] {
            let gear_kind = &gear_item.kind;
            match gear_kind {
                GearKind::Flashlight
                | GearKind::Thermometer
                | GearKind::EMFMeter
                | GearKind::UVTorch
                | GearKind::Videocam
                | GearKind::Recorder
                | GearKind::GeigerCounter
                | GearKind::SpiritBox
                | GearKind::RedTorch => {
                    if gear_item.is_enabled()
                        && walkie_play.set(
                            WalkieEvent::GearExplanation(*gear_kind),
                            time.elapsed_secs_f64(),
                        )
                    {
                        info!(
                            "Evidence gear explanation triggered for {:?} because it's enabled.",
                            gear_kind
                        );
                    }
                }
                _ => {} // Not an evidence tool of interest for this system
            }
        }
    }
}

fn trigger_support_item_explanations(
    mut walkie_play: ResMut<WalkiePlay>,
    current_difficulty_res: Res<CurrentDifficulty>,
    player_gear_query: Query<&PlayerGear, With<PlayerSprite>>,
    time: Res<Time>,
) {
    let difficulty_info = &current_difficulty_res.0;
    if !difficulty_info.difficulty.is_tutorial_difficulty() {
        return;
    }

    if let Ok(player_gear) = player_gear_query.get_single() {
        let gear_kind = player_gear.right_hand.kind;
        if matches!(
            gear_kind,
            GearKind::Salt | GearKind::QuartzStone | GearKind::SageBundle
        ) && walkie_play.set(
            WalkieEvent::GearExplanation(gear_kind),
            time.elapsed_secs_f64(),
        ) {
            info!(
                "Support item explanation triggered for {:?} because it's in an active hand.",
                gear_kind
            );
        }
    }
}
