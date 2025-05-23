//! This crate defines shared types used by the `walkie_voice_generator` tool
//! and the main `unwalkie` game crate for managing walkie-talkie voice lines.
//! It includes the `WalkieTag` enum for categorizing voice lines and the
//! `VoiceLineData` struct for holding metadata about each generated audio file.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WalkieTag {
    // --- Gameplay & Player State ---
    /// For mechanics the player might be encountering for the first time.
    FirstTimeHint,
    /// A general hint or piece of advice related to gameplay mechanics.
    Guidance,
    /// A gentle nudge if the player is inactive or seems unsure how to proceed.
    GentleProd,
    /// Player seems to be having trouble (e.g., repeated fails, low sanity, bumping into things).
    PlayerStruggling,
    /// Player hasn't made progress or performed key actions for a while.
    StuckOrInactive,
    /// Positive reinforcement or less hand-holding.
    PositiveReinforcement,
    /// A question or prompt to the player, possibly to elicit a response or make them think.
    Questioning,
    /// A more direct hint or suggestion for the player, often when stuck or missing something obvious.
    DirectHint,
    /// A hint or reminder related to a specific tutorial or training session.
    TutorialSpecific,
    /// An informative line.
    Informative,

    // --- Reminder Severity ---
    /// A reminder for a critical mechanic or event that the player might have forgotten.
    UrgentReminder,
    /// Gentle reminder for a commonly known or recently taught mechanic.
    ReminderLow,
    /// Slightly more insistent reminder if a previous low reminder was ignored.
    ReminderMedium,
    /// Urgent reminder, player might be in danger or missing something critical.
    ReminderHigh,
    /// A very gentle, often positive, reminder.
    FriendlyReminder,
    /// A somewhat more formal reminder.
    FormalishReminder,

    // --- Timing & Contextual ---
    /// A direct and immediate reaction to a player action or game event.
    ImmediateResponse,
    /// An observation made after some time has passed or a situation has developed.
    DelayedObservation,
    /// The hint is specifically relevant to the player's current location or environmental context.
    ContextualHint,

    // --- Line Characteristics ---
    /// Line is very short and to the point (e.g., during intense moments).
    ShortBrevity,
    /// Standard length voice line.
    MediumLength,
    /// More detailed explanation or observation.
    LongDetailed,

    // --- Personality & Tone ---
    /// Objective observation of the situation.
    NeutralObservation,
    /// Expressing concern or warning about potential danger.
    ConcernedWarning,
    /// Positive and encouraging tone, aimed at building confidence.
    Encouraging,
    /// Buddy is getting a bit fed up or impatient; use sparingly for repeated errors.
    SlightlyImpatient,
    /// Characteristic snarky/humorous tone; generally for more experienced players or less critical situations.
    SnarkyHumor,
    /// A purely humorous or light-hearted comment, not necessarily a hint.
    Humorous,
    /// A more formal or serious tone, possibly for critical situations.
    // FormalishReminder is already defined above

    // --- Tool Specificity Tags (NEW) ---
    /// Suggests player should *stop* primarily using the Thermometer for now.
    SuggestStopThermometer,
    /// Suggests player should *start* using the Thermometer.
    SuggestUseThermometer,
    /// Suggests player should *stop* primarily using the EMF Meter for now.
    SuggestStopEMFMeter,
    /// Suggests player should *start* using the EMF Meter.
    SuggestUseEMFMeter,
    /// Suggests player should *stop* primarily using the Flashlight (e.g., it's on but not needed).
    SuggestStopFlashlight,
    /// Suggests player should *start* using the Flashlight.
    SuggestUseFlashlight,
    // (We can add more for UVTorch, Videocam, Recorder, Geiger, etc. as needed for later chapters)
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
