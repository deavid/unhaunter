use crate::class::Class;
use crate::state::TileState;
use crate::traits::AutoSerialize;
use anyhow::Context;
use bevy::color::{LinearRgba, Srgba};
use bevy::ecs::component::Component;
use bevy::ecs::reflect::ReflectComponent;
use bevy::math::Vec3;
use bevy::reflect::Reflect;
use bevy::reflect::std_traits::ReflectDefault;
use bevy_platform::collections::HashMap;
use serde::{Deserialize, Serialize};
use unspatial_core::orientation::Orientation;

/// The `Behavior` component defines the behavior of an object in the game world.
#[derive(Component, Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Component)]
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
    /// WARNING: Do not use this function outside of the `unbehavior-core` crate.
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
        self.p.light.emission_power > 1.0
    }

    pub fn orientation(&self) -> Orientation {
        self.cfg.orientation
    }
}

/// Stores a collection of properties that define the behavior of an object.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
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
    /// Whether this object should be replicated over the network.
    pub is_replicated: bool,
}

/// Represents properties specific to objects in the game world.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct Object {
    pub pickable: bool,
    pub movable: bool,
    pub hidingspot: bool,
    pub weight: f32,
    pub name: String,
    pub throwable: bool,
    pub nudgeable: bool,
    pub haunt_movable: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub enum Util {
    RoomDef(String),
    PlayerSpawn,
    GhostSpawn,
    Van,
    #[default]
    None,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct Display {
    pub disable: bool,
    pub visual_priority: f32,
    pub auto_hide: bool,
    pub light_recv_offset: (i64, i64),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct Light {
    pub opaque: bool,
    pub see_through: bool,
    pub light_emission_enabled: bool,
    pub can_emit_light: bool,
    pub emission_power: f32,
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
            emission_power: 0.0,
            heat_coef: 0,
            flickering: false,
            color: LinearRgba::WHITE,
        }
    }
}

impl Light {
    pub fn emmisivity_lumens(&self) -> f32 {
        use fastapprox::faster;
        let emission_power = if self.emission_power.is_finite() {
            self.emission_power
        } else {
            0.0
        };
        if self.flickering {
            if self.light_emission_enabled {
                faster::exp(emission_power) * 0.4
            } else {
                faster::exp(emission_power) * 0.001
            }
        } else {
            match self.light_emission_enabled {
                true => faster::exp(emission_power),
                false => 0.0,
            }
        }
    }

    pub fn color(&self) -> (f32, f32, f32) {
        (self.color.red, self.color.green, self.color.blue)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct Movement {
    pub walkable: bool,
    pub player_collision: bool,
    pub ghost_collision: bool,
    pub is_dynamic: bool,
    pub stair_offset: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, Reflect)]
pub struct SpriteCVOKey {
    pub(crate) class: Class,
    pub variant: String,
    pub orientation: Orientation,
}

impl SpriteCVOKey {
    pub fn to_key_string(&self) -> String {
        format!("{:?}/{}/{:?}", self.class, self.variant, self.orientation)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, Reflect)]
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

    pub fn from_class_str(
        class: Option<&str>,
        tileset: String,
        tileuid: u32,
        properties: BehaviorProperties,
    ) -> Self {
        Self::try_from_class_str(class, tileset.clone(), tileuid, properties)
            .with_context(|| {
                format!(
                    "SpriteConfig: error loading sprite from tiled: {}:{} [c:{:?}]",
                    tileset, tileuid, class
                )
            })
            .unwrap()
    }

    pub fn try_from_class_str(
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
                p.display.visual_priority = -0.00035;
                p.is_floor = true;
            }
            Class::Wall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.opaque = true;
                p.display.visual_priority = -0.00005;
                p.is_wall = true;
            }
            Class::LowWall => {
                p.movement.player_collision = true;
                p.movement.ghost_collision = true;
                p.light.see_through = true;
                p.display.visual_priority = -0.00005;
                p.is_low_wall = true;
            }
            Class::Door => {
                p.display.visual_priority = 0.000015;
                p.movement.player_collision = self.state == TileState::Closed;
                p.movement.is_dynamic = true;
                p.light.opaque = self.state == TileState::Closed;
                p.is_door = true;
                p.is_replicated = true;
            }
            Class::Switch | Class::RoomSwitch | Class::Breaker => {
                p.display.visual_priority = 0.000002;
                p.is_breaker = self.class == Class::Breaker;
                p.is_switch = self.class == Class::Switch || self.class == Class::RoomSwitch;
                p.is_room_switch = self.class == Class::RoomSwitch;
                p.is_replicated = true;
            }
            Class::Doorway => {
                p.display.visual_priority = -0.00005;
            }
            Class::Decor | Class::Item => {
                p.display.visual_priority = 0.000065;
                p.is_stationary_collidable = true;
            }
            Class::Furniture | Class::NPC => {
                p.display.visual_priority = 0.000050;
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
                p.display.visual_priority = -0.00004;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = 3.0;
                p.light.heat_coef = -1;
                p.light.color = LinearRgba::new(1.0, 0.9, 0.75, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_wall_light = true;
                p.is_replicated = true;
            }
            Class::FloorLamp | Class::TableLamp => {
                p.display.visual_priority = 0.000050;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = match self.class {
                    Class::FloorLamp => 2.0,
                    _ => 1.0,
                };
                p.light.color = LinearRgba::new(1.0, 0.8, 0.6, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_floor_light = self.class == Class::FloorLamp;
                p.is_table_light = self.class == Class::TableLamp;
                p.is_replicated = true;
            }
            Class::WallDecor => {
                p.display.visual_priority = -0.00004;
            }
            Class::CeilingLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = self.state == TileState::On;
                p.light.emission_power = 3.5;
                p.light.heat_coef = -2;
                p.light.color = LinearRgba::new(1.0, 0.96, 0.905, 1.0);
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_light_source = true;
                p.is_ceiling_light = true;
                p.is_replicated = true;
            }
            Class::StreetLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = true;
                p.light.emission_power = 5.0;
                p.light.heat_coef = -6;
                p.light.color = LinearRgba::new(0.95, 0.98, 1.0, 1.0);
                p.is_electrical = true;
                p.is_house_powered = false;
                p.is_street_powered = true;
                p.is_light_source = true;
                p.is_street_light = true;
                p.is_replicated = true;
            }
            Class::CandleLight => {
                p.display.disable = true;
                p.light.can_emit_light = true;
                p.light.light_emission_enabled = true;
                p.light.emission_power = -0.5;
                p.light.heat_coef = 6;
                p.light.color = LinearRgba::new(1.0, 0.75, 0.1, 1.0);
                p.is_light_source = true;
                p.is_candle_light = true;
                p.is_replicated = true;
            }
            Class::Appliance => {
                p.display.visual_priority = 0.000070;
                p.is_electrical = true;
                p.is_house_powered = true;
                p.is_appliance = true;
            }
            Class::Van => {
                p.display.visual_priority = 0.000050;
                p.display.auto_hide = true;
                p.display.light_recv_offset = (5, 0);
            }
            Class::Window => {
                p.display.visual_priority = -0.00004;
            }
            Class::StairsDown | Class::StairsUp => {
                p.display.visual_priority = 0.000005;
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
        p.object.weight = self.properties.get_float("object:weight");
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

        if p.object.movable {
            p.is_replicated = true;
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Reflect)]
pub enum UnhaunterPropertyValue {
    BoolValue(bool),
    FloatValue(f32),
    IntValue(i32),
    StringValue(String),
    ColorValue([u8; 4]),
    FileValue(String),
    ObjectValue(u32),
    ClassValue { property_type: String },
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize, Reflect)]
#[reflect(Default)]
pub struct BehaviorProperties {
    properties: HashMap<String, UnhaunterPropertyValue>,
}

impl BehaviorProperties {
    pub fn from_map(properties: HashMap<String, UnhaunterPropertyValue>) -> Self {
        Self { properties }
    }

    pub fn get_bool(&self, key: &str) -> bool {
        self.properties
            .get(key)
            .map(|x| matches!(x, UnhaunterPropertyValue::BoolValue(true)))
            .unwrap_or(false)
    }

    pub fn get_float(&self, key: &str) -> f32 {
        self.properties
            .get(key)
            .map(|x| match x {
                UnhaunterPropertyValue::FloatValue(n) if n.is_finite() => *n,
                _ => 0.0,
            })
            .unwrap_or(0.0)
    }

    pub fn get_color(&self, key: &str) -> Option<LinearRgba> {
        self.properties.get(key).and_then(|x| match x {
            UnhaunterPropertyValue::ColorValue([red, green, blue, alpha]) => {
                Some(Srgba::rgba_u8(*red, *green, *blue, *alpha).into())
            }
            _ => None,
        })
    }

    pub fn get_string_opt(&self, key: &str) -> Option<String> {
        let parse = |x: &UnhaunterPropertyValue| -> String {
            match x {
                UnhaunterPropertyValue::BoolValue(x) => x.to_string(),
                UnhaunterPropertyValue::FloatValue(x) => x.to_string(),
                UnhaunterPropertyValue::IntValue(x) => x.to_string(),
                UnhaunterPropertyValue::ColorValue([red, green, blue, alpha]) => {
                    format!("{red},{green},{blue},{alpha}")
                }
                UnhaunterPropertyValue::StringValue(x) => x.to_string(),
                UnhaunterPropertyValue::FileValue(x) => x.to_string(),
                UnhaunterPropertyValue::ObjectValue(x) => x.to_string(),
                UnhaunterPropertyValue::ClassValue { property_type } => property_type.to_string(),
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
                UnhaunterPropertyValue::IntValue(n) => *n,
                _ => 0,
            })
            .unwrap_or(0)
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
    pub fn new(dialog: String) -> Self {
        Self {
            dialog,
            seen: false,
            trigger: 0.0,
        }
    }
}
