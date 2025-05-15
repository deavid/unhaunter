use crate::ConceptTrait; // Import from crate root
use crate::generated::base1::Base1Concept;
use crate::generated::environmental_awareness::EnvironmentalAwarenessConcept; // Added import
use crate::generated::locomotion_and_interaction::LocomotionAndInteractionConcept;
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

    // --- Player Wellbeing Events ---
    /// Player's health is low for a prolonged period while inside the location.
    LowHealthGeneralWarning,
    /// Player's sanity is critically low and they haven't returned to the truck.
    VeryLowSanityNoTruckReturn,

    // --- Consumables and Defense Events ---
    QuartzCrackedFeedback,
    QuartzShatteredFeedback,
    /// Player stays hidden for too long after a hunt ends.
    PlayerStaysHiddenTooLong,
    // TODO: Add other event categories here
}

impl WalkieEvent {
    fn to_concept(&self) -> Box<dyn ConceptTrait> {
        match self {
            WalkieEvent::GearInVan => Box::new(Base1Concept::GearInVan),
            WalkieEvent::GhostNearHunt => Box::new(Base1Concept::GhostNearHunt),
            WalkieEvent::MissionStartEasy => Box::new(Base1Concept::MissionStartEasy),
            // --- Locomotion and Interaction ---
            WalkieEvent::PlayerStuckAtStart => Box::new(LocomotionAndInteractionConcept::PlayerStuckAtStart),
            WalkieEvent::ErraticMovementEarly => Box::new(LocomotionAndInteractionConcept::ErraticMovementEarly),
            WalkieEvent::DoorInteractionHesitation => Box::new(LocomotionAndInteractionConcept::DoorInteractionHesitation),
            WalkieEvent::StrugglingWithGrabDrop => Box::new(LocomotionAndInteractionConcept::StrugglingWithGrabDrop),
            WalkieEvent::StrugglingWithHideUnhide => Box::new(LocomotionAndInteractionConcept::StrugglingWithHideUnhide),
            WalkieEvent::HuntActiveNearHidingSpotNoHide => Box::new(crate::generated::ghost_behavior_and_hunting::GhostBehaviorAndHuntingConcept::HuntActiveNearHidingSpotNoHide),
            // --- Environmental Awareness ---
            WalkieEvent::DarkRoomNoLightUsed => Box::new(EnvironmentalAwarenessConcept::DarkRoomNoLightUsed),
            WalkieEvent::BreachShowcase => Box::new(EnvironmentalAwarenessConcept::IgnoredObviousBreach),
            WalkieEvent::GhostShowcase => Box::new(EnvironmentalAwarenessConcept::IgnoredVisibleGhost),
            WalkieEvent::RoomLightsOnGearNeedsDark => Box::new(EnvironmentalAwarenessConcept::RoomLightsOnGearNeedsDark),
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => Box::new(crate::generated::player_wellbeing::PlayerWellbeingConcept::LowHealthGeneralWarning),
            WalkieEvent::VeryLowSanityNoTruckReturn => Box::new(crate::generated::player_wellbeing::PlayerWellbeingConcept::VeryLowSanityNoTruckReturn),
            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => Box::new(crate::generated::consumables_and_defense::ConsumablesAndDefenseConcept::QuartzCrackedFeedback),
            WalkieEvent::QuartzShatteredFeedback => Box::new(crate::generated::consumables_and_defense::ConsumablesAndDefenseConcept::QuartzShatteredFeedback),
            WalkieEvent::PlayerStaysHiddenTooLong => Box::new(crate::generated::ghost_behavior_and_hunting::GhostBehaviorAndHuntingConcept::PlayerStaysHiddenTooLong),
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

            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => 120.0 * count,
            WalkieEvent::VeryLowSanityNoTruckReturn => 120.0 * count,

            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => 60.0 * count,
            WalkieEvent::QuartzShatteredFeedback => 60.0 * count,
            WalkieEvent::PlayerStaysHiddenTooLong => 90.0 * count,
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
            // --- Player Wellbeing ---
            WalkieEvent::LowHealthGeneralWarning => WalkieEventPriority::Medium,
            WalkieEvent::VeryLowSanityNoTruckReturn => WalkieEventPriority::High,
            // --- Consumables and Defense ---
            WalkieEvent::QuartzCrackedFeedback => WalkieEventPriority::Medium,
            WalkieEvent::QuartzShatteredFeedback => WalkieEventPriority::High,
            WalkieEvent::PlayerStaysHiddenTooLong => WalkieEventPriority::Low,
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
