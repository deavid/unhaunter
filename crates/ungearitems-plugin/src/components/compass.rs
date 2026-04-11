use bevy::prelude::*;
use ungear_core::components::core::{GearSprite, ItemName, StatusText, StatusTextRefreshTimer};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungear_core::types::gear::utils::on_off;
pub(crate) use ungearitems_core::components::compass::Compass;
use uninteraction_core::interaction::Toggleable;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::resources::LocalPlayerRole;

use crate::metrics;

pub(crate) fn update_compass(
    mut commands: Commands,
    mut q_compass: Query<
        (
            Entity,
            &mut StatusText,
            &mut GearSprite,
            &Toggleable,
            &ItemName,
            Has<StatusTextRefreshTimer>,
        ),
        With<Compass>,
    >,
) {
    let measure = metrics::COMPASS_UPDATE.time_measure();
    for (entity, mut status, mut sprite, toggle, name, has_timer) in q_compass.iter_mut() {
        sprite.0 = GearSpriteID::Compass.to_visual_key();

        let on_s = on_off(toggle.is_on);
        let msg = if toggle.is_on {
            "You managed to 'turn on' a compass".to_string()
        } else {
            "".to_string()
        };
        let new_status = format!("{}: {}\n{}", name.0, on_s, msg);
        status.update(entity, &mut commands, !has_timer, new_status);
    }

    measure.end_ms();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        update_compass.run_if(resource_exists::<LocalPlayerRole>),
    );
}
