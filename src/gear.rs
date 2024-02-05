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

use self::playergear::{EquipmentPosition, Inventory, InventoryStats, PlayerGear};
use crate::board::{self, Position};
use crate::game::{GameConfig, PlayerSprite};
use bevy::ecs::system::SystemParam;
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
    #[default]
    None,
}

#[derive(Debug, Default, Clone)]
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

    fn set_trigger(&mut self, gs: &mut GearStuff) {
        let sound_file = "sounds/switch-on-1.ogg";
        gs.play_audio(sound_file.into(), 0.6);

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
    fn set_trigger(&mut self, gs: &mut GearStuff);
    fn update(&mut self, _gs: &mut GearStuff, _pos: &Position, _ep: &EquipmentPosition) {}
    fn get_sprite_idx(&self) -> GearSpriteID;
    fn box_clone(&self) -> Box<dyn GearUsable>;
}

pub fn keyboard_gear(
    keyboard_input: Res<Input<KeyCode>>,
    mut q_gear: Query<(&PlayerSprite, &mut PlayerGear)>,
    mut gs: GearStuff,
) {
    for (ps, mut playergear) in q_gear.iter_mut() {
        if keyboard_input.just_pressed(ps.controls.cycle) {
            playergear.cycle();
        }
        if keyboard_input.just_pressed(ps.controls.swap) {
            playergear.swap();
        }
        if keyboard_input.just_released(ps.controls.trigger) {
            playergear.right_hand.set_trigger(&mut gs);
        }
        if keyboard_input.just_released(ps.controls.torch) {
            playergear.left_hand.set_trigger(&mut gs);
        }
    }
}

pub fn update_gear_inventory(
    gc: Res<GameConfig>,
    mut q_gear: Query<(&PlayerSprite, &Position, &mut PlayerGear)>,
    mut qi: Query<(&Inventory, &mut UiTextureAtlasImage)>,
    mut qs: Query<(&InventoryStats, &mut Text)>,
    mut gs: GearStuff,
) {
    for (ps, position, mut playergear) in q_gear.iter_mut() {
        for (gear, epos) in playergear.as_vec_mut().into_iter() {
            gear.update(&mut gs, position, &epos);
        }

        if gc.player_id == ps.id {
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
    }
}

#[derive(SystemParam)]
pub struct GearStuff<'w, 's> {
    bf: ResMut<'w, board::BoardData>,
    commands: Commands<'w, 's>,
    asset_server: Res<'w, AssetServer>,
}

impl<'w, 's> GearStuff<'w, 's> {
    pub fn play_audio(&mut self, sound_file: String, volume: f32) {
        self.commands.spawn(AudioBundle {
            source: self.asset_server.load(sound_file),
            settings: PlaybackSettings {
                mode: bevy::audio::PlaybackMode::Once,
                volume: bevy::audio::Volume::Relative(bevy::audio::VolumeLevel::new(volume)),
                speed: 1.0,
                paused: false,
                spatial: false,
            },
        });
    }
}