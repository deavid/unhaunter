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

use anyhow::Context;
use bevy::ecs::component::Component;
use serde::{Deserialize, Serialize};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
struct Behavior {
    cfg: SpriteConfig,
    p: Properties,
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
struct Properties {
    // ---
    movement: Movement,
    light: Light,
    util: Util,
    /// If true marks that this is not a sprite, should never be drawn. Only for data.
    data_only: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
enum Util {
    PlayerSpawn,
    GhostSpawn,
    Van,
    #[default]
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Light {
    is_opaque: bool,
    emits_light: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct Movement {
    /// true for floors only, where the player can stand on this spot
    walkable: bool,

    /// true for walls and closed doors. It signals that the player cannot stand
    /// here. For detailed collision, there will be a map of collision.
    collision: bool,
    // 9x9 collision map on the sub-tile. This is using a subtile of 3x3, so it
    // means it can cover an area of 3x3 board tiles.
    // collision_map: [[bool; 9]; 9],
}

/// When several sprites go onto the same tile on the board, this represents
/// the combined data for a single part of the board
struct TileProperties {
    /// true if the player can walk through this tile position.
    /// In simplified form, false when there's a collision sprite on it.
    /// In more detailed form, false only if there's no walkable floor sprite in here.
    is_walkable: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
enum Class {
    Floor,
    Wall,
    Door,
    Switch,
    Breaker,
    #[default]
    None,
}

impl AutoSerialize for Class {}

impl Class {
    fn properties(&self) -> Properties {
        match self {
            Class::Floor => Properties {
                movement: Movement {
                    walkable: true,
                    ..Default::default()
                },
                ..Default::default()
            },
            _ => todo!(),
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
            Class::Floor => vec![None],
            Class::None => vec![None],
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
enum Orientation {
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
enum State {
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
