pub mod compass;
pub mod emfmeter;
pub mod estaticmeter;
pub mod flashlight;
pub mod geigercounter;
pub mod ionmeter;
pub mod motionsensor;
pub mod photocam;
pub mod playergear;
pub mod recorder;
pub mod redtorch;
pub mod spiritbox;
pub mod thermalimager;
pub mod thermometer;
pub mod uvtorch;
pub mod videocam;

use self::compass::Compass;
use self::emfmeter::EMFMeter;
use self::estaticmeter::EStaticMeter;
use self::flashlight::Flashlight;
use self::geigercounter::GeigerCounter;
use self::ionmeter::IonMeter;
use self::motionsensor::MotionSensor;
use self::photocam::Photocam;
use self::recorder::Recorder;
use self::redtorch::RedTorch;
use self::spiritbox::SpiritBox;
use self::thermalimager::ThermalImager;
use self::thermometer::Thermometer;
use self::uvtorch::UVTorch;
use self::videocam::Videocam;

use self::playergear::{Inventory, InventoryStats, PlayerGear};
use crate::game::ControlKeys;
use bevy::prelude::*;

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
    Recorder(Recorder),
    Flashlight(Flashlight),
    GeigerCounter(GeigerCounter),
    UVTorch(UVTorch),
    IonMeter(IonMeter),
    SpiritBox(SpiritBox),
    ThermalImager(ThermalImager),
    RedTorch(RedTorch),
    Photocam(Photocam),
    Compass(Compass),
    EStaticMeter(EStaticMeter),
    Videocam(Videocam),
    MotionSensor(MotionSensor),
    #[default]
    None,
}

impl GearUsable for Gear {
    fn get_display_name(&self) -> &'static str {
        match &self {
            Gear::Thermometer(x) => x.get_display_name(),
            Gear::Flashlight(x) => x.get_display_name(),
            Gear::EMFMeter(x) => x.get_display_name(),
            Gear::Recorder(x) => x.get_display_name(),
            Gear::GeigerCounter(x) => x.get_display_name(),
            Gear::UVTorch(x) => x.get_display_name(),
            Gear::IonMeter(x) => x.get_display_name(),
            Gear::Photocam(x) => x.get_display_name(),
            Gear::SpiritBox(x) => x.get_display_name(),
            Gear::RedTorch(x) => x.get_display_name(),
            Gear::Compass(x) => x.get_display_name(),
            Gear::ThermalImager(x) => x.get_display_name(),
            Gear::EStaticMeter(x) => x.get_display_name(),
            Gear::Videocam(x) => x.get_display_name(),
            Gear::MotionSensor(x) => x.get_display_name(),
            Gear::None => "",
        }
    }

    fn get_status(&self) -> String {
        match &self {
            Gear::Thermometer(x) => x.get_status(),
            Gear::Flashlight(x) => x.get_status(),
            Gear::EMFMeter(x) => x.get_status(),
            Gear::Recorder(x) => x.get_status(),
            Gear::GeigerCounter(x) => x.get_status(),
            Gear::UVTorch(x) => x.get_status(),
            Gear::IonMeter(x) => x.get_status(),
            Gear::Photocam(x) => x.get_status(),
            Gear::SpiritBox(x) => x.get_status(),
            Gear::RedTorch(x) => x.get_status(),
            Gear::Compass(x) => x.get_status(),
            Gear::ThermalImager(x) => x.get_status(),
            Gear::EStaticMeter(x) => x.get_status(),
            Gear::Videocam(x) => x.get_status(),
            Gear::MotionSensor(x) => x.get_status(),
            Gear::None => "".to_string(),
        }
    }

    fn set_trigger(&mut self) {
        let ni = || warn!("Trigger not implemented for {:?}", self);
        match self {
            Gear::Thermometer(x) => x.set_trigger(),
            Gear::Flashlight(x) => x.set_trigger(),
            Gear::ThermalImager(x) => x.set_trigger(),
            Gear::EMFMeter(x) => x.set_trigger(),
            Gear::Recorder(x) => x.set_trigger(),
            Gear::GeigerCounter(x) => x.set_trigger(),
            Gear::RedTorch(x) => x.set_trigger(),
            Gear::UVTorch(x) => x.set_trigger(),
            Gear::Photocam(x) => x.set_trigger(),
            Gear::IonMeter(x) => x.set_trigger(),
            Gear::SpiritBox(x) => x.set_trigger(),
            Gear::Compass(x) => x.set_trigger(),
            Gear::EStaticMeter(x) => x.set_trigger(),
            Gear::Videocam(x) => x.set_trigger(),
            Gear::MotionSensor(x) => x.set_trigger(),
            Gear::None => ni(),
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self {
            Gear::Thermometer(x) => x.get_sprite_idx(),
            Gear::Flashlight(x) => x.get_sprite_idx(),
            Gear::ThermalImager(x) => x.get_sprite_idx(),
            Gear::EMFMeter(x) => x.get_sprite_idx(),
            Gear::Recorder(x) => x.get_sprite_idx(),
            Gear::GeigerCounter(x) => x.get_sprite_idx(),
            Gear::RedTorch(x) => x.get_sprite_idx(),
            Gear::UVTorch(x) => x.get_sprite_idx(),
            Gear::Photocam(x) => x.get_sprite_idx(),
            Gear::IonMeter(x) => x.get_sprite_idx(),
            Gear::SpiritBox(x) => x.get_sprite_idx(),
            Gear::Compass(x) => x.get_sprite_idx(),
            Gear::EStaticMeter(x) => x.get_sprite_idx(),
            Gear::Videocam(x) => x.get_sprite_idx(),
            Gear::MotionSensor(x) => x.get_sprite_idx(),
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
            Gear::ThermalImager(x) => x.update(),
            Gear::EMFMeter(x) => x.update(),
            Gear::Recorder(x) => x.update(),
            Gear::GeigerCounter(x) => x.update(),
            Gear::RedTorch(x) => x.update(),
            Gear::UVTorch(x) => x.update(),
            Gear::Photocam(x) => x.update(),
            Gear::IonMeter(x) => x.update(),
            Gear::SpiritBox(x) => x.update(),
            Gear::Compass(x) => x.update(),
            Gear::EStaticMeter(x) => x.update(),
            Gear::Videocam(x) => x.update(),
            Gear::MotionSensor(x) => x.update(),
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
