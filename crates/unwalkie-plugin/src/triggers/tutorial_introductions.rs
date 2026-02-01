use bevy::prelude::*;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use untypes_core::states::{AppState, GameState};
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::resources::WalkiePlay;

pub(crate) struct TutorialIntroductionsTriggerPlugin;

impl Plugin for TutorialIntroductionsTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            trigger_chapter_intros
                .run_if(in_state(AppState::InGame).and(in_state(GameState::None))),
        );
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins(TutorialIntroductionsTriggerPlugin);
}

fn trigger_chapter_intros(
    mut walkie_play: ResMut<WalkiePlay>,
    app_state: Res<State<AppState>>,
    current_difficulty_res: Res<CurrentDifficulty>,
    time: Res<Time>,
) {
    if app_state.get() != &AppState::InGame {
        // We want to play this only when the player is in the game.
        return;
    }

    let difficulty = current_difficulty_res.0.difficulty;

    if walkie_play.set(
        WalkieEvent::ChapterIntro(difficulty),
        time.elapsed_secs_f64(),
    ) {
        info!("Intro for {:?} triggered.", difficulty);
    }
}
