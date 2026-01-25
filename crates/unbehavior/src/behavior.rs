use crate::class::Class;
use crate::state::TileState;
use crate::traits::AutoSerialize;
use anyhow::{Context, Ok};
use bevy::color::{LinearRgba, Srgba};
use bevy::ecs::component::Component;
use bevy::log::warn;
use bevy::math::Vec3;
use bevy_platform::collections::HashMap;
use ordered_float::NotNan;
use serde::{Deserialize, Serialize};
use unspatial_core::orientation::Orientation;

/// The `Behavior` component defines the behavior of an object in the game world.
#[derive(Component, Debug, Clone, PartialEq)]
pub struct Behavior {
    /// This `cfg` property is PRIVATE on purpose!
    cfg: SpriteConfig,
    /// The `p` field stores a collection of properties that define the object's
    /// behavior.
    pub p: Properties,
}

impl Behavior {
    pub fn cfg(&self) -> &SpriteConfig {
        &self.cfg
    }

    /// Creates a new `Behavior` component from a `SpriteConfig`.
    pub fn from_config(cfg: SpriteConfig) -> Self {
        let mut p = Properties::default();
        cfg.set_properties(&mut p);
        Self { cfg, p }
    }

    /// Flips horizontally a sprite.
    pub fn flip(&mut self, f: bool) {
        if f != self.p.flip {
            self.cfg.orientation.flip();
            self.p.flip = f;
        }
    }

    /// Returns the state (On/Off, Open/Closed) as a copy.
    pub fn state(&self) -> TileState {
        self.cfg.state.clone()
    }

    /// Returns the class of the behavior.
    ///
    /// WARNING: Do not use this function outside of the `unbehavior` crate.
    /// Instead, prefer adding a new property to `Behavior::Properties` to expose
    /// the specific functionality or characteristic you need.
    fn _class(&self) -> Class {
        self.cfg.class.clone()
    }

    pub fn key_cvo(&self) -> SpriteCVOKey {
        self.cfg.key_cvo()
    }

    pub fn key_tuid(&self) -> (String, u32) {
        self.cfg.key_tuid()
    }

    pub fn config(&self) -> &SpriteConfig {
        &self.cfg
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
        use fastapprox::faster;
        let heat_coeff = faster::exp(self.p.light.heat_coef as f32);
        self.p.light.emmisivity_lumens() / 10000.0 * heat_coeff
    }

    pub fn is_van_entry(&self) -> bool {
        self.p.is_van_entry
    }

    pub fn is_npc(&self) -> bool {
        self.p.is_npc
    }

    pub fn can_emit_light(&self) -> bool {
        self.p.light.emission_power.into_inner() > 1.0
    }

    pub fn orientation(&self) -> Orientation {
        self.cfg.orientation
    }
}

/// Stores a collection of properties that define the behavior of an object.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Properties {
    pub movement: Movement,
    pub light: Light,
    pub util: Util,
    pub display: Display,
    pub flip: bool,
    pub object: Object,
    /// Whether the object is electrical and depends on the map's power grid.
    pub is_electrical: bool,
    /// Whether the object depends on house power (main breaker).
    pub is_house_powered: bool,
    /// Whether the object depends on street power (external grid).
    pub is_street_powered: bool,
    /// Whether the object is a door.
    pub is_door: bool,
    /// Whether the object is a main power breaker.
    pub is_breaker: bool,
    /// Whether the object is a light switch.
    pub is_switch: bool,
    /// Whether the object is a primary light source (lamp, candle, street light).
    pub is_light_source: bool,
    /// Whether the object is the van entry point.
    pub is_van_entry: bool,
    /// Whether the object is an NPC.
    pub is_npc: bool,
    /// Whether the object is a floor.
    pub is_floor: bool,
    /// Whether the object is a wall.
    pub is_wall: bool,
    /// Whether the object is a low wall.
    pub is_low_wall: bool,
    /// Whether the object is a room light switch.
    pub is_room_switch: bool,
    /// Whether the object is a wall-mounted light.
    pub is_wall_light: bool,
    /// Whether the object is a floor-standing light.
    pub is_floor_light: bool,
    /// Whether the object is a table-top light.
    pub is_table_light: bool,
    /// Whether the object is a ceiling-mounted light.
    pub is_ceiling_light: bool,
    /// Whether the object is a street light.
    pub is_street_light: bool,
    /// Whether the object is a candle.
    pub is_candle_light: bool,
    /// Whether the object is an appliance.
    pub is_appliance: bool,
    /// Whether the object is a stationary collidable object (furniture, decor, item).
    pub is_stationary_collidable: bool,
}

/// Represents properties specific to objects in the game world.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Hash)]
pub struct Object {
    pub pickable: bool,
    pub movable: bool,
    pub hidingspot: bool,
    pub weight: NotNan<f32>,
    pub name: String,
    pub throwable: bool,
    pub nudgeable: bool,
    pub haunt_movable: bool,
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
    pub visual_priority: NotNan<f32>,
    pub auto_hide: bool,
    pub light_recv_offset: (i64, i64),
}

#[derive(Debug, Clone, PartialEq)]
pub struct Light {
    pub opaque: bool,
    pub see_through: bool,
    pub light_emission_enabled: bool,
    pub can_emit_light: bool,
    pub emission_power: NotNan<f32>,
    pub heat_coef: i32,
    pub flickering: bool,
    pub color: LinearRgba,
}

impl Default for Light {
    fn default() -> Self {
        Self {
            opaque: false,
            see_through: false,
            light_emission_enabled: false,
            can_emit_light: false,
            emission_power: NotNan::new(0.0).unwrap(),
            heat_coef: 0,
            flickering: false,
            color: LinearRgba::WHITE,
        }
    }
}

impl Light {
    pub fn emmisivity_lumens(&self) -> f32 {
        use fastapprox::faster;
        if self.flickering {
            if self.light_emission_enabled {
                faster::exp(self.emission_power.into_inner()) * 0.4
            } else {
                faster::exp(self.emission_power.into_inner()) * 0.001
            }
        } else {
            match self.light_emission_enabled {
                true => faster::exp(self.emission_power.into_inner()),
                false => 0.0,
            }
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        (self.color.red, self.color.green, self.color.blue)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Movement {
    pub walkable: bool,
    pub player_collision: bool,
    pub ghost_collision: bool,
    pub is_dynamic: bool,
    pub stair_offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct SpriteCVOKey {
    pub(crate) class: Class,
    pub variant: String,
    pub orientation: Orientation,
}

#[derive(Debug, Clone)]
pub struct SpriteConfig {
    pub(crate) class: Class,
    pub variant: String,
    pub orientation: Orientation,
    pub cvo_key: SpriteCVOKey,
    pub state: TileState,
    pub tileset: String,
    pub tileuid: u32,
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
        let properties = BehaviorProperties::from_tiled(tiled_tile);
        Self::from_tiled(
            tiled_tile.user_type.as_deref(),
            tset_name,
            tileuid,
            properties,
        )
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

    pub fn set_properties(&self, p: &mut Properties) {
        match self.class {
            Class::Floor => {
                p.movement.walkable = true;
                p.display.visual_priority = (-0.00035).try_into().unwrap();
                p.is_floor = true;
            }
            Class::Wall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.opaque = true;
                p.display.visual_priority = (-0.00005).try_into().unwrap();
                p.is_wall = true;
            }
            Class::LowWall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.see_through = true;
                p.display.visual_priority = (-0.00005).try_into().unwrap();
                p.is_low_wall = true;
            }
            Class::Door => {
                p.display.visual_priority = (0.000015).try_into().unwrap();
                p.movement.player_collision = self.state == TileState::Closed;
                p.movement.is_dynamic = true;
                p.light.opaque = self.state == TileState::Closed;
                p.is_door = true;
            }
            Class::Switch | Class::RoomSwitch | Class::Breaker => {
                p.display.visual_priority = (0.000040).try_into().unwrap();
                p.is_breaker = self.class == Class::Breaker;
                p.is_switch = self.class == Class::Switch || self.class == Class::RoomSwitch;
                p.is_room_switch = self.class == Class::RoomSwitch;
            }
            Class::Doorway => {
                p.display.visual_priority = (-0.00005).try_into().unwrap();
            }
            Class::Decor | Class::Item => {
                p.display.visual_priority = (0.000065).try_into().unwrap();
                p.is_stationary_collidable = true;
            }
            Class::Furniture | Class::NPC => {
                p.display.visual_priority = (0.000050).try_into().unwrap();
                p.is_npc = self.class == Class::NPC;
                p.is_stationary_collidable = self.class == Class::Furniture;
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
            Class::FakeGhost | Class::FakeBreach => {
                p.display.disable = true;
            }
            Class::VanEntry => {
                p.util = Util::Van;
                p.is_van_entry = true;
            }
            Class::RoomDef => {
                p.display.disable = true;
                p.util = Util::RoomDef(self.variant.clone());
            }
            Class::WallLamp => {
                p.display.visual_priority = (-0.00004).try_into().unwrap();
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (3.0).try_into().unwrap();
                p.light.heat_coef = -1;
                p.light.color = LinearRgba::new(1.0, 0.9, 0.75, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_wall_light = true;
            }
            Class::FloorLamp | Class::TableLamp => {
                p.display.visual_priority = (0.000050).try_into().unwrap();
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = match self.class {
                    Class::FloorLamp => (2.0).try_into().unwrap(),
                    _ => (1.0).try_into().unwrap(),
                };
                p.light.color = LinearRgba::new(1.0, 0.8, 0.6, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_floor_light = self.class == Class::FloorLamp;
                p.is_table_light = self.class == Class::TableLamp;
            }
            Class::WallDecor => {
                p.display.visual_priority = (-0.00004).try_into().unwrap();
            }
            Class::CeilingLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = (3.5).try_into().unwrap();
                p.light.heat_coef = -2;
                p.light.color = LinearRgba::new(1.0, 1.0, 1.0, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_ceiling_light = true;
            }
            Class::StreetLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = true;
                p.light.emission_power = (5.0).try_into().unwrap();
                p.light.heat_coef = -6;
                p.light.color = LinearRgba::new(0.95, 0.98, 1.0, 1.0);
                p.is_electrical = true;
                p.is_house_powered = false;
                p.is_street_powered = true;
                p.is_light_source = true;
                p.is_street_light = true;
            }
            Class::CandleLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = true;
                p.light.emission_power = (-0.5).try_into().unwrap();
                p.light.heat_coef = 6;
                p.light.color = LinearRgba::new(1.0, 0.75, 0.1, 1.0);
                p.is_light_source = true;
                p.is_candle_light = true;
            }
            Class::Appliance => {
                p.display.visual_priority = (0.000070).try_into().unwrap();
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_appliance = true;
            }
            Class::Van => {
                p.display.visual_priority = (0.000050).try_into().unwrap();
                p.display.auto_hide = true;
                p.display.light_recv_offset = (5, 0);
            }
            Class::Window => {
                p.display.visual_priority = (-0.00004).try_into().unwrap();
            }
            Class::StairsDown | Class::StairsUp => {
                p.display.visual_priority = (0.000005).try_into().unwrap();
                p.movement.stair_offset = match self.class {
                    Class::StairsDown => -1,
                    _ => 1,
                };
            }
            Class::None => {}
        }

        p.object.pickable = self.properties.get_bool("object:pickable");
        p.object.movable = self.properties.get_bool("object:movable");
        p.object.hidingspot = self.properties.get_bool("object:hidingspot");
        p.object.weight = NotNan::new(self.properties.get_float("object:weight")).unwrap();
        p.object.name = self.properties.get_string("object:name");
        if p.object.name.is_empty() {
            p.object.name.clone_from(&self.variant);
        }

        p.object.throwable = self.properties.get_bool("object:throwable");
        p.object.nudgeable = self.properties.get_bool("object:nudgeable");
        p.object.haunt_movable = self.properties.get_bool("object:haunt_movable");
        if p.object.movable {
            p.object.throwable = true;
            p.object.nudgeable = true;
            p.object.haunt_movable = true;
        }

        if self.properties.get_bool("light:can_emit_light") {
            p.light.can_emit_light = true;
        }

        if let Some(color) = self.properties.get_color("light:color") {
            p.light.color = color;
        }
    }
}

#[derive(Debug, Clone)]
pub struct BehaviorProperties {
    properties: HashMap<String, tiled::PropertyValue>,
}

impl BehaviorProperties {
    pub fn from_tiled(tiled_tile: &tiled::Tile) -> Self {
        let mut properties = HashMap::new();
        for (key, value) in &tiled_tile.properties {
            properties.insert(key.clone(), value.clone());
        }
        Self { properties }
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.properties
            .get(key)
            .map(|x| matches!(x, tiled::PropertyValue::BoolValue(true)))
            .unwrap_or(false)
    }

    pub fn get_float(&self, key: &str) -> f32 {
        self.properties
            .get(key)
            .map(|x| match x {
                tiled::PropertyValue::FloatValue(n) => *n,
                _ => 0.0,
            })
            .unwrap_or(0.0)
    }

    pub fn get_color(&self, key: &str) -> Option<LinearRgba> {
        self.properties.get(key).and_then(|x| match x {
            tiled::PropertyValue::ColorValue(c) => {
                Some(Srgba::rgba_u8(c.red, c.green, c.blue, c.alpha).into())
            }
            _ => None,
        })
    }

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

    pub fn get_string(&self, key: &str) -> String {
        self.get_string_opt(key).unwrap_or_default()
    }

    pub fn _get_int(&self, key: &str) -> i32 {
        self.properties
            .get(key)
            .map(|x| match x {
                tiled::PropertyValue::IntValue(n) => *n,
                _ => 0,
            })
            .unwrap_or(0)
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq)]
pub struct Interactive {
    pub on_activate_sound_file: String,
    pub on_deactivate_sound_file: String,
    pub hovered: bool,
}

impl Interactive {
    pub fn new(activate: &str, deactivate: &str) -> Self {
        let on_activate_sound_file = activate.to_string();
        let on_deactivate_sound_file = deactivate.to_string();
        Self {
            on_activate_sound_file,
            on_deactivate_sound_file,
            hovered: false,
        }
    }

    pub fn sound_for_moving_into_state(&self, behavior: &Behavior) -> String {
        match behavior.cfg.state {
            TileState::On => self.on_activate_sound_file.clone(),
            TileState::Off => self.on_deactivate_sound_file.clone(),
            TileState::Open => self.on_activate_sound_file.clone(),
            TileState::Closed => self.on_deactivate_sound_file.clone(),
            TileState::Full => self.on_activate_sound_file.clone(),
            TileState::Partial => self.on_activate_sound_file.clone(),
            TileState::Minimum => self.on_activate_sound_file.clone(),
            TileState::None => self.on_deactivate_sound_file.clone(),
        }
    }

    pub fn control_point_delta(&self, behavior: &Behavior) -> Vec3 {
        if behavior.p.is_door {
            match behavior.cfg.orientation {
                Orientation::XAxis => Vec3::new(0.0, -0.25, 0.0),
                Orientation::YAxis => Vec3::new(0.25, 0.0, 0.0),
                _ => Vec3::ZERO,
            }
        } else {
            Vec3::ZERO
        }
    }
}

#[derive(Component, Debug, Clone, PartialEq)]
pub struct NpcHelpDialog {
    pub dialog: String,
    pub seen: bool,
    pub trigger: f32,
}

impl NpcHelpDialog {
    pub fn new(
        classname: &str,
        variant: &str,
        user_properties: &HashMap<String, tiled::PropertyValue>,
    ) -> Self {
        let key = format!("{classname}:{variant}:dialog");
        let dialog = match user_properties.get(&key) {
            Some(p) => match p {
                tiled::PropertyValue::StringValue(v) => v.to_string(),
                _ => {
                    warn!(
                        "NPCHelpDialog was expecting a user property named {key:?} in the layer but it had an unsupported type - it must be text"
                    );
                    "".to_string()
                }
            },
            None => {
                warn!(
                    "NPCHelpDialog was expecting a user property named {key:?} in the layer but was not present"
                );
                "".to_string()
            }
        };
        Self {
            dialog,
            seen: false,
            trigger: 0.0,
        }
    }
}
