use bevy::prelude::*;
use ungearitems_core::components::salt::{
    SaltData, SaltPile, SaltPileArmed, SaltPileArmingTimer, SaltPileConsumed, SaltPileConsumedTimer,
};
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unreplicon_core::resources::AuthorityRole;
use unspatial_core::position::Position;

const SALT_PILE_ARMING_DELAY_SECS: f32 = 0.10;

pub(crate) fn update_salt_skeleton(
    mut q_salt: Query<
        (Entity, &mut SaltData, &Position),
        With<unreplicon_core::ownership::LocallyOwned>,
    >,
    q_triggered: Query<&uninteraction_core::interaction::Triggered>,
    mut commands: Commands,
    authority: Option<Res<AuthorityRole>>,
    mut salt_drop_writer: MessageWriter<unreplicon_core::messages::SaltDroppedMessage>,
) {
    for (entity, mut salt, pos) in q_salt.iter_mut() {
        if q_triggered.get(entity).is_ok() && salt.charges > 0 {
            salt.charges -= 1;
            commands
                .entity(entity)
                .remove::<uninteraction_core::interaction::Triggered>();

            if authority.is_some() {
                // Host/Singleplayer: Spawn directly
                commands.spawn((
                    SaltPile,
                    SaltPileArmingTimer(Timer::from_seconds(
                        SALT_PILE_ARMING_DELAY_SECS,
                        TimerMode::Once,
                    )),
                    *pos,
                    bevy_replicon::prelude::Replicated,
                ));
            } else {
                // Pure Client: Ask server to spawn
                salt_drop_writer.write(unreplicon_core::messages::SaltDroppedMessage {
                    pos: [pos.x, pos.y, pos.z, pos.visual_priority],
                });
            }
        }
    }
}

pub(crate) fn progress_salt_pile_arming(
    mut commands: Commands,
    time: Res<Time>,
    mut q_arming: Query<(Entity, &mut SaltPileArmingTimer), Without<SaltPileArmed>>,
) {
    for (entity, mut arming_timer) in q_arming.iter_mut() {
        arming_timer.0.tick(time.delta());
        if arming_timer.0.is_finished() {
            commands
                .entity(entity)
                .remove::<SaltPileArmingTimer>()
                .insert(SaltPileArmed);
        }
    }
}

pub(crate) fn salt_pile_system(
    mut commands: Commands,
    mut ghosts: Query<(&mut GhostSprite, &Position)>,
    salt_piles: Query<(Entity, &Position), (With<SaltPile>, With<SaltPileArmed>)>,
) {
    for (mut ghost, ghost_position) in ghosts.iter_mut() {
        for (salt_pile_entity, salt_pile_position) in salt_piles.iter() {
            if ghost_position.distance(salt_pile_position) < 2.0
                && (120.0 - ghost.salty_effect_remaining_secs) > 1.0
            {
                ghost.rage += 10.0;
                ghost.salty_effect_remaining_secs = 120.0;

                commands
                    .entity(salt_pile_entity)
                    .remove::<(SaltPile, SaltPileArmed)>();
                commands.entity(salt_pile_entity).insert((
                    SaltPileConsumed,
                    SaltPileConsumedTimer(Timer::from_seconds(2.0, TimerMode::Once)),
                ));
            }
        }
    }
}

// 4. Cleanup the consumed pile so it deletes itself over the network (Server only)
pub(crate) fn salt_pile_cleanup_system(
    mut commands: Commands,
    time: Res<Time>,
    mut q_consumed: Query<(Entity, &mut SaltPileConsumedTimer)>,
) {
    for (entity, mut timer) in q_consumed.iter_mut() {
        timer.0.tick(time.delta());
        if timer.0.just_finished() {
            commands.entity(entity).despawn();
        }
    }
}
