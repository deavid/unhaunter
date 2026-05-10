use bevy::prelude::*;
use unaudiospatial_core::emitter::LocalAudioEmitter;
use ungear_core::components::core::{GearSprite, StatusText};
use ungear_core::types::gear::sprite_id::GearSpriteID;
use ungearitems_core::components::quartz::{QuartzStoneData, QuartzStoneSkin};
use ungearitems_core::events::QuartzCrackedEvent;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;

pub(crate) fn play_quartz_crack_audio(
    mut ev_cracked: MessageReader<QuartzCrackedEvent>,
    mut gs_audio: LocalAudioEmitter,
) {
    for ev in ev_cracked.read() {
        let pos = Position {
            x: ev.position[0],
            y: ev.position[1],
            z: ev.position[2],
            visual_priority: ev.position[3],
        };
        gs_audio.play_audio("sounds/quartz_crack.ogg".into(), 1.0, &pos);
    }
}

pub(crate) fn update_quartz_skin(
    mut q_quartz: Query<(
        &QuartzStoneData,
        &QuartzStoneSkin,
        &mut StatusText,
        &mut GearSprite,
    )>,
) {
    for (quartz, skin, mut status, mut sprite) in q_quartz.iter_mut() {
        let state = match quartz.cracks {
            0 => "Pure",
            1 => "Used once",
            2 => "Used twice",
            3 => "Cracked, one use remaining",
            4 => "Shattered - Unusable",
            _ => "unknown",
        };
        status.0 = format!(
            "State: {state}\nEnergy absorbed: {energy:.1}",
            energy = skin.energy_absorbed - skin.cracked_time
        );

        sprite.0 = match quartz.cracks {
            0 => GearSpriteID::QuartzStone0.to_visual_key(),
            1 => GearSpriteID::QuartzStone1.to_visual_key(),
            2 => GearSpriteID::QuartzStone2.to_visual_key(),
            3 => GearSpriteID::QuartzStone3.to_visual_key(),
            _ => GearSpriteID::QuartzStone4.to_visual_key(),
        };
    }
}

pub(crate) fn hydrate_quartz_skin(
    mut commands: Commands,
    q_added: Query<Entity, (Added<QuartzStoneData>, Without<QuartzStoneSkin>)>,
) {
    for entity in q_added.iter() {
        commands.entity(entity).insert(QuartzStoneSkin::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, (play_quartz_crack_audio, hydrate_quartz_skin));
    app.add_systems(
        Update,
        update_quartz_skin.run_if(resource_exists::<LocalPlayerRole>),
    );
}
