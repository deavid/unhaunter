use bevy::prelude::*;

use crate::gear::Gear;

#[derive(Debug, Resource, Clone)]
pub struct TruckGear {
    pub inventory: Vec<Gear>,
}

impl TruckGear {
    pub fn new() -> Self {
        use crate::gear::compass::Compass;
        use crate::gear::emfmeter::EMFMeter;
        use crate::gear::estaticmeter::EStaticMeter;
        use crate::gear::flashlight::Flashlight;
        use crate::gear::geigercounter::GeigerCounter;
        use crate::gear::ionmeter::IonMeter;
        use crate::gear::motionsensor::MotionSensor;
        use crate::gear::photocam::Photocam;
        use crate::gear::recorder::Recorder;
        use crate::gear::redtorch::RedTorch;
        use crate::gear::repellentflask::RepellentFlask;
        use crate::gear::spiritbox::SpiritBox;
        use crate::gear::thermalimager::ThermalImager;
        use crate::gear::thermometer::Thermometer;
        use crate::gear::uvtorch::UVTorch;
        use crate::gear::videocam::Videocam;

        Self {
            inventory: vec![
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
                // Incomplete equipment:
                IonMeter::default().into(),
                ThermalImager::default().into(),
                Photocam::default().into(),
                Compass::default().into(),
                EStaticMeter::default().into(),
                MotionSensor::default().into(),
            ],
        }
    }
}
