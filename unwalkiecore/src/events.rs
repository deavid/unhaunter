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
use bevy::prelude::Event;
use unwalkie_types::VoiceLineData;

#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum WalkieEventPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl WalkieEventPriority {
    pub fn value(&self) -> f32 {
        match self {
            WalkieEventPriority::Low => 0.1,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 10.0,
            WalkieEventPriority::Urgent => 100.0,
        }
    }
    pub fn time_factor(&self) -> f32 {
        match self {
            WalkieEventPriority::Low => 1.5,
            WalkieEventPriority::Medium => 1.0,
            WalkieEventPriority::High => 0.2,
            WalkieEventPriority::Urgent => 0.05,
        }
    }
    pub fn is_urgent(&self) -> bool {
        matches!(self, WalkieEventPriority::Urgent)
    }
}

/// Sending this event will cause the walkie to play a message.
#[derive(Clone, Debug, Event, PartialEq, Eq, Hash)]
pub enum WalkieEvent {
    /// When the player forgets the stuff in the van.
    GearInVan,
    /// When the Ghost rage is near its limit.
    GhostNearHunt,
    /// Welcome message for easy difficulty.
    MissionStartEasy,

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
    // TODO: Add other event categories here
}

impl WalkieEvent {
    fn to_concept(&self) -> Box<dyn ConceptTrait> {
        match self {
            WalkieEvent::GearInVan => Box::new(Base1Concept::GearInVan),
            WalkieEvent::GhostNearHunt => Box::new(Base1Concept::GhostNearHunt),
            WalkieEvent::MissionStartEasy => Box::new(Base1Concept::MissionStartEasy),
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
        }
    }

    pub fn time_to_play(&self, count: u32) -> f64 {
        let count = count.max(1) as f64;
        match self {
            WalkieEvent::GearInVan => 120.0 * count,
            WalkieEvent::GhostNearHunt => 120.0 * count.cbrt(),
            WalkieEvent::MissionStartEasy => 3600.0 * 24.0 * 7.0, // Effectively once per week

            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => 60.0 * count,
            WalkieEvent::ErraticMovementEarly => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::DoorInteractionHesitation => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::StrugglingWithGrabDrop => 90.0 * count,
            WalkieEvent::StrugglingWithHideUnhide => 75.0 * count,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => 30.0 * count,

            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => 90.0 * count,
            WalkieEvent::BreachShowcase => 90.0 * count,
            WalkieEvent::GhostShowcase => 90.0 * count,
            WalkieEvent::RoomLightsOnGearNeedsDark => 90.0 * count,
            WalkieEvent::ThermometerNonFreezingFixation => 120.0 * count,
            WalkieEvent::GearSelectedNotActivated => 15.0 + 60.0 * count, // 15s, then +1min per trigger

            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => 120.0 * count,
            WalkieEvent::VeryLowSanityNoTruckReturn => 120.0 * count,

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
            WalkieEvent::HasRepellentEntersLocation => 120.0 * count,
            WalkieEvent::RepellentUsedTooFar => 60.0 * count, // Trigger every minute if conditions met
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => 90.0 * count, // Trigger every 1.5 minutes if conditions met
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => 90.0 * count, // Trigger every 1.5 minutes if conditions met
            WalkieEvent::GhostExpelledPlayerMissed => 90.0 * count, // Trigger every 1.5 minutes if conditions met
            WalkieEvent::DidNotSwitchStartingGearInHotspot => 180.0 * count, // Trigger every 3 minutes if conditions met
            WalkieEvent::DidNotCycleToOtherGear => 90.0 * count, // Trigger every 1.5 minutes if conditions met
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => 300.0 * count, // Trigger every 5 minutes if conditions met
            WalkieEvent::EMFNonEMF5Fixation => 120.0 * count, // Trigger every 2 minutes if conditions met
            WalkieEvent::JournalConflictingEvidence => 300.0 * count, // Trigger every 5 minutes if conditions met
        }
    }

    /// Get the list of voice line data for the event.
    pub fn sound_file_list(&self) -> Vec<VoiceLineData> {
        self.to_concept().get_lines()
    }

    pub fn priority(&self) -> WalkieEventPriority {
        match self {
            WalkieEvent::GearInVan => WalkieEventPriority::Low,
            WalkieEvent::GhostNearHunt => WalkieEventPriority::Urgent,
            WalkieEvent::MissionStartEasy => WalkieEventPriority::Medium,
            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => WalkieEventPriority::Medium,
            WalkieEvent::ErraticMovementEarly => WalkieEventPriority::Urgent,
            WalkieEvent::DoorInteractionHesitation => WalkieEventPriority::High,
            WalkieEvent::StrugglingWithGrabDrop => WalkieEventPriority::Low,
            WalkieEvent::StrugglingWithHideUnhide => WalkieEventPriority::Low,
            WalkieEvent::HuntActiveNearHidingSpotNoHide => WalkieEventPriority::High,
            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => WalkieEventPriority::Low,
            WalkieEvent::BreachShowcase => WalkieEventPriority::Medium,
            WalkieEvent::GhostShowcase => WalkieEventPriority::Medium,
            WalkieEvent::RoomLightsOnGearNeedsDark => WalkieEventPriority::Low,
            WalkieEvent::ThermometerNonFreezingFixation => WalkieEventPriority::Low,
            WalkieEvent::GearSelectedNotActivated => WalkieEventPriority::Low,
            WalkieEvent::EMFNonEMF5Fixation => WalkieEventPriority::Low,
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => WalkieEventPriority::Medium,
            WalkieEvent::VeryLowSanityNoTruckReturn => WalkieEventPriority::High,
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
            WalkieEvent::GhostExpelledPlayerLingers => WalkieEventPriority::Medium,
            WalkieEvent::HasRepellentEntersLocation => WalkieEventPriority::Medium,
            WalkieEvent::RepellentUsedTooFar => WalkieEventPriority::Low,
            WalkieEvent::RepellentUsedGhostEnragesPlayerFlees => WalkieEventPriority::High,
            WalkieEvent::RepellentExhaustedGhostPresentCorrectType => WalkieEventPriority::Medium,
            WalkieEvent::GhostExpelledPlayerMissed => WalkieEventPriority::Medium,
            WalkieEvent::DidNotSwitchStartingGearInHotspot => WalkieEventPriority::Medium,
            WalkieEvent::DidNotCycleToOtherGear => WalkieEventPriority::Medium,
            // --- Evidence Gathering ---
            WalkieEvent::JournalPointsToOneGhostNoCraft => WalkieEventPriority::Medium,
            WalkieEvent::JournalConflictingEvidence => WalkieEventPriority::Medium,
        }
    }
    /// This is just a prototype function that needs to be replaced, the idea being
    /// that whenever a voice line needs to add extra hints visually for the player,
    /// we send some kind of Bevy Event to an hypothetical hint system that will highlight
    /// the controls or the objects that are relevant to the voice line.
    pub fn _hint_event(&self) -> &'static str {
        match &self {
            WalkieEvent::PlayerStuckAtStart => "highlight WASD controls",
            _ => "",
        }
    }
}
