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
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::{maplight, tiledmap::MapLayer};

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Behavior {
    cfg: SpriteConfig,
    pub p: Properties,
}

impl Behavior {
    pub fn from_config(cfg: SpriteConfig) -> Self {
        let mut p = Properties::default();
        cfg.set_properties(&mut p);
        Self { cfg, p }
    }
    pub fn flip(&mut self, f: bool) {
        if f != self.p.flip {
            self.cfg.orientation.flip();
            self.p.flip = f;
        }
    }
    pub fn state(&self) -> State {
        self.cfg.state.clone()
    }
    pub fn default_components(
        &self,
        entity: &mut bevy::ecs::system::EntityCommands,
        layer: &MapLayer,
    ) {
        self.cfg.components(entity, layer)
    }
    pub fn key_cvo(&self) -> SpriteCVOKey {
        self.cfg.key_cvo()
    }
    pub fn key_tuid(&self) -> (String, u32) {
        self.cfg.key_tuid()
    }
    pub fn obsolete_occlusion_type(&self) -> Orientation {
        if !self.p.light.opaque {
            return Orientation::None;
        }
        self.cfg.orientation.clone()
    }
    /// Amount of "watts" of heat poured into the environment
    pub fn temp_heat_output(&self) -> f32 {
        let heat_coeff = (self.p.light.heat_coef as f32).exp();
        self.p.light.emmisivity_lumens() / 10000.0 * heat_coeff
    }
    /// Resistance to change temperature (how many Joules per Celsius)
    pub fn _temp_heat_capacity(&self) -> f32 {
        let f1 = match self.p.light.opaque {
            true => 10000.0,
            false => 10.0,
        };
        let f2 = match self.p.movement.walkable {
            true => 100.0,
            false => 0.0,
        };
        f1 + f2
    }
    /// Heat Conductivity, Watts per Meter*Kelvin (how many watts are transferred at a meter on a 1ºC difference)
    /// (f32, f32): (W/mK, weight), weight is used for averaging purposes.
    pub fn _temp_heat_conductivity(&self) -> (f32, f32) {
        match self.p.light.opaque {
            true => (0.001, 1000.0),
            false => (10.0, 0.1),
        }
    }

    pub fn is_van_entry(&self) -> bool {
        self.cfg.class == Class::VanEntry
    }

    pub fn is_npc(&self) -> bool {
        self.cfg.class == Class::NPC
    }
}

#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Properties {
    // ---
    pub movement: Movement,
    pub light: Light,
    pub util: Util,
    pub display: Display,
    pub flip: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Util {
    RoomDef(String),
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
    pub see_through: bool,
    pub emits_light: bool,
    pub emission_power: NotNan<f32>,
    pub heat_coef: i32,
}

impl Light {
    pub fn emmisivity_lumens(&self) -> f32 {
        match self.emits_light {
            true => self.emission_power.exp(),
            false => 0.0,
        }
    }
    pub fn transmissivity_factor(&self) -> f32 {
        match self.opaque {
            true => 0.00,
            false => 1.01,
        }
    }
    /// This represents if a light on the map is emitting visible light or other types.
    pub fn additional_data(&self) -> maplight::LightData {
        maplight::LightData::UNIT_VISIBLE
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Movement {
    /// true for floors only, where the player can stand on this spot
    pub walkable: bool,

    /// true for walls and closed doors. It signals that the player cannot stand
    /// here. For detailed collision, there will be a map of collision.
    pub player_collision: bool,

    pub ghost_collision: bool,
    // 9x9 collision map on the sub-tile. This is using a subtile of 3x3, so it
    // means it can cover an area of 3x3 board tiles.
    // collision_map: [[bool; 9]; 9],
}

pub mod component {
    use bevy::{ecs::component::Component, log::warn, math::Vec3};

    use crate::{board::BoardPosition, tiledmap::MapLayer};

    use super::Behavior;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Ground;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Collision;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Opaque;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct UVSurface;

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct RoomState {
        pub room_delta: BoardPosition,
    }

    impl RoomState {
        pub fn new() -> Self {
            Self {
                room_delta: BoardPosition::default(),
            }
        }
        pub fn new_for_room(orientation: &super::Orientation) -> Self {
            Self {
                room_delta: match orientation {
                    super::Orientation::XAxis => BoardPosition { x: -1, y: 1, z: 0 },
                    super::Orientation::YAxis => BoardPosition { x: -1, y: 1, z: 0 },
                    super::Orientation::Both => BoardPosition::default(),
                    super::Orientation::None => BoardPosition::default(),
                },
            }
        }
    }

    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct Interactive {
        pub on_activate_sound_file: String,
        pub on_deactivate_sound_file: String,
    }

    impl Interactive {
        pub fn new(activate: &str, deactivate: &str) -> Self {
            let on_activate_sound_file = activate.to_string();
            let on_deactivate_sound_file = deactivate.to_string();
            Self {
                on_activate_sound_file,
                on_deactivate_sound_file,
            }
        }
        pub fn sound_for_moving_into_state(&self, behavior: &Behavior) -> String {
            match behavior.cfg.state {
                super::State::On => self.on_activate_sound_file.clone(),
                super::State::Off => self.on_deactivate_sound_file.clone(),
                super::State::Open => self.on_activate_sound_file.clone(),
                super::State::Closed => self.on_deactivate_sound_file.clone(),
                super::State::Full => self.on_activate_sound_file.clone(),
                super::State::Partial => self.on_activate_sound_file.clone(),
                super::State::Minimum => self.on_activate_sound_file.clone(),
                super::State::None => self.on_deactivate_sound_file.clone(),
            }
        }
        pub fn control_point_delta(&self, behavior: &Behavior) -> Vec3 {
            match behavior.cfg.class {
                super::Class::Door => match behavior.cfg.orientation {
                    super::Orientation::XAxis => Vec3::new(0.0, -0.25, 0.0),
                    super::Orientation::YAxis => Vec3::new(0.25, 0.0, 0.0),
                    _ => Vec3::ZERO,
                },
                _ => Vec3::ZERO,
            }
        }
    }
    #[derive(Component, Debug, Clone, PartialEq, Eq)]
    pub struct NpcHelpDialog {
        pub dialog: String,
    }

    impl NpcHelpDialog {
        pub fn new(classname: &str, variant: &str, layer: &MapLayer) -> Self {
            let key = format!("{classname}:{variant}:dialog");
            let dialog = match layer.user_properties.get(&key) {
                Some(p) => match p {
                    tiled::PropertyValue::StringValue(v) => v.to_string(),
                    _ => {
                        warn!("NPCHelpDialog was expecting a user property named {key:?} in the layer but it had an unsupported type - it must be text");
                        "".to_string()
                    }
                },
                None => {
                    warn!("NPCHelpDialog was expecting a user property named {key:?} in the layer but was not present");
                    "".to_string()
                }
            };
            Self { dialog }
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Class {
    Floor,
    Wall,
    LowWall,
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
    StreetLight,
    Appliance,
    Van,
    Window,
    InvisibleWall,
    CornerWall,
    #[allow(clippy::upper_case_acronyms)]
    NPC,
    FakeGhost,
    FakeBreach,
    #[default]
    None,
}

impl AutoSerialize for Class {}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum Orientation {
    XAxis,
    YAxis,
    Both,
    #[default]
    None,
}

impl Orientation {
    fn flip(&mut self) {
        match self {
            Orientation::XAxis => *self = Orientation::YAxis,
            Orientation::YAxis => *self = Orientation::XAxis,
            Orientation::Both => {}
            Orientation::None => {}
        }
    }
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
pub struct SpriteCVOKey {
    pub class: Class,
    pub variant: String,
    pub orientation: Orientation,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteConfig {
    /// Main behavior class
    class: Class,
    /// Custom variant name - must be the same across all sprites that represent the same object
    variant: String,
    /// Orientation of the sprite - if it's facing one axis or another.
    orientation: Orientation,

    // Backup of the original data for the key
    cvo_key: SpriteCVOKey,
    /// Current state of the sprite - or the initial state.
    pub state: State,

    // other interesting metadata
    /// Tileset name
    pub tileset: String,
    /// UID of the tileset for this sprite
    pub tileuid: u32,
}

impl SpriteConfig {
    pub fn key_cvo(&self) -> SpriteCVOKey {
        self.cvo_key.clone()
    }
    pub fn key_tuid(&self) -> (String, u32) {
        (self.tileset.clone(), self.tileuid)
    }
    pub fn from_tiled_auto(tset_name: String, tileuid: u32, tiled_tile: &tiled::Tile) -> Self {
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
            tset_name,
            tileuid,
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
        let cvo_key = SpriteCVOKey {
            class: class.clone(),
            variant: variant.clone(),
            orientation: orientation.clone(),
        };
        Ok(SpriteConfig {
            class,
            variant,
            orientation,
            state,
            tileset,
            tileuid,
            cvo_key,
        })
    }

    pub fn components(&self, entity: &mut bevy::ecs::system::EntityCommands, layer: &MapLayer) {
        match self.class {
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
            Class::Door => entity.insert(component::Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            )),
            Class::Switch => entity
                .insert(component::Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::RoomState::new()),
            Class::RoomSwitch => entity
                .insert(component::Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::RoomState::new_for_room(&self.orientation)),
            Class::Breaker => entity.insert(component::Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            )),
            Class::Doorway => entity,
            Class::Decor => entity,
            Class::Item => entity,
            Class::Furniture => entity,
            Class::PlayerSpawn => entity,
            Class::GhostSpawn => entity,
            Class::VanEntry => entity.insert(component::Interactive::new(
                "sounds/door-open.ogg",
                "sounds/door-close.ogg",
            )),
            Class::RoomDef => entity,
            Class::WallLamp => entity.insert(component::RoomState::new()),
            Class::FloorLamp => entity.insert(component::Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            )),
            Class::TableLamp => entity.insert(component::Interactive::new(
                "sounds/switch-on-1.ogg",
                "sounds/switch-off-1.ogg",
            )),
            Class::WallDecor => entity,
            Class::CeilingLight => entity.insert(component::RoomState::new()),
            Class::StreetLight => entity,
            Class::Appliance => entity,
            Class::Van => entity,
            Class::Window => entity,
            Class::None => entity,
            Class::InvisibleWall => entity,
            Class::CornerWall => entity,
            Class::FakeBreach => entity,
            Class::FakeGhost => entity,
            Class::NPC => entity
                .insert(component::NpcHelpDialog::new("NPC", &self.variant, layer))
                .insert(component::Interactive::new(
                    "sounds/effects-dongdongdong.ogg",
                    "sounds/effects-dongdongdong.ogg",
                )),
        };
    }
    pub fn set_properties(&self, p: &mut Properties) {
        match self.class {
            Class::Floor => {
                p.movement.walkable = true;
                p.display.global_z = (-0.00035).try_into().unwrap();
            }
            Class::Wall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.opaque = true;
                p.display.global_z = (-0.00005).try_into().unwrap();
            }
            Class::LowWall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.see_through = true;
                p.display.global_z = (-0.00005).try_into().unwrap();
            }
            Class::Door => {
                p.display.global_z = (0.000015).try_into().unwrap();
                p.movement.player_collision = self.state == State::Closed;
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
            Class::NPC => {
                p.display.global_z = (0.000050).try_into().unwrap();
            }
            Class::InvisibleWall => {
                p.movement.player_collision = true;
                p.light.see_through = true;
                p.display.disable = true;
            }
            Class::CornerWall => {
                p.movement.player_collision = true;
                p.light.see_through = false;
                p.display.disable = true;
            }
            Class::PlayerSpawn => {
                p.display.disable = true;
                p.util = Util::PlayerSpawn;
            }
            Class::GhostSpawn => {
                p.display.disable = true;
                p.util = Util::GhostSpawn;
            }
            Class::FakeGhost => {
                p.display.disable = true;
                // p.util = Util::GhostSpawn;
            }
            Class::FakeBreach => {
                p.display.disable = true;
                // p.util = Util::GhostSpawn;
            }
            Class::VanEntry => {
                // p.display.disable = true;
                p.util = Util::Van;
            }
            Class::RoomDef => {
                p.display.disable = true;
                p.util = Util::RoomDef(self.variant.clone());
            }
            Class::WallLamp => {
                p.display.global_z = (-0.00004).try_into().unwrap();
                p.light.emits_light = self.state == State::On;
                p.light.emission_power = (5.5).try_into().unwrap();
                p.light.heat_coef = -1;
            }
            Class::FloorLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.light.emits_light = self.state == State::On;
                p.light.emission_power = (6.0).try_into().unwrap();
            }
            Class::TableLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.light.emits_light = self.state == State::On;
                p.light.emission_power = (6.5).try_into().unwrap();
            }
            Class::WallDecor => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::CeilingLight => {
                p.display.disable = true;
                p.light.emits_light = self.state == State::On;
                p.light.emission_power = (7.0).try_into().unwrap();
                p.light.heat_coef = -2;
            }
            Class::StreetLight => {
                p.display.disable = true;
                p.light.emits_light = true;
                p.light.emission_power = (6.0).try_into().unwrap();
                p.light.heat_coef = -6;
            }
            Class::Appliance => {
                p.display.global_z = (0.000070).try_into().unwrap();
            }
            Class::Van => {
                p.display.global_z = (0.000050).try_into().unwrap();
            }
            Class::Window => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::None => {}
        }
    }

    /// A class requires a set of states. Not only these are the only valid ones for the given class, also they need all to be included.
    fn _required_states(&self) -> Vec<State> {
        use State::*;

        match self.class {
            Class::Wall => vec![Full, Partial, Minimum],
            Class::Door => vec![Open, Closed],
            Class::Switch => vec![On, Off],
            Class::Breaker => vec![On, Off],
            _ => vec![None],
        }
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
