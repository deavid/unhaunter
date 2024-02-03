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

#[derive(Debug, Default, PartialEq, Eq, Clone, Copy, Sequence)]
pub enum Gear {
    Thermometer,
    ThermalImager,
    EMFMeter,
    Recorder,
    Flashlight,
    GeigerCounter,
    RedTorch,
    UVTorch,
    Photocam,
    IonMeter,
    SpiritBox,
    Compass,
    EStaticMeter,
    Videocam,
    MotionSensor,
    #[default]
    None,
}

impl Gear {
    pub fn get_sprite_idx(&self) -> usize {
        (match &self {
            Gear::Thermometer => GearSpriteID::ThermometerOn,
            Gear::ThermalImager => GearSpriteID::ThermalImagerOn,
            Gear::EMFMeter => GearSpriteID::EMFMeter0,
            Gear::Recorder => GearSpriteID::Recorder1,
            Gear::Flashlight => GearSpriteID::Flashlight1,
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
        }) as usize
    }
    pub fn status(&self) -> String {
        match &self {
            Gear::Thermometer => "Thermometer: ON\nTemperature: 15.5ºC".to_string(),
            Gear::ThermalImager => "Thermal Imager: ON\nBattery: 53%".to_string(),
            Gear::EMFMeter => "EMF Meter: ON\nField Strength: 215mG (EMF 1)".to_string(),
            Gear::Recorder => "Recorder: ON\nVolume: 35dB".to_string(),
            Gear::Flashlight => "Flashlight: ON\nPower Setting: 1".to_string(),
            Gear::GeigerCounter => "Geiger Counter: ON\nClicks: 30c/m".to_string(),
            Gear::RedTorch => "Red Torch: ON\nBattery: 20%".to_string(),
            Gear::UVTorch => "UV Torch: ON\nBattery: 75%".to_string(),
            Gear::Photocam => "Photo Camera: Ready\nPhotos remaining: 32".to_string(),
            Gear::IonMeter => "Ion Meter: ON\nReading: 35V/m".to_string(),
            Gear::SpiritBox => "Spirit Box: ON\nScanning...".to_string(),
            Gear::Compass => "Compass\nHeading: N".to_string(),
            Gear::EStaticMeter => "Electrostatic Meter: ON\nReading: 120V/m".to_string(),
            Gear::Videocam => "Video Camera: ON\nBattery: 32%".to_string(),
            Gear::MotionSensor => "Motion Sensor\nBattery: 10%".to_string(),
            Gear::None => "".to_string(),
        }
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
    pub inventory: [Gear; 6],
}

impl PlayerGear {
    pub fn new() -> Self {
        Self {
            left_hand: Gear::Flashlight,
            right_hand: Gear::Thermometer,
            inventory: [
                Gear::EMFMeter,
                Gear::GeigerCounter,
                Gear::UVTorch,
                Gear::Recorder,
                Gear::IonMeter,
                Gear::SpiritBox,
            ],
        }
    }
    pub fn cycle(&mut self) {
        let old_right = self.right_hand;
        let last_idx = self.inventory.len() - 1;
        self.right_hand = self.inventory[0];
        for i in 0..last_idx {
            self.inventory[i] = self.inventory[i + 1]
        }
        self.inventory[last_idx] = old_right;
    }
    pub fn swap(&mut self) {
        std::mem::swap(&mut self.right_hand, &mut self.left_hand);
    }
    pub fn get_hand(&self, hand: &Hand) -> Gear {
        match hand {
            Hand::Left => self.left_hand,
            Hand::Right => self.right_hand,
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
}

pub fn update_gear_inventory(
    playergear: Res<PlayerGear>,
    mut qi: Query<(&Inventory, &mut UiTextureAtlasImage)>,
    mut qs: Query<(&InventoryStats, &mut Text)>,
) {
    for (inv, mut utai) in qi.iter_mut() {
        let gear = playergear.get_hand(&inv.hand);
        let idx = gear.get_sprite_idx();
        utai.index = idx;
    }
    for (_, mut txt) in qs.iter_mut() {
        let gear = playergear.right_hand;
        txt.sections[0].value = gear.status();
    }
}
