use uncore::types::gear_kind::{GearKind, PlayerGearKind};
use crate::gear::{ext::types::gear::Gear, playergear::PlayerGear};

impl From<GearKind> for Gear {
    fn from(value: GearKind) -> Self {
        use super::prelude::*;
        match value {
            GearKind::Thermometer => Thermometer::default().into(),
            GearKind::EMFMeter => EMFMeter::default().into(),
            GearKind::Recorder => Recorder::default().into(),
            GearKind::Flashlight => Flashlight::default().into(),
            GearKind::GeigerCounter => GeigerCounter::default().into(),
            GearKind::UVTorch => UVTorch::default().into(),
            GearKind::IonMeter => IonMeter::default().into(),
            GearKind::SpiritBox => SpiritBox::default().into(),
            GearKind::ThermalImager => ThermalImager::default().into(),
            GearKind::RedTorch => RedTorch::default().into(),
            GearKind::Photocam => Photocam::default().into(),
            GearKind::Compass => Compass::default().into(),
            GearKind::EStaticMeter => EStaticMeter::default().into(),
            GearKind::Videocam => Videocam::default().into(),
            GearKind::MotionSensor => MotionSensor::default().into(),
            GearKind::RepellentFlask => RepellentFlask::default().into(),
            GearKind::QuartzStone => QuartzStoneData::default().into(),
            GearKind::Salt => SaltData::default().into(),
            GearKind::SageBundle => SageBundleData::default().into(),
            GearKind::None => Gear::none(),
        }
    }
}

impl From<PlayerGearKind> for PlayerGear {
    fn from(value: PlayerGearKind) -> Self {
        Self {
            left_hand: value.left_hand.into(),
            right_hand: value.right_hand.into(),
            inventory: value.inventory.into_iter().map(Gear::from).collect(),
            held_item: None,
        }
    }
}
