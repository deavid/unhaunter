//! Functionality to load Tiled maps (tilesets and tilemaps) into Bevy for
//! Unhaunter.
//!
//! Most of the classes here are almost a redefinition (for now) of the tiled
//! library. Currently serve as an example on how to load/store data.

pub(crate) mod bevy;
pub(crate) mod init_maps;
pub(crate) mod load;
pub(crate) mod load_level;
pub(crate) mod map_loader;
pub mod plugin;
