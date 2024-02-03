use std::default;

use bevy::ecs::component::Component;
use enum_iterator::Sequence;

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
