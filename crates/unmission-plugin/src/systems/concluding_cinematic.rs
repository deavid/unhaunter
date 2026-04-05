use bevy::prelude::*;
use uncommon_states_core::UIContextState;
use unmission_core::resources::MissionConcludingCinematic;
use unmission_core::summary::SummaryData;
use unreplicon_core::components::{MissionGoalEntity, ServerGamePhase};
use unreplicon_core::resources::AuthorityRole;

pub(crate) fn on_mission_concluding(
    q_phase: Query<&ServerGamePhase, Changed<ServerGamePhase>>,
    mut commands: Commands,
) {
    for phase in q_phase.iter() {
        if *phase == ServerGamePhase::Concluding {
            info!("ServerGamePhase::Concluding observed — starting cinematic");
            commands.insert_resource(MissionConcludingCinematic {
                timer: Timer::from_seconds(2.5, TimerMode::Once),
                inputs_blocked: true,
            });
        }
    }
}

pub(crate) fn tick_mission_concluding(
    mut commands: Commands,
    cinematic: Option<ResMut<MissionConcludingCinematic>>,
    summary_data: Option<Res<SummaryData>>,
    authority: Option<Res<AuthorityRole>>,
    q_goal: Query<&SummaryData, With<MissionGoalEntity>>,
    mut next_app_state: ResMut<NextState<UIContextState>>,
    time: Res<Time>,
) {
    let Some(mut cinematic) = cinematic else {
        return;
    };

    cinematic.timer.tick(time.delta());
    if !cinematic.timer.just_finished() {
        return;
    }

    // summary_data on authority or summary_data replicated component on client
    let ready = if authority.is_some() {
        summary_data.is_some()
    } else {
        q_goal.iter().next().is_some()
    };
    if ready {
        next_app_state.set(UIContextState::Summary);
        commands.remove_resource::<MissionConcludingCinematic>();
    }
}
