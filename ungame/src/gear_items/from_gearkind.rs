use uncore::types::gear_kind::{GearKind, PlayerGearKind};
use ungear::{components::playergear::PlayerGear, types::gear::Gear};

pub trait FromGearKind {
    fn from_gearkind(value: GearKind) -> Gear;
}

impl FromGearKind for Gear {
    fn from_gearkind(value: GearKind) -> Self {
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

pub trait FromPlayerGearKind {
    fn from_playergearkind(value: PlayerGearKind) -> PlayerGear;
}

impl FromPlayerGearKind for PlayerGear {
    fn from_playergearkind(value: PlayerGearKind) -> Self {
        Self {
            left_hand: Gear::from_gearkind(value.left_hand),
            right_hand: Gear::from_gearkind(value.right_hand),
            inventory: value
                .inventory
                .into_iter()
                .map(Gear::from_gearkind)
                .collect(),
            held_item: None,
        }
    }
}
