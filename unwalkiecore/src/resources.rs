use crate::events::WalkieEvent;
use bevy::{prelude::*, utils::HashMap};
use rand::Rng;
use uncore::random_seed;
use uncore::types::evidence::Evidence;
use unwalkie_types::VoiceLineData;

#[derive(Clone, Debug, Default)]
pub struct WalkieEventStats {
    pub count: u32,
    pub other_count: u32,
    pub last_played: f64,
}

#[derive(Clone, Debug, Resource)]
pub struct WalkiePlay {
    pub event: Option<WalkieEvent>,
    pub played_events: HashMap<WalkieEvent, WalkieEventStats>,
    pub other_mission_event_count: HashMap<WalkieEvent, u32>,
    pub state: Option<WalkieSoundState>,
    pub current_voice_line: Option<VoiceLineData>,
    pub last_message_time: f64,
    pub truck_accessed: bool,
    pub urgent_pending: bool,
    pub evidence_hinted_not_logged_via_walkie: Option<(Evidence, f64)>,
    pub highlight_craft_button: bool,
}

impl Default for WalkiePlay {
    fn default() -> Self {
        Self {
            event: Default::default(),
            played_events: Default::default(),
            state: Default::default(),
            current_voice_line: Default::default(),
            // Set to a negative value so the first message can be played immediately
            last_message_time: -100.0,
            truck_accessed: Default::default(),
            urgent_pending: Default::default(),
            other_mission_event_count: Default::default(),
            evidence_hinted_not_logged_via_walkie: None,
            highlight_craft_button: false, // Added
        }
    }
}

impl WalkiePlay {
    /// Try to set the event to be played. If it's not ready, the system needs to keep retrying.
    pub fn set(&mut self, event: WalkieEvent, time: f64) -> bool {
        // TODO: Priority should lower by the count of times played, and we need to know what is the highest priority event that is attempting to play.
        // ... to know the highest priority that tries to play we need to compute some kind of running average of the inverse of priority, so it decays over time.
        // ... if an event of lower priority attempts to play, compared to the average priority in the queue, we should ignore it.
        self.urgent_pending = false;
        let mut count = 0;
        if let Some(event_stats) = self.played_events.get(&event) {
            count = event_stats.count + event_stats.other_count;
            let next_time_to_play = event.time_to_play(count);
            if time - event_stats.last_played < next_time_to_play {
                // Wait for the next time to play
                return false;
            }
        }
        let min_delay_mult = event.priority().time_factor() as f64;
        let saved_count = self
            .other_mission_event_count
            .get(&event)
            .copied()
            .unwrap_or_default();

        if time - self.last_message_time
            < (10.0 + count as f64 * 20.0 + saved_count as f64 * 2.0) * min_delay_mult
        {
            // Wait between messages
            return false;
        }
        count += 1;
        let mut rng = random_seed::rng();
        let max_dice_value = saved_count * saved_count.clamp(0, 10);
        let dice = rng.random_range(0..=max_dice_value);
        if dice > 3 {
            // Skip playing this event, played too many times.
            info!(
                "WalkiePlay: skipped: {:?}  play dice: {}/{}",
                event, dice, max_dice_value
            );
            let event_stats = self.played_events.entry(event).or_default();
            event_stats.last_played = time;
            event_stats.other_count += 1;

            return true;
        }
        if let Some(in_event) = &self.event {
            if event.priority().is_urgent() && !in_event.priority().is_urgent() {
                self.urgent_pending = true;
            }
            return false;
        }

        warn!(
            "WalkiePlay: {:?} - play dice: {}/{}",
            event,
            dice,
            saved_count.pow(2)
        );
        self.event = Some(event.clone());
        self.played_events.insert(
            event,
            WalkieEventStats {
                count,
                other_count: 0,
                last_played: time,
            },
        );
        self.state = None;
        // Ensure this is reset:
        self.current_voice_line = None;
        true
    }

    /// Marks the event as played, even tough it wasn't. This is a the condition is already met and this makes no sense to trigger later.
    pub fn mark(&mut self, event: WalkieEvent, time: f64) {
        self.played_events.entry(event).or_default().last_played = time;
    }

    /// Reset all the state of the walkie play, so it will play again on a new mission.
    pub fn reset(&mut self) {
        let omec = self.other_mission_event_count.clone();
        let new_self = Self::default();
        *self = new_self;
        // Ensure current_voice_line is also reset, though Default::default() handles it.
        self.current_voice_line = None;
        // Keep the other mission event count, so it can be used in the next mission.
        self.other_mission_event_count = omec;
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub enum WalkieSoundState {
    Intro,
    Talking,
    Outro,
}
