use unbehavior::behavior::{Behavior, Interactive, NpcHelpDialog};
use unbehavior::components;
use untiled_core::tiledmap::map::MapLayer;

pub(crate) fn apply_components_to_entity(
    behavior: &Behavior,
    entity: &mut bevy::ecs::system::EntityCommands,
    layer: &MapLayer,
) {
    use bevy::picking::Pickable;
    let cfg = behavior.cfg();

    if behavior.p.is_floor {
        entity
            .insert(components::Ground)
            .insert(components::UVSurface);
    } else if behavior.p.is_wall || behavior.p.is_low_wall {
        entity
            .insert(components::Collision)
            .insert(components::Opaque)
            .insert(components::UVSurface);
    } else if behavior.p.is_door {
        entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            ))
            .insert(components::FloorItemCollidable)
            .insert(components::Door);
    } else if behavior.p.is_room_switch {
        // Check for opposite_side property
        let opposite_side = cfg.properties.get_bool("switch:opposite_side");

        entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(components::RoomState::with_opposite_side(
                &cfg.orientation,
                opposite_side,
            ));
    } else if behavior.p.is_switch {
        entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(components::RoomState::default());
    } else if behavior.p.is_breaker {
        entity.insert(Pickable::default()).insert(Interactive::new(
            "sounds/switch-on-2.ogg",
            "sounds/switch-off-1.ogg",
        ));
    } else if behavior.p.is_van_entry {
        entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            ))
            .insert(components::FloorItemCollidable);
    } else if behavior.p.is_wall_light {
        entity
            .insert(components::RoomState::default())
            .insert(components::Light);
    } else if behavior.p.is_floor_light || behavior.p.is_table_light {
        entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(components::FloorItemCollidable)
            .insert(components::Light);
    } else if behavior.p.is_npc {
        entity
            .insert(Pickable::default())
            .insert(NpcHelpDialog::new(
                "NPC",
                &cfg.variant,
                &layer.user_properties,
            ))
            .insert(Interactive::new(
                "sounds/effects-dongdongdong.ogg",
                "sounds/effects-dongdongdong.ogg",
            ))
            .insert(components::FloorItemCollidable);
    } else if behavior.p.movement.stair_offset != 0 {
        entity.insert(components::Stairs {
            z: behavior.p.movement.stair_offset,
        });
    }

    // Additional generic component attachment based on properties
    if behavior.p.is_ceiling_light {
        entity
            .insert(components::RoomState::default())
            .insert(components::Light);
    } else if behavior.p.is_street_light || behavior.p.is_candle_light {
        entity.insert(components::Light);
    } else if behavior.p.is_appliance || behavior.p.is_stationary_collidable {
        entity.insert(components::FloorItemCollidable);
    }

    // Add InteractableByGhost marker component for entities that ghosts can interact with
    let should_add_ghost_interaction =
        if behavior.p.is_door || behavior.p.is_switch || behavior.p.is_breaker {
            true
        } else {
            // For other classes, check properties
            let has_light = behavior.p.light.can_emit_light;
            let is_movable = behavior.p.object.movable;
            let is_throwable = behavior.p.object.throwable;
            let is_nudgeable = behavior.p.object.nudgeable;
            let haunt_movable = behavior.p.object.haunt_movable;

            has_light || is_movable || is_throwable || is_nudgeable || haunt_movable
        };

    if should_add_ghost_interaction {
        entity.insert(components::InteractableByGhost);
    }
}
