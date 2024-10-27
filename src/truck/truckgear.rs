use crate::gear::Gear;
use bevy::prelude::*;

#[derive(Debug, Resource, Clone)]
pub struct TruckGear {
    pub inventory: Vec<Gear>,
}

impl TruckGear {
    pub fn new() -> Self {
        const ENABLE_INCOMPLETE: bool = false;

        use crate::gear::compass::Compass;
        use crate::gear::emfmeter::EMFMeter;
        use crate::gear::estaticmeter::EStaticMeter;
        use crate::gear::flashlight::Flashlight;
        use crate::gear::geigercounter::GeigerCounter;
        use crate::gear::ionmeter::IonMeter;
        use crate::gear::motionsensor::MotionSensor;
        use crate::gear::photocam::Photocam;
        use crate::gear::quartz::QuartzStoneData;
        use crate::gear::recorder::Recorder;
        use crate::gear::redtorch::RedTorch;
        use crate::gear::repellentflask::RepellentFlask;
        use crate::gear::sage::SageBundleData;
        use crate::gear::salt::SaltData;
        use crate::gear::spiritbox::SpiritBox;
        use crate::gear::thermalimager::ThermalImager;
        use crate::gear::thermometer::Thermometer;
        use crate::gear::uvtorch::UVTorch;
        use crate::gear::videocam::Videocam;

        let mut inventory: Vec<Gear> = vec![
            Flashlight::default().into(),
            Thermometer::default().into(),
            EMFMeter::default().into(),
            UVTorch::default().into(),
            SpiritBox::default().into(),
            Recorder::default().into(),
            Videocam::default().into(),
            RedTorch::default().into(),
            GeigerCounter::default().into(),
            RepellentFlask::default().into(),
            QuartzStoneData::default().into(),
            SaltData::default().into(),
            SageBundleData::default().into(),
        ];
        if ENABLE_INCOMPLETE {
            let mut incomplete: Vec<Gear> = vec![
                // Incomplete equipment:
                IonMeter::default().into(),
                ThermalImager::default().into(),
                Photocam::default().into(),
                Compass::default().into(),
                EStaticMeter::default().into(),
                MotionSensor::default().into(),
            ];
            inventory.append(&mut incomplete);
        }
        Self { inventory }
    }
}
