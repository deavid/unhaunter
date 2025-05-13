use crate::ConceptTrait; // Import from crate root
use crate::generated::base1::Base1Concept;
use crate::generated::locomotion_and_interaction::LocomotionAndInteractionConcept;
use bevy::prelude::Event;
use unwalkie_types::VoiceLineData;

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
    /// Player is bumping into objects in a dark room.
    BumpingInDarkness,
    /// Player is struggling with picking up or dropping items.
    StrugglingWithGrabDrop,
    /// Player is struggling with hiding or unhiding mechanics.
    StrugglingWithHideUnhide,
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
            WalkieEvent::BumpingInDarkness => {
                Box::new(LocomotionAndInteractionConcept::BumpingInDarkness)
            }
            WalkieEvent::StrugglingWithGrabDrop => {
                Box::new(LocomotionAndInteractionConcept::StrugglingWithGrabDrop)
            }
            WalkieEvent::StrugglingWithHideUnhide => {
                Box::new(LocomotionAndInteractionConcept::StrugglingWithHideUnhide)
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
            WalkieEvent::PlayerStuckAtStart => 180.0 * count, // Long cooldown, especially after first play
            WalkieEvent::ErraticMovementEarly => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::DoorInteractionHesitation => 3600.0 * 24.0, // Effectively once per day (mission)
            WalkieEvent::BumpingInDarkness => 60.0 * count.powf(1.5), // Moderate, increases significantly
            WalkieEvent::StrugglingWithGrabDrop => 90.0 * count,
            WalkieEvent::StrugglingWithHideUnhide => 75.0 * count,
        }
    }

    /// Get the list of voice line data for the event.
    pub fn sound_file_list(&self) -> Vec<VoiceLineData> {
        self.to_concept().get_lines()
    }
}
