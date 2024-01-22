//! Behavior module
//! ----------------
//!
//! The objective of this module is to replace the TileSprite code in board.rs
//! so all the logic and behavior functionality comes here.
//!
//! This must represent all possible functionality that can be expressed from
//! Tiled.
//!
//! In Tiled, it should be possible to have only the following defined:
//! - Class / user_type: Door
//! - sprite:orientation: Y
//! - sprite:state: closed
//! - sprite:variant: wooden
//!
//! When loaded, we should be able to transform that into a property list that
//! defines the behavior.
//!

use crate::{board::TileSprite, tiledmap::MapTile};
use anyhow::Context;
use bevy::ecs::component::{Component, TableStorage};
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Behavior {
    cfg: SpriteConfig,
    pub p: Properties,
}

impl Behavior {
    pub fn from_config(cfg: SpriteConfig) -> Self {
        let mut p = Properties::default();
        cfg.class.set_properties(&mut p);
        Self { cfg, p }
    }
    pub fn default_components(&self) -> Vec<Box<dyn Component<Storage = TableStorage>>> {
        self.cfg.class.components()
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Properties {
    // ---
    pub movement: Movement,
    pub light: Light,
    pub util: Util,
    pub display: Display,
    pub obsolete: Obsolete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Obsolete {
    pub sprite: TileSprite,
}

impl Default for Obsolete {
    fn default() -> Self {
        Self {
            sprite: TileSprite::FloorTile,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Util {
    PlayerSpawn,
    GhostSpawn,
    Van,
    #[default]
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Display {
    pub disable: bool,
    pub global_z: NotNan<f32>,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Light {
    pub opaque: bool,
    pub emits_light: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Movement {
    /// true for floors only, where the player can stand on this spot
    pub walkable: bool,

    /// true for walls and closed doors. It signals that the player cannot stand
    /// here. For detailed collision, there will be a map of collision.
    pub collision: bool,
    // 9x9 collision map on the sub-tile. This is using a subtile of 3x3, so it
    // means it can cover an area of 3x3 board tiles.
    // collision_map: [[bool; 9]; 9],
}

pub mod component {
    use bevy::ecs::component::Component;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Ground;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Collision;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Opaque;
    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct UVSurface;
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Class {
    Floor,
    Wall,
    Door,
    Switch,
    RoomSwitch,
    Breaker,
    Doorway,
    Decor,
    Item,
    Furniture,
    PlayerSpawn,
    GhostSpawn,
    VanEntry,
    RoomDef,
    WallLamp,
    FloorLamp,
    TableLamp,
    WallDecor,
    CeilingLight,
    Appliance,
    Van,
    Window,
    #[default]
    None,
}

impl AutoSerialize for Class {}

impl Class {
    pub fn components(&self) -> Vec<Box<dyn Component<Storage = TableStorage>>> {
        match self {
            Class::Floor => vec![Box::new(component::Ground), Box::new(component::UVSurface)],
            Class::Wall => vec![
                Box::new(component::Collision),
                Box::new(component::Opaque),
                Box::new(component::UVSurface),
            ],
            Class::Door => vec![],
            Class::Switch => vec![],
            Class::RoomSwitch => vec![],
            Class::Breaker => vec![],
            Class::Doorway => vec![],
            Class::Decor => vec![],
            Class::Item => vec![],
            Class::Furniture => vec![],
            Class::PlayerSpawn => vec![],
            Class::GhostSpawn => vec![],
            Class::VanEntry => vec![],
            Class::RoomDef => vec![],
            Class::WallLamp => vec![],
            Class::FloorLamp => vec![],
            Class::TableLamp => vec![],
            Class::WallDecor => vec![],
            Class::CeilingLight => vec![],
            Class::Appliance => vec![],
            Class::Van => vec![],
            Class::Window => vec![],
            Class::None => vec![],
        }
    }
    pub fn set_properties(&self, p: &mut Properties) {
        match self {
            Class::Floor => {
                p.movement.walkable = true;
                p.display.global_z = (-0.00025).try_into().unwrap();
            }
            Class::Wall => {
                p.movement.collision = true;
                p.light.opaque = true;
                p.obsolete.sprite = TileSprite::Pillar;
                p.display.global_z = (-0.00005).try_into().unwrap();
            }
            Class::Door => {
                p.display.global_z = (0.000015).try_into().unwrap();
            }
            Class::Switch => {
                p.display.global_z = (0.000040).try_into().unwrap();
            }
            Class::RoomSwitch => {
                p.display.global_z = (0.000040).try_into().unwrap();
            }
            Class::Breaker => {
                p.display.global_z = (0.000040).try_into().unwrap();
            }
            Class::Doorway => {
                p.display.global_z = (-0.00005).try_into().unwrap();
            }
            Class::Decor => {
                p.display.global_z = (0.000065).try_into().unwrap();
            }
            Class::Item => {
                p.display.global_z = (0.000065).try_into().unwrap();
            }
            Class::Furniture => {
                p.display.global_z = (0.000050).try_into().unwrap();
            }
            Class::PlayerSpawn => {
                p.display.global_z = (-1.0).try_into().unwrap();
                p.display.disable = true;
                p.obsolete.sprite = TileSprite::Util;
                p.util = Util::PlayerSpawn;
            }
            Class::GhostSpawn => {
                p.display.global_z = (-1.0).try_into().unwrap();
                p.display.disable = true;
                p.obsolete.sprite = TileSprite::Util;
                p.util = Util::GhostSpawn;
            }
            Class::VanEntry => {
                p.display.global_z = (-1.0).try_into().unwrap();
                p.display.disable = true;
                p.obsolete.sprite = TileSprite::Util;
                p.util = Util::Van;
            }
            Class::RoomDef => {
                p.display.global_z = (-1.0).try_into().unwrap();
                p.display.disable = true;
                p.obsolete.sprite = TileSprite::Util;
            }
            Class::WallLamp => {
                p.display.global_z = (-0.00004).try_into().unwrap();
                p.obsolete.sprite = TileSprite::Lamp;
            }
            Class::FloorLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.obsolete.sprite = TileSprite::Lamp;
            }
            Class::TableLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.obsolete.sprite = TileSprite::Lamp;
            }
            Class::WallDecor => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::CeilingLight => {
                p.display.global_z = (-1.0).try_into().unwrap();
                p.display.disable = true;
                p.obsolete.sprite = TileSprite::CeilingLight;
            }
            Class::Appliance => {
                p.display.global_z = (0.000070).try_into().unwrap();
            }
            Class::Van => {
                p.display.global_z = (0.000200).try_into().unwrap();
            }
            Class::Window => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::None => {}
        }
    }

    /// A class requires a set of states. Not only these are the only valid ones for the given class, also they need all to be included.
    fn required_states(&self) -> Vec<State> {
        use State::*;

        match self {
            Class::Wall => vec![Full, Partial, Minimum],
            Class::Door => vec![Open, Closed],
            Class::Switch => vec![On, Off],
            Class::Breaker => vec![On, Off],
            _ => vec![None],
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Orientation {
    XAxis,
    YAxis,
    Both,
    #[default]
    None,
}

impl AutoSerialize for Orientation {}

trait AutoSerialize: Serialize + for<'a> Deserialize<'a> + Default {
    fn from_text(text: Option<&str>) -> anyhow::Result<Self> {
        let Some(text) = text else {
            return Ok(Self::default());
        };
        let t = format!("\"{text}\"");
        serde_json::from_str(&t).context("Auto deserialize error")
    }
    fn to_text(&self) -> anyhow::Result<String> {
        serde_json::to_string(self)
            .map(|x| x.replace('"', ""))
            .context("Auto serialize error")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum State {
    // Switch states
    On,
    Off,
    // Door states
    Open,
    Closed,
    // Wall states
    Full,
    Partial,
    Minimum,

    // Default state
    #[default]
    None,
}

impl AutoSerialize for State {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteConfig {
    /// Main behavior class
    pub class: Class,
    /// Custom variant name - must be the same across all sprites that represent the same object
    pub variant: String,
    /// Orientation of the sprite - if it's facing one axis or another.
    pub orientation: Orientation,
    /// Current state of the sprite - or the initial state.
    pub state: State,

    // other interesting metadata
    /// Tileset name
    pub tileset: String,
    /// UID of the tileset for this sprite
    pub tileuid: u32,
}

impl SpriteConfig {
    pub fn from_tiled_auto(tile: &MapTile, tiled_tile: &tiled::Tile) -> Self {
        let parse = |x: &tiled::PropertyValue| -> String {
            match x {
                tiled::PropertyValue::BoolValue(x) => x.to_string(),
                tiled::PropertyValue::FloatValue(x) => x.to_string(),
                tiled::PropertyValue::IntValue(x) => x.to_string(),
                tiled::PropertyValue::ColorValue(x) => {
                    format!("{},{},{},{}", x.red, x.green, x.blue, x.alpha)
                }
                tiled::PropertyValue::StringValue(x) => x.to_string(),
                tiled::PropertyValue::FileValue(x) => x.to_string(),
                tiled::PropertyValue::ObjectValue(x) => x.to_string(),
            }
        };
        let get_property_str =
            |key: &str| -> Option<String> { tiled_tile.properties.get(key).map(parse) };
        Self::from_tiled(
            tiled_tile.user_type.as_deref(),
            get_property_str("sprite:variant").as_deref(),
            get_property_str("sprite:orientation").as_deref(),
            get_property_str("sprite:state").as_deref(),
            tile.tileset.clone(),
            tile.tileuid,
        )
    }

    pub fn from_tiled(
        class: Option<&str>,
        variant: Option<&str>,
        orientation: Option<&str>,
        state: Option<&str>,
        tileset: String,
        tileuid: u32,
    ) -> Self {
        Self::try_from_tiled(class, variant, orientation, state, tileset.clone(), tileuid)
            .with_context(|| {
                format!(
                    "SpriteConfig: error loading sprite from tiled: {}:{} [c:{:?}, v:{:?}, o:{:?}, s:{:?}]",
                    tileset, tileuid, class, variant, orientation, state
                )
            })
            .unwrap()
    }
    pub fn try_from_tiled(
        class: Option<&str>,
        variant: Option<&str>,
        orientation: Option<&str>,
        state: Option<&str>,
        tileset: String,
        tileuid: u32,
    ) -> anyhow::Result<Self> {
        let tilesetuid_key = format!("{}:{}", tileset, tileuid);
        let class = Class::from_text(class).context("parsing Class")?;
        let variant = variant.unwrap_or(&tilesetuid_key).to_owned();
        let orientation = Orientation::from_text(orientation).context("parsing Orientation")?;
        let state = State::from_text(state).context("parsing State")?;

        Ok(SpriteConfig {
            class,
            variant,
            orientation,
            state,
            tileset,
            tileuid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn class_serialization() {
        let c = Class::Breaker;
        let ct = c.to_text().unwrap();
        assert_eq!(ct, "Breaker");
        let d = Class::from_text(Some(&ct)).unwrap();
        assert_eq!(d, c);
    }
    #[test]
    fn state_serialization() {
        let c = State::On;
        let ct = c.to_text().unwrap();
        assert_eq!(ct, "On");
        let d = State::from_text(Some(&ct)).unwrap();
        assert_eq!(d, c);
    }
}
