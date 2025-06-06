use crate::ConceptTrait;
use crate::generated::base1::Base1Concept;
use crate::generated::basic_gear_usage::BasicGearUsageConcept;
use crate::generated::consumables_and_defense::ConsumablesAndDefenseConcept;
use crate::generated::environmental_awareness::EnvironmentalAwarenessConcept;
use crate::generated::evidence_gathering_and_logic::EvidenceGatheringAndLogicConcept;
use crate::generated::ghost_behavior_and_hunting::GhostBehaviorAndHuntingConcept;
use crate::generated::locomotion_and_interaction::LocomotionAndInteractionConcept;
use crate::generated::player_wellbeing::PlayerWellbeingConcept;
use crate::generated::repellent_and_expulsion::RepellentAndExpulsionConcept;
use crate::generated::tutorial_chapter_intros::TutorialChapterIntrosConcept;
use crate::generated::tutorial_gear_explanations::TutorialGearExplanationsConcept;
use bevy::log::warn;
use bevy::prelude::Event;
use enum_iterator::Sequence;
use uncore::difficulty::Difficulty;
use uncore::types::gear_kind::GearKind;
use unwalkie_types::VoiceLineData;

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

impl WalkieEventPriority {
    pub fn value(&self) -> f32 {
        match self {
            WalkieEventPriority::VeryLow => 0.01,
            WalkieEventPriority::Low => 0.1,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 10.0,
            WalkieEventPriority::VeryHigh => 100.0,
            WalkieEventPriority::Urgent => 1000.0,
        }
    }
    pub fn time_factor(&self) -> f32 {
        match self {
            WalkieEventPriority::VeryLow => 1.4,
            WalkieEventPriority::Low => 1.2,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 0.5,
            WalkieEventPriority::VeryHigh => 0.2,
            WalkieEventPriority::Urgent => 0.05,
        }
    }
    pub fn is_urgent(&self) -> bool {
        matches!(self, WalkieEventPriority::Urgent)
    }
}

struct NullVoice;

impl ConceptTrait for NullVoice {
    fn get_lines(&self) -> Vec<VoiceLineData> {
        vec![]
    }
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
    /// Player hasn\'t moved significantly from the start.
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
    /// Player is using the Thermometer, it\'s showing cold (1-10°C) but not freezing, and lingers too long.
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
}

impl WalkieEvent {
    fn to_concept(&self) -> Box<dyn ConceptTrait> {
        match self {
            WalkieEvent::GearInVan => Box::new(Base1Concept::GearInVan),
            WalkieEvent::GhostNearHunt => Box::new(Base1Concept::GhostNearHunt),
            WalkieEvent::ChapterIntro(difficulty) => match difficulty {
                Difficulty::TutorialChapter1 => {
                    Box::new(TutorialChapterIntrosConcept::TutorialChapter1Intro)
                }
                Difficulty::TutorialChapter2 => {
                    Box::new(TutorialChapterIntrosConcept::TutorialChapter2Intro)
                }
                Difficulty::TutorialChapter3 => {
                    Box::new(TutorialChapterIntrosConcept::TutorialChapter3Intro)
                }
                Difficulty::TutorialChapter4 => {
                    Box::new(TutorialChapterIntrosConcept::TutorialChapter4Intro)
                }
                Difficulty::TutorialChapter5 => {
                    Box::new(TutorialChapterIntrosConcept::TutorialChapter5Intro)
                }
                Difficulty::StandardChallenge => {
                    Box::new(TutorialChapterIntrosConcept::StandardChallengeIntro)
                }
                Difficulty::HardChallenge => {
                    Box::new(TutorialChapterIntrosConcept::HardChallengeIntro)
                }
                Difficulty::ExpertChallenge => {
                    Box::new(TutorialChapterIntrosConcept::ExpertChallengeIntro)
                }
                Difficulty::MasterChallenge => {
                    Box::new(TutorialChapterIntrosConcept::MasterChallengeIntro)
                }
            },
            WalkieEvent::GearExplanation(gear_kind) => match gear_kind {
                GearKind::Flashlight => {
                    Box::new(TutorialGearExplanationsConcept::FlashlightEnabledIntro)
                }
                GearKind::Thermometer => {
                    Box::new(TutorialGearExplanationsConcept::ThermometerEnabledIntro)
                }
                GearKind::EMFMeter => {
                    Box::new(TutorialGearExplanationsConcept::EMFMeterEnabledIntro)
                }
                GearKind::UVTorch => Box::new(TutorialGearExplanationsConcept::UVTorchEnabledIntro),
                GearKind::Videocam => {
                    Box::new(TutorialGearExplanationsConcept::VideocamEnabledIntro)
                }
                GearKind::Recorder => {
                    Box::new(TutorialGearExplanationsConcept::RecorderEnabledIntro)
                }
                GearKind::GeigerCounter => {
                    Box::new(TutorialGearExplanationsConcept::GeigerCounterEnabledIntro)
                }
                GearKind::SpiritBox => {
                    Box::new(TutorialGearExplanationsConcept::SpiritBoxEnabledIntro)
                }
                GearKind::RedTorch => {
                    Box::new(TutorialGearExplanationsConcept::RedTorchEnabledIntro)
                }
                GearKind::Salt => Box::new(TutorialGearExplanationsConcept::SaltSelectedIntro),
                GearKind::QuartzStone => {
                    Box::new(TutorialGearExplanationsConcept::QuartzStoneSelectedIntro)
                }
                GearKind::SageBundle => {
                    Box::new(TutorialGearExplanationsConcept::SageBundleSelectedIntro)
                }

                _ => {
                    warn!("No gear explanation concept for gear kind: {:?}", gear_kind);
                    Box::new(NullVoice)
                }
            },
            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => {
                Box::new(LocomotionAndInteractionConcept::PlayerStuckAtStart)
            }
            WalkieEvent::ErraticMovementEarly => {
                Box::new(LocomotionAndInteractionConcept::ErraticMovementEarly)
            }
            WalkieEvent::DoorInteractionHesitation => {
                Box::new(LocomotionAndInteractionConcept::DoorInteractionHesitation)
            }
            WalkieEvent::StrugglingWithGrabDrop => {
                Box::new(LocomotionAndInteractionConcept::StrugglingWithGrabDrop)
            }
            WalkieEvent::StrugglingWithHideUnhide => {
                Box::new(LocomotionAndInteractionConcept::StrugglingWithHideUnhide)
            }
            WalkieEvent::HuntActiveNearHidingSpotNoHide => {
                Box::new(GhostBehaviorAndHuntingConcept::HuntActiveNearHidingSpotNoHide)
            }
            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => {
                Box::new(EnvironmentalAwarenessConcept::DarkRoomNoLightUsed)
            }
            WalkieEvent::BreachShowcase => {
                Box::new(EnvironmentalAwarenessConcept::IgnoredObviousBreach)
            }
            WalkieEvent::GhostShowcase => {
                Box::new(EnvironmentalAwarenessConcept::IgnoredVisibleGhost)
            }
            WalkieEvent::RoomLightsOnGearNeedsDark => {
                Box::new(EnvironmentalAwarenessConcept::RoomLightsOnGearNeedsDark)
            }
            WalkieEvent::ThermometerNonFreezingFixation => {
                Box::new(BasicGearUsageConcept::ThermometerNonFreezingFixation)
            }
            WalkieEvent::GearSelectedNotActivated => {
                Box::new(BasicGearUsageConcept::GearSelectedNotActivated)
            }
            WalkieEvent::DidNotSwitchStartingGearInHotspot => {
                Box::new(BasicGearUsageConcept::DidNotSwitchStartingGearInHotspot)
            }
            WalkieEvent::DidNotCycleToOtherGear => {
                Box::new(BasicGearUsageConcept::DidNotCycleToOtherGear)
            }
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => {
                Box::new(PlayerWellbeingConcept::LowHealthGeneralWarning)
            }
            WalkieEvent::VeryLowSanityNoTruckReturn => {
                Box::new(PlayerWellbeingConcept::VeryLowSanityNoTruckReturn)
            }
            WalkieEvent::SanityDroppedBelowThresholdDarkness => {
                Box::new(PlayerWellbeingConcept::SanityDroppedBelowThresholdDarkness)
            }
            WalkieEvent::SanityDroppedBelowThresholdGhost => {
                Box::new(PlayerWellbeingConcept::SanityDroppedBelowThresholdGhost)
            }
            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => {
                Box::new(ConsumablesAndDefenseConcept::QuartzCrackedFeedback)
            }
            WalkieEvent::QuartzShatteredFeedback => {
                Box::new(ConsumablesAndDefenseConcept::QuartzShatteredFeedback)
            }
            WalkieEvent::QuartzUnusedInRelevantSituation => {
                Box::new(ConsumablesAndDefenseConcept::QuartzUnusedInRelevantSituation)
            }
            WalkieEvent::SageUnusedInRelevantSituation => {
                Box::new(ConsumablesAndDefenseConcept::SageUnusedInRelevantSituation)
            }
            WalkieEvent::SageActivatedIneffectively => {
                Box::new(ConsumablesAndDefenseConcept::SageActivatedIneffectively)
            }
            WalkieEvent::SageUnusedDefensivelyDuringHunt => {
                Box::new(ConsumablesAndDefenseConcept::SageUnusedDefensivelyDuringHunt)
            }
            // --- Ghost Behavior and Hunting ---
            WalkieEvent::PlayerStaysHiddenTooLong => {
                Box::new(GhostBehaviorAndHuntingConcept::PlayerStaysHiddenTooLong)
            }
            // --- Repellent and Expulsion ---
            WalkieEvent::GhostExpelledPlayerLingers => {
                Box::new(RepellentAndExpulsionConcept::GhostExpelledPlayerLingers)
            }
            WalkieEvent::HasRepellentEntersLocation => {
                Box::new(RepellentAndExpulsionConcept::HasRepellentEntersLocation)
            }
            WalkieEvent::RepellentUsedTooFar => {
                Box::new(RepellentAndExpulsionConcept::RepellentUsedTooFar)
            }
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => {
                Box::new(RepellentAndExpulsionConcept::RepellentUsedGhostEnragesPlayerFlees)
            }
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => {
                Box::new(RepellentAndExpulsionConcept::RepellentExhaustedGhostPresentCorrectType)
            }
            WalkieEvent::GhostExpelledPlayerMissed => {
                Box::new(RepellentAndExpulsionConcept::GhostExpelledPlayerMissed)
            }
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => {
                Box::new(EvidenceGatheringAndLogicConcept::JournalPointsToOneGhostNoCraft)
            }
            WalkieEvent::EMFNonEMF5Fixation => Box::new(BasicGearUsageConcept::EMFNonEMF5Fixation),
            WalkieEvent::JournalConflictingEvidence => {
                Box::new(EvidenceGatheringAndLogicConcept::JournalConflictingEvidence)
            }
            // --- Evidence Confirmation Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::FreezingTempsEvidenceConfirmed)
            }
            WalkieEvent::FloatingOrbsEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::FloatingOrbsEvidenceConfirmed)
            }
            WalkieEvent::UVEctoplasmEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::UVEctoplasmEvidenceConfirmed)
            }
            WalkieEvent::EMFLevel5EvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::EMFLevel5EvidenceConfirmed)
            }
            WalkieEvent::EVPEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::EVPEvidenceConfirmed)
            }
            WalkieEvent::SpiritBoxEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::SpiritBoxEvidenceConfirmed)
            }
            WalkieEvent::RLPresenceEvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::RLPresenceEvidenceConfirmed)
            }
            WalkieEvent::CPM500EvidenceConfirmed => {
                Box::new(EvidenceGatheringAndLogicConcept::CPM500EvidenceConfirmed)
            }
            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => {
                Box::new(EvidenceGatheringAndLogicConcept::PotentialGhostIDWithNewEvidencePrompt)
            }
            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => {
                Box::new(EvidenceGatheringAndLogicConcept::ClearEvidenceFoundNoActionCKey)
            }
            WalkieEvent::ClearEvidenceFoundNoActionTruck => {
                Box::new(EvidenceGatheringAndLogicConcept::ClearEvidenceFoundNoActionTruck)
            }
            WalkieEvent::InTruckWithEvidenceNoJournal => {
                Box::new(EvidenceGatheringAndLogicConcept::InTruckWithEvidenceNoJournal)
            }
            WalkieEvent::HuntWarningNoPlayerEvasion => {
                Box::new(GhostBehaviorAndHuntingConcept::HuntWarningNoPlayerEvasion)
            }
            WalkieEvent::AllObjectivesMetReminderToEndMission => {
                Box::new(Base1Concept::AllObjectivesMetReminderToEndMission)
            }
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => {
                Box::new(Base1Concept::PlayerLeavesTruckWithoutChangingLoadout)
            }
        }
    }

    pub fn time_to_play(&self, count: u32) -> f64 {
        let count = count.max(1) as f64;
        match self {
            WalkieEvent::GearInVan => 120.0 * count,
            WalkieEvent::GhostNearHunt => 120.0 * count.cbrt(),
            // WalkieEvent::MissionStartEasy => 3600.0 * 24.0 * 7.0, // Removed
            WalkieEvent::ChapterIntro(_) => 3600.0 * 24.0 * 365.0, // Effectively once (very long cooldown)
            WalkieEvent::GearExplanation(_) => 3600.0 * 24.0 * 365.0, // Effectively once (very long cooldown)

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => 180.0 * count,
            WalkieEvent::ErraticMovementEarly => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::DoorInteractionHesitation => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::StrugglingWithGrabDrop => 180.0 * count,
            WalkieEvent::StrugglingWithHideUnhide => 180.0 * count,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => 30.0 * count,

            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => 180.0 * count,
            WalkieEvent::BreachShowcase => 9000.0 * count,
            WalkieEvent::GhostShowcase => 9000.0 * count,
            WalkieEvent::RoomLightsOnGearNeedsDark => 90.0 * count,
            WalkieEvent::ThermometerNonFreezingFixation => 120.0 * count,
            WalkieEvent::GearSelectedNotActivated => 300.0 * count,

            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => 120.0 * count,
            WalkieEvent::VeryLowSanityNoTruckReturn => 120.0 * count,
            WalkieEvent::SanityDroppedBelowThresholdDarkness => 120.0 * count,
            WalkieEvent::SanityDroppedBelowThresholdGhost => 120.0 * count,

            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => 60.0 * count,
            WalkieEvent::QuartzShatteredFeedback => 60.0 * count,
            WalkieEvent::QuartzUnusedInRelevantSituation => 180.0 * count, // Every 3 minutes if conditions met
            WalkieEvent::SageUnusedInRelevantSituation => 180.0 * count, // Every 3 minutes if conditions met
            WalkieEvent::SageActivatedIneffectively => 180.0 * count, // Trigger every 3 minutes if conditions met
            WalkieEvent::SageUnusedDefensivelyDuringHunt => 180.0 * count, // Trigger every 3 minutes if conditions met

            // --- Ghost Behavior and Hunting ---
            WalkieEvent::PlayerStaysHiddenTooLong => 90.0 * count,

            // --- Repellent and Expulsion ---
            WalkieEvent::GhostExpelledPlayerLingers => 120.0 * count,
            WalkieEvent::HasRepellentEntersLocation => 300.0 * count,
            WalkieEvent::RepellentUsedTooFar => 60.0 * count,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => 90.0 * count,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => 90.0 * count,
            WalkieEvent::GhostExpelledPlayerMissed => 180.0 * count,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => 180.0 * count,
            WalkieEvent::DidNotCycleToOtherGear => 180.0 * count,
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => 300.0 * count, // Trigger every 5 minutes if conditions met
            WalkieEvent::EMFNonEMF5Fixation => 120.0 * count, // Trigger every 2 minutes if conditions met
            WalkieEvent::JournalConflictingEvidence => 300.0 * count, // Trigger every 5 minutes if conditions met

            // --- Evidence Confirmation Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => 180.0 * count,
            WalkieEvent::FloatingOrbsEvidenceConfirmed => 180.0 * count,
            WalkieEvent::UVEctoplasmEvidenceConfirmed => 180.0 * count,
            WalkieEvent::EMFLevel5EvidenceConfirmed => 180.0 * count,
            WalkieEvent::EVPEvidenceConfirmed => 180.0 * count,
            WalkieEvent::SpiritBoxEvidenceConfirmed => 180.0 * count,
            WalkieEvent::RLPresenceEvidenceConfirmed => 180.0 * count,
            WalkieEvent::CPM500EvidenceConfirmed => 180.0 * count,

            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => 180.0 * count,

            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => 120.0 * count,
            WalkieEvent::ClearEvidenceFoundNoActionTruck => 120.0 * count,
            WalkieEvent::InTruckWithEvidenceNoJournal => 120.0 * count,
            WalkieEvent::HuntWarningNoPlayerEvasion => 120.0 * count,
            WalkieEvent::AllObjectivesMetReminderToEndMission => 180.0 * count,
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => 120.0 * count,
        }
    }

    /// Get the list of voice line data for the event.
    pub fn sound_file_list(&self) -> Vec<VoiceLineData> {
        self.to_concept().get_lines()
    }

    pub fn priority(&self) -> WalkieEventPriority {
        match self {
            WalkieEvent::GearInVan => WalkieEventPriority::Low,
            WalkieEvent::GhostNearHunt => WalkieEventPriority::VeryLow,
            WalkieEvent::ChapterIntro(_) => WalkieEventPriority::Low,
            WalkieEvent::GearExplanation(_) => WalkieEventPriority::VeryLow,

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => WalkieEventPriority::Medium,
            WalkieEvent::ErraticMovementEarly => WalkieEventPriority::Urgent,
            WalkieEvent::DoorInteractionHesitation => WalkieEventPriority::High,
            WalkieEvent::StrugglingWithGrabDrop => WalkieEventPriority::Medium,
            WalkieEvent::StrugglingWithHideUnhide => WalkieEventPriority::Medium,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => WalkieEventPriority::High,
            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => WalkieEventPriority::Medium,
            WalkieEvent::BreachShowcase => WalkieEventPriority::VeryLow,
            WalkieEvent::GhostShowcase => WalkieEventPriority::VeryLow,
            WalkieEvent::RoomLightsOnGearNeedsDark => WalkieEventPriority::Low,
            WalkieEvent::ThermometerNonFreezingFixation => WalkieEventPriority::Medium,
            WalkieEvent::GearSelectedNotActivated => WalkieEventPriority::Medium,
            WalkieEvent::EMFNonEMF5Fixation => WalkieEventPriority::Low,
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => WalkieEventPriority::Medium,
            WalkieEvent::VeryLowSanityNoTruckReturn => WalkieEventPriority::High,
            WalkieEvent::SanityDroppedBelowThresholdDarkness => WalkieEventPriority::Medium,
            WalkieEvent::SanityDroppedBelowThresholdGhost => WalkieEventPriority::High,
            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => WalkieEventPriority::Medium,
            WalkieEvent::QuartzShatteredFeedback => WalkieEventPriority::High,
            WalkieEvent::QuartzUnusedInRelevantSituation => WalkieEventPriority::Medium,
            WalkieEvent::SageUnusedInRelevantSituation => WalkieEventPriority::Medium,
            WalkieEvent::SageActivatedIneffectively => WalkieEventPriority::Low,
            WalkieEvent::SageUnusedDefensivelyDuringHunt => WalkieEventPriority::Medium,
            // --- Ghost Behavior and Hunting ---
            WalkieEvent::PlayerStaysHiddenTooLong => WalkieEventPriority::Low,
            // --- Repellent and Expulsion ---
            WalkieEvent::GhostExpelledPlayerLingers => WalkieEventPriority::High,
            WalkieEvent::HasRepellentEntersLocation => WalkieEventPriority::Medium,
            WalkieEvent::RepellentUsedTooFar => WalkieEventPriority::High,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => WalkieEventPriority::High,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => WalkieEventPriority::Medium,
            WalkieEvent::GhostExpelledPlayerMissed => WalkieEventPriority::Medium,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => WalkieEventPriority::High,
            WalkieEvent::DidNotCycleToOtherGear => WalkieEventPriority::Medium,
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => WalkieEventPriority::Low,
            WalkieEvent::JournalConflictingEvidence => WalkieEventPriority::Medium,

            // --- Evidence Confirmation Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::FloatingOrbsEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::UVEctoplasmEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::EMFLevel5EvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::EVPEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::SpiritBoxEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::RLPresenceEvidenceConfirmed => WalkieEventPriority::VeryHigh,
            WalkieEvent::CPM500EvidenceConfirmed => WalkieEventPriority::VeryHigh,

            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => WalkieEventPriority::High,

            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => WalkieEventPriority::VeryLow,
            WalkieEvent::ClearEvidenceFoundNoActionTruck => WalkieEventPriority::VeryLow,
            WalkieEvent::InTruckWithEvidenceNoJournal => WalkieEventPriority::Medium,
            WalkieEvent::HuntWarningNoPlayerEvasion => WalkieEventPriority::Urgent,
            WalkieEvent::AllObjectivesMetReminderToEndMission => WalkieEventPriority::High,
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => WalkieEventPriority::High,
        }
    }
    /// This function returns hint text to display to the player for various events.
    /// These hints are displayed alongside the walkie-talkie voice to help guide the player
    /// on what controls or actions they should take.
    pub fn get_on_screen_actionable_hint_text(&self) -> &'static str {
        match &self {
            // --- Base1 ---
            WalkieEvent::GearInVan => "Return to van; check Loadout tab for gear.",
            WalkieEvent::GhostNearHunt => {
                "Ghost is about to hunt! Find a hiding spot (Hold [E] to hide) or create distance!"
            }
            WalkieEvent::ChapterIntro(difficulty) => match difficulty {
                Difficulty::TutorialChapter1 => {
                    "Approach the building to start investigating. Gear limited to: Flashlight, EMF & Thermometer."
                }
                Difficulty::TutorialChapter2 => {
                    "Enter the location when ready. Gear has been extended with UV Torch & Video Cam."
                }
                Difficulty::TutorialChapter3 => {
                    "Find the ghost and the breach. Recorder & Geiger Counter have been added to the truck."
                }
                Difficulty::TutorialChapter4 => {
                    "Carefully explore the location. All main gear is now available: Spirit Box & Red Torch added."
                }
                Difficulty::TutorialChapter5 => {
                    "Identify the ghost. Defensive items: Salt, Quartz & Sage are now available."
                }
                Difficulty::StandardChallenge => "Standard contract. Identify the entity.",
                Difficulty::HardChallenge => "This will be a challenge. Expect aggression.",
                Difficulty::ExpertChallenge => "Expert level: a highly dangerous entity awaits.",
                Difficulty::MasterChallenge => {
                    "Master challenge: face the ultimate paranormal threat."
                }
            },
            WalkieEvent::GearExplanation(gear_kind) => match gear_kind {
                GearKind::Flashlight => "Flashlight: [TAB] to toggle modes. High-beam overheats.",
                GearKind::Thermometer => "Thermometer: Detects Freezing Temps (<0°C).",
                GearKind::EMFMeter => "EMF Meter: Reads energy. \"EMF5\" is key evidence.",
                GearKind::UVTorch => "UV Torch: Makes some ghosts turn green.",
                GearKind::Videocam => {
                    "Video Cam: Look using the night vision to see if there are floating orbs."
                }
                GearKind::Recorder => "Digital Recorder: Record for EVPs (ghost voices).",
                GearKind::GeigerCounter => "Geiger Counter: Finds High Radiation (>500cpm).",
                GearKind::SpiritBox => "Spirit Box: Some ghosts talk throught it.",
                GearKind::RedTorch => "Red Torch: Some ghosts glow orange under it.",
                GearKind::Salt => "Salt: Place to track the ghost (UV might be needed).",
                GearKind::QuartzStone => "Quartz Stone: Hold for protection. Cracks/shatters.",
                GearKind::SageBundle => "Sage Bundle: Light to cleanse area, deters ghost.",
                _ => "This gear has specific uses. Check Journal for details.",
            },

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => "Use [WASD] or Arrow Keys to move.",
            WalkieEvent::ErraticMovementEarly => {
                "Movement is isometric by default, you'll get used to in no time;\nSettings -> Gameplay -> Movement Style"
            }
            WalkieEvent::DoorInteractionHesitation => "Press [E] near door to open.",
            WalkieEvent::StrugglingWithGrabDrop => "Use [F] to grab small items, [G] to drop.",
            WalkieEvent::StrugglingWithHideUnhide => {
                "Press & Hold [E] near tables/beds to hide. Press [E] again to unhide. (Cannot hide while holding items with [F])."
            }
            WalkieEvent::HuntActiveNearHidingSpotNoHide => {
                "Ghost hunting! Press & Hold [E] at the nearby hiding spot NOW!"
            }

            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => {
                "Dark! Use Flashlight [Tab] or find a room light switch [E]."
            }
            WalkieEvent::BreachShowcase => {
                "The shimmering distortion is the Ghost Breach, its entry point."
            }
            WalkieEvent::GhostShowcase => {
                "That was the ghost! Observe its appearance and behavior."
            }
            WalkieEvent::RoomLightsOnGearNeedsDark => {
                "This gear works best in darkness. Turn off room lights."
            }
            WalkieEvent::ThermometerNonFreezingFixation => {
                "Journal Check: Cold is good, but <0°C is \'Freezing Temps\' evidence."
            }
            WalkieEvent::GearSelectedNotActivated => "Activate gear in your right hand with [R].",

            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => "Health low! Return to the van to recover.",
            WalkieEvent::VeryLowSanityNoTruckReturn => {
                "Sanity critical! Go to the van immediately!"
            }
            WalkieEvent::SanityDroppedBelowThresholdDarkness => {
                "Sanity dropping in darkness! Find light or return to truck."
            }
            WalkieEvent::SanityDroppedBelowThresholdGhost => {
                "Sanity dropping near ghost! Increase distance or use Quartz."
            }

            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => {
                "Quartz took damage protecting you! It has limited uses."
            }
            WalkieEvent::QuartzShatteredFeedback => {
                "Quartz shattered! It no longer offers protection."
            }
            WalkieEvent::PlayerStaysHiddenTooLong => "Hunt over. Press [E] to stop hiding.",
            WalkieEvent::QuartzUnusedInRelevantSituation => {
                "Ghost is aggressive! Consider grabbing a Quartz Stone from the truck for defense."
            }
            WalkieEvent::SageUnusedInRelevantSituation => {
                "Ghost is agitated! Sage from the truck can help calm it or aid escape."
            }
            WalkieEvent::SageActivatedIneffectively => {
                "For Sage to work, ensure its smoke reaches the ghost\'s area."
            }
            WalkieEvent::SageUnusedDefensivelyDuringHunt => {
                "Sage could have helped during that hunt! Use [R] to activate if equipped."
            }

            // --- Repellent and Expulsion Events ---
            WalkieEvent::GhostExpelledPlayerLingers => {
                "Ghost gone! Return to van and select \'End Mission\'."
            }
            WalkieEvent::HasRepellentEntersLocation => {
                "Repellent equipped! Use [R] to open the bottle and ensure to be close to the ghost."
            }
            WalkieEvent::RepellentUsedTooFar => {
                "Repellent used too far away! Get closer to the ghost or its breach."
            }
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => {
                "Strong ghost reaction to repellent! It might be the correct type."
            }
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => {
                "Correct repellent type, but you ran out! Craft more in the van."
            }
            WalkieEvent::GhostExpelledPlayerMissed => {
                "Ghost expelled! Return to the area to visually confirm it and the breach are gone."
            }

            // --- Basic Gear Usage (continued) & Journal Logic ---
            WalkieEvent::DidNotSwitchStartingGearInHotspot => {
                "Current tool ineffective here. Try your other starting tool with [Q]."
            }
            WalkieEvent::DidNotCycleToOtherGear => {
                "Stuck on one tool? Press [Q] to cycle to other gear."
            }
            WalkieEvent::JournalPointsToOneGhostNoCraft => {
                "Journal identified ghost! Go to truck and \'Craft Repellent\'."
            }
            WalkieEvent::EMFNonEMF5Fixation => {
                "Journal Check: EMF activity noted, but EMF Level 5 is the specific evidence."
            }
            WalkieEvent::JournalConflictingEvidence => {
                "Journal evidence conflicts. Please review your findings carefully."
            }

            // --- "Evidence Confirmed" Events ---
            WalkieEvent::FreezingTempsEvidenceConfirmed => {
                "Freezing Temps found! Log with [C] or in truck journal."
            }
            WalkieEvent::FloatingOrbsEvidenceConfirmed => {
                "Floating Orbs spotted! Log with [C] or in truck journal."
            }
            WalkieEvent::UVEctoplasmEvidenceConfirmed => {
                "UV Ectoplasm detected! Log with [C] or in truck journal."
            }
            WalkieEvent::EMFLevel5EvidenceConfirmed => {
                "EMF Level 5 confirmed! Log with [C] or in truck journal."
            }
            WalkieEvent::EVPEvidenceConfirmed => "EVP recorded! Log with [C] or in truck journal.",
            WalkieEvent::SpiritBoxEvidenceConfirmed => {
                "Spirit Box response! Log with [C] or in truck journal."
            }
            WalkieEvent::RLPresenceEvidenceConfirmed => {
                "RL Presence observed! Log with [C] or in truck journal."
            }
            WalkieEvent::CPM500EvidenceConfirmed => {
                "500+ CPM on Geiger! Log with [C] or in truck journal."
            }

            // --- Proactive Crafting Prompts ---
            WalkieEvent::PotentialGhostIDWithNewEvidence => {
                "Potential ID! Log new evidence in truck to confirm & craft repellent."
            }

            // --- Mission Progression and Truck Events ---
            WalkieEvent::ClearEvidenceFoundNoActionCKey => {
                "Evidence found! Press [C] to quick-log it."
            }
            WalkieEvent::ClearEvidenceFoundNoActionTruck => {
                "Evidence found! Log it in the Journal in the truck."
            }
            WalkieEvent::InTruckWithEvidenceNoJournal => {
                "New evidence collected. Go to the truck and update your findings."
            }
            WalkieEvent::HuntWarningNoPlayerEvasion => {
                "Hunt starting! Hide or run! Press and Hold [E] near big furniture to hide."
            }
            WalkieEvent::AllObjectivesMetReminderToEndMission => {
                "All objectives complete! Return to truck and 'End Mission'."
            }
            WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout => {
                "Remember to check the Loadout tab in the truck to equip different gear."
            }
        }
    }
}
