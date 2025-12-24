use bevy::prelude::*;
use uncore_components::*;
use uncore_foundation::types::evidence::Evidence;
use uncore_foundation::types::gear::GearKind;
use uncore_foundation::types::gear::GearSpriteID;
use ungear::resources::spawner::{GearMetadata, GearSpawnerRegistry};

use crate::components::compass::Compass;
use crate::components::emfmeter::EMFMeter as EMFMeterInternal;
use crate::components::estaticmeter::EStaticMeter;
use crate::components::flashlight::Flashlight as FlashlightInternal;
use crate::components::geigercounter::GeigerCounter;
use crate::components::ionmeter::IonMeter;
use crate::components::motionsensor::MotionSensor;
use crate::components::photocam::Photocam;
use crate::components::quartz::QuartzStoneData;
use crate::components::recorder::Recorder;
use crate::components::redtorch::RedTorch;
use crate::components::repellentflask::RepellentFlask;
use crate::components::sage::SageBundleData;
use crate::components::salt::SaltData;
use crate::components::spiritbox::SpiritBox;
use crate::components::thermalimager::ThermalImager;
use crate::components::thermometer::Thermometer as ThermometerInternal;
use crate::components::uvtorch::UVTorch;
use crate::components::videocam::Videocam;

use uncore_foundation::types::light::LightType;

pub fn register_all(app: &mut App) {
    let mut registry = app.world_mut().resource_mut::<GearSpawnerRegistry>();

    registry.register(
        GearKind::Flashlight,
        GearMetadata {
            name: "Flashlight".into(),
            description: "Iluminates the way. Imprescindible tool to work in the dark.".into(),
            sprite_idx: GearSpriteID::FlashlightOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Flashlight"));
            cmd.insert(ItemDescription::new(
                "Iluminates the way. Imprescindible tool to work in the dark.",
            ));
            cmd.insert(GearSprite(GearSpriteID::FlashlightOff));
            cmd.insert(Flashlight {
                power: 10.0,
                color: Color::WHITE,
                light_type: LightType::Visible,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0001,
            });
            cmd.insert(Handheld);
            cmd.insert(FlashlightInternal::default());
        },
    );

    registry.register(
        GearKind::Thermometer,
        GearMetadata {
            name: "Thermometer".into(),
            description: "Reads the temperature of the room. Most paranormal interactions have been correlated with unusual cold temperatures.".into(),
            sprite_idx: GearSpriteID::ThermometerOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Thermometer"));
            cmd.insert(ItemDescription::new("Reads the temperature of the room. Most paranormal interactions have been correlated with unusual cold temperatures."));
            cmd.insert(GearSprite(GearSpriteID::ThermometerOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::FreezingTemp,
            });
            cmd.insert(Electronic {
                sensitivity: 0.5,
                ..default()
            });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.00005,
            });
            cmd.insert(Handheld);
            cmd.insert(ThermometerInternal::default());
        },
    );

    registry.register(
        GearKind::EMFMeter,
        GearMetadata {
            name: "EMF Meter".into(),
            description: "Used to find electric wires behind walls. Ghosts might disturb the electromagnetic field.".into(),
            sprite_idx: GearSpriteID::EMFMeterOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("EMF Meter"));
            cmd.insert(ItemDescription::new("Used to find electric wires behind walls. Ghosts might disturb the electromagnetic field."));
            cmd.insert(GearSprite(GearSpriteID::EMFMeterOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::EMFLevel5,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0001,
            });
            cmd.insert(Handheld);
            cmd.insert(EMFMeterInternal::default());
        },
    );

    registry.register(
        GearKind::Recorder,
        GearMetadata {
            name: "Recorder".into(),
            description:
                "Records ambient sounds and conversations. Sometimes it can capture EVP phenomena."
                    .into(),
            sprite_idx: GearSpriteID::RecorderOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Recorder"));
            cmd.insert(ItemDescription::new(
                "Records ambient sounds and conversations. Sometimes it can capture EVP phenomena.",
            ));
            cmd.insert(GearSprite(GearSpriteID::RecorderOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::EVPRecording,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                glitch_intensity: 0.0,
                glitch_timer: 0.0,
            });
            cmd.insert(Battery::default());
            cmd.insert(Handheld);
            cmd.insert(Recorder::default());
        },
    );

    registry.register(
        GearKind::GeigerCounter,
        GearMetadata {
            name: "Geiger Counter".into(),
            description: "Measures radioactivity by counting alpha and beta particles. It can be used to roughly locate the ghost with patience.".into(),
            sprite_idx: GearSpriteID::GeigerOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Geiger Counter"));
            cmd.insert(ItemDescription::new("Measures radioactivity by counting alpha and beta particles. It can be used to roughly locate the ghost with patience."));
            cmd.insert(GearSprite(GearSpriteID::GeigerOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::CPM500,
            });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(GeigerCounter::default());
        },
    );

    registry.register(
        GearKind::UVTorch,
        GearMetadata {
            name: "UV Torch".into(),
            description: "Ultraviolet light that can be used to expose evidence invisible to the naked eye since some substances react to it and glow.".into(),
            sprite_idx: GearSpriteID::UVTorchOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("UV Torch"));
            cmd.insert(ItemDescription::new("Ultraviolet light that can be used to expose evidence invisible to the naked eye since some substances react to it and glow."));
            cmd.insert(GearSprite(GearSpriteID::UVTorchOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::UVEctoplasm,
            });
            cmd.insert(Flashlight {
                power: 5.0,
                color: Color::srgb(0.60, 0.25, 1.00),
                light_type: LightType::UltraViolet,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(UVTorch::default());
        },
    );

    registry.register(
        GearKind::IonMeter,
        GearMetadata {
            name: "Ion Meter".into(),
            description: "Detects charged particles in the air. Ghost leave a trace as they move and this tool may help following the ghost.".into(),
            sprite_idx: GearSpriteID::IonMeterOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Ion Meter"));
            cmd.insert(ItemDescription::new("Detects charged particles in the air. Ghost leave a trace as they move and this tool may help following the ghost."));
            cmd.insert(GearSprite(GearSpriteID::IonMeterOff));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(IonMeter::default());
        },
    );

    registry.register(
        GearKind::SpiritBox,
        GearMetadata {
            name: "Spirit Box".into(),
            description: "A modified AM Radio that constantly changes radio stations. It is said that the ghost can manipulate this to send messages to the living if you're close to its breach, and with the lights off.".into(),
            sprite_idx: GearSpriteID::SpiritBoxOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Spirit Box"));
            cmd.insert(ItemDescription::new("A modified AM Radio that constantly changes radio stations. It is said that the ghost can manipulate this to send messages to the living if you're close to its breach, and with the lights off."));
            cmd.insert(GearSprite(GearSpriteID::SpiritBoxOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::SpiritBox,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(SpiritBox::default());
        },
    );

    registry.register(
        GearKind::ThermalImager,
        GearMetadata {
            name: "Thermal Imager".into(),
            description: "Heat vision to see easily what's hot and what's cold. Might improve visibility of the paranormal and haunted objects.".into(),
            sprite_idx: GearSpriteID::ThermalImagerOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Thermal Imager"));
            cmd.insert(ItemDescription::new("Heat vision to see easily what's hot and what's cold. Might improve visibility of the paranormal and haunted objects."));
            cmd.insert(GearSprite(GearSpriteID::ThermalImagerOff));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(ThermalImager::default());
        },
    );

    registry.register(
        GearKind::RedTorch,
        GearMetadata {
            name: "Red Torch".into(),
            description: "A simple red light used by astronomers to see on the dark without losing night vision eye's adaptation. But the ghost might also react to this too.".into(),
            sprite_idx: GearSpriteID::RedTorchOff,
        },
        |cmd| {
            cmd.insert(ItemName::new("Red Torch"));
            cmd.insert(ItemDescription::new("A simple red light used by astronomers to see on the dark without losing night vision eye's adaptation. But the ghost might also react to this too."));
            cmd.insert(GearSprite(GearSpriteID::RedTorchOff));
            cmd.insert(EvidenceSensor {
                evidence: Evidence::RLPresence,
            });
            cmd.insert(Flashlight {
                power: 2.5,
                color: Color::srgb(1.0, 0.20, 0.07),
                light_type: LightType::Red,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(RedTorch::default());
        },
    );

    registry.register(
        GearKind::Photocam,
        GearMetadata {
            name: "Photo Camera".into(),
            description: "Takes photos, hopefully of something paranormal.".into(),
            sprite_idx: GearSpriteID::Photocam,
        },
        |cmd| {
            cmd.insert(ItemName::new("Photo Camera"));
            cmd.insert(ItemDescription::new(
                "Takes photos, hopefully of something paranormal.",
            ));
            cmd.insert(GearSprite(GearSpriteID::Photocam));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(Photocam::default());
        },
    );

    registry.register(
        GearKind::Compass,
        GearMetadata {
            name: "Compass".into(),
            description: "Measures the Earth's magnetic field, and sometimes the ghost.".into(),
            sprite_idx: GearSpriteID::Compass,
        },
        |cmd| {
            cmd.insert(ItemName::new("Compass"));
            cmd.insert(ItemDescription::new(
                "Measures the Earth's magnetic field, and sometimes the ghost.",
            ));
            cmd.insert(GearSprite(GearSpriteID::Compass));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Handheld);
            cmd.insert(Compass::default());
        },
    );

    registry.register(
        GearKind::EStaticMeter,
        GearMetadata {
            name: "Electrostatic Meter".into(),
            description:
                "Measures static electricity in the air. Might warn if the ghost is angering."
                    .into(),
            sprite_idx: GearSpriteID::EStaticMeter,
        },
        |cmd| {
            cmd.insert(ItemName::new("Electrostatic Meter"));
            cmd.insert(ItemDescription::new(
                "Measures static electricity in the air. Might warn if the ghost is angering.",
            ));
            cmd.insert(GearSprite(GearSpriteID::EStaticMeter));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(EStaticMeter::default());
        },
    );

    registry.register(
        GearKind::Videocam,
        GearMetadata {
            name: "Video Camera NV".into(),
            description: "Mainly used for its infrared night vision, it can also transmit images to the van in real time.".into(),
            sprite_idx: GearSpriteID::Videocam,
        },
        |cmd| {
            cmd.insert(ItemName::new("Video Camera NV"));
            cmd.insert(ItemDescription::new("Mainly used for its infrared night vision, it can also transmit images to the van in real time."));
            cmd.insert(GearSprite(GearSpriteID::Videocam));
            cmd.insert(Flashlight {
                power: 35.0,
                color: Color::srgb(0.01, 1.00, 0.70),
                light_type: LightType::InfraRedNV,
            });
            cmd.insert(EvidenceSensor {
                evidence: Evidence::FloatingOrbs,
            });
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Handheld);
            cmd.insert(Videocam::default());
        },
    );

    registry.register(
        GearKind::MotionSensor,
        GearMetadata {
            name: "Motion Sensor".into(),
            description: "Shoots an infrared beam that if cut will make the device beep. Can alert if a presence passes through.".into(),
            sprite_idx: GearSpriteID::MotionSensor,
        },
        |cmd| {
            cmd.insert(ItemName::new("Motion Sensor"));
            cmd.insert(ItemDescription::new("Shoots an infrared beam that if cut will make the device beep. Can alert if a presence passes through."));
            cmd.insert(GearSprite(GearSpriteID::MotionSensor));
            cmd.insert(Toggleable { is_on: false });
            cmd.insert(Battery {
                level: 1.0,
                drain_rate: 0.0002,
            });
            cmd.insert(Electronic {
                sensitivity: 1.0,
                ..default()
            });
            cmd.insert(Handheld);
            cmd.insert(MotionSensor::default());
        },
    );

    registry.register(
        GearKind::RepellentFlask,
        GearMetadata {
            name: "Repellent".into(),
            description: "Crafted in the van, specifically targeting a single ghost type to be effective enough to expel a ghost.".into(),
            sprite_idx: GearSpriteID::RepelentFlaskEmpty,
        },
        |cmd| {
            cmd.insert(ItemName::new("Repellent"));
            cmd.insert(ItemDescription::new("Crafted in the van, specifically targeting a single ghost type to be effective enough to expel a ghost."));
            cmd.insert(GearSprite(GearSpriteID::RepelentFlaskEmpty));
            cmd.insert(Handheld);
            cmd.insert(RepellentFlask::default());
        },
    );

    registry.register(
        GearKind::QuartzStone,
        GearMetadata {
            name: "Quartz Stone".into(),
            description: "A protective charm that absorbs the ghost's hunting energy, preventing or shortening hunts. The stone gradually cracks and eventually breaks after repeated uses.".into(),
            sprite_idx: GearSpriteID::QuartzStone0,
        },
        |cmd| {
            cmd.insert(ItemName::new("Quartz Stone"));
            cmd.insert(ItemDescription::new("A protective charm that absorbs the ghost's hunting energy, preventing or shortening hunts. The stone gradually cracks and eventually breaks after repeated uses."));
            cmd.insert(GearSprite(GearSpriteID::QuartzStone0));
            cmd.insert(Handheld);
            cmd.insert(QuartzStoneData::default());
        },
    );

    registry.register(
        GearKind::Salt,
        GearMetadata {
            name: "Salt".into(),
            description: "A bottle containing four charges of salt. Players can drop salt piles to repel the ghost and create temporary trails of UV-reactive salt particles.".into(),
            sprite_idx: GearSpriteID::Salt4,
        },
        |cmd| {
            cmd.insert(ItemName::new("Salt"));
            cmd.insert(ItemDescription::new("A bottle containing four charges of salt. Players can drop salt piles to repel the ghost and create temporary trails of UV-reactive salt particles."));
            cmd.insert(GearSprite(GearSpriteID::Salt4));
            cmd.insert(Handheld);
            cmd.insert(SaltData::default());
        },
    );

    registry.register(
        GearKind::SageBundle,
        GearMetadata {
            name: "Sage Bundle".into(),
            description: "A bundle of sage that, when activated, burns slowly and emits soothing smoke particles that calm the ghost over time.".into(),
            sprite_idx: GearSpriteID::SageBundle0,
        },
        |cmd| {
            cmd.insert(ItemName::new("Sage Bundle"));
            cmd.insert(ItemDescription::new("A bundle of sage that, when activated, burns slowly and emits soothing smoke particles that calm the ghost over time."));
            cmd.insert(GearSprite(GearSpriteID::SageBundle0));
            cmd.insert(Handheld);
            cmd.insert(SageBundleData::default());
        },
    );
}
