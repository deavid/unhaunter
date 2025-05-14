
Trigger events for voices
==============================


**RON File: `locomotion_and_interaction.ron`**

**5. `WalkieEventConceptEntry: StrugglingWithGrabDrop`**

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

**6. `WalkieEventConceptEntry: StrugglingWithHideUnhide`**

Needs full refactor, split into separate events, different voice lines for each, etc.

*   *Not Hiding During Hunt:* Ghost is actively hunting, player is near a valid `Behavior.p.object.hidingspot == true` but doesn't press and hold `[E]`.
*   *Immediately Unhiding:* Player successfully hides, but presses `[E]` again very quickly while hunt is still active.
*   *Trying to Hide While Carrying:* Player attempts to `[E]` interact with a hiding spot while `PlayerGear.held_item` is `Some`.

---






**RON File: `environmental_awareness.ron`**

---

**2. `WalkieEventConceptEntry: IgnoredObviousBreach`**

Needs to be renamed and repurposed.

The name of the event should be BreachShowcase

This should be to teach the player what a breach looks like.


It needs to detect when the player is in the same room as the breach, then trigger the voice line.

Additionally, we need to send a secondary event for the breach to lit up for 30 seconds in a way that's very clear to see.


---

**3. `WalkieEventConceptEntry: IgnoredVisibleGhost`**

Needs to be renamed and repurposed.

Should be named GhostShowcase. This should teach the player what the ghost looks like.

It needs to be triggered when the ghost is in the same room as the player.

An event should be triggered to showcase the ghost, such that the ghost becomes really visible for 30 seconds.
During this period the ghost is not allowed to leave the room where the player is.


---

**4. `WalkieEventConceptEntry: WildFlashlightSweeping`**


To be removed, this makes no sense at all.

---

**5. `WalkieEventConceptEntry: RoomLightsOnGearNeedsDark`**

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

**2. `WalkieEventConceptEntry: ThermometerNonFreezingFixation`**

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

**6. `WalkieEventConceptEntry: SpiritBoxResponseNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` if SpiritBox sets a clear "EVP detected" state that can be queried)

**7. `WalkieEventConceptEntry: RLPresenceFoundNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` if Red Torch interaction sets a clear "RL Presence detected" state)

**9. `WalkieEventConceptEntry: EVPRecordedNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` when `Recorder.evp_recorded_display == true`)

**10. `WalkieEventConceptEntry: CPM500FoundNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` when `GeigerCounter.sound_display > 500`)

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
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player has recently activated `RepellentFlask`.
    *   `GhostSprite.rage` spikes significantly OR `GhostSprite.hunting` state changes shortly after repellent use.
    *   Player `Position` rapidly increases distance from `GhostSprite.Position` immediately following the ghost's reaction.
    *   This is the first or second time this strong reaction + flee sequence has occurred in the mission.
    *   `WalkiePlay.can_play(WalkieEvent::RepellentUsedGhostEnragesPlayerFlees, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `PlayerGear`, `GhostSprite.Position`, `GhostSprite.rage`, `GhostSprite.hunting` state, `WalkiePlay`, mission attempt counter for this specific reaction.
*   **`WalkieEvent` Enum Variant:** `RepellentUsedGhostEnragesPlayerFlees`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for this reaction), `Guidance`, `ConcernedWarning` (about misinterpreting), `PlayerStruggling`.
*   **Repetition Strategy:** Once or twice per mission for this specific "fleeing an enraged ghost" scenario.
*   **Priority/Severity:** Medium. Helps player understand ghost reaction.

---

**4. `WalkieEventConceptEntry: GhostExpelledPlayerMissed`**

*   **Scenario Description (Recap):** The `GhostSprite` and `GhostBreach` entities are despawned (expulsion successful), but the player was either far away, facing the wrong direction, or left the room immediately before/during the despawn animation and thus might not have visually confirmed the expulsion.
*   **Goal of Hint:** Inform the player that the expulsion was likely successful and they should go back to confirm visually.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostSprite` and `GhostBreach` entities no longer exist (or have a special "expelled" state).
    *   At the moment of despawn, Player `Position` was > X distance from ghost/breach OR Player `Direction` was not facing ghost/breach.
    *   Timer `TimeSinceExpulsionNoConfirmation` > Y seconds (e.g., 10-15s).
    *   `WalkiePlay.can_play(WalkieEvent::GhostExpelledPlayerMissed, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `Direction` (at time of despawn), status of `GhostSprite` & `GhostBreach` entities, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `GhostExpelledPlayerMissed`
*   **Primary `WalkieTag`(s) for Line Selection:** `DelayedObservation`, `Guidance`, `PositiveReinforcement`.
*   **Repetition Strategy:** Once per successful expulsion if player seems to have missed it.
*   **Priority/Severity:** Medium. Important for mission completion confirmation.

---

**5. `WalkieEventConceptEntry: GhostExpelledPlayerLingers`**

*   **Scenario Description (Recap):** `GhostSprite` and `GhostBreach` are confirmed gone (either player saw it, or previous "PlayerMissed" hint was given and some time passed). Player remains in the haunted location for an extended period (e.g., X minutes) instead of returning to the truck to click "End Mission".
*   **Goal of Hint:** Prompt the player to return to the truck and end the mission.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player is inside the location (`RoomDB`).
    *   `GhostSprite` and `GhostBreach` entities are confirmed gone (internal flag `MissionObjectivesComplete` is true).
    *   Timer `TimeSinceObjectivesCompletePlayerLingers` > X minutes (e.g., 1-2 mins).
    *   `WalkiePlay.can_play(WalkieEvent::GhostExpelledPlayerLingers, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position`, `RoomDB`, internal flag for objectives complete, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `GhostExpelledPlayerLingers`
*   **Primary `WalkieTag`(s) for Line Selection:** `ReminderLow`, `DirectHint`, `Humorous`.
*   **Repetition Strategy:** Can play a couple of times with increasing cooldown if player continues to linger.
*   **Priority/Severity:** Low-Medium.

---

**6. `WalkieEventConceptEntry: RepellentExhaustedGhostPresentCorrectType`**

*   **Scenario Description (Recap):** Player's `RepellentFlask.qty` reaches 0. The `GhostSprite` still exists. The `GhostSprite.repellent_hits` for the *current ghost type* (matching `RepellentFlask.liquid_content` before it became `None`) is > 0, indicating the player was using the correct repellent type.
*   **Goal of Hint:** Encourage the player, confirm their repellent choice was right, and instruct them to return to the truck to craft more.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player's `RepellentFlask.qty == 0` (event fires once when it transitions to 0).
    *   `GhostSprite` entity still exists.
    *   A temporary variable/check confirms that the `RepellentFlask.liquid_content` (just before becoming `None`) matched `GhostSprite.class` AND `GhostSprite.repellent_hits > 0` (or a similar stat showing the correct repellent was being effective).
    *   `WalkiePlay.can_play(WalkieEvent::RepellentExhaustedGhostPresentCorrectType, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`RepellentFlask` state), `GhostSprite` entity status & `GhostSprite.class` & `GhostSprite.repellent_hits` (or a new field like `last_repellent_type_effective: bool`), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `RepellentExhaustedGhostPresentCorrectType`
*   **Primary `WalkieTag`(s) for Line Selection:** `PlayerStruggling` (because ran out), `Encouraging`, `DirectHint`, `PositiveReinforcement` (for correct ID).
*   **Repetition Strategy:** Once per "correct repellent exhausted" event.
*   **Priority/Severity:** High. Critical for player to not give up if they were on the right track.

---





Excellent! Let's maintain this momentum and move on to **`consumables_and_defense.ron`**.

This file will cover hints related to the player's use (or lack thereof) of defensive and utility consumables like Salt, Quartz, and Sage, which are typically introduced in later chapters.

---

**RON File: `consumables_and_defense.ron`**

**1. `WalkieEventConceptEntry: SaltUnusedInRelevantSituation`**

*   **Scenario Description (Recap):** Player has `GearKind::Salt` in their inventory, is in a relevant situation for its use (e.g., narrow corridor, doorway, suspected ghost path, trying to track a roaming ghost) but hasn't used any charges for an extended period or after significant ghost activity in that area.
*   **Goal of Hint:** Remind the player about Salt's utility for tracking and suggest its use.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player `PlayerGear` contains `GearKind::Salt` with `SaltData.charges > 0`.
    *   Player is in a "strategically relevant area" for salt (e.g., near a doorway in the ghost's room, in a narrow hallway the ghost frequents, or after observing ghost pass a point). This might require some heuristics or map annotation.
    *   Timer `TimeSinceLastSaltUseInRelevantAreaOrGhostActivity` > X seconds/minutes.
    *   AND/OR Ghost has recently passed through a "chokepoint" near the player, and player has salt.
    *   `WalkiePlay.can_play(WalkieEvent::SaltUnusedInRelevantSituation, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`SaltData`), Player `Position`, `RoomDB` (for room/path context), ghost `Position` history (for pathing), local timers, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SaltUnusedInRelevantSituation`
*   **Primary `WalkieTag`(s) for Line Selection:** `ReminderLow`, `Guidance`, `ContextualHint`, `FirstTimeHint` (if it's the first time salt is relevant after being acquired).
*   **Repetition Strategy:** Occasionally if relevant situations persist and salt remains unused.
*   **Priority/Severity:** Medium-Low. Salt is useful but not always critical.

---

**2. `WalkieEventConceptEntry: QuartzUnusedInRelevantSituation`** (More accurately, reminding it's equipped if a hunt is likely/starts)

*   **Scenario Description (Recap):** A hunt is starting (`GhostSprite.hunt_warning_active == true` or `GhostSprite.hunting > 0`), and the player has a `GearKind::QuartzStone` in their `PlayerGear` (hands or inventory) which is not yet shattered (`QuartzStoneData.cracks < MAX_CRACKS`).
*   **Goal of Hint:** Remind the player that the Quartz Stone they are carrying offers passive protection during a hunt.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   (`GhostSprite.hunt_warning_active == true` OR `GhostSprite.hunting > 0.1`).
    *   Player `PlayerGear` contains `GearKind::QuartzStone` with `QuartzStoneData.cracks < MAX_CRACKS`.
    *   This hint should trigger shortly after the hunt/warning begins if quartz is available.
    *   `WalkiePlay.can_play(WalkieEvent::QuartzUnusedInRelevantSituation, current_time)` returns true (using "Unused" in the event name is a bit of a misnomer as it's passive, but it's about awareness).
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`QuartzStoneData`), `GhostSprite` state, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `QuartzEquippedDuringHuntWarning` (renamed for clarity)
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for quartz's role), `FriendlyReminder`, `Encouraging`, `ConcernedWarning` (implicitly, as a hunt is dangerous).
*   **Repetition Strategy:** Once per hunt if quartz is available and not shattered.
*   **Priority/Severity:** Medium. Can increase survivability.

---

**3. `WalkieEventConceptEntry: SageUnusedInRelevantSituation`**

*   **Scenario Description (Recap):** Ghost's `rage` is high (e.g., > 75% of `rage_limit`) OR `GhostSprite.hunt_warning_active == true`, and the player has `GearKind::SageBundle` in inventory with `SageBundleData.consumed == false` but hasn't activated it.
*   **Goal of Hint:** Suggest using Sage to potentially calm the ghost or provide cover/confusion if a hunt starts.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player `PlayerGear` contains `GearKind::SageBundle` with `SageBundleData.consumed == false`.
    *   (`GhostSprite.rage / GhostSprite.rage_limit > 0.75` AND `GhostSprite.calm_time_secs <= 0`) OR `GhostSprite.hunt_warning_active == true`.
    *   Timer `TimeSinceHighRageOrHuntWarningWithSageUnused` > X seconds.
    *   `WalkiePlay.can_play(WalkieEvent::SageUnusedInRelevantSituation, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`SageBundleData`), `GhostSprite` state, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SageUnusedInRelevantSituation`
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `ConcernedWarning`, `ContextualHint`, `ReminderLow`.
*   **Repetition Strategy:** Once per high-rage/pre-hunt situation if sage is available.
*   **Priority/Severity:** Medium. Sage can be a useful tactical tool.

---

**4. `WalkieEventConceptEntry: SaltDroppedIneffectively`**

*   **Scenario Description (Recap):** Player uses a charge of Salt (`SaltData.spawn_salt` was true, now false), but the `Position` where it was dropped is in a very open area (e.g., center of a large room) rather than a chokepoint like a doorway or narrow corridor.
*   **Goal of Hint:** Advise the player on more strategic salt placement for better tracking.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `SaltData.spawn_salt` just transitioned from true to false (salt was just dropped).
    *   The drop `Position` is checked against map geometry: if it's > X units away from any wall/doorway in an open room (room size also a factor).
    *   `WalkiePlay.can_play(WalkieEvent::SaltDroppedIneffectively, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` (at time of drop), `BoardData.collision_field`, `RoomDB` (for room geometry), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SaltDroppedIneffectively`
*   **Primary `WalkieTag`(s) for Line Selection:** `PlayerStruggling`, `Guidance`, `ContextualHint`, `Humorous`.
*   **Repetition Strategy:** If player repeatedly places salt poorly. Cooldown.
*   **Priority/Severity:** Low-Medium. Helps optimize salt use.

---

**5. `WalkieEventConceptEntry: QuartzCrackedFeedback`**

*   **Scenario Description (Recap):** The player's `QuartzStoneData.cracks` count increases by one due to absorbing hunt energy.
*   **Goal of Hint:** Inform the player that their Quartz Stone took damage but successfully protected them, and that it has limited durability.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `QuartzStoneData.cracks` value increases. This event fires once per crack.
    *   `WalkiePlay.can_play(WalkieEvent::QuartzCrackedFeedback, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`QuartzStoneData`), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `QuartzCrackedFeedback`
*   **Primary `WalkieTag`(s) for Line Selection:** `ImmediateResponse`, `PositiveReinforcement` (it worked!), `NeutralObservation`, `ConcernedWarning` (it's getting used up).
*   **Repetition Strategy:** Once per crack level.
*   **Priority/Severity:** Medium. Important feedback on consumable use.

---

**6. `WalkieEventConceptEntry: QuartzShatteredFeedback`**

*   **Scenario Description (Recap):** The player's `QuartzStoneData.cracks` reaches `MAX_CRACKS` (or a `is_shattered` flag becomes true), meaning it's now useless.
*   **Goal of Hint:** Inform the player their Quartz Stone is broken and no longer offers protection.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `QuartzStoneData.cracks >= MAX_CRACKS` (event fires once when this threshold is met/exceeded).
    *   `WalkiePlay.can_play(WalkieEvent::QuartzShatteredFeedback, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`QuartzStoneData`), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `QuartzShatteredFeedback`
*   **Primary `WalkieTag`(s) for Line Selection:** `ImmediateResponse`, `ConcernedWarning`, `PlayerStruggling` (now more vulnerable).
*   **Repetition Strategy:** Once when the stone shatters.
*   **Priority/Severity:** Medium-High. Player needs to know their defense is gone.

---

**7. `WalkieEventConceptEntry: SageActivatedIneffectively`**

*   **Scenario Description (Recap):** Player activates `SageBundleData` (`is_active == true`), but the spawned `SageSmokeParticle` entities are not near the `GhostSprite.Position` or its recent path for X seconds.
*   **Goal of Hint:** Advise the player to get the sage smoke closer to the ghost for it to have an effect.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `SageBundleData.is_active == true` and `burn_timer` is running.
    *   A system checks the average distance of recent `SageSmokeParticle` entities from the `GhostSprite.Position` (or its last known strong activity area). If this average distance is > Y units for X seconds.
    *   `WalkiePlay.can_play(WalkieEvent::SageActivatedIneffectively, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`SageBundleData`), `SageSmokeParticle` positions, `GhostSprite.Position`, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SageActivatedIneffectively`
*   **Primary `WalkieTag`(s) for Line Selection:** `PlayerStruggling`, `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** If player repeatedly uses sage far from the ghost. Cooldown.
*   **Priority/Severity:** Medium. Helps with effective consumable use.

---

**8. `WalkieEventConceptEntry: SageUnusedDefensivelyDuringHunt`**

*   **Scenario Description (Recap):** `GhostSprite.hunting > 0` (actively hunting), player has `SageBundleData` available (`consumed == false`), but does not activate it within X seconds of the hunt starting or when ghost is very close.
*   **Goal of Hint:** Remind the player that Sage can be used defensively during a hunt to create confusion or break line of sight.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostSprite.hunting > 0.1`.
    *   Player `PlayerGear` contains `SageBundleData` with `consumed == false` and `is_active == false`.
    *   Timer `TimeSinceHuntStartWithSageAvailable` > X seconds (e.g., 3-5s) OR `GhostSprite.Position.distance(Player.Position) < CloseRangeDuringHuntThreshold`.
    *   `WalkiePlay.can_play(WalkieEvent::SageUnusedDefensivelyDuringHunt, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`SageBundleData`), `GhostSprite.hunting` state & `Position`, Player `Position`, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SageUnusedDefensivelyDuringHunt`
*   **Primary `WalkieTag`(s) for Line Selection:** `ConcernedWarning`, `DirectHint`, `ImmediateResponse`, `PlayerStruggling`.
*   **Repetition Strategy:** Once per hunt if applicable.
*   **Priority/Severity:** Medium-High. Can be a lifesaver.

---










---

**RON File: `ghost_behavior_and_hunting.ron`**

**1. `WalkieEventConceptEntry: HuntWarningNoPlayerEvasion`**

*   **Scenario Description (Recap):** The `GhostSprite.hunt_warning_active == true` (e.g., ghost is roaring, lights flickering intensely as a pre-hunt signal), but the player doesn't take evasive action (move towards a known hiding spot, attempt to leave the current room/area, or use a defensive item like Sage) for X seconds after the warning starts.
*   **Goal of Hint:** Urgently prompt the player to react to the hunt warning by hiding or creating distance.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostSprite.hunt_warning_active == true`.
    *   `GhostSprite.hunt_warning_timer` is, for example, > Y seconds from its start OR < Z seconds from its end (to give a chance before hunt fully starts).
    *   Player's `Position` has not significantly changed vector towards an exit or known `Behavior.p.object.hidingspot`.
    *   Player has not activated `SageBundleData` (if available).
    *   `WalkiePlay.can_play(WalkieEvent::HuntWarningNoPlayerEvasion, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `GhostSprite` state (`hunt_warning_active`, `hunt_warning_timer`), Player `Position` & movement vector, known hiding spot locations/entities, `PlayerGear` (for Sage status), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `HuntWarningNoPlayerEvasion`
*   **Primary `WalkieTag`(s) for Line Selection:** `ConcernedWarning`, `DirectHint`, `ImmediateResponse`, `UrgentReminder` (or `ReminderHigh`).
*   **Repetition Strategy:** Once per hunt warning phase if player is unresponsive.
*   **Priority/Severity:** Very High. Critical for survival.

---

**2. `WalkieEventConceptEntry: HuntActiveNearHidingSpotNoHide`**

*   **Scenario Description (Recap):** `GhostSprite.hunting > 0.1` (actively hunting and visible/audible), player is within close proximity (e.g., 1-2 units) to a valid `Behavior.p.object.hidingspot == true` but does not initiate the hide action (`[E]` hold) for X seconds.
*   **Goal of Hint:** Urgently direct the player to use the nearby hiding spot.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostSprite.hunting > 0.1`.
    *   Player `Position` is within interaction range of an `Entity` with `Behavior.p.object.hidingspot == true`.
    *   Player does not have the `Hiding` component.
    *   Timer `TimeNearHidingSpotDuringHuntNoHide` > X seconds (e.g., 1-2s, needs to be quick).
    *   `WalkiePlay.can_play(WalkieEvent::HuntActiveNearHidingSpotNoHide, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `GhostSprite.hunting` state, Player `Position` & presence of `Hiding` component, nearby entity `Behavior.p.object.hidingspot`, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `HuntActiveNearHidingSpotNoHide`
*   **Primary `WalkieTag`(s) for Line Selection:** `UrgentReminder`, `DirectHint`, `ContextualHint`, `PlayerStruggling`, `ImmediateResponse`.
*   **Repetition Strategy:** Can play quickly if player remains near a hiding spot during a hunt without using it.
*   **Priority/Severity:** Critical. Key survival mechanic.

---

**3. `WalkieEventConceptEntry: PlayerStaysHiddenTooLong`**

*   **Scenario Description (Recap):** Player has the `Hiding` component. The `GhostSprite.hunting` state has returned to 0 (or very low) for X seconds (e.g., 10-15s), indicating the hunt is over, but the player remains hidden.
*   **Goal of Hint:** Inform the player that the immediate danger has likely passed and they can safely unhide.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player has `Hiding` component.
    *   `GhostSprite.hunting < 0.01` (or some "not hunting" threshold).
    *   Timer `TimeSinceHuntEndedWhileHidden` > X seconds (e.g., 10-15s).
    *   `WalkiePlay.can_play(WalkieEvent::PlayerStaysHiddenTooLong, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player presence of `Hiding` component, `GhostSprite.hunting` state, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `PlayerStaysHiddenTooLong`
*   **Primary `WalkieTag`(s) for Line Selection:** `DelayedObservation`, `Guidance`, `Encouraging`.
*   **Repetition Strategy:** Once after a hunt if player remains hidden for an extended period.
*   **Priority/Severity:** Low-Medium.

---

**4. `WalkieEventConceptEntry: EMFMinorFluctuationsIgnored`**

*   **Scenario Description (Recap):** Player is using the EMF Meter, and it's showing minor, non-evidence-level activity (e.g., `EMFMeter.emf_level` is `EMF2`, `EMF3`, or `EMF4` but not `EMF5`) in an area, but the player quickly moves on or switches gear without further investigation or noting the general activity.
*   **Goal of Hint:** Teach the player that even non-EMF5 readings can indicate ghost presence/activity and are useful for tracking or pinpointing areas of interest.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player is using an active `EMFMeter`.
    *   `EMFMeter.emf_level` registers as `EMF2`-`EMF4` for a brief period.
    *   Player then deactivates EMF, switches gear, or moves > Y distance away from that spot within Z (short) seconds.
    *   This pattern (brief minor EMF, then disengagement) happens N times in the mission without leading to stronger evidence.
    *   `WalkiePlay.can_play(WalkieEvent::EMFMinorFluctuationsIgnored, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (`EMFMeter` state), Player `Position` & movement history, counter for this specific pattern, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `EMFMinorFluctuationsIgnored`
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `NeutralObservation`, `PlayerStruggling` (if repeated often), `FirstTimeHint` (for interpreting minor EMF).
*   **Repetition Strategy:** Can play a few times per mission if player consistently ignores these subtle cues.
*   **Priority/Severity:** Medium-Low. Helps with nuanced investigation.

---












---

**RON File: `player_wellbeing.ron`**

**1. `WalkieEventConceptEntry: SanityDroppedBelowThresholdDarkness`**

*   **Scenario Description (Recap):** Player's sanity (`PlayerSprite.sanity()`) drops below a certain threshold (e.g., 70%) for the first time in the mission, and the primary contributing factor seems to be prolonged exposure to darkness (player has been in low `BoardData.light_field` areas for a significant portion of the time leading up to the sanity drop).
*   **Goal of Hint:** Inform the player about sanity loss in darkness and suggest using room lights or returning to the truck.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerSprite.sanity()` drops below (e.g., 70%) for the first time this mission (requires tracking a `sanity_threshold_warning_given_darkness` flag per mission).
    *   A heuristic determines darkness is the likely cause (e.g., average `BoardData.light_field[player_pos].lux` over the last X seconds has been below `DarkThreshold`, and ghost proximity/events have been minimal).
    *   `WalkiePlay.can_play(WalkieEvent::SanityDroppedBelowThresholdDarkness, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerSprite.sanity()`, `BoardData.light_field`, Player `Position` history (to average light exposure), mission-specific warning flag, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SanityDroppedBelowThresholdDarkness`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for this specific cause of sanity loss), `ConcernedWarning`, `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** Once per mission if primarily due to darkness.
*   **Priority/Severity:** Medium. Important for teaching core sanity mechanic.

---

**2. `WalkieEventConceptEntry: SanityDroppedBelowThresholdGhost`**

*   **Scenario Description (Recap):** Player's sanity drops below a certain threshold (e.g., 70%) for the first time, and the primary contributing factor appears to be recent ghost proximity or significant paranormal events (e.g., ghost manifestation, objects thrown, loud noises attributed to ghost).
*   **Goal of Hint:** Inform the player that ghost interactions drain sanity and suggest returning to the truck to recover.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerSprite.sanity()` drops below (e.g., 70%) for the first time this mission (requires tracking a `sanity_threshold_warning_given_ghost` flag per mission).
    *   A heuristic determines ghost activity is the likely cause (e.g., player was recently within X distance of `GhostSprite` for Y seconds, OR a major `GhostEvent` occurred nearby recently, and light levels have been adequate).
    *   `WalkiePlay.can_play(WalkieEvent::SanityDroppedBelowThresholdGhost, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerSprite.sanity()`, `GhostSprite.Position`, recent `GhostEvent` log, Player `Position` history, mission-specific warning flag, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `SanityDroppedBelowThresholdGhost`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint`, `ConcernedWarning`, `Guidance`.
*   **Repetition Strategy:** Once per mission if primarily due to ghost activity.
*   **Priority/Severity:** Medium.

---

**3. `WalkieEventConceptEntry: VeryLowSanityNoTruckReturn`**

*   **Scenario Description (Recap):** Player's sanity (`PlayerSprite.sanity()`) is critically low (e.g., < 30%), visual/audio "insanity" effects are likely active, and the player has not moved towards the van/exit or entered the truck for X seconds/minutes since sanity became critical.
*   **Goal of Hint:** Urgently warn the player about their critical sanity and strongly advise returning to the truck immediately.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerSprite.sanity() < CriticalSanityThreshold` (e.g., 30%).
    *   Timer `TimeSinceSanityCriticalNoTruckReturn` > X seconds (e.g., 20-30s). This timer starts/resets when sanity drops below critical or player enters truck.
    *   Player is inside the location (not in "VanArea").
    *   `WalkiePlay.can_play(WalkieEvent::VeryLowSanityNoTruckReturn, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerSprite.sanity()`, Player `Position`, "VanArea" definition, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `VeryLowSanityNoTruckReturn`
*   **Primary `WalkieTag`(s) for Line Selection:** `UrgentReminder` (or `ReminderHigh`), `ConcernedWarning`, `DirectHint`, `PlayerStruggling`.
*   **Repetition Strategy:** Can play multiple times with a shorter cooldown if sanity remains critical and player doesn't act.
*   **Priority/Severity:** Very High. Critical for player survival/mission success.

---

**4. `WalkieEventConceptEntry: LowHealthGeneralWarning`** (More for non-hunt damage, or post-hunt if player doesn't heal)

*   **Scenario Description (Recap):** Player's health (`PlayerSprite.health`) drops below a certain threshold (e.g., < 50% or < 30%) due to any reason (could be a lingering effect from a hunt, an environmental hazard if any exist, etc.), and they are not in the truck.
*   **Goal of Hint:** Advise the player that their health is low and they should consider returning to the truck to recover.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerSprite.health < LowHealthThreshold` (e.g., 40%).
    *   Player is inside the location (not in "VanArea").
    *   Timer `TimeSinceLowHealthNoTruckReturn` > X seconds (e.g., 15-20s). Timer resets if health recovers above threshold or player enters truck.
    *   This hint might have a lower priority or longer cooldown if a `VeryLowSanity` hint is also eligible, to avoid hint spam.
    *   `WalkiePlay.can_play(WalkieEvent::LowHealthGeneralWarning, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerSprite.health`, Player `Position`, "VanArea" definition, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `LowHealthGeneralWarning`
*   **Primary `WalkieTag`(s) for Line Selection:** `ConcernedWarning`, `Guidance`, `ReminderLow`.
*   **Repetition Strategy:** Once or twice per significant health drop if player doesn't recover.
*   **Priority/Severity:** Medium. Health is important, but sanity often drives more immediate threats.

---








---

**RON File: `mission_progression_and_truck.ron`**

**1. `WalkieEventConceptEntry: PlayerLeavesTruckWithoutChangingLoadout`**

*   **Scenario Description (Recap):** Player starts a mission (after their very first one, where loadouts become more customizable), enters the truck, then leaves the truck to enter the haunted location *without* having interacted with the "Loadout" tab in the `TruckUI`. This hint is more relevant if their previous mission's loadout was significantly different or if they have new gear available.
*   **Goal of Hint:** Gently remind the player that they can customize their gear via the loadout tab, especially if they seem to be repeatedly using a suboptimal or default kit.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None` (player just exited truck).
    *   This is *not* the player's absolute first mission (e.g., check `PlayerProfileData.statistics.total_missions_completed > 0` or a similar progression metric).
    *   A flag ` interacted_with_loadout_tab_this_truck_visit` is `false`. This flag is set to `true` when the Loadout tab is opened, and reset when the player leaves the truck.
    *   *Optional advanced condition:* The player's current `PlayerGear` is identical to the default starting gear for the difficulty OR identical to their loadout from the *previous* mission, AND new gear types are available in `TruckGear` that they haven't equipped.
    *   `WalkiePlay.can_play(WalkieEvent::PlayerLeavesTruckWithoutChangingLoadout, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerProfileData` (for mission count/progression), `TruckUI` tab interaction state (local flag), `PlayerGear`, `TruckGear` (available items in truck), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `PlayerLeavesTruckWithoutChangingLoadout`
*   **Primary `WalkieTag`(s) for Line Selection:** `FriendlyReminder`, `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** Infrequently. Perhaps once per mission if conditions are met, or if player consistently ignores new gear unlocks across several missions.
*   **Priority/Severity:** Low. This is more of a quality-of-life / optimization hint.

---

**2. `WalkieEventConceptEntry: AllObjectivesMetReminderToEndMission`**

*   **Scenario Description (Recap):** All primary mission objectives are complete (e.g., `GhostSprite` and `GhostBreach` are despawned/neutralized), the player has returned to the truck (`GameState::Truck`), but they haven't clicked the "End Mission" button in the `TruckUI` for X seconds/minutes.
*   **Goal of Hint:** Prompt the player to finalize the mission by clicking the "End Mission" button.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::Truck`.
    *   A global flag `MissionObjectivesComplete` is `true` (set when ghost and breach are gone).
    *   Timer `TimeSinceObjectivesCompleteInTruckNoEnd` > X seconds (e.g., 20-30s). This timer starts when player enters truck *after* objectives are complete.
    *   `WalkiePlay.can_play(WalkieEvent::AllObjectivesMetReminderToEndMission, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, global flag for `MissionObjectivesComplete`, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `AllObjectivesMetReminderToEndMission`
*   **Primary `WalkieTag`(s) for Line Selection:** `ReminderLow`, `DirectHint`, `PositiveReinforcement`.
*   **Repetition Strategy:** Can play a couple of times if player idles in the truck after completing objectives. Increasing cooldown.
*   **Priority/Severity:** Medium. Ensures player completes the mission loop.

---












---

**RON File: `tutorial_specific_flow.ron`**

**1. `WalkieEventConceptEntry: FirstMissionPlayerNotEnteringCabin`**

*   **Scenario Description (Recap):** It's the player's absolute first mission (e.g., `PlayerProfileData.statistics.total_missions_completed == 0` and `PlayerProfileData.achievements.expelled_first_ghost == false`). The player has started the mission, is in the van or immediate vicinity, and has not moved towards or interacted with the entrance to the actual haunted location for an extended period (e.g., 45-60 seconds).
*   **Goal of Hint:** Very gently guide the brand-new player to physically enter the mission area.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   This is the player's first ever mission (checked via `PlayerProfileData` stats/achievements).
    *   `Time.elapsed_seconds_since_level_ready > X` (e.g., 45-60s, a bit longer to allow initial fumbling).
    *   Player's `Position` is still within a small radius of their initial spawn point (or a designated "VanArea").
    *   Player has not interacted with the main entrance door of the location.
    *   `WalkiePlay.can_play(WalkieEvent::FirstMissionPlayerNotEnteringCabin, current_time)` returns true (this event should have a very high `time_to_play` after its first trigger, effectively making it play only once per profile).
*   **Key Game Data/Resources Needed:** `GameState`, `Time` (since `LevelReadyEvent`), Player `Position`, initial player spawn `Position`, "VanArea" definition, main entrance door interaction state, `PlayerProfileData`, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `FirstMissionPlayerNotEnteringCabin`
*   **Primary `WalkieTag`(s) for Line Selection:** `TutorialSpecific`, `FirstTimeHint`, `DirectHint`, `Encouraging`.
*   **Repetition Strategy:** Ideally, only once per player profile, ever. The `WalkiePlay.time_to_play` for this event should be set astronomically high after the first trigger, or a global "tutorial step completed" flag in `PlayerProfileData` should prevent it.
*   **Priority/Severity:** High (for the very first mission). Essential to get the player started.

---

**Considerations for `tutorial_specific_flow.ron`:**

*   **Minimalism:** This file should remain very small. Most tutorial hints, even for first-time mechanics, are better handled by `FirstTimeHint` tags within the functionally-categorized RON files (e.g., first time using EMF, first time seeing breach). This keeps `tutorial_specific_flow.ron` for only the absolute, unmissable, "player is literally not starting the game" scenarios.
*   **Profile-Wide Tracking:** The repetition strategy for hints in this file almost certainly needs to be tied to `PlayerProfileData` achievements or flags to ensure they don't replay for experienced players starting a new game or replaying tutorial maps. The `WalkiePlay`'s per-mission tracking might not be sufficient for "once ever" hints. This is a more advanced feature but important for this category. For an MVP, a very long `time_to_play` cooldown after the first trigger might suffice.

---
