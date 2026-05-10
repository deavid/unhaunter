use bevy::prelude::*;
use bevy_replicon::prelude::{SendMode, ToClients};
use unaudiospatial_core::emitter::LocalAudioEmitter;
use unbehavior_core::behavior::{Behavior, Interactive};
use uncommon_states_core::UIContextState;
use unplayer_core::components::Hiding;
use unplayer_core::components::PlayerSprite;
use unreplicon_core::resources::{AuthorityRole, LocalPlayerRole};
use unspatial_core::position::Position;
use untruck_core::components::in_truck::InTruck;
use untruck_core::events::truck::TruckAudioMessage;

const MAX_VAN_ENTRY_DISTANCE: f32 = 1.5;

fn on_intruck_added(trigger: On<Add, InTruck>, mut commands: Commands) {
    commands
        .entity(trigger.entity)
        .insert(Hiding { hiding_spot: None });
}

fn emit_truck_audio(
    audio: TruckAudioMessage,
    local_audio: &mut LocalAudioEmitter,
    ev_audio: &mut MessageWriter<ToClients<TruckAudioMessage>>,
    has_local_player: bool,
) {
    if has_local_player {
        local_audio.play_audio(audio.sound_file.clone(), audio.volume, &audio.position);
    }

    ev_audio.write(ToClients {
        mode: SendMode::Broadcast,
        message: audio,
    });
}

fn on_intruck_added_authoritative_audio(
    trigger: On<Add, InTruck>,
    q_player_pos: Query<&Position, With<PlayerSprite>>,
    q_van_entries: Query<(&Position, &Interactive, &Behavior), Without<PlayerSprite>>,
    authority: Option<Res<AuthorityRole>>,
    local_player_role: Option<Res<LocalPlayerRole>>,
    mut local_audio: LocalAudioEmitter,
    mut ev_audio: MessageWriter<ToClients<TruckAudioMessage>>,
) {
    if authority.is_none() {
        return;
    }

    let Ok(player_pos) = q_player_pos.get(trigger.entity) else {
        warn!(
            "on_intruck_added_authoritative_audio: entity {:?} gained InTruck without a PlayerSprite/Position",
            trigger.entity
        );
        return;
    };

    let mut nearest_audio = None;
    let mut nearest_distance = MAX_VAN_ENTRY_DISTANCE;
    for (item_pos, interactive, behavior) in q_van_entries.iter() {
        if !behavior.is_van_entry() {
            continue;
        }

        let cp_delta = interactive.control_point_delta(behavior);
        let sound_pos = Position {
            x: item_pos.x + cp_delta.x,
            y: item_pos.y + cp_delta.y,
            z: item_pos.z + cp_delta.z,
            visual_priority: item_pos.visual_priority,
        };
        let distance = player_pos.distance_zf(&sound_pos, 6.0);
        if distance >= nearest_distance {
            continue;
        }

        nearest_distance = distance;
        nearest_audio = Some(TruckAudioMessage {
            sound_file: interactive.sound_for_moving_into_state(behavior),
            volume: 1.0,
            position: sound_pos,
        });
    }

    let Some(audio) = nearest_audio else {
        warn!(
            "on_intruck_added_authoritative_audio: no van-entry interactive found within {:.1} units for player {:?}",
            MAX_VAN_ENTRY_DISTANCE, trigger.entity
        );
        return;
    };

    emit_truck_audio(
        audio,
        &mut local_audio,
        &mut ev_audio,
        local_player_role.is_some(),
    );
}

fn on_intruck_removed(trigger: On<Remove, InTruck>, mut commands: Commands) {
    commands.entity(trigger.entity).remove::<Hiding>();
}

fn receive_truck_audio_broadcast(
    mut reader: MessageReader<TruckAudioMessage>,
    mut audio: LocalAudioEmitter,
    authority: Option<Res<AuthorityRole>>,
) {
    if authority.is_some() {
        for _ in reader.read() {}
        return;
    }

    for msg in reader.read() {
        audio.play_audio(msg.sound_file.clone(), msg.volume, &msg.position);
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_observer(on_intruck_added);
    app.add_observer(on_intruck_added_authoritative_audio);
    app.add_observer(on_intruck_removed);
    app.add_systems(
        Update,
        receive_truck_audio_broadcast.run_if(in_state(UIContextState::InGame)),
    );
}
