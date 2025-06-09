use crate::events::WalkieEvent;
use bevy::{prelude::*};
use std::collections::HashMap;
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
    pub priority_bar: f32,
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
            priority_bar: 0.0,
        }
    }
}

impl WalkiePlay {
    /// Try to set the event to be played. If it's not ready, the system needs to keep retrying.
    pub fn set(&mut self, event: WalkieEvent, time: f64) -> bool {
        if self.priority_bar > event.priority().value() {
            // dbg!(&self.priority_bar, event);
            return false;
        }
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

        if self.priority_bar < event.priority().value() {
            self.priority_bar = self.priority_bar * 0.8 + event.priority().value() * 0.199;
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
            if event.priority().value() > in_event.priority().value() * 50.0
                && event.priority().value() > 5.0
            {
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

    /// Mark evidence as hinted via walkie for potential journal blinking
    pub fn set_evidence_hint(&mut self, evidence: Evidence, time: f64) {
        self.evidence_hinted_not_logged_via_walkie = Some((evidence, time));
    }

    /// Clear evidence hint when it's been acknowledged in journal
    pub fn clear_evidence_hint(&mut self) -> Option<Evidence> {
        if let Some((evidence, _)) = self.evidence_hinted_not_logged_via_walkie.take() {
            Some(evidence)
        } else {
            None
        }
    }

    /// Check if there's a pending evidence hint
    pub fn has_evidence_hint(&self, evidence: Evidence) -> bool {
        self.evidence_hinted_not_logged_via_walkie
            .map(|(e, _)| e == evidence)
            .unwrap_or(false)
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub enum WalkieSoundState {
    Intro,
    Talking,
    Outro,
}
