use crate::{
    ConceptTrait,
    events::walkie_types::WalkieEvent,
    generated::{
        base1::Base1Concept, basic_gear_usage::BasicGearUsageConcept,
        consumables_and_defense::ConsumablesAndDefenseConcept,
        environmental_awareness::EnvironmentalAwarenessConcept,
        evidence_gathering_and_logic::EvidenceGatheringAndLogicConcept,
        ghost_behavior_and_hunting::GhostBehaviorAndHuntingConcept,
        incorrect_repellent_hint::IncorrectRepellentHintConcept,
        locomotion_and_interaction::LocomotionAndInteractionConcept,
        player_wellbeing::PlayerWellbeingConcept,
        repellent_and_expulsion::RepellentAndExpulsionConcept,
        tutorial_chapter_intros::TutorialChapterIntrosConcept,
        tutorial_gear_explanations::TutorialGearExplanationsConcept,
    },
};
use bevy::log::warn;
use uncore::{
    difficulty::Difficulty,
    types::{evidence::Evidence, gear_kind::GearKind},
};
use unwalkie_types::VoiceLineData;

struct NullVoice;

impl ConceptTrait for NullVoice {
    fn get_lines(&self) -> Vec<VoiceLineData> {
        vec![]
    }
}

impl WalkieEvent {
    fn to_concept(&self) -> Box<dyn ConceptTrait> {
        match self {
            WalkieEvent::GearInVan => Box::new(Base1Concept::GearInVan),
            WalkieEvent::GhostNearHunt => Box::new(Base1Concept::GhostNearHunt),
            WalkieEvent::IncorrectRepellentHint(evidence) => Box::new(
                Self::evidence_to_incorrect_repellent_hint_concept(*evidence),
            ),
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

    /// Get the list of voice line data for the event.
    pub fn sound_file_list(&self) -> Vec<VoiceLineData> {
        self.to_concept().get_lines()
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
                "Ghost gone! Return to van and select \'End Mission\'.."
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
                "Journal identified ghost! Go to truck and \'Craft Repellent\'.."
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
            WalkieEvent::IncorrectRepellentHint(evidence) => {
                // This will be dynamically formatted by the caller or a helper.
                // For now, return a generic placeholder or a format string.
                // The actual formatting will happen in the trigger system or WalkiePlay.
                // However, the plan asks for the formatted string here.
                // Let's assume a helper function `format_evidence_name` exists for now.
                Box::leak(
                    format!(
                        "Journal Updated: {} has been ruled out as a possibility.",
                        evidence.name()
                    )
                    .into_boxed_str(),
                )
            }
        }
    }

    /// Maps an Evidence enum to the corresponding IncorrectRepellentHintConcept variant
    fn evidence_to_incorrect_repellent_hint_concept(
        evidence: Evidence,
    ) -> IncorrectRepellentHintConcept {
        match evidence {
            Evidence::FreezingTemp => {
                IncorrectRepellentHintConcept::IncorrectRepellentHintFreezingTemp
            }
            Evidence::FloatingOrbs => {
                IncorrectRepellentHintConcept::IncorrectRepellentHintFloatingOrbs
            }
            Evidence::UVEctoplasm => {
                IncorrectRepellentHintConcept::IncorrectRepellentHintUVEctoplasm
            }
            Evidence::EMFLevel5 => IncorrectRepellentHintConcept::IncorrectRepellentHintEMFLevel5,
            Evidence::EVPRecording => {
                IncorrectRepellentHintConcept::IncorrectRepellentHintEVPRecording
            }
            Evidence::SpiritBox => IncorrectRepellentHintConcept::IncorrectRepellentHintSpiritBox,
            Evidence::RLPresence => IncorrectRepellentHintConcept::IncorrectRepellentHintRLPresence,
            Evidence::CPM500 => IncorrectRepellentHintConcept::IncorrectRepellentHintCPM500,
        }
    }
}
