use bevy::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use uncommon_states_core::UIContextState;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::presentation::ghost_dying::GhostDying;
use unreplicon_core::resources::LocalPlayerRole;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            start_ghost_dying_visuals_system,
            tick_ghost_dying_visuals_system,
            ghost_dying_visuals_system,
        )
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}

fn start_ghost_dying_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &GhostDeathSignal), (Added<GhostDeathSignal>, Without<GhostDying>)>,
) {
    for (entity, death_signal) in query.iter() {
        commands
            .entity(entity)
            .insert(GhostDying::new(death_signal.duration_secs));
    }
}

fn tick_ghost_dying_visuals_system(time: Res<Time>, mut query: Query<&mut GhostDying>) {
    for mut dying in query.iter_mut() {
        dying.timer.tick(time.delta());
    }
}

fn ghost_dying_visuals_system(mut query: Query<(&GhostDying, &mut MapColor)>) {
    for (dying, mut map_color) in query.iter_mut() {
        let rem_f = dying.timer.remaining_secs() / dying.timer.duration().as_secs_f32();
        map_color.color.set_alpha(rem_f.powi(2));
    }
}
