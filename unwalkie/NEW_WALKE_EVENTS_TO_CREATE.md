Trigger events for voices
==============================


**RON File: `locomotion_and_interaction.ron`**

**5. `WalkieEventConceptEntry: StrugglingWithGrabDrop`** ( ** DONE ** )

This needs a full refactor, the voice lines, the event itself...

Possible Triggers: (each of these might become their own event with their own lines)
[GRAB]
* [F] pressed several times but nothing to pick up. That's probably because it's the wrong key.
* Pressing [E] trying to pick up stuff, and [E] it's doing nothing. No idea how to make voice lines that are in-character.
* Possessed object nearby, the player getting really close to it for a while, and not picking it up.
* Trying to grab gear from the ground but no space on the inventory.

[DROP]
* Insistently trying to drop where there's no space to.
* Not dropping the object mid-hunt - player would be slow to move.



---

**6. `WalkieEventConceptEntry: StrugglingWithHideUnhide`** ( ** DONE ** )

Needs full refactor, split into separate events, different voice lines for each, etc.

*   *Not Hiding During Hunt:* Ghost is actively hunting, player is near a valid `Behavior.p.object.hidingspot == true` but doesn't press and hold `[E]`.
*   *Immediately Unhiding:* Player successfully hides, but presses `[E]` again very quickly while hunt is still active.
*   *Trying to Hide While Carrying:* Player attempts to `[E]` interact with a hiding spot while `PlayerGear.held_item` is `Some`.

---






**RON File: `environmental_awareness.ron`**

---

**2. `WalkieEventConceptEntry: IgnoredObviousBreach`**  ( ** DONE ** )

Needs to be renamed and repurposed.

The name of the event should be BreachShowcase

This should be to teach the player what a breach looks like.


It needs to detect when the player is in the same room as the breach, then trigger the voice line.

Additionally, we need to send a secondary event for the breach to lit up for 30 seconds in a way that's very clear to see.


---

**3. `WalkieEventConceptEntry: IgnoredVisibleGhost`**  ( ** DONE ** )

Needs to be renamed and repurposed.

Should be named GhostShowcase. This should teach the player what the ghost looks like.

It needs to be triggered when the ghost is in the same room as the player.

An event should be triggered to showcase the ghost, such that the ghost becomes really visible for 30 seconds.
During this period the ghost is not allowed to leave the room where the player is.


---

**4. `WalkieEventConceptEntry: WildFlashlightSweeping`**


To be removed, this makes no sense at all.

---

**5. `WalkieEventConceptEntry: RoomLightsOnGearNeedsDark`**  ( ** DONE ** )

*   **Scenario Description (Recap):** Player activates a piece of gear that requires darkness for optimal use (e.g., Video Camera for Orbs, UV Torch for certain evidence types if that becomes a mechanic) while the current room's lights are ON.
*   **Goal of Hint:** Inform the player that the current room lighting might interfere with their gear's effectiveness.

We will keep this for last, as currently this is not really mandatory for players.

---

**6. `WalkieEventConceptEntry: FlashlightOnInLitRoom`**

*   **Scenario Description (Recap):** Player turns on their flashlight in a room where the main room lights (via light switch) are already on and providing sufficient illumination.
*   **Goal of Hint:** Gently suggest conserving flashlight battery when room lights are adequate.


Not doing this for now since batteries are not a concern currently.

---






---

**RON File: `basic_gear_usage.ron`**

**1. `WalkieEventConceptEntry: GearSelectedNotActivated`**

*   **Scenario Description (Recap):** Player has a piece of gear (e.g., Thermometer, EMF Meter) selected in their active hand but hasn't pressed the activate button (`[R]`) for X seconds, especially when in a relevant context (e.g., in a potentially haunted room).
*   **Goal of Hint:** Remind the player that most gear needs to be actively turned on to function.

This one is a hard one. The main objective here would be to teach the usage of the [R] button.

But the problem is knowing when the hint is really needed. Also, another big problem is that gear internals are hard to get, there are too many dependencies.

The trait GearUsable will need a new method "is_enabled() -> bool" and another one "can_enable() -> bool" to know if the gear is enabled or disabled, and
also to know if this particular one can be enabled at all.

We probably need to wait a long time (30 seconds) before triggering this, and ensuring the same piece of gear is always on the hand, i.e. the player did not cycle.

We will need to collect data somewhere of the last time the player activated gear, and the last time the player cycled gear [Q] - but that has to be done elsewhere, not in unwalkie.

We only want to check the right hand.

We want to ensure they're inside the location, but I don't think we need to wait until they're in the same room as the breach.

---

**2. `WalkieEventConceptEntry: ThermometerNonFreezingFixation`**  ( ** DONE ** )

*   **Scenario Description (Recap):** Player is using the Thermometer, it's showing cold temperatures (e.g., 1-10°C) but not sub-zero (Freezing Temps evidence). *   **Goal of Hint:** Guide the player to understand that "cold" isn't "freezing" for evidence, and suggest trying other tools or recognizing this might not be the evidence type.

Trigger: Player marks Feezing evidence without the ghost having this evidence.

We need to limit the amount of times we can give this guidance. Something needs to be stored on disk for the amount of times this was played - the number of missions. Or the list of different map names where it was triggered - maybe this is more stable and easy to get right.


---

**3. `WalkieEventConceptEntry: EMFNonEMF5Fixation`**

*   **Scenario Description (Recap):** Player is using the EMF Meter, it's showing activity (EMF 2-4) but not EMF Level 5 evidence.
*   **Goal of Hint:** Guide the player that while activity is noted, EMF5 is the specific evidence, and suggest trying other tools or recognizing this might not be the evidence type.

Trigger: Player marks EMF5 but ghost doesn't have this evidence.

As above, we need to limit the amount of times the player receives this feedback. This needs to be stored on disk. Storing the list of maps where this was triggered is enough.

We should, however, cross reference this with the maps that were completed, looking at the Grade. The maps that were not succeeded should not count. However, we should store it even if the player does not complete the mission. Same for Thermometer.

---

**4. `WalkieEventConceptEntry: DidNotSwitchStartingGearInHotspot`**

*   **Scenario Description (Recap):** Player gets a *promising but not definitive* reading with one starting tool (e.g., Thermometer shows 5°C, or EMF shows Level 3) in a "hotspot" (near breach, in ghost room) but doesn't switch to the *other primary starting tool* (EMF if using Thermometer, or vice-versa) within X seconds to cross-reference.
*   **Goal of Hint:** Encourage the player to use their *other* basic tool in an area that's already showing some activity.

This aims to teach the player the [Q] Cycle button. The trigger should focus on if the key has been used or not.

We need to wait first until the player gets enough reading from the primary tool (i.e. Thermometer) before triggering this. However this is hard to track.

We can assume safely that this is for the lowest tutorial difficulty, therefore we only care about Thermometer->EMF. We need to ensure the thermometer reads under 5ºC for some time, or negative if there's evidence, before starting to count time towards triggering this.

---

**5. `WalkieEventConceptEntry: DidNotCycleToOtherGear`**

*   **Scenario Description (Recap):** Player has been actively using one or two specific pieces of gear for an extended period (e.g., 1-2 minutes) in the location without pressing `[Q]` to cycle to other available gear in their inventory (especially if they have more than just two items, or if one hand is empty and they have backpack items).
*   **Goal of Hint:** Remind the player they have other tools in their inventory accessible via the cycle key.

This is the same as DidNotSwitchStartingGearInHotspot, and we should merge these two events into one.






---

**RON File: `evidence_gathering_and_logic.ron`**

**1. `WalkieEventConceptEntry: ClearEvidenceFoundNoActionCKey`**

*   **Scenario Description (Recap):** Player's active gear clearly indicates a piece of evidence (e.g., Thermometer shows <0°C, EMF Meter shows EMF5, Recorder shows "EVP RECORDED", etc.), but the player does not press the `[C]` key (Change Evidence/Log Quick Evidence) within X seconds.
*   **Goal of Hint:** Remind the player they can log evidence directly from their gear for convenience.

This needs to happen in Chapter 3 and later, not earlier. Part of the trigger has to be the player going back to the van to set evidence.

---

**2. `WalkieEventConceptEntry: ClearEvidenceFoundNoActionTruck`**

*   **Scenario Description (Recap):** Player has found one or more pieces of clear evidence (not logged with `[C]`, just observed), has been in the location for a significant time afterwards (e.g., 1 minute), and has not returned to the truck to use the journal.
*   **Goal of Hint:** Encourage the player to return to the truck to log evidence in the journal, which helps in ghost identification.

This is the counterpart of ClearEvidenceFoundNoActionCKey but for Chapters 1 and 2.

In this case we want the player to go back to the truck to set the evidence.

---

**3. `WalkieEventConceptEntry: InTruckWithEvidenceNoJournal`**

*   **Scenario Description (Recap):** Player just entered the truck after finding a piece of evidence during the current mission, but has not set any evidence in the journal.
*   **Goal of Hint:** Guide the player to use the Journal tab in the truck to log their findings and identify the ghost.

This is just a welcome message when they enter the truck to set evidence. We need to know if they have evidence to set, and if true, the next time they're in the truck and no prior evidence set, then we trigger. This message would play only once per mission.

---

**4. `WalkieEventConceptEntry: JournalConflictingEvidence`**

*   **Scenario Description (Recap):** Player has marked a combination of "Found" and/or "Discarded" evidence in the `TruckUI` Journal such that the list of possible ghosts no longer includes the correct ghost.
*   **Goal of Hint:** Alert the player that their current evidence selection is possibly wrong and they need to re-evaluate.

This hint would need to be reduced over time by the level of the player. So as their level grows, the more time needs to pass for the walkie to make this remark.


---

**5. `WalkieEventConceptEntry: JournalPointsToOneGhostNoCraft`**

*   **Scenario Description (Recap):** Player is in the `TruckUI` Journal tab, and the selected/discarded evidence has narrowed the possible ghost types down to exactly one, but the player doesn't click the "Craft Repellent" button and exits the truck.
*   **Goal of Hint:** Prompt the player to proceed with crafting the repellent now that a single ghost type is identified.

We need to remind the player of the next step, crafting the repellent.

To check, we just need to know that the player was in the truck <30 s ago, the evidences they set filter down to only one ghost, and they don't have a repellent in the inventory.

This one should be marked as done if the player had the repellent once in the mission, to avoid over repeating this.

---

**(Specific events to prompt the player to mark the evidence for each gear type):**

See ClearEvidenceFoundNoActionCKey for ideas on how to implement.

**6. `WalkieEventConceptEntry: SpiritBoxResponseNoAction`**


**7. `WalkieEventConceptEntry: RLPresenceFoundNoAction`**

**9. `WalkieEventConceptEntry: EVPRecordedNoAction`**


**10. `WalkieEventConceptEntry: CPM500FoundNoAction`**

**We're missing EMF, Thermometer, Orbs, UVPresence**



**8. `WalkieEventConceptEntry: UVTraceFoundNoFollow`** (This is more about *acting* on UV, fits better in `environmental_awareness.ron` or a new `tracking_and_observation.ron`)

This one probably should be removed.








---

**RON File: `repellent_and_expulsion.ron`**

**1. `WalkieEventConceptEntry: HasRepellentEntersLocation`**

*   **Scenario Description (Recap):** Player has successfully crafted a ghost-specific repellent in the truck and then enters the haunted location with it equipped or in inventory.
*   **Goal of Hint:** Remind the player about the repellent's purpose and the need to get close to the ghost/breach for it to be effective.


This one could also work very well just after the player exits the truck with the repellent.


---

**2. `WalkieEventConceptEntry: RepellentUsedTooFar`**

*   **Scenario Description (Recap):** Player activates the `RepellentFlask` (`RepellentFlask.active == true`), but their `Position` is too far from the `GhostSprite.Position` or `GhostBreach.Position` for the repellent particles to likely have an effect.
*   **Goal of Hint:** Advise the player to get closer to the target for the repellent to be effective.

We can probably trigger this by the repellent not hitting the ghost.

---

**3. `WalkieEventConceptEntry: RepellentUsedGhostEnragesPlayerFlees`** (Focus on first-time strong reaction)

*   **Scenario Description (Recap):** Player uses the `RepellentFlask` (could be correct or incorrect type), the `GhostSprite.rage` increases significantly or it enters a hunt-like visual/audio state, and the player immediately moves away a significant distance, possibly thinking the repellent failed or made things worse.
*   **Goal of Hint:** Explain that a strong reaction is expected, and if they believe the repellent is correct, they should persist (or if unsure, retreat to truck to re-evaluate evidence).

There's no need to know if the player is fleeing, we could trigger this just by getting the ghost rage high while the repellant is taking effect (good or not).

We just need to limit this help message by the number of missions completed, let's say 3.

---

**4. `WalkieEventConceptEntry: GhostExpelledPlayerMissed`**

*   **Scenario Description (Recap):** The `GhostSprite` and `GhostBreach` entities are despawned (expulsion successful), but the player was either far away, facing the wrong direction, or left the room immediately before/during the despawn animation and thus might not have visually confirmed the expulsion.
*   **Goal of Hint:** Inform the player that the expulsion was likely successful and they should go back to confirm visually.

We need to check how visible was the ghost or the breach during the event of the ghost fading away.

So this means we need to detect somehow the animation of dying was happening, probably GhostSprite has the information here.

And we need to check the visibility of both the GhostSprite and the Breach, was it clearly visible?

We need to to bascially integrate the visibility of them by the amount of time - we could add them both together and compute an average.

If at least one of them was 20% visible or more, we probably don't need to trigger this. But otherwise, if the animation ended and the ghost is gone, we will need to trigger this.

We will limit this hint by the level of the player, only for players below level 10.

---

**5. `WalkieEventConceptEntry: GhostExpelledPlayerLingers`**

*   **Scenario Description (Recap):** `GhostSprite` and `GhostBreach` are confirmed gone (either player saw it, or previous "PlayerMissed" hint was given and some time passed). Player remains in the haunted location for an extended period (e.g., X minutes) instead of returning to the truck to click "End Mission".
*   **Goal of Hint:** Prompt the player to return to the truck and end the mission.

We can just count 30 seconds of the player being in the location since the GhostSprite is gone.

If GhostExpelledPlayerMissed we could give 30 seconds lead time before counting, for this, GhostExpelledPlayerMissed could mark GhostExpelledPlayerLingers once.

---

**6. `WalkieEventConceptEntry: RepellentExhaustedGhostPresentCorrectType`**

*   **Scenario Description (Recap):** Player's `RepellentFlask.qty` reaches 0. The `GhostSprite` still exists. The `GhostSprite.repellent_hits` for the *current ghost type* (matching `RepellentFlask.liquid_content` before it became `None`) is > 0, indicating the player was using the correct repellent type.
*   **Goal of Hint:** Encourage the player, confirm their repellent choice was right, and instruct them to return to the truck to craft more.


Seems straightforward enough. The repellent being empty does not mean the repellent will not complete it's job, since it's in the air. So we need to also wait until the amount of particles of the repellent is low enough (<20).

---








---

**RON File: `consumables_and_defense.ron`**

**1. `WalkieEventConceptEntry: SaltUnusedInRelevantSituation`**

*   **Scenario Description (Recap):** Player has `GearKind::Salt` in their inventory, is in a relevant situation for its use (e.g., narrow corridor, doorway, suspected ghost path, trying to track a roaming ghost) but hasn't used any charges for an extended period or after significant ghost activity in that area.
*   **Goal of Hint:** Remind the player about Salt's utility for tracking and suggest its use.

This one is not really clear how it would work, or if it is needed, or even if it's helpful.

Also the Salt is one of these items that might change use later.

Probably we should skip doing this one.


---

**2. `WalkieEventConceptEntry: QuartzUnusedInRelevantSituation`** (More accurately, reminding it's equipped if a hunt is likely/starts)

*   **Scenario Description (Recap):** A hunt is starting (`GhostSprite.hunt_warning_active == true` or `GhostSprite.hunting > 0`), and the player has a `GearKind::QuartzStone` in their `PlayerGear` (hands or inventory) which is not yet shattered (`QuartzStoneData.cracks < MAX_CRACKS`).
*   **Goal of Hint:** Remind the player that the Quartz Stone they are carrying offers passive protection during a hunt.

This needs to be refactored. We probably should change this to tell the player that they're not using a Quartz stone and they could. This means they have not picked one from the Truck in this mission, and the ghost is getting angry.

Some voice lines might need change here.

This should only play once per mission. And of course it should play only if Quartz is available in the truck.

We should limit this to happen only after the ghost has hunted once.

---

**3. `WalkieEventConceptEntry: SageUnusedInRelevantSituation`**

*   **Scenario Description (Recap):** Ghost's `rage` is high (e.g., > 50% of `rage_limit`) OR `GhostSprite.hunt_warning_active == true`, and the player has `GearKind::SageBundle` in inventory with `SageBundleData.consumed == false` but hasn't activated it.
*   **Goal of Hint:** Suggest using Sage to potentially calm the ghost or provide cover/confusion if a hunt starts.

We should limit this to happen only after the ghost has hunted once.

We could also refactor this to be when the player doesn't use it, meaning it's in the truck but not on their inventory, never used.

We need to see how to avoid this and QuartzUnusedInRelevantSituation from playing in the same hunt number. If one plays, the other should be marked.

---

**4. `WalkieEventConceptEntry: SaltDroppedIneffectively`**

*   **Scenario Description (Recap):** Player uses a charge of Salt (`SaltData.spawn_salt` was true, now false), but the `Position` where it was dropped is in a very open area (e.g., center of a large room) rather than a chokepoint like a doorway or narrow corridor.
*   **Goal of Hint:** Advise the player on more strategic salt placement for better tracking.

No, I don't think we should do this at all. Probably it's best if we remove this. We can't give these kinds of hints effectively.

---

**5. `WalkieEventConceptEntry: QuartzCrackedFeedback`** ( ** DONE ** )

*   **Scenario Description (Recap):** The player's `QuartzStoneData.cracks` count increases by one due to absorbing hunt energy.
*   **Goal of Hint:** Inform the player that their Quartz Stone took damage but successfully protected them, and that it has limited durability.

We need to wait until the ghost calms down to do this, or the player exited the location.

---

**6. `WalkieEventConceptEntry: QuartzShatteredFeedback`** ( ** DONE ** )

*   **Scenario Description (Recap):** The player's `QuartzStoneData.cracks` reaches `MAX_CRACKS` (or a `is_shattered` flag becomes true), meaning it's now useless.
*   **Goal of Hint:** Inform the player their Quartz Stone is broken and no longer offers protection.

We need to wait until the ghost calms down to do this, or the player exited the location.


---

**7. `WalkieEventConceptEntry: SageActivatedIneffectively`**

*   **Scenario Description (Recap):** Player activates `SageBundleData` (`is_active == true`), but the spawned `SageSmokeParticle` entities are not near the `GhostSprite.Position` or its recent path for X seconds.
*   **Goal of Hint:** Advise the player to get the sage smoke closer to the ghost for it to have an effect.

Let's do it simple: We need to wait until the sage runs out. And check if the ghost was smoked or not, if it wasn't, then we play the message.

---

**8. `WalkieEventConceptEntry: SageUnusedDefensivelyDuringHunt`**

*   **Scenario Description (Recap):** `GhostSprite.hunting > 0` (actively hunting), player has `SageBundleData` available (`consumed == false`), but does not activate it within X seconds of the hunt starting or when ghost is very close.
*   **Goal of Hint:** Remind the player that Sage can be used defensively during a hunt to create confusion or break line of sight.

We should wait until the hunt is over, then remind the player.


---










---

**RON File: `ghost_behavior_and_hunting.ron`**

**1. `WalkieEventConceptEntry: HuntWarningNoPlayerEvasion`**

*   **Scenario Description (Recap):** The `GhostSprite.hunt_warning_active == true` (e.g., ghost is roaring, lights flickering intensely as a pre-hunt signal), but the player doesn't take evasive action (move towards a known hiding spot, attempt to leave the current room/area, or use a defensive item like Sage) for X seconds after the warning starts.
*   **Goal of Hint:** Urgently prompt the player to react to the hunt warning by hiding or creating distance.

Most players will understand this on their own, so there's not much point to this. Additionally, the amount of time we have to give the warning and the ghost attacking is pretty low. So this probably won't work.

Knowing that the player is taking evsive action is from hard to impossible. And even if doable, it needs several seconds to analyze, of which are crucial to deliver the message on time.

We could, optionally, reserve this for big maps, and trigger only if the ghost rages far away from the player. Otherwise I don't think this is usable.

---

**2. `WalkieEventConceptEntry: HuntActiveNearHidingSpotNoHide`**

*   **Scenario Description (Recap):** `GhostSprite.hunting > 10.0` (actively hunting and visible/audible), player is within close proximity (e.g., 1-2 units) to a valid `Behavior.p.object.hidingspot == true` but does not initiate the hide action (`[E]` hold) for X seconds.
*   **Goal of Hint:** Urgently direct the player to use the nearby hiding spot.

This is an interesting concept. We don't need to see if the player tries or does not try to hide, we could point it out straight away.

We should trigger an event to highlight the place to hide for 10 seconds.



---

**3. `WalkieEventConceptEntry: PlayerStaysHiddenTooLong`**

*   **Scenario Description (Recap):** Player has the `Hiding` component. The `GhostSprite.hunting` state has returned to 0 (or very low) for X seconds (e.g., 10-15s), indicating the hunt is over, but the player remains hidden.
*   **Goal of Hint:** Inform the player that the immediate danger has likely passed and they can safely unhide.

A very good idea here. Doesn't seem too complicated to do.

---

**4. `WalkieEventConceptEntry: EMFMinorFluctuationsIgnored`**

*   **Scenario Description (Recap):** Player is using the EMF Meter, and it's showing minor, non-evidence-level activity (e.g., `EMFMeter.emf_level` is `EMF2`, `EMF3`, or `EMF4` but not `EMF5`) in an area, but the player quickly moves on or switches gear without further investigation or noting the general activity.
*   **Goal of Hint:** Teach the player that even non-EMF5 readings can indicate ghost presence/activity and are useful for tracking or pinpointing areas of interest.

We should scrap this. The EMF tool is very bad at finding anything.

---












---

**RON File: `player_wellbeing.ron`**

**1. `WalkieEventConceptEntry: SanityDroppedBelowThresholdDarkness`**

*   **Scenario Description (Recap):** Player's sanity (`PlayerSprite.sanity()`) drops below a certain threshold (e.g., 50%) for the first time in the mission, and the primary contributing factor seems to be prolonged exposure to darkness (player has been in low `BoardData.light_field` areas for a significant portion of the time leading up to the sanity drop).
*   **Goal of Hint:** Inform the player about sanity loss in darkness and suggest using room lights or returning to the truck.

Mainly we check sanity dropped below 50%. That will be triggering either SanityDroppedBelowThresholdDarkness or SanityDroppedBelowThresholdGhost.

The main decision on when to trigger each one is the avg proximity to the ghost in the last 10 seconds, if it was close, then SanityDroppedBelowThresholdGhost otherwise SanityDroppedBelowThresholdDarkness

---

**2. `WalkieEventConceptEntry: SanityDroppedBelowThresholdGhost`**

*   **Scenario Description (Recap):** Player's sanity drops below a certain threshold (e.g., 50%) for the first time, and the primary contributing factor appears to be recent ghost proximity or significant paranormal events (e.g., ghost manifestation, objects thrown, loud noises attributed to ghost).
*   **Goal of Hint:** Inform the player that ghost interactions drain sanity and suggest returning to the truck to recover.

---

This one needs to be developed at the same time with SanityDroppedBelowThresholdDarkness.


**3. `WalkieEventConceptEntry: VeryLowSanityNoTruckReturn`** (DONE)

*   **Scenario Description (Recap):** Player's sanity (`PlayerSprite.sanity()`) is critically low (e.g., < 30%), visual/audio "insanity" effects are likely active, and the player has not moved towards the van/exit or entered the truck for X seconds/minutes since sanity became critical.
*   **Goal of Hint:** Urgently warn the player about their critical sanity and strongly advise returning to the truck immediately.

We can wait for 20 seconds to see if the player exits the location, if not and the sanity is below 30%, then we trigger the message.


---

**4. `WalkieEventConceptEntry: LowHealthGeneralWarning`** (DONE)

*   **Scenario Description (Recap):** Player's health (`PlayerSprite.health`) drops below a certain threshold (e.g., < 50% or < 30%) due to any reason (could be a lingering effect from a hunt, an environmental hazard if any exist, etc.), and they are not in the truck.
*   **Goal of Hint:** Advise the player that their health is low and they should consider returning to the truck to recover.

Seems straightforward. If the player health is below 50% for 30 seconds, and inside the location for the whole time, we play the message.

---








---

**RON File: `mission_progression_and_truck.ron`**

**1. `WalkieEventConceptEntry: PlayerLeavesTruckWithoutChangingLoadout`**

*   **Scenario Description (Recap):** Player starts a mission (after their very first one, where loadouts become more customizable), enters the truck, then leaves the truck to enter the haunted location *without* having interacted with the "Loadout" tab in the `TruckUI`. This hint is more relevant if their previous mission's loadout was significantly different or if they have new gear available.
*   **Goal of Hint:** Gently remind the player that they can customize their gear via the loadout tab, especially if they seem to be repeatedly using a suboptimal or default kit.

This needs to be merged with the existing GearInVan event. It's the same thing. We can merge the voice lines.


---

**2. `WalkieEventConceptEntry: AllObjectivesMetReminderToEndMission`**

*   **Scenario Description (Recap):** All primary mission objectives are complete (e.g., `GhostSprite` and `GhostBreach` are despawned/neutralized), the player has returned to the truck (`GameState::Truck`), but they haven't clicked the "End Mission" button in the `TruckUI` for X seconds/minutes.
*   **Goal of Hint:** Prompt the player to finalize the mission by clicking the "End Mission" button.

This seems to be an addendum to GhostExpelledPlayerLingers.

We need to analyze if these should be still separate events, or if they should be merged.











---

**RON File: `tutorial_specific_flow.ron`**

**1. `WalkieEventConceptEntry: FirstMissionPlayerNotEnteringCabin`**

*   **Scenario Description (Recap):** It's the player's absolute first mission (e.g., `PlayerProfileData.statistics.total_missions_completed == 0` and `PlayerProfileData.achievements.expelled_first_ghost == false`). The player has started the mission, is in the van or immediate vicinity, and has not moved towards or interacted with the entrance to the actual haunted location for an extended period (e.g., 45-60 seconds).
*   **Goal of Hint:** Very gently guide the brand-new player to physically enter the mission area.


We already have hints for this, we can remove these.

