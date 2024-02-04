use bevy::prelude::*;
use bevy::{
    ecs::{
        component::Component,
        system::{Res, Resource},
    },
    input::{keyboard::KeyCode, Input},
};
use enum_iterator::Sequence;

use crate::game::ControlKeys;

#[allow(dead_code)]
#[derive(Debug, Default, PartialEq, Eq, Clone, Copy)]
pub enum GearSpriteID {
    ThermometerOff = 0,
    ThermometerOn,

    ThermalImagerOff,
    ThermalImagerOn,

    EMFMeterOff = 10,
    EMFMeter0,
    EMFMeter1,
    EMFMeter2,
    EMFMeter3,
    EMFMeter4,

    RecorderOff = 20,
    Recorder1,
    Recorder2,
    Recorder3,
    Recorder4,

    FlashlightOff = 30,
    Flashlight1,
    Flashlight2,
    Flashlight3,

    GeigerOff,
    GeigerOn,
    GeigerTick,

    RedTorchOff = 40,
    RedTorchOn,

    UVTorchOff,
    UVTorchOn,

    Photocam,
    PhotocamFlash1,
    PhotocamFlash2,

    IonMeterOff = 50,
    IonMeter0,
    IonMeter1,
    IonMeter2,

    SpiritBoxOff,
    SpiritBoxScan1,
    SpiritBoxScan2,
    SpiritBoxScan3,
    SpiritBoxAns1,
    SpiritBoxAns2,

    Compass = 80,

    EStaticMeter = 90,
    Videocam,
    MotionSensor,

    #[default]
    None,
}

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub enum Gear {
    Thermometer(Thermometer),
    EMFMeter,
    Recorder,
    Flashlight(Flashlight),
    GeigerCounter,
    UVTorch,
    IonMeter,
    SpiritBox,
    ThermalImager,
    RedTorch,
    Photocam,
    Compass,
    EStaticMeter,
    Videocam,
    MotionSensor,
    #[default]
    None,
}

impl GearUsable for Gear {
    fn get_display_name(&self) -> &'static str {
        todo!()
    }

    fn get_status(&self) -> String {
        match &self {
            Gear::Thermometer(t) => t.get_status(),
            Gear::Flashlight(f) => f.get_status(),
            Gear::EMFMeter => "EMF Meter: ON\nField Strength: 215mG (EMF 1)".to_string(),
            Gear::Recorder => "Recorder: ON\nVolume: 35dB".to_string(),
            Gear::GeigerCounter => "Geiger Counter: ON\nClicks: 30c/m".to_string(),
            Gear::UVTorch => "UV Torch: ON\nBattery: 75%".to_string(),
            Gear::IonMeter => "Ion Meter: ON\nReading: 35V/m".to_string(),
            Gear::Photocam => "Photo Camera: Ready\nPhotos remaining: 32".to_string(),
            Gear::SpiritBox => "Spirit Box: ON\nScanning...".to_string(),
            Gear::RedTorch => "Red Torch: ON\nBattery: 20%".to_string(),
            Gear::Compass => "Compass\nHeading: N".to_string(),
            Gear::ThermalImager => "Thermal Imager: ON\nBattery: 53%".to_string(),
            Gear::EStaticMeter => "Electrostatic Meter: ON\nReading: 120V/m".to_string(),
            Gear::Videocam => "Video Camera: ON\nBattery: 32%".to_string(),
            Gear::MotionSensor => "Motion Sensor\nBattery: 10%".to_string(),
            Gear::None => "".to_string(),
        }
    }

    fn set_trigger(&mut self) {
        match self {
            Gear::Thermometer(t) => t.set_trigger(),
            Gear::Flashlight(t) => t.set_trigger(),
            _ => {
                warn!("Trigger not implemented for {:?}", self)
            }
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self {
            Gear::Thermometer(t) => t.get_sprite_idx(),
            Gear::Flashlight(f) => f.get_sprite_idx(),
            Gear::ThermalImager => GearSpriteID::ThermalImagerOn,
            Gear::EMFMeter => GearSpriteID::EMFMeter0,
            Gear::Recorder => GearSpriteID::Recorder1,
            Gear::GeigerCounter => GearSpriteID::GeigerOn,
            Gear::RedTorch => GearSpriteID::RedTorchOn,
            Gear::UVTorch => GearSpriteID::UVTorchOn,
            Gear::Photocam => GearSpriteID::Photocam,
            Gear::IonMeter => GearSpriteID::IonMeter0,
            Gear::SpiritBox => GearSpriteID::SpiritBoxScan1,
            Gear::Compass => GearSpriteID::Compass,
            Gear::EStaticMeter => GearSpriteID::EStaticMeter,
            Gear::Videocam => GearSpriteID::Videocam,
            Gear::MotionSensor => GearSpriteID::MotionSensor,
            Gear::None => GearSpriteID::None,
        }
    }
    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

impl Gear {}

pub fn on_off(s: bool) -> &'static str {
    match s {
        true => "ON",
        false => "OFF",
    }
}

pub trait GearUsable: std::fmt::Debug + Sync + Send {
    fn get_display_name(&self) -> &'static str;
    fn get_status(&self) -> String;
    fn set_trigger(&mut self);
    fn get_sprite_idx(&self) -> GearSpriteID;
    fn box_clone(&self) -> Box<dyn GearUsable>;
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Thermometer {
    pub enabled: bool,
}

impl GearUsable for Thermometer {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.enabled {
            true => GearSpriteID::ThermometerOn,
            false => GearSpriteID::ThermometerOff,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Thermometer"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = on_off(self.enabled);
        let msg = if self.enabled {
            "Temperature: 22.2ºC".to_string()
        } else {
            "".to_string()
        };
        format!("{name}: {on_s}\n{msg}")
    }

    fn set_trigger(&mut self) {
        self.enabled = !self.enabled;
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Sequence)]
pub enum FlashlightStatus {
    #[default]
    Off,
    Low,
    Mid,
    High,
}

impl FlashlightStatus {
    pub fn string(&self) -> &'static str {
        match self {
            FlashlightStatus::Off => "OFF",
            FlashlightStatus::Low => "LOW",
            FlashlightStatus::Mid => "MID",
            FlashlightStatus::High => "HI",
        }
    }
}

#[derive(Component, Debug, Clone, Default, PartialEq, Eq)]
pub struct Flashlight {
    pub status: FlashlightStatus,
}

impl GearUsable for Flashlight {
    fn get_sprite_idx(&self) -> GearSpriteID {
        match self.status {
            FlashlightStatus::Off => GearSpriteID::FlashlightOff,
            FlashlightStatus::Low => GearSpriteID::Flashlight1,
            FlashlightStatus::Mid => GearSpriteID::Flashlight2,
            FlashlightStatus::High => GearSpriteID::Flashlight3,
        }
    }

    fn get_display_name(&self) -> &'static str {
        "Flashlight"
    }

    fn get_status(&self) -> String {
        let name = self.get_display_name();
        let on_s = self.status.string();
        format!("{name}: {on_s}")
    }

    fn set_trigger(&mut self) {
        self.status = self.status.next().unwrap_or_default();
    }

    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum Hand {
    Left,
    Right,
}

#[derive(Component, Debug, Clone)]
pub struct Inventory {
    hand: Hand,
}

impl Inventory {
    pub fn new_left() -> Self {
        Inventory { hand: Hand::Left }
    }
    pub fn new_right() -> Self {
        Inventory { hand: Hand::Right }
    }
}

#[derive(Component, Debug, Clone)]
pub struct InventoryStats;

#[derive(Clone, Debug, Resource, Default)]
pub struct PlayerGear {
    pub left_hand: Gear,
    pub right_hand: Gear,
    pub inventory: Vec<Gear>,
}

impl PlayerGear {
    pub fn new() -> Self {
        Self {
            left_hand: Gear::Flashlight(Flashlight::default()),
            right_hand: Gear::Thermometer(Thermometer::default()),
            inventory: vec![
                Gear::EMFMeter,
                Gear::GeigerCounter,
                Gear::UVTorch,
                Gear::Recorder,
                Gear::IonMeter,
                Gear::SpiritBox,
                Gear::ThermalImager,
                Gear::RedTorch,
                Gear::Photocam,
                Gear::Compass,
                Gear::EStaticMeter,
                Gear::Videocam,
                Gear::MotionSensor,
            ],
        }
    }
    pub fn cycle(&mut self) {
        let old_right = self.right_hand.clone();
        let last_idx = self.inventory.len() - 1;
        self.right_hand = self.inventory[0].clone();
        for i in 0..last_idx {
            self.inventory[i] = self.inventory[i + 1].clone()
        }
        self.inventory[last_idx] = old_right;
    }
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.right_hand, &mut self.left_hand);
    }
    pub fn get_hand(&self, hand: &Hand) -> Gear {
        match hand {
            Hand::Left => self.left_hand.clone(),
            Hand::Right => self.right_hand.clone(),
        }
    }
}

pub fn keyboard_gear(keyboard_input: Res<Input<KeyCode>>, mut playergear: ResMut<PlayerGear>) {
    const CONTROLS: ControlKeys = ControlKeys::WASD;
    if keyboard_input.just_pressed(CONTROLS.cycle) {
        playergear.cycle();
    }
    if keyboard_input.just_pressed(CONTROLS.swap) {
        playergear.swap();
    }
    if keyboard_input.just_pressed(CONTROLS.trigger) {
        playergear.right_hand.set_trigger();
    }
    if keyboard_input.just_pressed(CONTROLS.torch) {
        playergear.left_hand.set_trigger();
    }
}

pub fn update_gear_inventory(
    playergear: Res<PlayerGear>,
    mut qi: Query<(&Inventory, &mut UiTextureAtlasImage)>,
    mut qs: Query<(&InventoryStats, &mut Text)>,
) {
    for (inv, mut utai) in qi.iter_mut() {
        let gear = playergear.get_hand(&inv.hand);
        let idx = gear.get_sprite_idx() as usize;
        utai.index = idx;
    }
    for (_, mut txt) in qs.iter_mut() {
        let gear = &playergear.right_hand;
        txt.sections[0].value = gear.get_status();
    }
}
