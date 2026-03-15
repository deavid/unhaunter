use bevy::prelude::*;
use unbehavior::behavior::Behavior;
use unbehavior::state::TileState;
use unrender_std::board::spritedb::SpriteDB;
use unrender_std::components::game::MapTileSprite;
use unrender_std::materials::CustomMaterial1;

use unlight_core::resources::light_grid::LightGrid;

fn update_single_visual(
    behavior: &Behavior,
    material_handle: &mut MeshMaterial2d<CustomMaterial1>,
    has_power: bool,
    sdb: &SpriteDB,
    materials1: &mut Assets<CustomMaterial1>,
) {
    let visual_state = if behavior.p.is_house_powered && !has_power {
        TileState::Off
    } else {
        behavior.state()
    };

    let cvo = behavior.key_cvo();
    let Some(variants) = sdb.cvo_idx.get(&cvo) else {
        return;
    };

    for tuid in variants {
        let Some(tile) = sdb.map_tile.get(tuid) else {
            continue;
        };

        if tile.behavior.state() == visual_state {
            let target_material = tile.bundle.material.clone();

            // We only update if the material handle is different.
            // Note: This might still trigger if target_material is a shared material
            // and we previously cloned it. But it avoids most redundant work.
            if material_handle.0 != target_material.0 {
                let current_alpha = materials1
                    .get(&material_handle.0)
                    .map(|m| m.data.color.alpha)
                    .unwrap_or(1.0);

                let mut new_mat = materials1
                    .get(&target_material.0)
                    .expect("Material from SpriteDB should exist")
                    .clone();

                new_mat.data.color.alpha = current_alpha;
                *material_handle = MeshMaterial2d(materials1.add(new_mat));
            }
            break;
        }
    }
}

pub(crate) fn update_power_visuals(
    lg: If<Res<LightGrid>>,
    sdb: Res<SpriteDB>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut q_set: ParamSet<(
        Query<(&Behavior, &mut MeshMaterial2d<CustomMaterial1>), With<MapTileSprite>>,
        Query<
            (&Behavior, &mut MeshMaterial2d<CustomMaterial1>),
            (With<MapTileSprite>, Changed<Behavior>),
        >,
    )>,
    q_breaker_behavior: Query<&Behavior>,
    mut last_has_power: Local<Option<bool>>,
) {
    let has_power = lg.has_power(&q_breaker_behavior);
    let power_changed = Some(has_power) != *last_has_power;
    *last_has_power = Some(has_power);

    if power_changed {
        for (behavior, mut material_handle) in q_set.p0().iter_mut() {
            update_single_visual(
                behavior,
                &mut material_handle,
                has_power,
                &sdb,
                &mut materials1,
            );
        }
    } else {
        for (behavior, mut material_handle) in q_set.p1().iter_mut() {
            update_single_visual(
                behavior,
                &mut material_handle,
                has_power,
                &sdb,
                &mut materials1,
            );
        }
    }
}
