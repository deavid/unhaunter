use crate::events::walkie::WalkieEvent;
use bevy::{prelude::*, utils::HashSet};

#[derive(Clone, Debug, Resource, Default)]
pub struct WalkiePlay {
    pub event: Option<WalkieEvent>,
    pub played_events: HashSet<WalkieEvent>,
    pub state: Option<WalkieSoundState>,
    pub truck_accessed: bool,
}

impl WalkiePlay {
    /// Try to set the event to be played. If it's not ready, the system needs to keep retrying.
    pub fn set(&mut self, event: WalkieEvent) -> bool {
        if self.event.is_some() {
            return false;
        }
        if self.played_events.contains(&event) {
            return false;
        }
        warn!("WalkiePlay: {:?}", event);
        self.event = Some(event.clone());
        self.played_events.insert(event);
        self.state = None;
        true
    }

    /// Marks the event as played, even tough it wasn't. This is a the condition is already met and this makes no sense to trigger later.
    pub fn mark(&mut self, event: WalkieEvent) {
        self.played_events.insert(event);
    }

    /// Reset all the state of the walkie play, so it will play again on a new mission.
    pub fn reset(&mut self) {
        let new_self = Self::default();
        *self = new_self;
    }
}

#[derive(Clone, Debug, Component)]
pub enum WalkieSoundState {
    Intro,
    Talking,
    Outro,
}
