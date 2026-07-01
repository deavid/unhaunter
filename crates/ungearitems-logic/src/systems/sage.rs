use bevy::prelude::*;
use unfog_core::miasma::MiasmaGrid;
use ungearitems_core::components::sage::SageBundleData;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unreplicon_core::resources::AuthorityRole;
use unspatial_core::position::Position;

pub(crate) fn update_sage_skeleton(
    mut q_sage: Query<
        (Entity, &mut SageBundleData),
        With<unreplicon_core::ownership::LocallyOwned>,
    >,
    q_triggered: Query<&uninteraction_core::interaction::Triggered>,
    mut commands: Commands,
) {
    for (entity, mut sage) in q_sage.iter_mut() {
        if q_triggered.get(entity).is_ok() && !sage.is_active && !sage.consumed {
            sage.is_active = true;
            commands
                .entity(entity)
                .remove::<uninteraction_core::interaction::Triggered>();
        }
    }
}

pub(crate) fn sage_authority_system(
    q_sage: Query<(&SageBundleData, &Position)>,
    mut q_ghost: Query<(&mut GhostSprite, &Position)>,
    _authority: Res<AuthorityRole>,
    time: Res<Time>,
    mut miasma_grid: ResMut<MiasmaGrid>,
) {
    let dt = time.delta_secs();
    for (sage, sage_pos) in q_sage.iter() {
        if sage.is_active && !sage.consumed {
            // Apply smoke to grid at sage location
            let bpos = sage_pos.to_board_position();
            let (width, height, depth) = miasma_grid.smoke_field.dim();
            if bpos.is_valid((width, height, depth)) {
                miasma_grid.smoke_field[bpos.ndidx()] += 0.12 * dt;
                miasma_grid.pressure_field[bpos.ndidx()] *= 0.9;
            }

            for (mut ghost, ghost_pos) in q_ghost.iter_mut() {
                let dist2 = sage_pos.distance2(ghost_pos);
                if dist2 < 6.0 * 6.0 {
                    let dist = dist2.sqrt();
                    let factor = 1.0 / (1.0 + dist);

                    ghost.rage = (ghost.rage - (30.0 * dt * factor)).max(0.0);
                    ghost.calm_time_secs = (ghost.calm_time_secs + (10.0 * dt * factor)).min(30.0);
                }
            }
        }
    }
}
