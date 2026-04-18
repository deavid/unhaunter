use bevy::prelude::*;
use unbehavior_core::behavior::Behavior;
use unbehavior_core::state::TileState;
use unboard_core::entity::MapTileSprite;
use unmapload_core::resources::SpriteDB;
use unrender_std::custom_material1::CustomMaterial1;
use untmxmap_core::tiled::AtlasData;
use untmxmap_core::tiled::MapTileSetDb;

use unlight_core::resources::light_grid::LightGrid;
type TileVisualMutableQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Behavior,
        &'static mut MeshMaterial2d<CustomMaterial1>,
    ),
    With<MapTileSprite>,
>;
type TileVisualChangedQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Behavior,
        &'static mut MeshMaterial2d<CustomMaterial1>,
    ),
    (With<MapTileSprite>, Changed<Behavior>),
>;

fn resolve_visual_tuid(
    sprite_db: &SpriteDB,
    behavior: &Behavior,
    visual_state: TileState,
) -> Option<(String, u32)> {
    let key = (behavior.key_cvo(), visual_state);
    let variants = sprite_db.cvo_idx.get(&key.0)?;

    variants.iter().find_map(|tuid| {
        let candidate = sprite_db.map_tile.get(tuid)?;
        (candidate.behavior.state() == key.1).then(|| tuid.clone())
    })
}

fn canonical_material_for_tuid(
    tilesetdb: &MapTileSetDb,
    tuid: &(String, u32),
) -> Option<CustomMaterial1> {
    let (tileset_name, tileuid) = tuid;
    let tileset = tilesetdb.db.get(tileset_name)?;

    let mut material = match &tileset.data {
        AtlasData::Sheet((_handle, cmat)) => {
            let mut cmat = cmat.clone();
            cmat.data.upscale_factor = tileset.factor;
            cmat.data.sheet_idx = *tileuid;
            cmat.data.y_anchor = tileset.y_anchor;
            cmat
        }
        AtlasData::Tiles(v_img) => {
            let (_image_handle, mut cmat) = v_img.get(*tileuid as usize)?.clone();
            cmat.data.sheet_cols = 1;
            cmat.data.sheet_rows = 1;
            cmat.data.sheet_idx = 0;
            cmat.data.y_anchor = tileset.y_anchor;
            cmat
        }
        AtlasData::Headless => return None,
    };

    material.data.color.set_alpha(0.0);
    material.data.gamma = 0.1;
    material.data.gbl = 0.1;
    material.data.gbr = 0.1;
    material.data.gtl = 0.1;
    material.data.gtr = 0.1;
    Some(material)
}

fn preserve_runtime_lighting(current: &CustomMaterial1, target: &mut CustomMaterial1) {
    target.data.color = current.data.color;
    target.data.ctl = current.data.ctl;
    target.data.ctr = current.data.ctr;
    target.data.cbl = current.data.cbl;
    target.data.cbr = current.data.cbr;
    target.data.ambient_color = current.data.ambient_color;
    target.data.gamma = current.data.gamma;
    target.data.gtl = current.data.gtl;
    target.data.gtr = current.data.gtr;
    target.data.gbl = current.data.gbl;
    target.data.gbr = current.data.gbr;
}

fn matches_visual_identity(current: &CustomMaterial1, target: &CustomMaterial1) -> bool {
    current.texture() == target.texture()
        && current.data.sheet_rows == target.data.sheet_rows
        && current.data.sheet_cols == target.data.sheet_cols
        && current.data.sheet_idx == target.data.sheet_idx
        && current.data.sprite_width == target.data.sprite_width
        && current.data.sprite_height == target.data.sprite_height
        && current.data.padding == target.data.padding
        && current.data.margin == target.data.margin
        && current.data.y_anchor == target.data.y_anchor
        && current.data.upscale_factor == target.data.upscale_factor
}

fn update_single_visual(
    behavior: &Behavior,
    material_handle: &mut MeshMaterial2d<CustomMaterial1>,
    has_power: bool,
    sprite_db: &SpriteDB,
    tilesetdb: &MapTileSetDb,
    materials1: &mut Assets<CustomMaterial1>,
) {
    let visual_state = if behavior.p.is_house_powered && !has_power {
        TileState::Off
    } else {
        behavior.state()
    };

    let Some(target_tuid) = resolve_visual_tuid(sprite_db, behavior, visual_state.clone()) else {
        warn!(
            "update_single_visual: missing visual variant for {} -> {:?}",
            behavior.key_cvo().to_key_string(),
            visual_state
        );
        return;
    };

    let Some(current_mat) = materials1.get(&material_handle.0).cloned() else {
        warn!(
            "update_single_visual: current material handle missing for {}",
            behavior.key_cvo().to_key_string()
        );
        return;
    };

    let Some(mut target_mat) = canonical_material_for_tuid(tilesetdb, &target_tuid) else {
        warn!(
            "update_single_visual: missing canonical material for tileset '{}' tile {}",
            target_tuid.0, target_tuid.1
        );
        return;
    };

    if matches_visual_identity(&current_mat, &target_mat) {
        return;
    }

    // If the current sheet_idx already belongs to a valid tile in the same CVO group
    // with the same visual state, it's just a different random variant — keep it.
    let current_tuid = (behavior.key_tuid().0, current_mat.data.sheet_idx);
    if sprite_db.map_tile.get(&current_tuid).is_some_and(|c| {
        c.behavior.key_cvo() == behavior.key_cvo() && c.behavior.state() == visual_state
    }) {
        return;
    }

    preserve_runtime_lighting(&current_mat, &mut target_mat);
    *material_handle = MeshMaterial2d(materials1.add(target_mat));
}

pub(crate) fn update_power_visuals(
    lg: If<Res<LightGrid>>,
    sprite_db: Res<SpriteDB>,
    tilesetdb: Res<MapTileSetDb>,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut q_set: ParamSet<(
        TileVisualMutableQuery<'_, '_>,
        TileVisualChangedQuery<'_, '_>,
    )>,
    q_breaker_behavior: Query<&Behavior>,
    mut last_has_power: Local<Option<bool>>,
) {
    let has_power = lg.has_power(&q_breaker_behavior);
    let power_changed = Some(has_power) != *last_has_power;
    *last_has_power = Some(has_power);

    if power_changed {
        for (_entity, behavior, mut material_handle) in q_set.p0().iter_mut() {
            update_single_visual(
                behavior,
                &mut material_handle,
                has_power,
                &sprite_db,
                &tilesetdb,
                &mut materials1,
            );
        }
    } else {
        for (_entity, behavior, mut material_handle) in q_set.p1().iter_mut() {
            update_single_visual(
                behavior,
                &mut material_handle,
                has_power,
                &sprite_db,
                &tilesetdb,
                &mut materials1,
            );
        }
    }
}
