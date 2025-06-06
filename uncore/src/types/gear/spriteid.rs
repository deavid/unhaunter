/// Unique identifiers for different gear sprites.
///
/// Each variant represents a specific sprite or animation frame for a piece of
/// gear. The values are used to index into the gear spritesheet.
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
    RepelentFlaskEmpty = 60,
    RepelentFlaskFull,
    // Quartz Stone
    QuartzStone0 = 65,
    QuartzStone1,
    QuartzStone2,
    QuartzStone3,
    QuartzStone4,
    // Salt
    Salt4 = 75,
    Salt3,
    Salt2,
    Salt1,
    Salt0,
    Compass = 80,
    // Sage Bundle
    SageBundle0 = 85,
    SageBundle1,
    SageBundle2,
    SageBundle3,
    SageBundle4,
    EStaticMeter = 90,
    Videocam,
    MotionSensor,
    #[default]
    None,
}
