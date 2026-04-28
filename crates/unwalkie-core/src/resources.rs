use crate::events::walkie_types::WalkieEvent;
use bevy::prelude::*;
use bevy_platform::collections::HashMap;
use rand::prelude::*;
use uncommon_app_core::random_seed;
use uninvestigation_core::evidence::Evidence;
use unwalkie_types::types::VoiceLineData;

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
    pub current_seed: u64,
    pub last_message_time: f64,
    pub last_proposed_time: HashMap<WalkieEvent, f64>,
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
            current_seed: 0,
            last_message_time: -100.0,
            last_proposed_time: Default::default(),
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
        // Get previous mission play count for priority calculation
        let saved_count = self
            .other_mission_event_count
            .get(&event)
            .copied()
            .unwrap_or_default();

        // Calculate effective priority based on previous mission play count
        let effective_priority = event.effective_priority(saved_count);

        if self.priority_bar > effective_priority.value() {
            debug!(
                "WalkiePlay: rejected {:?}: priority_bar ({}) > event priority ({})",
                event,
                self.priority_bar,
                effective_priority.value()
            );
            return false;
        }
        self.urgent_pending = false;
        let mut count = 0;
        if let Some(event_stats) = self.played_events.get(&event) {
            count = event_stats.count + event_stats.other_count;
            let next_time_to_play = event.time_to_play(count);
            if time - event_stats.last_played < next_time_to_play {
                debug!(
                    "WalkiePlay: rejected {:?}: too soon since last play (elapsed: {}, need: {})",
                    event,
                    time - event_stats.last_played,
                    next_time_to_play
                );
                return false;
            }
        }
        let min_delay_mult = effective_priority.time_factor() as f64;
        let repeat_behavior = event.repeat_behavior();
        let timing_mult = repeat_behavior.timing_multiplier();

        let inter_message_limit =
            (20.0 + count as f64 * 30.0 + saved_count as f64 * 10.0) * min_delay_mult * timing_mult;

        if time - self.last_message_time < inter_message_limit {
            debug!(
                "WalkiePlay: rejected {:?}: inter-message delay (elapsed: {}, need: {})",
                event,
                time - self.last_message_time,
                inter_message_limit
            );
            return false;
        }

        if self.priority_bar < effective_priority.value() {
            self.priority_bar = self.priority_bar * 0.8 + effective_priority.value() * 0.199;
        }

        count += 1;
        let mut rng = random_seed::rng();
        let max_dice_value = saved_count * saved_count.clamp(0, 4);
        let dice_threshold = repeat_behavior.dice_threshold();
        let dice = rng.random_range(0..=max_dice_value);
        if dice > dice_threshold {
            // Skip playing this event, played too many times.
            debug!(
                "WalkiePlay: skipped: {:?}  play dice: {}/{} (threshold: {})",
                event, dice, max_dice_value, dice_threshold
            );
            let event_stats = self.played_events.entry(event).or_default();
            event_stats.last_played = time;
            event_stats.other_count += 1;

            return true;
        }
        if let Some(in_event) = &self.event {
            // Calculate effective priority for the current in-progress event for comparison
            let in_event_saved_count = self
                .other_mission_event_count
                .get(in_event)
                .copied()
                .unwrap_or_default();
            let in_event_effective_priority = in_event.effective_priority(in_event_saved_count);

            if effective_priority.value() > in_event_effective_priority.value() * 50.0
                && effective_priority.value() > 5.0
            {
                self.urgent_pending = true;
            }
            debug!(
                "WalkiePlay: rejected {:?}: already playing {:?}",
                event, in_event
            );
            return false;
        }

        debug!(
            "WalkiePlay: {:?} - play dice: {}/{} (threshold: {})",
            event, dice, max_dice_value, dice_threshold
        );
        info!("WALKIE_PLAY: queuing event {:?}", event);
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
        self.current_seed = random_seed::heavy_rng_seed();
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

    /// Progress the walkie state machine.
    /// Returns true if the state changed.
    pub fn tick_state(&mut self) -> bool {
        let mut rng = random_seed::rng_from_seed(self.current_seed);

        let Some(walkie_event) = self.event.clone() else {
            return false;
        };

        let new_state = match &self.state {
            None => Some(WalkieSoundState::Intro),
            Some(WalkieSoundState::Intro) => {
                let voice_lines = walkie_event.sound_file_list();
                if let Some(chosen_line) = voice_lines.choose(&mut rng).cloned() {
                    self.current_voice_line = Some(chosen_line);
                } else {
                    self.current_voice_line = Some(VoiceLineData {
                        ogg_path: "sounds/radio-on-zzt.ogg".to_string(),
                        subtitle_text: "[NO SUBTITLE AVAILABLE]".to_string(),
                        tags: vec![],
                        length_seconds: 2,
                    });
                }
                Some(WalkieSoundState::Talking)
            }
            Some(WalkieSoundState::Talking) => Some(WalkieSoundState::Outro),
            Some(WalkieSoundState::Outro) => Some(WalkieSoundState::Outro),
        };

        if new_state != self.state {
            self.state = new_state;
            true
        } else {
            false
        }
    }

    /// For client use: runs the same cooldown checks as `set()` but does **not** queue the
    /// event for local audio. Returns `true` if the checks passed and a `ProposeWalkieEvent`
    /// should be sent to the server. Updates `played_events.last_played` to prevent proposal spam.
    pub fn set_client_propose(&mut self, event: WalkieEvent, time: f64) -> bool {
        // Don't propose while the walkie is currently playing something.
        if self.event.is_some() {
            return false;
        }

        // Prevent proposal spam while waiting for the server to logic it
        if self
            .last_proposed_time
            .get(&event)
            .is_some_and(|last_prop| time - last_prop < 5.0)
        {
            return false;
        }

        let saved_count = self
            .other_mission_event_count
            .get(&event)
            .copied()
            .unwrap_or_default();
        let effective_priority = event.effective_priority(saved_count);

        if self.priority_bar > effective_priority.value() {
            return false;
        }
        let mut count = 0;
        if let Some(event_stats) = self.played_events.get(&event) {
            count = event_stats.count + event_stats.other_count;
            let next_time_to_play = event.time_to_play(count);
            if time - event_stats.last_played < next_time_to_play {
                return false;
            }
        }
        let min_delay_mult = effective_priority.time_factor() as f64;
        let timing_mult = event.repeat_behavior().timing_multiplier();
        if time - self.last_message_time
            < (20.0 + count as f64 * 30.0 + saved_count as f64 * 10.0)
                * min_delay_mult
                * timing_mult
        {
            return false;
        }
        if self.priority_bar < effective_priority.value() {
            self.priority_bar = self.priority_bar * 0.8 + effective_priority.value() * 0.199;
        }
        // Update last_proposed_time to prevent proposal spam while waiting for the server to respond.
        self.last_proposed_time.insert(event, time);
        true
    }

    /// Force-queue an event for local audio playback (called when the server broadcasts a
    /// `BroadcastWalkieEvent`). Bypasses all cooldown checks.
    pub fn set_forced(&mut self, event: WalkieEvent, time: f64, seed: u64) {
        let count = self
            .played_events
            .get(&event)
            .map(|s| s.count + s.other_count + 1)
            .unwrap_or(1);
        self.played_events.insert(
            event.clone(),
            WalkieEventStats {
                count,
                other_count: 0,
                last_played: time,
            },
        );
        info!("WALKIE_PLAY: force-queuing event {:?}", event);
        self.event = Some(event);
        self.state = None;
        self.current_voice_line = None;
        self.current_seed = seed;
        // last_message_time is updated when playback ends in walkie_play.rs
    }
}

#[derive(Clone, Debug, Component, PartialEq, Eq)]
pub enum WalkieSoundState {
    Intro,
    Talking,
    Outro,
}
