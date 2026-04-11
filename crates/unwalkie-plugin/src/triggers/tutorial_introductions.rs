use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::messages::ProposeWalkieEvent;
use unwalkie_core::resources::WalkiePlay;

pub(crate) struct TutorialIntroductionsTriggerPlugin;

impl Plugin for TutorialIntroductionsTriggerPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            Update,
            trigger_chapter_intros.run_if(in_state(UIContextState::InGame)),
        );
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_plugins(TutorialIntroductionsTriggerPlugin);
}

fn trigger_chapter_intros(
    mut walkie_play: ResMut<WalkiePlay>,
    mut ev_propose: MessageWriter<ProposeWalkieEvent>,
    app_state: Res<State<UIContextState>>,
    current_difficulty_res: Res<CurrentDifficulty>,
    time: Res<Time>,
) {
    if app_state.get() != &UIContextState::InGame {
        // We want to play this only when the player is in the game.
        return;
    }

    let difficulty = current_difficulty_res.0;

    if crate::triggers::net::walkie_set_or_propose(
        WalkieEvent::ChapterIntro(difficulty),
        time.elapsed_secs_f64(),
        &mut walkie_play,
        &mut ev_propose,
    ) {
        info!("Intro for {:?} triggered.", difficulty);
    }
}
