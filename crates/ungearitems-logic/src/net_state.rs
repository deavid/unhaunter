use bevy::prelude::*;
use bevy_replicon::prelude::{Channel, ClientMessageAppExt, FromClient};
use ungearitems_core::events::SageHitNetMessage;
use unreplicon_core::resources::AuthorityRole;

pub(crate) fn app_setup(app: &mut App) {
    app.add_client_message::<SageHitNetMessage>(Channel::Unreliable);
    app.add_client_message::<unreplicon_core::messages::SaltDroppedMessage>(Channel::Ordered);

    app.add_systems(
        Update,
        (handle_sage_hits, handle_salt_drop_requests)
            .run_if(resource_exists::<AuthorityRole>)
            .run_if(in_state(uncommon_states_core::UIContextState::InGame)),
    );
}

pub(crate) fn handle_sage_hits(
    mut ev_hits: MessageReader<SageHitNetMessage>,
    mut q_ghost: Query<&mut unghost_core::components::logic::ghost_sprite::GhostSprite>,
) {
    for hit in ev_hits.read() {
        if hit.rage_reduction_this_frame > 0.0 || hit.calm_this_frame > 0.0 {
            for mut ghost in q_ghost.iter_mut() {
                ghost.rage = (ghost.rage - hit.rage_reduction_this_frame).max(0.0);
                ghost.calm_time_secs = (ghost.calm_time_secs + hit.calm_this_frame).min(30.0);
            }
        }
    }
}

pub(crate) fn handle_salt_drop_requests(
    mut commands: Commands,
    mut ev_salt: MessageReader<FromClient<unreplicon_core::messages::SaltDroppedMessage>>,
) {
    for ev in ev_salt.read() {
        let pos = ev.pos;
        commands.spawn((
            ungearitems_core::components::salt::SaltPile,
            ungearitems_core::components::salt::SaltPileArmingTimer(Timer::from_seconds(
                0.10,
                TimerMode::Once,
            )),
            unspatial_core::position::Position {
                x: pos[0],
                y: pos[1],
                z: pos[2],
                visual_priority: pos[3],
            },
            bevy_replicon::prelude::Replicated,
        ));
    }
}
