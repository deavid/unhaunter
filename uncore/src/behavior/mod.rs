//! ## Behavior module
//!
//! This module defines the `Behavior` component and its associated data
//! structures, which are used to represent the behavior of objects in the game
//! world.
//!
//! The `Behavior` component stores information about the object's type, variant,
//! orientation, state, and a collection of properties that determine how it
//! interacts with the player and the environment. This component is crucial for
//! separating the object's visual representation (its sprite) from its logical
//! behavior.
//!
//! The information stored in the `Behavior` component is loaded from Tiled map
//! data. Each tile in Tiled can be assigned a "class" (e.g., "Door", "Wall",
//! "Light"), a "variant" (e.g., "wooden", "brick", "fluorescent"), an
//! "orientation", and a "state" (e.g., "open", "closed", "on", "off").
//!
//! This data is used to create a `SpriteConfig` struct, which is then used to
//! initialize the `Behavior` component. The `Behavior` component, in turn, is used
//! to add other Bevy components to the object's entity, such as `Collision`,
//! `Interactive`, `Light`, etc., based on its configuration.
pub mod component;

use anyhow::Context;
use bevy::{ecs::component::Component, utils::HashMap};
use fastapprox::faster;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};

use crate::types::board::light::LightData;
use crate::types::tiledmap::map::MapLayer;

/// The `Behavior` component defines the behavior of an object in the game world.
///
/// It stores a `SpriteConfig` struct, which contains the object's basic
/// configuration (class, variant, orientation, state), and a `Properties` struct,
/// which holds a collection of properties that determine how the object interacts
/// with the player and the environment.
#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Behavior {
    /// This `cfg` property is PRIVATE on purpose! We need to separate the "what it is"
    /// from "what it does". There is always a tendency to read and write cfg from
    /// everywhere because it is "the easiest" thing to do, but that creates a mess in
    /// the code later on because we are not separating behavior from raw data. Always
    /// place behavioral traits in "p: Properties" and never read or write from Cfg.
    cfg: SpriteConfig,
    /// The `p` field stores a collection of properties that define the object's
    /// behavior.
    pub p: Properties,
}

impl Behavior {
    /// Creates a new `Behavior` component from a `SpriteConfig`.
    ///
    /// The `cfg` field is set to the given `SpriteConfig`, and the `p` field is
    /// initialized based on the properties defined in the `SpriteConfig`.
    pub fn from_config(cfg: SpriteConfig) -> Self {
        let mut p = Properties::default();
        cfg.set_properties(&mut p);
        Self { cfg, p }
    }

    /// Flips horizontally a sprite. Some sprites work just by flipping the image,
    /// however other sprites have an alternative sprite for when it is flipped.
    pub fn flip(&mut self, f: bool) {
        if f != self.p.flip {
            self.cfg.orientation.flip();
            self.p.flip = f;
        }
    }

    /// Returns the state (On/Off, Open/Closed) as a copy so that it is not possible to
    /// modify the original from outside code.
    pub fn state(&self) -> TileState {
        self.cfg.state.clone()
    }

    /// Creates the default components as required by this behavior for a new entity.
    /// This is often used to spawn new map tiles to add the required components
    /// automatically.
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
        self.cfg.orientation
    }

    /// Amount of "watts" of heat poured into the environment
    pub fn temp_heat_output(&self) -> f32 {
        // FIXME: Precompute this value and store it. This is slow and it's computed every frame by the temperature system.
        let heat_coeff = faster::exp(self.p.light.heat_coef as f32);
        self.p.light.emmisivity_lumens() / 10000.0 * heat_coeff
    }

    /// Resistance to change temperature (how many Joules per Kelvin)
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

    /// Heat Conductivity, Watts per Meter*Kelvin (how many watts are transferred at a
    /// meter on a 1ºC difference) (f32, f32): (W/mK, weight), weight is used for
    /// averaging purposes.
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

    pub fn can_emit_light(&self) -> bool {
        self.p.light.emission_power.into_inner() > 1.0
    }

    pub fn orientation(&self) -> Orientation {
        self.cfg.orientation
    }
}

/// Stores a collection of properties that define the behavior of an object.
///
/// These properties determine how the object interacts with the player, the
/// environment, and other systems in the game.
#[derive(Default, Debug, Clone, PartialEq, Eq)]
pub struct Properties {
    // --- Movement Properties ---
    /// Properties related to movement and collision.
    pub movement: Movement,
    // --- Light Properties ---
    /// Properties related to light emission, opacity, and visibility.
    pub light: Light,
    // --- Utility Properties ---
    /// Properties that define the object's utility or purpose in the game world.
    pub util: Util,
    // --- Display Properties ---
    /// Properties related to the object's visual display, such as visibility and
    /// global Z position.
    pub display: Display,
    /// Whether the sprite should be horizontally flipped.
    pub flip: bool,
    /// Properties specific to objects in the game world.
    pub object: Object,
}

/// Represents properties specific to objects in the game world.
///
/// These properties determine how the player can interact with objects, such as
/// whether they can be picked up, moved, or used as hiding spots.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Object {
    pub pickable: bool,
    pub movable: bool,
    pub hidingspot: bool,
    pub weight: NotNan<f32>,
    pub name: String,
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
    /// Used to partially make transparent big objects when the player walks behind them
    pub auto_hide: bool,
    /// Mainly for the van, to make the light computation look at a different tile.
    pub light_recv_offset: (i64, i64),
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Light {
    pub opaque: bool,
    pub see_through: bool,
    pub light_emission_enabled: bool,
    pub can_emit_light: bool,
    pub emission_power: NotNan<f32>,
    pub heat_coef: i32,
    pub flickering: bool,
}

impl Light {
    pub fn emmisivity_lumens(&self) -> f32 {
        if self.flickering {
            // Reduced emission when flickering, with a slight glow even when off
            if self.light_emission_enabled {
                self.emission_power.exp() * 0.4
            } else {
                self.emission_power.exp() * 0.001
            }
        } else {
            // Normal emission based on emits_light
            match self.light_emission_enabled {
                true => self.emission_power.exp(),
                false => 0.0,
            }
        }
    }

    pub fn transmissivity_factor(&self) -> f32 {
        match self.opaque {
            true => 0.00,
            false => 1.01,
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        (1.0, 1.0, 1.0)
    }

    /// This represents if a light on the map is emitting visible light or other types.
    pub fn additional_data(&self) -> LightData {
        LightData::UNIT_VISIBLE
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Movement {
    /// true for floors only, where the player can stand on this spot
    pub walkable: bool,
    /// true for walls and closed doors. It signals that the player cannot stand here.
    /// For detailed collision, there will be a map of collision.
    pub player_collision: bool,
    pub ghost_collision: bool,
    // 9x9 collision map on the sub-tile. This is using a subtile of 3x3, so it means
    // it can cover an area of 3x3 board tiles. collision_map: [[bool; 9]; 9],
    /// Indicates that this will make collision dynamic. This is used for doors.
    pub is_dynamic: bool,
    pub stair_offset: i32,
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
    StairsUp,
    StairsDown,
    #[allow(clippy::upper_case_acronyms)]
    NPC,
    FakeGhost,
    FakeBreach,
    #[default]
    None,
}

impl AutoSerialize for Class {}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
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

    #[allow(dead_code)]
    fn to_text(&self) -> anyhow::Result<String> {
        // FIXME: This is not used at all.
        serde_json::to_string(self)
            .map(|x| x.replace('"', ""))
            .context("Auto serialize error")
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub enum TileState {
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

impl AutoSerialize for TileState {}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteCVOKey {
    pub class: Class,
    pub variant: String,
    pub orientation: Orientation,
}

#[derive(Debug, Clone)]
pub struct SpriteConfig {
    /// Main behavior class
    class: Class,
    /// Custom variant name - must be the same across all sprites that represent the
    /// same object
    variant: String,
    /// Orientation of the sprite - if it's facing one axis or another.
    orientation: Orientation,
    // Backup of the original data for the key
    cvo_key: SpriteCVOKey,
    /// Current state of the sprite - or the initial state.
    pub state: TileState,
    // Other interesting metadata:
    /// Tileset name
    pub tileset: String,
    /// UID of the tileset for this sprite
    pub tileuid: u32,
    /// All other tiled properties live here
    pub properties: BehaviorProperties,
}

impl PartialEq for SpriteConfig {
    fn eq(&self, other: &Self) -> bool {
        self.class == other.class
            && self.variant == other.variant
            && self.orientation == other.orientation
            && self.cvo_key == other.cvo_key
            && self.state == other.state
    }
}

impl Eq for SpriteConfig {}

impl std::hash::Hash for SpriteConfig {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.class.hash(state);
        self.variant.hash(state);
        self.orientation.hash(state);
        self.cvo_key.hash(state);
        self.state.hash(state);
    }
}

impl SpriteConfig {
    pub fn key_cvo(&self) -> SpriteCVOKey {
        self.cvo_key.clone()
    }

    pub fn key_tuid(&self) -> (String, u32) {
        (self.tileset.clone(), self.tileuid)
    }

    pub fn from_tiled_auto(tset_name: String, tileuid: u32, tiled_tile: &tiled::Tile) -> Self {
        // --- Load properties
        let properties = BehaviorProperties::from_tiled(tiled_tile);
        let sprite_config = Self::from_tiled(
            tiled_tile.user_type.as_deref(),
            tset_name,
            tileuid,
            properties,
        );
        sprite_config
    }

    pub fn from_tiled(
        class: Option<&str>,
        tileset: String,
        tileuid: u32,
        properties: BehaviorProperties,
    ) -> Self {
        Self::try_from_tiled(class, tileset.clone(), tileuid, properties)
            .with_context(|| {
                format!(
                    "SpriteConfig: error loading sprite from tiled: {}:{} [c:{:?}]",
                    tileset, tileuid, class
                )
            })
            .unwrap()
    }

    pub fn try_from_tiled(
        class: Option<&str>,
        tileset: String,
        tileuid: u32,
        properties: BehaviorProperties,
    ) -> anyhow::Result<Self> {
        let variant = properties.get_string_opt("sprite:variant");
        let orientation = properties.get_string_opt("sprite:orientation");
        let state = properties.get_string_opt("sprite:state");
        let orientation = orientation.as_deref();
        let state = state.as_deref();
        let tilesetuid_key = format!("{}:{}", tileset, tileuid);
        let class = Class::from_text(class).context("parsing Class")?;
        let variant = variant.unwrap_or(tilesetuid_key).to_owned();
        let orientation = Orientation::from_text(orientation).context("parsing Orientation")?;
        let state = TileState::from_text(state).context("parsing State")?;
        let cvo_key = SpriteCVOKey {
            class: class.clone(),
            variant: variant.clone(),
            orientation,
        };
        Ok(SpriteConfig {
            class,
            variant,
            orientation,
            state,
            tileset,
            tileuid,
            cvo_key,
            properties,
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
            Class::Door => entity
                .insert(component::Interactive::new(
                    "sounds/door-open.ogg",
                    "sounds/door-close.ogg",
                ))
                .insert(component::FloorItemCollidable)
                .insert(component::Door),
            Class::Switch => entity
                .insert(component::Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::RoomState::default()),
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
            Class::Decor => entity.insert(component::FloorItemCollidable),
            Class::Item => entity.insert(component::FloorItemCollidable),
            Class::Furniture => entity.insert(component::FloorItemCollidable),
            Class::PlayerSpawn => entity,
            Class::GhostSpawn => entity,
            Class::VanEntry => entity
                .insert(component::Interactive::new(
                    "sounds/door-open.ogg",
                    "sounds/door-close.ogg",
                ))
                .insert(component::FloorItemCollidable),
            Class::RoomDef => entity,
            Class::WallLamp => entity
                .insert(component::RoomState::default())
                .insert(component::Light),
            Class::FloorLamp => entity
                .insert(component::Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::FloorItemCollidable)
                .insert(component::Light),
            Class::TableLamp => entity
                .insert(component::Interactive::new(
                    "sounds/switch-on-1.ogg",
                    "sounds/switch-off-1.ogg",
                ))
                .insert(component::FloorItemCollidable)
                .insert(component::Light),
            Class::WallDecor => entity,
            Class::CeilingLight => entity
                .insert(component::RoomState::default())
                .insert(component::Light),
            Class::StreetLight => entity.insert(component::Light),
            Class::Appliance => entity.insert(component::FloorItemCollidable),
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
                ))
                .insert(component::FloorItemCollidable),
            Class::StairsDown => entity.insert(component::Stairs { z: -1 }),
            Class::StairsUp => entity.insert(component::Stairs { z: 1 }),
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
                p.movement.player_collision = self.state == TileState::Closed;
                p.movement.is_dynamic = true;
                p.light.opaque = self.state == TileState::Closed;
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
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (3.0).try_into().unwrap();
                p.light.heat_coef = -1;
            }
            Class::FloorLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (2.0).try_into().unwrap();
            }
            Class::TableLamp => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (1.0).try_into().unwrap();
            }
            Class::WallDecor => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::CeilingLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (3.5).try_into().unwrap();
                p.light.heat_coef = -2;
            }
            Class::StreetLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = true;
                p.light.emission_power = (5.0).try_into().unwrap();
                p.light.heat_coef = -6;
            }
            Class::Appliance => {
                p.display.global_z = (0.000070).try_into().unwrap();
            }
            Class::Van => {
                p.display.global_z = (0.000050).try_into().unwrap();
                p.display.auto_hide = true;
                p.display.light_recv_offset = (5, 0);
            }
            Class::Window => {
                p.display.global_z = (-0.00004).try_into().unwrap();
            }
            Class::StairsDown => {
                p.display.global_z = (0.000005).try_into().unwrap();
                p.movement.stair_offset = -1;
            }
            Class::StairsUp => {
                p.display.global_z = (0.000005).try_into().unwrap();
                p.movement.stair_offset = 1;
            }
            Class::None => {}
        }

        // --- Load object properties from Tiled data ---
        p.object.pickable = self.properties.get_bool("object:pickable");
        p.object.movable = self.properties.get_bool("object:movable");
        p.object.hidingspot = self.properties.get_bool("object:hidingspot");
        p.object.weight = NotNan::new(self.properties.get_float("object:weight")).unwrap();
        p.object.name = self.properties.get_string("object:name");
        if p.object.name.is_empty() {
            p.object.name.clone_from(&self.variant.clone());
        }
    }

    /// A class requires a set of states. Not only these are the only valid ones for
    /// the given class, also they need all to be included.
    fn _required_states(&self) -> Vec<TileState> {
        use TileState::*;

        match self.class {
            Class::Wall => vec![Full, Partial, Minimum],
            Class::Door => vec![Open, Closed],
            Class::Switch => vec![On, Off],
            Class::Breaker => vec![On, Off],
            _ => vec![None],
        }
    }
}

/// Stores a collection of properties loaded from Tiled map data.
///
/// This struct provides helper functions for accessing property values by key.
#[derive(Debug, Clone)]
pub struct BehaviorProperties {
    properties: HashMap<String, tiled::PropertyValue>,
}

impl BehaviorProperties {
    /// Creates a new `BehaviorProperties` instance from the properties of a
    /// `tiled::Tile`.
    pub fn from_tiled(tiled_tile: &tiled::Tile) -> Self {
        let mut properties = HashMap::new();
        for (key, value) in &tiled_tile.properties {
            properties.insert(key.clone(), value.clone());
        }
        Self { properties }
    }

    /// Returns the boolean value of a property with the given key.
    ///
    /// Returns `false` if the property is not found or is not a boolean value.
    pub fn get_bool(&self, key: &str) -> bool {
        self.properties
            .get(key)
            .map(|x| matches!(x, tiled::PropertyValue::BoolValue(true)))
            .unwrap_or(false)
    }

    /// Returns the floating-point value of a property with the given key.
    ///
    /// Returns `0.0` if the property is not found or is not a floating-point value.
    pub fn get_float(&self, key: &str) -> f32 {
        self.properties
            .get(key)
            .map(|x| match x {
                tiled::PropertyValue::FloatValue(n) => *n,
                _ => 0.0,
            })
            .unwrap_or(0.0)
    }

    /// Returns the string value of a property with the given key, or None if not
    /// present.
    pub fn get_string_opt(&self, key: &str) -> Option<String> {
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
                tiled::PropertyValue::ClassValue { property_type, .. } => property_type.to_string(),
            }
        };
        self.properties.get(key).map(parse)
    }

    /// Returns the string value of a property with the given key.
    ///
    /// Returns an empty string if the property is not found or is not a string value.
    pub fn get_string(&self, key: &str) -> String {
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
                tiled::PropertyValue::ClassValue { property_type, .. } => property_type.to_string(),
            }
        };
        self.properties
            .get(key)
            .map(parse)
            .unwrap_or("".to_string())
    }

    /// Returns the integer value of a property with the given key.
    ///
    /// Returns `0` if the property is not found or is not an integer value.
    pub fn get_int(&self, key: &str) -> i32 {
        self.properties
            .get(key)
            .map(|x| match x {
                tiled::PropertyValue::IntValue(n) => *n,
                _ => 0,
            })
            .unwrap_or(0)
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
        let c = TileState::On;
        let ct = c.to_text().unwrap();
        assert_eq!(ct, "On");
        let d = TileState::from_text(Some(&ct)).unwrap();
        assert_eq!(d, c);
    }
}
