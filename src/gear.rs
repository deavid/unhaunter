pub mod emfmeter;
pub mod flashlight;
pub mod playergear;
pub mod thermometer;

use bevy::prelude::*;

use crate::game::ControlKeys;

use self::emfmeter::EMFMeter;
use self::flashlight::Flashlight;
use self::playergear::{Inventory, InventoryStats, PlayerGear};
use self::thermometer::Thermometer;

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
    EMFMeter(EMFMeter),
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
            Gear::Thermometer(x) => x.get_status(),
            Gear::Flashlight(x) => x.get_status(),
            Gear::EMFMeter(x) => x.get_status(),
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
        let ni = || warn!("Trigger not implemented for {:?}", self);
        match self {
            Gear::Thermometer(x) => x.set_trigger(),
            Gear::Flashlight(x) => x.set_trigger(),
            Gear::ThermalImager => ni(),
            Gear::EMFMeter(x) => x.set_trigger(),
            Gear::Recorder => ni(),
            Gear::GeigerCounter => ni(),
            Gear::RedTorch => ni(),
            Gear::UVTorch => ni(),
            Gear::Photocam => ni(),
            Gear::IonMeter => ni(),
            Gear::SpiritBox => ni(),
            Gear::Compass => ni(),
            Gear::EStaticMeter => ni(),
            Gear::Videocam => ni(),
            Gear::MotionSensor => ni(),
            Gear::None => ni(),
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self {
            Gear::Thermometer(x) => x.get_sprite_idx(),
            Gear::Flashlight(x) => x.get_sprite_idx(),
            Gear::ThermalImager => GearSpriteID::ThermalImagerOn,
            Gear::EMFMeter(x) => x.get_sprite_idx(),
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
    fn update(&mut self) {
        match self {
            Gear::Thermometer(x) => x.update(),
            Gear::Flashlight(x) => x.update(),
            Gear::ThermalImager => {}
            Gear::EMFMeter(x) => x.update(),
            Gear::Recorder => {}
            Gear::GeigerCounter => {}
            Gear::RedTorch => {}
            Gear::UVTorch => {}
            Gear::Photocam => {}
            Gear::IonMeter => {}
            Gear::SpiritBox => {}
            Gear::Compass => {}
            Gear::EStaticMeter => {}
            Gear::Videocam => {}
            Gear::MotionSensor => {}
            Gear::None => {}
        }
    }
}

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
    fn update(&mut self) {}
    fn get_sprite_idx(&self) -> GearSpriteID;
    fn box_clone(&self) -> Box<dyn GearUsable>;
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
    mut playergear: ResMut<PlayerGear>,
    mut qi: Query<(&Inventory, &mut UiTextureAtlasImage)>,
    mut qs: Query<(&InventoryStats, &mut Text)>,
) {
    playergear.left_hand.update();
    playergear.right_hand.update();

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
