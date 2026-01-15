use unbehavior::behavior::{Behavior, Interactive, NpcHelpDialog};
use unbehavior::class::Class;
use unboard_core::behavior::component;
use untiled_core::tiledmap::map::MapLayer;

pub(crate) fn apply_components_to_entity(
    behavior: &Behavior,
    entity: &mut bevy::ecs::system::EntityCommands,
    layer: &MapLayer,
) {
    use bevy::picking::Pickable;
    let cfg = behavior.cfg();

    match cfg.class {
        Class::Floor => entity
            .insert(component::Ground)
            .insert(component::UVSurface),
        Class::Wall => entity
            .insert(component::Collision)
            .insert(component::Opaque)
            .insert(component::UVSurface),
        Class::LowWall => entity
            .insert(component::Collision)
            .insert(component::Opaque)
            .insert(component::UVSurface),
        Class::Door => entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            ))
            .insert(component::FloorItemCollidable)
            .insert(component::Door),
        Class::Switch => entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(component::RoomState::default()),
        Class::RoomSwitch => {
            // Check for opposite_side property
            let opposite_side = cfg.properties.get_bool("switch:opposite_side");

            entity
                .insert(Pickable::default())
                .insert(Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::RoomState::with_opposite_side(
                    &cfg.orientation,
                    opposite_side,
                ))
        }
        Class::Breaker => entity.insert(Pickable::default()).insert(Interactive::new(
            "sounds/switch-on-1.ogg",
            "sounds/switch-off-1.ogg",
        )),
        Class::Doorway => entity,
        Class::Decor => entity.insert(component::FloorItemCollidable),
        Class::Item => entity.insert(component::FloorItemCollidable),
        Class::Furniture => entity.insert(component::FloorItemCollidable),
        Class::PlayerSpawn => entity,
        Class::GhostSpawn => entity,
        Class::VanEntry => entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            ))
            .insert(component::FloorItemCollidable),
        Class::RoomDef => entity,
        Class::WallLamp => entity
            .insert(component::RoomState::default())
            .insert(component::Light),
        Class::FloorLamp => entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(component::FloorItemCollidable)
            .insert(component::Light),
        Class::TableLamp => entity
            .insert(Pickable::default())
            .insert(Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            ))
            .insert(component::FloorItemCollidable)
            .insert(component::Light),
        Class::WallDecor => entity,
        Class::CeilingLight => entity
            .insert(component::RoomState::default())
            .insert(component::Light),
        Class::StreetLight => entity.insert(component::Light),
        Class::CandleLight => entity.insert(component::Light),
        Class::Appliance => entity.insert(component::FloorItemCollidable),
        Class::Van => entity,
        Class::Window => entity,
        Class::None => entity,
        Class::InvisibleWall => entity,
        Class::CornerWall => entity,
        Class::FakeBreach => entity,
        Class::FakeGhost => entity,
        Class::NPC => entity
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
            .insert(component::FloorItemCollidable),
        Class::StairsDown => entity.insert(component::Stairs { z: -1 }),
        Class::StairsUp => entity.insert(component::Stairs { z: 1 }),
    };

    // Add InteractableByGhost marker component for entities that ghosts can interact with
    let should_add_ghost_interaction = match cfg.class {
        // Doors, switches, and breakers can always be interacted with by ghosts
        Class::Door | Class::Switch | Class::RoomSwitch | Class::Breaker => true,
        // For other classes, check properties
        _ => {
            // Check for light capabilities (can be toggled by ghosts)
            let has_light = cfg.properties.get_bool("light:can_emit_light");

            // Check for object interaction properties
            let has_object_interaction = cfg.properties.get_bool("object:throwable")
                || cfg.properties.get_bool("object:nudgeable")
                || cfg.properties.get_bool("object:haunt_movable");

            has_light || has_object_interaction
        }
    };

    if should_add_ghost_interaction {
        entity.insert(component::InteractableByGhost);
    }
}
