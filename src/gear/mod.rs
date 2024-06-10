//! Gear Module
//! -----------
//!
//! This module defines the gear system for the game, including:
//! * Different types of gear available to the player.
//! * A common interface for interacting with gear (`GearUsable` trait).
//! * Functions for updating gear state based on player actions and game conditions.
//! * Visual representations of gear using sprites and animations.
//!
//! The gear system allows players to equip and use various tools to investigate paranormal activity,
//! gather evidence, and ultimately banish ghosts.

pub mod compass;
pub mod emfmeter;
pub mod estaticmeter;
pub mod flashlight;
pub mod geigercounter;
pub mod ionmeter;
pub mod motionsensor;
pub mod photocam;
pub mod playergear;
pub mod quartz;
pub mod recorder;
pub mod redtorch;
pub mod repellentflask;
pub mod sage;
pub mod salt;
pub mod spiritbox;
pub mod thermalimager;
pub mod thermometer;
pub mod ui;
pub mod uvtorch;
pub mod videocam;

use std::mem::swap;

use self::compass::Compass;
use self::emfmeter::EMFMeter;
use self::estaticmeter::EStaticMeter;
use self::flashlight::Flashlight;
use self::geigercounter::GeigerCounter;
use self::ionmeter::IonMeter;
use self::motionsensor::MotionSensor;
use self::photocam::Photocam;
use self::quartz::QuartzStoneData;
use self::recorder::Recorder;
use self::redtorch::RedTorch;
use self::repellentflask::RepellentFlask;
use self::sage::SageBundleData;
use self::salt::SaltData;
use self::spiritbox::SpiritBox;
use self::thermalimager::ThermalImager;
use self::thermometer::Thermometer;
use self::uvtorch::UVTorch;
use self::videocam::Videocam;

use self::playergear::{EquipmentPosition, PlayerGear};
use crate::board::{self, Position};
use crate::game::GameConfig;
use crate::player::{DeployedGear, DeployedGearData, PlayerSprite};
use crate::summary;
use bevy::ecs::system::SystemParam;
use bevy::prelude::*;

/// Unique identifiers for different gear sprites.
///
/// Each variant represents a specific sprite or animation frame for a piece of gear.
/// The values are used to index into the gear spritesheet.
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

/// Represents the different types of gear available in the game.
///
/// Each variant holds a specific gear struct with its own attributes and behavior.
#[derive(Debug, Default, Clone)]
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
    RepellentFlask(RepellentFlask),
    QuartzStone(QuartzStoneData),
    Salt(SaltData),
    SageBundle(SageBundleData),
    #[default]
    None,
}

impl GearKind {
    pub fn is_none(&self) -> bool {
        matches!(self, GearKind::None)
    }

    pub fn is_some(&self) -> bool {
        !self.is_none()
    }
}

/// A wrapper struct for holding a `GearKind`.
#[derive(Debug, Default, Clone)]
pub struct Gear {
    /// The type of gear being held.
    pub kind: GearKind,
}

impl Gear {
    /// Creates a new empty Gear
    pub fn none() -> Self {
        Self {
            kind: GearKind::None,
        }
    }

    /// Creates a new Gear of the specified Kind
    pub fn new_from_kind(kind: GearKind) -> Self {
        Self { kind }
    }

    /// Takes the content of the current Gear and returns it, leaving None.
    pub fn take(&mut self) -> Self {
        let mut new = Self::none();
        swap(&mut new, self);
        new
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
            GearKind::RepellentFlask(x) => x.get_display_name(),
            GearKind::QuartzStone(x) => x.get_display_name(),
            GearKind::Salt(x) => x.get_display_name(),
            GearKind::SageBundle(x) => x.get_display_name(),
            GearKind::None => "",
        }
    }
    fn get_description(&self) -> &'static str {
        match &self.kind {
            GearKind::Thermometer(x) => x.get_description(),
            GearKind::Flashlight(x) => x.get_description(),
            GearKind::EMFMeter(x) => x.get_description(),
            GearKind::Recorder(x) => x.get_description(),
            GearKind::GeigerCounter(x) => x.get_description(),
            GearKind::UVTorch(x) => x.get_description(),
            GearKind::IonMeter(x) => x.get_description(),
            GearKind::Photocam(x) => x.get_description(),
            GearKind::SpiritBox(x) => x.get_description(),
            GearKind::RedTorch(x) => x.get_description(),
            GearKind::Compass(x) => x.get_description(),
            GearKind::ThermalImager(x) => x.get_description(),
            GearKind::EStaticMeter(x) => x.get_description(),
            GearKind::Videocam(x) => x.get_description(),
            GearKind::MotionSensor(x) => x.get_description(),
            GearKind::RepellentFlask(x) => x.get_description(),
            GearKind::QuartzStone(x) => x.get_description(),
            GearKind::Salt(x) => x.get_description(),
            GearKind::SageBundle(x) => x.get_description(),
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
            GearKind::RepellentFlask(x) => x.get_status(),
            GearKind::QuartzStone(x) => x.get_status(),
            GearKind::Salt(x) => x.get_status(),
            GearKind::SageBundle(x) => x.get_status(),
            GearKind::None => "".to_string(),
        }
    }

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        let sound_file = "sounds/switch-on-1.ogg";
        gs.play_audio_nopos(sound_file.into(), 0.6);

        let ni = |s| warn!("Trigger not implemented for {:?}", s);
        match &mut self.kind {
            GearKind::Thermometer(x) => x.set_trigger(gs),
            GearKind::Flashlight(x) => x.set_trigger(gs),
            GearKind::ThermalImager(x) => x.set_trigger(gs),
            GearKind::EMFMeter(x) => x.set_trigger(gs),
            GearKind::Recorder(x) => x.set_trigger(gs),
            GearKind::GeigerCounter(x) => x.set_trigger(gs),
            GearKind::RedTorch(x) => x.set_trigger(gs),
            GearKind::UVTorch(x) => x.set_trigger(gs),
            GearKind::Photocam(x) => x.set_trigger(gs),
            GearKind::IonMeter(x) => x.set_trigger(gs),
            GearKind::SpiritBox(x) => x.set_trigger(gs),
            GearKind::Compass(x) => x.set_trigger(gs),
            GearKind::EStaticMeter(x) => x.set_trigger(gs),
            GearKind::Videocam(x) => x.set_trigger(gs),
            GearKind::MotionSensor(x) => x.set_trigger(gs),
            GearKind::RepellentFlask(x) => x.set_trigger(gs),
            GearKind::QuartzStone(x) => x.set_trigger(gs),
            GearKind::Salt(x) => x.set_trigger(gs),
            GearKind::SageBundle(x) => x.set_trigger(gs),
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
            GearKind::RepellentFlask(x) => x.get_sprite_idx(),
            GearKind::QuartzStone(x) => x.get_sprite_idx(),
            GearKind::Salt(x) => x.get_sprite_idx(),
            GearKind::SageBundle(x) => x.get_sprite_idx(),
            GearKind::None => GearSpriteID::None,
        }
    }
    fn _box_clone(&self) -> Box<dyn GearUsable> {
        // FIXME: This is not used at all.
        Box::new(self.clone())
    }
    fn update(&mut self, gs: &mut GearStuff, pos: &Position, ep: &EquipmentPosition) {
        match &mut self.kind {
            GearKind::Thermometer(x) => x.update(gs, pos, ep),
            GearKind::Flashlight(x) => x.update(gs, pos, ep),
            GearKind::ThermalImager(x) => x.update(gs, pos, ep),
            GearKind::EMFMeter(x) => x.update(gs, pos, ep),
            GearKind::Recorder(x) => x.update(gs, pos, ep),
            GearKind::GeigerCounter(x) => x.update(gs, pos, ep),
            GearKind::RedTorch(x) => x.update(gs, pos, ep),
            GearKind::UVTorch(x) => x.update(gs, pos, ep),
            GearKind::Photocam(x) => x.update(gs, pos, ep),
            GearKind::IonMeter(x) => x.update(gs, pos, ep),
            GearKind::SpiritBox(x) => x.update(gs, pos, ep),
            GearKind::Compass(x) => x.update(gs, pos, ep),
            GearKind::EStaticMeter(x) => x.update(gs, pos, ep),
            GearKind::Videocam(x) => x.update(gs, pos, ep),
            GearKind::MotionSensor(x) => x.update(gs, pos, ep),
            GearKind::RepellentFlask(x) => x.update(gs, pos, ep),
            GearKind::QuartzStone(x) => x.update(gs, pos, ep),
            GearKind::Salt(x) => x.update(gs, pos, ep),
            GearKind::SageBundle(x) => x.update(gs, pos, ep),
            GearKind::None => {}
        }
    }
}

/// Utility function to convert a boolean value to "ON" or "OFF".
pub fn on_off(s: bool) -> &'static str {
    match s {
        true => "ON",
        false => "OFF",
    }
}

/// Provides a common interface for all gear types, enabling consistent interactions.
pub trait GearUsable: std::fmt::Debug + Sync + Send {
    /// Returns the display name of the gear (e.g., "EMF Reader").
    fn get_display_name(&self) -> &'static str;
    /// Returns a brief description of the gear's functionality.
    fn get_description(&self) -> &'static str;
    /// Returns a string representing the current status of the gear (e.g., "ON", "OFF", "Reading: 5.0 mG").
    fn get_status(&self) -> String;
    /// Triggers the gear's primary action (e.g., turn on/off, take a reading).
    fn set_trigger(&mut self, gs: &mut GearStuff);
    /// Updates the gear's internal state based on time, player actions, or game conditions.
    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}
    /// Returns the `GearSpriteID` for the gear's current state.
    fn get_sprite_idx(&self) -> GearSpriteID;
    /// Creates a boxed clone of the `GearUsable` object. (Unused for now)
    fn _box_clone(&self) -> Box<dyn GearUsable>;
}

/// System for updating the internal state of all gear carried by the player.
///
/// This system iterates through the player's gear and calls the `update` method for each piece of gear,
/// allowing gear to update their state based on time, player actions, or environmental conditions.
pub fn update_playerheld_gear_data(
    mut q_gear: Query<(&Position, &mut PlayerGear)>,
    mut gs: GearStuff,
) {
    for (position, mut playergear) in q_gear.iter_mut() {
        for (gear, epos) in playergear.as_vec_mut().into_iter() {
            gear.update(&mut gs, position, &epos);
        }
    }
}

/// System for updating the internal state of all gear deployed in the environment.
pub fn update_deployed_gear_data(
    mut q_gear: Query<(&Position, &DeployedGear, &mut DeployedGearData)>,
    mut gs: GearStuff,
) {
    for (position, _deployed_gear, mut gear_data) in q_gear.iter_mut() {
        gear_data
            .gear
            .update(&mut gs, position, &playergear::EquipmentPosition::Deployed);
    }
}

/// System for updating the sprites of deployed gear to reflect their internal state.
pub fn update_deployed_gear_sprites(mut q_gear: Query<(&mut TextureAtlas, &DeployedGearData)>) {
    for (mut texture_atlas, gear_data) in q_gear.iter_mut() {
        let new_index = gear_data.gear.get_sprite_idx() as usize;
        if texture_atlas.index != new_index {
            texture_atlas.index = new_index;
        }
    }
}

/// A collection of resources and commands frequently used by gear-related systems.
#[derive(SystemParam)]
pub struct GearStuff<'w, 's> {
    /// Access to the game's board data, including collision, lighting, and temperature fields.
    pub bf: ResMut<'w, board::BoardData>,
    /// Access to summary data, which tracks game progress and statistics.
    pub summary: ResMut<'w, summary::SummaryData>,
    /// Allows gear systems to spawn new entities (e.g., for sound effects).
    pub commands: Commands<'w, 's>,
    /// Provides access to the asset server for loading sound effects.
    pub asset_server: Res<'w, AssetServer>,
    /// Access to the current game time.
    pub time: Res<'w, Time>,
    /// Event writer for sending sound events.
    pub sound_events: EventWriter<'w, SoundEvent>,
}

impl<'w, 's> GearStuff<'w, 's> {
    /// Plays a sound effect using the specified file path and volume from the given position.
    pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: Some(*position),
        };
        // Send the SoundEvent to be handled by the sound playback system
        self.sound_events.send(sound_event);
    }

    /// Plays a sound effect without having a position volume modifier.
    pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
        // Create a SoundEvent with the required data
        let sound_event = SoundEvent {
            sound_file,
            volume,
            position: None,
        };
        // Send the SoundEvent to be handled by the sound playback system
        self.sound_events.send(sound_event);
    }
}

/// Represents an event to play a sound effect at a specific location in the game world.
///
/// This event is used to trigger the playback of a sound with volume adjusted
/// based on the distance to the player's position.
#[derive(Debug, Event, Clone)]
pub struct SoundEvent {
    /// The path to the sound file to be played.
    pub sound_file: String,
    /// The initial volume of the sound effect (this will be adjusted based on distance).
    pub volume: f32,
    /// The position in the game world where the sound is originating from.
    pub position: Option<Position>,
}

/// System to handle the SoundEvent, playing the sound with volume adjusted by distance.
pub fn sound_playback_system(
    mut sound_events: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    gc: Res<GameConfig>,
    qp: Query<(&Position, &PlayerSprite)>,
    mut commands: Commands,
) {
    for sound_event in sound_events.read() {
        // Get player position (Match against the player ID from GameConfig)
        let Some((player_position, _)) = qp.iter().find(|(_, p)| p.id == gc.player_id) else {
            return;
        };
        let adjusted_volume = match sound_event.position {
            Some(position) => {
                const MIN_DIST: f32 = 25.0;
                // Calculate distance from player to sound source
                let distance2 = player_position.distance2(&position) + MIN_DIST;
                let distance = distance2.powf(0.7) + MIN_DIST;

                // Calculate adjusted volume based on distance
                (sound_event.volume / distance2 * MIN_DIST
                    + sound_event.volume / distance * MIN_DIST)
                    .clamp(0.0, 1.0)
            }
            None => sound_event.volume,
        };

        // Spawn an AudioBundle with the adjusted volume
        commands.spawn(AudioBundle {
            source: asset_server.load(sound_event.sound_file.clone()),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Despawn,
                volume: bevy::audio::Volume::new(adjusted_volume),
                speed: 1.0,
                paused: false,
                spatial: false,
                spatial_scale: None,
            },
        });
    }
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<GameConfig>()
        .add_systems(FixedUpdate, update_playerheld_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_data)
        .add_systems(FixedUpdate, update_deployed_gear_sprites)
        .add_systems(Update, quartz::update_quartz_and_ghost)
        .add_systems(Update, salt::salt_particle_system)
        .add_systems(Update, salt::salt_pile_system)
        .add_systems(Update, salt::salty_trace_system)
        .add_systems(Update, sage::sage_smoke_system)
        .add_systems(Update, thermometer::temperature_update)
        .add_systems(Update, recorder::sound_update)
        .add_systems(Update, repellentflask::repellent_update)
        .add_systems(Update, sound_playback_system)
        .add_event::<SoundEvent>();
    ui::app_setup(app);
}
