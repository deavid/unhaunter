//! ## Behavior module
//!
//! This module defines the `Behavior` component and its associated data
//! structures, which are used to represent the behavior of objects in the game
//! world.
//!
//! The `Behavior` component stores information about the object's type, variant,
//! orientation, state, and a collection of properties that determine how it
//! interacts with the player and the environment. This component is crucial for
//! separating the object's visual representation (its sprite) from its logical
//! behavior.
//!
//! The information stored in the `Behavior` component is loaded from Tiled map
//! data. Each tile in Tiled can be assigned a "class" (e.g., "Door", "Wall",
//! "Light"), a "variant" (e.g., "wooden", "brick", "fluorescent"), an
//! "orientation", and a "state" (e.g., "open", "closed", "on", "off").
//!
//! This data is used to create a `SpriteConfig` struct, which is then used to
//! initialize the `Behavior` component. The `Behavior` component, in turn, is used
//! to add other Bevy components to the object's entity, such as `Collision`,
//! `Interactive`, `Light`, etc., based on its configuration.
pub mod component;

pub use unbehavior::behavior::*;
pub use unbehavior::class::Class;
pub use unbehavior::state::TileState;
pub use unspatial_core::orientation::Orientation;

pub fn apply_components_to_entity(
    behavior: &Behavior,
    entity: &mut bevy::ecs::system::EntityCommands,
    layer: &crate::types::tiledmap::map::MapLayer,
) {
    use crate::behavior::component;
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
