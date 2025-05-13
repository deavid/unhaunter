//! This crate defines shared types used by the `walkie_voice_generator` tool
//! and the main `unwalkie` game crate for managing walkie-talkie voice lines.
//! It includes the `WalkieTag` enum for categorizing voice lines and the
//! `VoiceLineData` struct for holding metadata about each generated audio file.

use serde::{Deserialize, Serialize};

/// Enum representing various tags to categorize walkie-talkie voice lines.
/// These tags help in selecting appropriate lines based on game context,
/// player state, desired tone, and other characteristics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WalkieTag {
    // Gameplay & Player State
    /// For mechanics the player might be encountering for the first time.
    FirstTimeHint,
    /// Gentle reminder for a commonly known mechanic.
    ReminderLow,
    /// Slightly more insistent reminder.
    ReminderMedium,
    /// Urgent reminder, player might be in danger or missing something critical.
    ReminderHigh,
    /// Player hasn't made progress or performed key actions for a while.
    StuckOrInactive,
    /// Positive reinforcement or less hand-holding.
    PositiveReinforcement, // Renamed from PlayerPerformingWell to match review's list more closely
    /// Player seems to be having trouble (e.g., repeated fails, low sanity).
    PlayerStruggling,
    /// A more direct hint or suggestion for the player.
    DirectHint,
    /// A question or prompt to the player, possibly to elicit a response.
    Questioning,
    /// A more neutral observation, not necessarily positive or negative.
    ObservationNeutral,

    // Timing & Contextual
    /// A direct and immediate reaction to a player action or game event.
    ImmediateResponse,
    /// An observation made after some time has passed or a situation has developed.
    DelayedObservation,

    // Line Characteristics
    /// Line is very short and to the point (e.g., during intense moments).
    ShortBrevity,
    /// Standard length voice line.
    MediumLength,
    /// More detailed explanation or observation.
    LongDetailed,

    // Personality & Tone
    /// Objective observation of the situation.
    NeutralObservation,
    /// Expressing concern or warning about potential danger.
    ConcernedWarning,
    /// Positive and encouraging tone.
    Encouraging,
    /// Buddy is getting a bit fed up or impatient.
    SlightlyImpatient,
    /// Characteristic snarky/humorous tone.
    SnarkyHumor,
}

/// Holds metadata for a single generated voice line.
/// This structure is used by the `walkie_voice_generator` tool to create a manifest
/// and by the game to load and manage voice lines.
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VoiceLineData {
    /// The path to the generated OGG audio file, relative to the game's asset directory.
    /// Example: "walkie/generated/low_visibility_dark_torch_reminder_01.ogg"
    pub ogg_path: String,
    /// The subtitle text to display in the game UI when this voice line is played.
    pub subtitle_text: String,
    /// A list of `WalkieTag`s associated with this voice line, used for contextual selection.
    pub tags: Vec<WalkieTag>,
    /// The duration of the audio file in seconds.
    pub length_seconds: u32,
}
