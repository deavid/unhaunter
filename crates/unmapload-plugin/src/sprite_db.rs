//! # Sprite Database Module
//!
//! This module is responsible for populating the SpriteDB resource with tile data from tilesets.
//! The SpriteDB serves as a behavior lookup/index used by map loading and interaction logic.

use unbehavior_core::behavior::Behavior;
use unmapload_core::components::MapTileComponents;

use crate::level_setup::LoadLevelSystemParam;

/// Pre-computes and populates the SpriteDB resource with tile data from all available tilesets.
///
/// This function is called during level loading to prepare all tile visual data and behavior properties
/// that will be used when spawning tile entities.
///
/// # Arguments
/// * `p` - System parameters containing resources needed for sprite loading
pub(crate) fn populate_sprite_db(p: &mut LoadLevelSystemParam) {
    // Iterate through all tilesets in the database
    for (tset_name, tileset) in &p.tilesetdb.db {
        for (tileuid, tiled_tile) in tileset.tileset.tiles() {
            // Create sprite configuration from the tile data
            let sprite_config = unbehavior::behavior::sprite_config_from_tiled_auto(
                tset_name.clone(),
                tileuid,
                &tiled_tile,
            );
            let behavior = Behavior::from_config(sprite_config);

            // Store the tile data in the sprite database
            let key_tuid = behavior.key_tuid();
            p.sdb
                .cvo_idx
                .entry(behavior.key_cvo())
                .or_default()
                .push(key_tuid.clone());
            let mt = MapTileComponents { behavior };
            p.sdb.map_tile.insert(key_tuid, mt);
        }
    }
}
