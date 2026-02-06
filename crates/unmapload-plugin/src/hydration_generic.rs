use bevy::prelude::*;
use unbehavior::behavior::{Behavior, Interactive};
use unbehavior::components;
use untypes_core::hydration::HydrationStage;

fn hydration_generic_logic_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    use bevy::picking::Pickable;
    for (entity, behavior) in q.iter_mut() {
        let mut cmd = commands.entity(entity);
        let cfg = behavior.cfg();

        if behavior.p.is_door {
            cmd.insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/door-open.ogg",
                    "sounds/door-close.ogg",
                ))
                .insert(components::FloorItemCollidable)
                .insert(components::Door);
        } else if behavior.p.is_room_switch {
            // Check for opposite_side property
            let opposite_side = cfg.properties.get_bool("switch:opposite_side");

            cmd.insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(components::RoomState::with_opposite_side(
                    &cfg.orientation,
                    opposite_side,
                ));
        } else if behavior.p.is_switch {
            cmd.insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(components::RoomState::default());
        } else if behavior.p.is_breaker {
            cmd.insert(Pickable::default()).insert(Interactive::new(
                "sounds/switch-on-2.ogg",
                "sounds/switch-off-1.ogg",
            ));
        } else if behavior.p.is_wall_light {
            cmd.insert(components::RoomState::default())
                .insert(components::Light);
        } else if behavior.p.is_floor_light || behavior.p.is_table_light {
            cmd.insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(components::FloorItemCollidable)
                .insert(components::Light);
        } else if behavior.p.movement.stair_offset != 0 {
            cmd.insert(components::Stairs {
                z: behavior.p.movement.stair_offset,
            });
        }

        // Additional generic component attachment based on properties
        if behavior.p.is_ceiling_light {
            cmd.insert(components::RoomState::default())
                .insert(components::Light);
        } else if behavior.p.is_street_light || behavior.p.is_candle_light {
            cmd.insert(components::Light);
        } else if behavior.p.is_appliance || behavior.p.is_stationary_collidable {
            cmd.insert(components::FloorItemCollidable);
        }

        // Add Movable marker
        if behavior.p.object.movable {
            cmd.insert(components::Movable);
        }

        // Add HidingSpot marker
        if behavior.p.object.hidingspot {
            cmd.insert(components::HidingSpot);
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, hydration_generic_logic_system);
}
