// THIS FILE IS AUTOMATICALLY GENERATED BY THE WALKIE VOICE GENERATOR TOOL
// DO NOT EDIT MANUALLY

use unwalkie_types::{VoiceLineData, WalkieTag};

use crate::ConceptTrait;

/// Defines the different voice line concepts available in this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RepellentAndExpulsionConcept {
    GhostExpelledPlayerLingers,
    GhostExpelledPlayerMissed,
    HasRepellentEntersLocation,
    RepellentExhaustedGhostPresentCorrectType,
    RepellentUsedGhostEnragesPlayerFlees,
    RepellentUsedTooFar,
}

impl RepellentAndExpulsionConcept {
    /// Retrieves a vector of `VoiceLineData` for this concept variant.
    pub fn get_lines(&self) -> Vec<VoiceLineData> {
        match self {
            Self::GhostExpelledPlayerLingers => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_01.ogg".to_string(),
                    subtitle_text: "Pretty sure the coast is clear now. Time to call it a day back at the truck, I reckon, and hit that 'End Mission' button.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::ReminderLow],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_02.ogg".to_string(),
                    subtitle_text: "Job done, by the looks of it. Head back to the van and we can wrap this one up.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::PositiveReinforcement],
                    length_seconds: 5,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_03.ogg".to_string(),
                    subtitle_text: "Nothing more to see here, I think. The truck is waiting for you to officially close the case.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::NeutralObservation],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_04.ogg".to_string(),
                    subtitle_text: "Are you admiring your handiwork? Fair enough. But the 'End Mission' button is in the truck when you're ready.".to_string(),
                    tags: vec![WalkieTag::Humorous, WalkieTag::MediumLength, WalkieTag::ReminderLow],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_05.ogg".to_string(),
                    subtitle_text: "Looks like a successful operation. All that's left is to make it official back at the van.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_06.ogg".to_string(),
                    subtitle_text: "The site seems secure. Return to the truck to complete the mission debrief.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::NeutralObservation],
                    length_seconds: 5,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_07.ogg".to_string(),
                    subtitle_text: "Unless you're planning on redecorating, I think your work here is done. Back to the truck for the final step.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::SnarkyHumor],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_08.ogg".to_string(),
                    subtitle_text: "With the ghost gone, there's not much else to do but hit that 'End Mission' button in the van.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::ReminderMedium],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_09.ogg".to_string(),
                    subtitle_text: "Feels good to clear a haunting, eh? Don't forget to finalize the mission in the truck.".to_string(),
                    tags: vec![WalkieTag::MediumLength, WalkieTag::PositiveReinforcement, WalkieTag::ReminderLow],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayerlingers_10.ogg".to_string(),
                    subtitle_text: "All quiet on the paranormal front. That means it's time to head to the truck and clock out.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength],
                    length_seconds: 6,
                },
            ],
            Self::GhostExpelledPlayerMissed => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_01.ogg".to_string(),
                    subtitle_text: "Hang on... I think that might have done it. Things have gone very quiet all of a sudden. You might want to peek back in and confirm.".to_string(),
                    tags: vec![WalkieTag::DelayedObservation, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::Questioning],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_02.ogg".to_string(),
                    subtitle_text: "My readings just flatlined for our friend in there. Pretty sure it's checked out. Did you see it go?".to_string(),
                    tags: vec![WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::Questioning],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_03.ogg".to_string(),
                    subtitle_text: "I think you got it! The energy signature just vanished. Might be worth a look to make sure it's actually gone.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PositiveReinforcement],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_04.ogg".to_string(),
                    subtitle_text: "Is it... gone? The activity seems to have stopped completely. You should double-check the area.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::Questioning],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_05.ogg".to_string(),
                    subtitle_text: "Pretty sure that was a successful eviction. The place feels different, does it not? Go have a look, make sure it's clear.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::Questioning],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_06.ogg".to_string(),
                    subtitle_text: "That last dose of repellent seems to have done the trick. I'm not picking up any paranormal signs. Confirm visual?".to_string(),
                    tags: vec![WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::Questioning],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_07.ogg".to_string(),
                    subtitle_text: "It feels like the air has cleared. I think our ghost has departed. Best to make sure the breach is gone too.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_08.ogg".to_string(),
                    subtitle_text: "Victory! Or so my instruments suggest. Go take a final sweep, ensure no lingering spookiness.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::Humorous, WalkieTag::MediumLength, WalkieTag::PositiveReinforcement],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_09.ogg".to_string(),
                    subtitle_text: "I think our uninvited guest has finally left the building. You should verify the site is clear before we call it.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::NeutralObservation],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/ghostexpelledplayermissed_10.ogg".to_string(),
                    subtitle_text: "The atmosphere in there just changed significantly. I reckon that means job done. But a quick confirmation look wouldn't go amiss.".to_string(),
                    tags: vec![WalkieTag::DelayedObservation, WalkieTag::Guidance, WalkieTag::MediumLength],
                    length_seconds: 9,
                },
            ],
            Self::HasRepellentEntersLocation => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_01.ogg".to_string(),
                    subtitle_text: "Right, you've got the ghost-specific cocktail. Remember, you'll need to get reasonably close for it to... *mingle* properly with our friend.".to_string(),
                    tags: vec![WalkieTag::FirstTimeHint, WalkieTag::Guidance, WalkieTag::Humorous, WalkieTag::MediumLength],
                    length_seconds: 9,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_02.ogg".to_string(),
                    subtitle_text: "Armed with the repellent, I see. This is the pointy end of the stick. Good luck, and try to get a good dose on it.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::Guidance, WalkieTag::MediumLength],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_03.ogg".to_string(),
                    subtitle_text: "That repellent needs to make contact, or at least get very near the ghost or its breach to work effectively.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::ReminderLow],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_04.ogg".to_string(),
                    subtitle_text: "Time to serve the eviction notice. Head towards where you last saw it, or near its breach, and get ready to use that vial.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Guidance, WalkieTag::MediumLength],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_05.ogg".to_string(),
                    subtitle_text: "You're going in with the strong stuff. Remember, aim for the ghost or its main haunt. Don't just spray it at the walls.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::Humorous, WalkieTag::MediumLength],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_06.ogg".to_string(),
                    subtitle_text: "Repellent in hand, nice. The closer you are when you use it, the better the chances it'll be effective.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PositiveReinforcement],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_07.ogg".to_string(),
                    subtitle_text: "This is it then. You've got the repellent. Find your target and give it a good spray.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength],
                    length_seconds: 5,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_08.ogg".to_string(),
                    subtitle_text: "The repellent is for close encounters. Don't waste it from across the room.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::ReminderMedium, WalkieTag::ShortBrevity],
                    length_seconds: 5,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_09.ogg".to_string(),
                    subtitle_text: "With that vial, you're ready to face it. Get near its core area and activate the repellent.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/hasrepellententerslocation_10.ogg".to_string(),
                    subtitle_text: "Okay, repellent acquired. Now for the tricky part: actually using it on the spooky blighter.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::NeutralObservation],
                    length_seconds: 7,
                },
            ],
            Self::RepellentExhaustedGhostPresentCorrectType => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_01.ogg".to_string(),
                    subtitle_text: "Blast, ran out of the stuff! But you were definitely on the right track with that mix. Back to the truck, brew another batch, and give it another go. You've got this.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::PositiveReinforcement],
                    length_seconds: 10,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_02.ogg".to_string(),
                    subtitle_text: "You were so close! Just needed a bit more of that correct repellent. Head back to the van and whip up some more.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_03.ogg".to_string(),
                    subtitle_text: "That repellent was working, you just didn't have enough. Don't give up now, a quick refill at the truck should do it.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::PositiveReinforcement],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_04.ogg".to_string(),
                    subtitle_text: "It seems you had the right idea with the repellent, just not quite enough of it. Another vial from the truck might finish the job.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_05.ogg".to_string(),
                    subtitle_text: "That was the right stuff, it was definitely weakening! Just need to top up your repellent in the van and go again.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_06.ogg".to_string(),
                    subtitle_text: "So close to banishing it! Your repellent choice was spot on, just need a larger dose. Back to the truck for a refill.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::PositiveReinforcement],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_07.ogg".to_string(),
                    subtitle_text: "Don't be disheartened, you were using the correct repellent. It just needs a bit more persuading. Craft another batch.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_08.ogg".to_string(),
                    subtitle_text: "The ghost was reacting to that repellent, which means your ID was good. You just ran out. Time for a quick resupply at the van.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_09.ogg".to_string(),
                    subtitle_text: "You almost had it! That repellent was the right one. Another go with a full vial should send it packing.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentexhaustedghostpresentcorrecttype_10.ogg".to_string(),
                    subtitle_text: "It's a tough one, but your repellent was correct. Just needs a bit more of a push. Make another vial and finish it off.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::Encouraging, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
            ],
            Self::RepellentUsedGhostEnragesPlayerFlees => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_01.ogg".to_string(),
                    subtitle_text: "Whoa, it's properly furious now! That stuff definitely got its attention. If you think you've got the right mix, hold your nerve and keep at it.".to_string(),
                    tags: vec![WalkieTag::ConcernedWarning, WalkieTag::FirstTimeHint, WalkieTag::Guidance, WalkieTag::ImmediateResponse, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 9,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_02.ogg".to_string(),
                    subtitle_text: "It's not happy, is it? That means something is happening! Don't leg it just yet, see if you need to apply more or if you chose correctly.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 9,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_03.ogg".to_string(),
                    subtitle_text: "That's a strong reaction! It could be the right repellent, or it might just be very angry. Assess the situation before running!".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_04.ogg".to_string(),
                    subtitle_text: "Okay, it's throwing a tantrum. That's to be expected when you try to evict an angry spirit.".to_string(),
                    tags: vec![WalkieTag::Humorous, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::Questioning],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_05.ogg".to_string(),
                    subtitle_text: "It's definitely upset by that. If you're confident in your ghost ID, you might need to persist with the repellent.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_06.ogg".to_string(),
                    subtitle_text: "That angry display means the repellent is having some effect, good or bad. Don't just run away, think about your next move.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_07.ogg".to_string(),
                    subtitle_text: "A lot of noise and fury! That's common when the repellent starts working... or when it's the wrong one. They don't like it either way.".to_string(),
                    tags: vec![WalkieTag::ConcernedWarning, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_08.ogg".to_string(),
                    subtitle_text: "It's fighting back! If you're sure it's the correct repellent, you might just need to apply more pressure.".to_string(),
                    tags: vec![WalkieTag::Encouraging, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_09.ogg".to_string(),
                    subtitle_text: "That reaction means you're on its radar now. If that was the right stuff, keep it up.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedghostenragesplayerflees_10.ogg".to_string(),
                    subtitle_text: "It's certainly not pleased with that repellent. The question is, is it the *right kind* of not pleased? Only one way to find out!".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::Questioning],
                    length_seconds: 8,
                },
            ],
            Self::RepellentUsedTooFar => vec![
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_01.ogg".to_string(),
                    subtitle_text: "Bit of a long shot from there, love. You'll need to be a tad closer for that repellent to have any effect.".to_string(),
                    tags: vec![WalkieTag::ContextualHint, WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::SlightlyImpatient],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_02.ogg".to_string(),
                    subtitle_text: "I don't think it felt that from way over there. The repellent works best up close and personal.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::NeutralObservation, WalkieTag::PlayerStruggling],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_03.ogg".to_string(),
                    subtitle_text: "You're trying to fumigate the whole county, or just the ghost? Get closer with that stuff!".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::SnarkyHumor],
                    length_seconds: 6,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_04.ogg".to_string(),
                    subtitle_text: "That repellent has a limited range, you know. You need to be practically breathing down its neck for it to really work.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::Humorous, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_05.ogg".to_string(),
                    subtitle_text: "Wasting good repellent spraying it from that distance. The ghost needs to be nearby for it to take effect.".to_string(),
                    tags: vec![WalkieTag::ConcernedWarning, WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_06.ogg".to_string(),
                    subtitle_text: "Think of it like perfume, but for banishing things. You wouldn't spray perfume from across the street, would you? Get closer.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::Humorous, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 8,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_07.ogg".to_string(),
                    subtitle_text: "It needs to be in the thick of that repellent cloud. You're a bit too far away for it to have much impact.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_08.ogg".to_string(),
                    subtitle_text: "Are you sure it can even smell that from there? The repellent needs to be used in close proximity to the target.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::Questioning],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_09.ogg".to_string(),
                    subtitle_text: "That was a practice shot, right? Because it was nowhere near. Get in closer before you use the repellent again.".to_string(),
                    tags: vec![WalkieTag::DirectHint, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::SnarkyHumor],
                    length_seconds: 7,
                },
                VoiceLineData {
                    ogg_path: "walkie/generated/repellent_and_expulsion/repellentusedtoofar_10.ogg".to_string(),
                    subtitle_text: "The effective range on that repellent isn't what you'd call 'long'. You need to be much nearer the ghost or breach.".to_string(),
                    tags: vec![WalkieTag::Guidance, WalkieTag::MediumLength, WalkieTag::PlayerStruggling, WalkieTag::ReminderMedium],
                    length_seconds: 7,
                },
            ],
        }
    }
}

// Auto-generated implementation of ConceptTrait
impl ConceptTrait for RepellentAndExpulsionConcept {
    fn get_lines(&self) -> Vec<VoiceLineData> {
        // Delegate to the generated get_lines method
        self.get_lines()
    }
}
