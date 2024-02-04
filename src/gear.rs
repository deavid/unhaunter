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
pub enum GearKind {
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

#[derive(Debug, Default, PartialEq, Eq, Clone)]
pub struct Gear {
    pub kind: GearKind,
}

impl Gear {
    pub fn new_from_kind(kind: GearKind) -> Self {
        Self { kind }
    }
}

impl GearUsable for Gear {
    fn get_display_name(&self) -> &'static str {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_display_name(),
            GearKind::Flashlight(x) => x.get_display_name(),
            GearKind::EMFMeter(x) => x.get_display_name(),
            GearKind::Recorder(x) => x.get_display_name(),
            GearKind::GeigerCounter(x) => x.get_display_name(),
            GearKind::UVTorch(x) => x.get_display_name(),
            GearKind::IonMeter(x) => x.get_display_name(),
            GearKind::Photocam(x) => x.get_display_name(),
            GearKind::SpiritBox(x) => x.get_display_name(),
            GearKind::RedTorch(x) => x.get_display_name(),
            GearKind::Compass(x) => x.get_display_name(),
            GearKind::ThermalImager(x) => x.get_display_name(),
            GearKind::EStaticMeter(x) => x.get_display_name(),
            GearKind::Videocam(x) => x.get_display_name(),
            GearKind::MotionSensor(x) => x.get_display_name(),
            GearKind::None => "",
        }
    }

    fn get_status(&self) -> String {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_status(),
            GearKind::Flashlight(x) => x.get_status(),
            GearKind::EMFMeter(x) => x.get_status(),
            GearKind::Recorder(x) => x.get_status(),
            GearKind::GeigerCounter(x) => x.get_status(),
            GearKind::UVTorch(x) => x.get_status(),
            GearKind::IonMeter(x) => x.get_status(),
            GearKind::Photocam(x) => x.get_status(),
            GearKind::SpiritBox(x) => x.get_status(),
            GearKind::RedTorch(x) => x.get_status(),
            GearKind::Compass(x) => x.get_status(),
            GearKind::ThermalImager(x) => x.get_status(),
            GearKind::EStaticMeter(x) => x.get_status(),
            GearKind::Videocam(x) => x.get_status(),
            GearKind::MotionSensor(x) => x.get_status(),
            GearKind::None => "".to_string(),
        }
    }

    fn set_trigger(&mut self) {
        let ni = |s| warn!("Trigger not implemented for {:?}", s);
        match &mut self.kind {
            GearKind::Thermometer(x) => x.set_trigger(),
            GearKind::Flashlight(x) => x.set_trigger(),
            GearKind::ThermalImager(x) => x.set_trigger(),
            GearKind::EMFMeter(x) => x.set_trigger(),
            GearKind::Recorder(x) => x.set_trigger(),
            GearKind::GeigerCounter(x) => x.set_trigger(),
            GearKind::RedTorch(x) => x.set_trigger(),
            GearKind::UVTorch(x) => x.set_trigger(),
            GearKind::Photocam(x) => x.set_trigger(),
            GearKind::IonMeter(x) => x.set_trigger(),
            GearKind::SpiritBox(x) => x.set_trigger(),
            GearKind::Compass(x) => x.set_trigger(),
            GearKind::EStaticMeter(x) => x.set_trigger(),
            GearKind::Videocam(x) => x.set_trigger(),
            GearKind::MotionSensor(x) => x.set_trigger(),
            GearKind::None => ni(&self),
        }
    }

    fn get_sprite_idx(&self) -> GearSpriteID {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_sprite_idx(),
            GearKind::Flashlight(x) => x.get_sprite_idx(),
            GearKind::ThermalImager(x) => x.get_sprite_idx(),
            GearKind::EMFMeter(x) => x.get_sprite_idx(),
            GearKind::Recorder(x) => x.get_sprite_idx(),
            GearKind::GeigerCounter(x) => x.get_sprite_idx(),
            GearKind::RedTorch(x) => x.get_sprite_idx(),
            GearKind::UVTorch(x) => x.get_sprite_idx(),
            GearKind::Photocam(x) => x.get_sprite_idx(),
            GearKind::IonMeter(x) => x.get_sprite_idx(),
            GearKind::SpiritBox(x) => x.get_sprite_idx(),
            GearKind::Compass(x) => x.get_sprite_idx(),
            GearKind::EStaticMeter(x) => x.get_sprite_idx(),
            GearKind::Videocam(x) => x.get_sprite_idx(),
            GearKind::MotionSensor(x) => x.get_sprite_idx(),
            GearKind::None => GearSpriteID::None,
        }
    }
    fn box_clone(&self) -> Box<dyn GearUsable> {
        Box::new(self.clone())
    }
    fn update(&mut self) {
        match &mut self.kind {
            GearKind::Thermometer(x) => x.update(),
            GearKind::Flashlight(x) => x.update(),
            GearKind::ThermalImager(x) => x.update(),
            GearKind::EMFMeter(x) => x.update(),
            GearKind::Recorder(x) => x.update(),
            GearKind::GeigerCounter(x) => x.update(),
            GearKind::RedTorch(x) => x.update(),
            GearKind::UVTorch(x) => x.update(),
            GearKind::Photocam(x) => x.update(),
            GearKind::IonMeter(x) => x.update(),
            GearKind::SpiritBox(x) => x.update(),
            GearKind::Compass(x) => x.update(),
            GearKind::EStaticMeter(x) => x.update(),
            GearKind::Videocam(x) => x.update(),
            GearKind::MotionSensor(x) => x.update(),
            GearKind::None => {}
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
