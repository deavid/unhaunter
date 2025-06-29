use bevy::prelude::Event;
use enum_iterator::Sequence;
use uncore::{
    difficulty::Difficulty,
    types::{evidence::Evidence, gear_kind::GearKind},
};

/// Event that is fired when a walkie-talkie message starts talking (transitions from Intro to Talking state).
/// This allows other systems to react when a specific walkie message starts playing.
#[derive(Event, Debug, Clone)]
pub struct WalkieTalkingEvent {
    /// The walkie event that is currently playing
    pub event: WalkieEvent,
}

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WalkieEventPriority {
    VeryLow,
    Low,
    Medium,
    High,
    VeryHigh,
    Urgent,
}

/// Controls how often a walkie event should repeat across missions
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum WalkieRepeatBehavior {
    /// Very low repeat frequency - almost never repeat (introductions, one-time tips)
    VeryLowRepeat,
    /// Low repeat frequency - rarely repeat (basic tutorials, simple reminders)
    LowRepeat,
    /// Normal repeat frequency - moderate repeat (standard gameplay hints)
    NormalRepeat,
    /// High repeat frequency - can repeat often (important feedback, confirmations)
    HighRepeat,
    /// Always repeat - no cross-mission suppression (critical warnings, urgent messages)
    AlwaysRepeat,
}

/// Sending this event will cause the walkie to play a message.
#[derive(Clone, Debug, PartialEq, Eq, Hash, Sequence)]
pub enum WalkieEvent {
    /// When the player forgets the stuff in the van.
    GearInVan,
    /// When the Ghost rage is near its limit.
    GhostNearHunt,

    // --- Tutorial and Difficulty Intros ---
    /// Chapter intros based on difficulty/chapter number
    ChapterIntro(Difficulty),
    /// Explanations for specific gear items
    GearExplanation(GearKind),

    // --- Locomotion and Interaction Events ---
    /// Player hasn't moved significantly from the start.
    PlayerStuckAtStart,
    /// Player exhibits erratic movement early on.
    ErraticMovementEarly,
    /// Player hesitates to interact with the main door.
    DoorInteractionHesitation,
    /// Player is struggling with picking up or dropping items.
    StrugglingWithGrabDrop,
    /// Player is struggling with hiding or unhiding mechanics.
    StrugglingWithHideUnhide,
    /// Player is near a hiding spot during a hunt but does not hide.
    HuntActiveNearHidingSpotNoHide,

    // --- Environmental Awareness Events ---
    /// Player is in a dark room without using a light source for a specified duration.
    DarkRoomNoLightUsed,
    /// Player is in the same room as a breach, should be shown what a breach looks like.
    BreachShowcase,
    /// Player is in the same room as the ghost, should be shown what the ghost looks like.
    GhostShowcase,
    /// Player uses gear that requires darkness in a lit room.
    RoomLightsOnGearNeedsDark,
    /// Player is using the Thermometer, it's showing cold (1-10°C) but not freezing, and lingers too long.
    ThermometerNonFreezingFixation,
    /// Player selects gear but does not activate it.
    GearSelectedNotActivated,

    // --- Player Wellbeing Events ---
    /// Player's health is low for a prolonged period while inside the location.
    LowHealthGeneralWarning,
    /// Player's sanity is critically low and they haven't returned to the truck.
    VeryLowSanityNoTruckReturn,
    /// Player's sanity dropped due to darkness.
    SanityDroppedBelowThresholdDarkness,
    /// Player's sanity dropped due to ghost proximity.
    SanityDroppedBelowThresholdGhost,

    // --- Consumables and Defense Events ---
    /// Player used quartz, and it cracked during use.
    QuartzCrackedFeedback,
    /// Player used quartz, and it shattered during use.
    QuartzShatteredFeedback,
    /// Player stays hidden for too long after a hunt ends.
    PlayerStaysHiddenTooLong,
    /// Player has not picked up quartz from the truck when a hunt is likely and they have experienced a hunt before.
    QuartzUnusedInRelevantSituation,
    /// Player has not used Sage in a relevant situation.
    SageUnusedInRelevantSituation,
    /// Player activated Sage, but it was used ineffectively.
    SageActivatedIneffectively,
    /// Player did not use Sage defensively during a hunt.
    SageUnusedDefensivelyDuringHunt,

    // --- Repellent and Expulsion Events ---
    GhostExpelledPlayerLingers,
    /// Player enters the location with a repellent.
    HasRepellentEntersLocation,
    /// Player used a repellent too far from the ghost.
    RepellentUsedTooFar,
    /// Player used a repellent, enraging the ghost, causing the player to flee.
    RepellentUsedGhostEnragesPlayerFlees,
    /// Player exhausted a repellent while the ghost was present, and it was the correct type.
    RepellentExhaustedGhostPresentCorrectType,
    /// Player missed the ghost after it was expelled.
    GhostExpelledPlayerMissed,
    /// Player did not switch starting gear in a hotspot.
    DidNotSwitchStartingGearInHotspot,
    /// Player did not cycle to other gear when it was necessary.
    DidNotCycleToOtherGear,
    /// Journal points to one ghost, but no crafting has been done.
    JournalPointsToOneGhostNoCraft,
    /// Player is fixated on EMF readings that are not EMF Level 5.
    EMFNonEMF5Fixation,
    /// Journal contains conflicting evidence entries.
    JournalConflictingEvidence,

    // --- Evidence Confirmation Events ---
    /// Freezing temperatures evidence has been confirmed.
    FreezingTempsEvidenceConfirmed,
    /// Floating orbs evidence has been confirmed.
    FloatingOrbsEvidenceConfirmed,
    /// UV Ectoplasm evidence has been confirmed.
    UVEctoplasmEvidenceConfirmed,
    /// EMF Level 5 evidence has been confirmed.
    EMFLevel5EvidenceConfirmed,
    /// EVP evidence has been confirmed.
    EVPEvidenceConfirmed,
    /// Spirit Box evidence has been confirmed.
    SpiritBoxEvidenceConfirmed,
    /// RL Presence evidence has been confirmed.
    RLPresenceEvidenceConfirmed,
    /// CPM 500 evidence has been confirmed.
    CPM500EvidenceConfirmed,

    // --- Proactive Crafting Prompts ---
    PotentialGhostIDWithNewEvidence,

    // --- Mission Progression and Truck Events ---
    /// Player has found evidence but not logged it via C key.
    ClearEvidenceFoundNoActionCKey,
    /// Player has found evidence but not logged it in the truck.
    ClearEvidenceFoundNoActionTruck,
    /// Player is in the truck with evidence but hasn't updated the journal.
    InTruckWithEvidenceNoJournal,
    /// Player is warned about a hunt but doesn't evade.
    HuntWarningNoPlayerEvasion,
    /// All objectives are met, reminding the player to end the mission.
    AllObjectivesMetReminderToEndMission,
    /// Player leaves the truck without changing their loadout.
    PlayerLeavesTruckWithoutChangingLoadout,
    /// Player is using the wrong repellent, hint to discard a specific evidence.
    IncorrectRepellentHint(Evidence),
}
