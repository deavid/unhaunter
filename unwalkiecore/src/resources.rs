// unwalkiecore/src/resources.rs
use crate::events::WalkieEvent;
use bevy::{prelude::*, utils::HashMap};

#[derive(Clone, Debug, Default)]
pub struct WalkieEventStats {
    pub count: u32,
    pub last_played: f64,
}

#[derive(Clone, Debug, Resource)]
pub struct WalkiePlay {
    pub event: Option<WalkieEvent>,
    pub played_events: HashMap<WalkieEvent, WalkieEventStats>,
    pub state: Option<WalkieSoundState>,
    pub last_message_time: f64,
    pub truck_accessed: bool,
}

impl Default for WalkiePlay {
    fn default() -> Self {
        Self {
            event: Default::default(),
            played_events: Default::default(),
            state: Default::default(),
            // Set to a negative value so the first message can be played immediately
            last_message_time: -100.0,
            truck_accessed: Default::default(),
        }
    }
}

impl WalkiePlay {
    /// Try to set the event to be played. If it's not ready, the system needs to keep retrying.
    pub fn set(&mut self, event: WalkieEvent, time: f64) -> bool {
        if self.event.is_some() {
            return false;
        }
        let mut count = 0;
        if let Some(event_stats) = self.played_events.get(&event) {
            count = event_stats.count;
            let next_time_to_play = event.time_to_play(count);
            if time - event_stats.last_played < next_time_to_play {
                // Wait for the next time to play
                return false;
            }
        }
        if time - self.last_message_time < 10.0 + count as f64 * 20.0 {
            // Wait between messages
            return false;
        }
        count += 1;
        warn!("WalkiePlay: {:?}", event);
        self.event = Some(event.clone());
        self.played_events.insert(
            event,
            WalkieEventStats {
                count,
                last_played: time,
            },
        );
        self.state = None;
        true
    }

    /// Marks the event as played, even tough it wasn't. This is a the condition is already met and this makes no sense to trigger later.
    pub fn mark(&mut self, event: WalkieEvent, time: f64) {
        self.played_events.entry(event).or_default().last_played = time;
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
