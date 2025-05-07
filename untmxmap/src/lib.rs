//! Functionality to load Tiled maps (tilesets and tilemaps) into Bevy for
//! Unhaunter.
//!
//! Most of the classes here are almost a redefinition (for now) of the tiled
//! library. Currently serve as an example on how to load/store data.

pub mod bevy;
pub mod init_maps;
pub mod load;
pub mod load_level;
pub mod map_loader;
pub mod plugin;
pub mod campaign_loader;
