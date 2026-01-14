use bevy::prelude::*;
use unfoundation_core::types::ghost::types::GhostType;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearSpawnerRegistry;
use ungear_core::types::GearKind;
use ungearitems_core::components::repellentflask::RepellentFlask;

/// Crafts a repellent for the specified ghost type.
/// Returns true if a new bottle was consumed (should count as a craft).
pub(crate) fn craft_repellent(
    commands: &mut Commands,
    gear_registry: &GearSpawnerRegistry,
    playergear: &mut PlayerGear,
    ghost_type: GhostType,
    q_repellent: &mut Query<&mut RepellentFlask>,
    q_gearkind: &Query<&GearKind>,
) -> bool {
    // 1) Find or create a repellent flask
    let mut flask_entity = None;

    // Check right hand
    if let Some(e) = playergear.right_hand
        && let Ok(kind) = q_gearkind.get(e)
        && *kind == GearKind::RepellentFlask
    {
        flask_entity = Some(e);
    }

    // Check left hand
    if flask_entity.is_none()
        && let Some(e) = playergear.left_hand
        && let Ok(kind) = q_gearkind.get(e)
        && *kind == GearKind::RepellentFlask
    {
        flask_entity = Some(e);
    }

    // Check inventory
    if flask_entity.is_none() {
        for &e in &playergear.inventory {
            if let Ok(kind) = q_gearkind.get(e)
                && *kind == GearKind::RepellentFlask
            {
                flask_entity = Some(e);
                break;
            }
        }
    }

    let mut is_new = false;
    if flask_entity.is_none() {
        // Spawn new flask
        let entity = gear_registry.spawn(commands, GearKind::RepellentFlask);

        // Put in right hand (swap if needed)
        if let Some(old_rh) = playergear.right_hand.take() {
            playergear.inventory.push(old_rh);
        }
        playergear.right_hand = Some(entity);
        flask_entity = Some(entity);
        is_new = true;
    }

    // 2) Call do_fill_liquid and return the result
    if let Some(entity) = flask_entity {
        if !is_new {
            if let Ok(mut flask) = q_repellent.get_mut(entity) {
                flask.liquid_content = Some(ghost_type);
                flask.qty = 400; // MAX_QTY
                flask.active = false;
                return true;
            }
        } else {
            // It's a new entity. We can't get it from query yet.
            // We overwrite the component with the desired state.
            let flask = RepellentFlask {
                liquid_content: Some(ghost_type),
                qty: 400,
                active: false,
            };
            commands.entity(entity).insert(flask);
            return true;
        }
    }

    false
}
