use bevy::prelude::*;
use uncore::difficulty::CurrentDifficulty;
use uncore::states::{AppState, GameState};
use unwalkiecore::{WalkieEvent, WalkiePlay};

pub struct TutorialIntroductionsTriggerPlugin;

impl Plugin for TutorialIntroductionsTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            trigger_chapter_intros
                .run_if(in_state(AppState::InGame).and(in_state(GameState::None))),
        );
    }
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
