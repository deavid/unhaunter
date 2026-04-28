use bevy::prelude::*;
use bevy_replicon::prelude::{
    Channel, ClientMessageAppExt, FromClient, SendMode, ServerMessageAppExt, ToClients,
};
use unreplicon_core::resources::AuthorityRole;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::messages::{BroadcastWalkieEvent, ProposeWalkieEvent};
use unwalkie_core::resources::WalkiePlay;

pub(crate) fn app_setup_messages(app: &mut App) {
    app.add_client_message::<ProposeWalkieEvent>(Channel::Ordered);
    app.add_server_message::<BroadcastWalkieEvent>(Channel::Ordered);
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (receive_walkie_proposals, server_broadcast_on_event_set)
            .chain()
            .run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(Update, receive_walkie_broadcast);
}

/// Server system: reads proposals from clients, gates them via the authoritative `WalkiePlay`
/// resource (same cooldown/priority logic). If accepted, the event is set on `WalkiePlay` and
/// `server_broadcast_on_event_set` will broadcast it to all clients on the same frame.
fn receive_walkie_proposals(
    mut reader: MessageReader<FromClient<ProposeWalkieEvent>>,
    mut walkie_play: ResMut<WalkiePlay>,
    time: Res<Time>,
) {
    for msg in reader.read() {
        let event = msg.message.event.clone();
        if walkie_play.set(event.clone(), time.elapsed_secs_f64()) {
            debug!(
                "WALKIE_NET: accepted proposal {:?} from client {:?}",
                event, msg.client_id
            );
        } else {
            warn!(
                "WALKIE_NET: rejected proposal {:?} from client {:?} (likely priority/cooldown)",
                event, msg.client_id
            );
        }
    }
}

/// Server system: watches `WalkiePlay` for newly queued events and broadcasts them
/// to all clients so they can play local audio. Runs after `receive_walkie_proposals`.
fn server_broadcast_on_event_set(
    walkie_play: Res<WalkiePlay>,
    mut ev_broadcast: MessageWriter<ToClients<BroadcastWalkieEvent>>,
    mut last_broadcast: Local<Option<WalkieEvent>>,
) {
    if !walkie_play.is_changed() {
        return;
    }
    let current = walkie_play.event.clone();
    if current != *last_broadcast {
        *last_broadcast = current.clone();
        if let Some(event) = current {
            debug!("WALKIE_NET: broadcasting {:?} to all clients", event);
            ev_broadcast.write(ToClients {
                mode: SendMode::Broadcast,
                message: BroadcastWalkieEvent {
                    event,
                    seed: walkie_play.current_seed,
                },
            });
        }
    }
}

/// Client system: receives broadcast from server and force-queues the event for local audio.
/// Skipped on authority since authority already set the event locally.
fn receive_walkie_broadcast(
    mut reader: MessageReader<BroadcastWalkieEvent>,
    mut walkie_play: ResMut<WalkiePlay>,
    authority: Option<Res<AuthorityRole>>,
    time: Res<Time>,
) {
    if authority.is_some() {
        // Authority already queued the event via set() when it was accepted.
        for _ in reader.read() {}
        return;
    }
    for msg in reader.read() {
        debug!(
            "WALKIE_NET: received broadcast {:?}, force-playing",
            msg.event
        );
        walkie_play.set_forced(msg.event.clone(), time.elapsed_secs_f64(), msg.seed);
    }
}
