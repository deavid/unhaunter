use bevy::prelude::*;
use unbehavior_core::behavior::{Behavior, Interactive};
use unbehavior_core::components;
use unmapload_core::hydration::HydrationStage;

fn hydration_generic_logic_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    use bevy::picking::Pickable;
    for (entity, behavior) in q.iter_mut() {
        let mut cmd = commands.entity(entity);
        let cfg = behavior.cfg();

        if behavior.p.is_door {
            cmd.insert_if_new(Pickable::default())
                .insert_if_new(Interactive::new(
                    "sounds/door-open.ogg",
                    "sounds/door-close.ogg",
                ))
                .insert_if_new(components::FloorItemCollidable)
                .insert_if_new(components::Door);
        } else if behavior.p.is_room_switch {
            // Check for opposite_side property
            let opposite_side = cfg.properties.get_bool("switch:opposite_side");

            cmd.insert_if_new(Pickable::default())
                .insert_if_new(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert_if_new(components::RoomStateDelta::with_opposite_side(
                    &cfg.orientation,
                    opposite_side,
                ));
        } else if behavior.p.is_switch {
            cmd.insert_if_new(Pickable::default())
                .insert_if_new(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert_if_new(components::RoomStateDelta::default());
        } else if behavior.p.is_breaker {
            cmd.insert_if_new(Pickable::default())
                .insert_if_new(Interactive::new(
                    "sounds/switch-on-2.ogg",
                    "sounds/switch-off-1.ogg",
                ));
        } else if behavior.p.is_wall_light {
            cmd.insert_if_new(components::RoomStateDelta::default())
                .insert_if_new(components::Light);
        } else if behavior.p.is_floor_light || behavior.p.is_table_light {
            cmd.insert_if_new(Pickable::default())
                .insert_if_new(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert_if_new(components::FloorItemCollidable)
                .insert_if_new(components::Light);
        } else if behavior.p.movement.stair_offset != 0 {
            cmd.insert_if_new(components::Stairs {
                z: behavior.p.movement.stair_offset,
            });
        }

        // Additional generic component attachment based on properties
        if behavior.p.is_ceiling_light {
            cmd.insert_if_new(components::RoomStateDelta::default())
                .insert_if_new(components::Light);
        } else if behavior.p.is_street_light || behavior.p.is_candle_light {
            cmd.insert_if_new(components::Light);
        } else if behavior.p.is_appliance || behavior.p.is_stationary_collidable {
            cmd.insert_if_new(components::FloorItemCollidable);
        }

        if behavior.can_emit_light() {
            cmd.insert_if_new(components::HeatEmitter);
        }

        // Add Movable marker
        if behavior.p.object.movable {
            cmd.insert_if_new(components::Movable);
        }

        // Add HidingSpot marker
        if behavior.p.object.hidingspot {
            cmd.insert_if_new(components::HidingSpot);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_generic_logic_system);
}
