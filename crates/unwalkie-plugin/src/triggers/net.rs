use bevy::prelude::*;
use unwalkie_core::events::walkie_types::WalkieEvent;
use unwalkie_core::messages::ProposeWalkieEvent;
use unwalkie_core::resources::WalkiePlay;

/// Presentation-layer walkie set.
///
/// On client: runs local cooldown checks and if passed, sends a `ProposeWalkieEvent` to
/// the server. The server gates it and, if accepted, broadcasts back via `BroadcastWalkieEvent`.
///
/// Returns `true` if a proposal was sent locally without immediately hitting a cooldown limit.
#[allow(dead_code)]
pub(crate) fn walkie_set_or_propose(
    event: WalkieEvent,
    time: f64,
    walkie_play: &mut WalkiePlay,
    ev_propose: &mut MessageWriter<ProposeWalkieEvent>,
) -> bool {
    if walkie_play.set_client_propose(event.clone(), time) {
        ev_propose.write(ProposeWalkieEvent { event });
        true
    } else {
        false
    }
}
