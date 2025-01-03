//! ## Gear Module
//!
//! This module defines the gear system for the game, including:
//!
//! * Different types of gear available to the player.
//!
//! * A common interface for interacting with gear (`GearUsable` trait).
//!
//! * Functions for updating gear state based on player actions and game conditions.
//!
//! * Visual representations of gear using sprites and animations.
//!
//! The gear system allows players to equip and use various tools to investigate
//! paranormal activity, gather evidence, and ultimately banish ghosts.
pub mod ext;
pub mod playergear;
pub mod systems;
pub mod ui;

use bevy::prelude::*;
use uncore::components::game_config::GameConfig;
use uncore::events::sound::SoundEvent;

pub use uncore::types::gear::spriteid::GearSpriteID;

pub fn app_setup(app: &mut App) {
    use crate::gear::ext::types::items::*;

    app.init_resource::<GameConfig>()
        .add_systems(FixedUpdate, systems::update_playerheld_gear_data)
        .add_systems(FixedUpdate, systems::update_deployed_gear_data)
        .add_systems(FixedUpdate, systems::update_deployed_gear_sprites)
        .add_systems(Update, quartz::update_quartz_and_ghost)
        .add_systems(Update, salt::salt_particle_system)
        .add_systems(Update, salt::salt_pile_system)
        .add_systems(Update, salt::salty_trace_system)
        .add_systems(Update, sage::sage_smoke_system)
        .add_systems(Update, thermometer::temperature_update)
        .add_systems(Update, recorder::sound_update)
        .add_systems(Update, repellentflask::repellent_update)
        .add_systems(Update, systems::sound_playback_system)
        .add_event::<SoundEvent>();
    ui::app_setup(app);
}
