State that we know exists:
=============================

Done:
------

`GameState` is `GameState::None` -> Use game_state: `Res<State<GameState>>` - in uncore.

Time.elapsed_seconds_since_level_ready -> BoardData.level_ready_time (use Res<Time> to compute the delta and know the actual seconds passed)

Player's `Position` is still within a small radius of their initial spawn point (or a designated "VanArea") -> PlayerSprite now includes spawn_position:Position.

`WalkiePlay.can_play(WalkieEvent::PlayerStuckAtStart, current_time)` -> this check is done automatically. We can forget on these.

"MainEntranceDoor" entity `Position` and `Behavior` -> Just check all doors by component, filter those that are inside a RoomDB and at the same time have a neighbor that is outside the room (no room). behavior::component::Door exists.

DarkRoomNoLightUsed: This event will now encompass scenarios where the player is in a dark room without using a light source, regardless of collision events. The logic will focus on light levels and the player's use of light sources.

BumpingInDarkness: The trigger here should be much easier, if the flashlight or equivalent is on, the area would be brighter already. There's no need to check the state of the flashlight. There's no need to query if the room has the lights on or off - we just need to check the exposure level.


Not Done:
-------------

Player `Direction` (or input state) -> For this we probably want some component to track the average length of the Direction vector over 5 seconds or so.

collision events with van boundaries -> For this we probably want a component to track the amount of collision (magnitude, float) over the last 5-10 seconds. We really don't care if it's a van boundary or not.

Breach visibility: We need to compute in maplight and store somewhere the current visibility of the breach, taking into account color brightness, transparency, and player visibility field.

Ghost visibility: We need to compute in maplight and store somewhere the current visibility of the ghost, taking into account transparency, and player visibility field.

Breach and Ghost reverse visibility trigger: When we trigger the voice line, we need to send some kind of event or state to make them be super visible for 10 seconds or so.

RoomLightsOnGearNeedsDark: Probably we need to make the gear inform that they need darkness somehow; locating what gear is used is tricky, so it's easier if the gear pushes the state somewhere. But only if it's active and on hand, not stowed or on the ground. Knowing that the room is lit might be a bit trickier? first of all, which room? the gear needs to explain what is the target here - is it the breach or the ghost? we could filter by knowing if we have the correct target visible; and then locating in which room we are and looking up the light state; but it would be even better if we look at the cached lighting, because that includes other stuff that aren't the lights of the room.

GearSelectedNotActivated: This seems it would benefit from using the trait we have for gear, we need to know if the item is enabled, if it is gear, and if it can be enabled. Probably we need to tweak the trait here.



Other:
-------------

StrugglingWithGrabDrop: This trigger is quite unclear on how to really implement - the voice lines suggest plenty of different stuff for different scenarios. We need to leave this for later on.

StrugglingWithHideUnhide: This needs more work. Probably this is several separate triggers; "Struggling" is too generic here.
  - event when the player should be looking for a place to hide.
  - event for when the player went out of hiding too early.
  - event for when the player tries to hide in places that are not valid.

DarkRoomNoLightUsed: Isn't this just a variation of "BumpingInDarkness"?

IgnoredObviousBreach: We can't know when the player ignores the breach. All we need to do is to point out when the breach becomes highly visible.

WildFlashlightSweeping: I don't get this one. Doesn't feel applicable. The flashlight is very wide, the player naturally when moving will swing it. This feels like it needs to be removed.

FlashlightOnInLitRoom: This one can be done much simpler by looking at the lighting data (the cached one, the permanent one), and seeing how lit is the tile we're on before applying the flashlight on top. That's enough to do this.



--- environmental_awareness

*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `PlayerGear`, `Flashlight.status`, `BoardData.light_field`, `RoomDB`, state of `Interactive` light switches in the current room, local timer, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `Direction`, `GhostBreach` entity `Position` & `Visibility`, `VisibilityData`, `WalkiePlay`, current mission/tutorial stage.

*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `Direction` & `PlayerGear` (to check for gear activation), `GhostSprite` `Position` & visibility state & `hunting` state, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` & `Flashlight.status`, `PlayerSprite.direction` (and its rate of change), `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` (and specific gear states), `RoomDB` (or `Interactive` light switch states), `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` & `Flashlight.status`, `RoomDB`, `BoardData.light_field`, `WalkiePlay`.

--- basic_gear_usage

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and internal states of specific gear like `Thermometer.enabled`, `Flashlight.status`), local timer, `WalkiePlay`, potentially `BoardData.light_field`, `RoomDB`.

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, `Thermometer.enabled` & `Thermometer.temp`, `RoomDB` (for ghost room context), local timer for this state, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, `EMFMeter.enabled` & `EMFMeter.emf_level`, `RoomDB`, local timer, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and states of Thermometer/EMF), `RoomDB`, breach `Position`, local timer, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, local timer for gear cycling, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `Difficulty`, `PlayerGear` (and specific gear states), local timers per new gear type, `WalkiePlay`, potentially `RoomDB`/`BoardData` for context.

--- evidence_gathering_and_logic

*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and specific gear states like `Thermometer.temp`, `EMFMeter.emf_level`, `Recorder.evp_recorded_display`), local timer, current mission's evidence state (from `TruckUI` or a shared resource), `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, Player `Position`, `RoomDB`, internal flag for new evidence, local timer, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab state, internal count/flag of evidence found, local timer, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab & evidence states, journal's internal ghost filtering logic result, `WalkiePlay`.

*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab & evidence states, `GhostGuess` resource, journal's internal ghost filtering logic result, local timer, `WalkiePlay`.




Trigger events for voices
==============================


**RON File: `locomotion_and_interaction.ron`**

**4. `WalkieEventConceptEntry: BumpingInDarkness`**

*   **Scenario Description (Recap):** Player is in a dark room (`BoardData.light_field` at player's `Position` is very low), flashlight is OFF (`PlayerGear.left_hand` or `right_hand` is Flashlight but `Flashlight.status == Off`), room lights are OFF (checked via `RoomDB` or interactive light switches in the room), AND player has had multiple collision events with static objects in a short time.
*   **Goal of Hint:** Strongly suggest the player use a light source (flashlight or room lights) to navigate.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `BoardData.light_field[player_bpos].lux < DarkThreshold`.
    *   Player's equipped Flashlight `Flashlight.status == Off` (if flashlight is equipped).
    *   Current room's `RoomDB.room_state` indicates lights are off (or query local light switches).
    *   Player has accumulated N collision events with non-dynamic `Behavior` entities within the last Y seconds.
    *   `WalkiePlay.can_play(WalkieEvent::BumpingInDarkness, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `BoardData`, Player `Position` & `PlayerGear`, `Flashlight.status` (if equipped), `RoomDB` (or `Interactive` light switch states), collision event tracker, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `BumpingInDarkness`
*   **Primary `WalkieTag`(s) for Line Selection:** `PlayerStruggling`, `DirectHint`, `SnarkyHumor`, `ConcernedWarning`.
*   **Repetition Strategy:** Can play multiple times per mission if the situation reoccurs, but with an increasing cooldown.
*   **Priority/Severity:** High. Prevents frustration and is important for basic gameplay.

---

**5. `WalkieEventConceptEntry: StrugglingWithGrabDrop`** (Assuming this is introduced around Chapter 2/3)

*   **Scenario Description (Recap):** Player is near a clearly "pickable" small object (`Behavior.p.object.pickable == true`, low `Behavior.p.object.weight`), perhaps even looking at it, but doesn't use `[F]` for X seconds; OR player is holding an item and tries to activate gear (fails) or bumps into things.
*   **Goal of Hint:** Guide the player on how to use `[F]` to pick up and `[G]` to drop items, and the implication of holding items (slower, can't use gear).
*   **Trigger Logic (Conceptual):**
    *   *Not Picking Up:* Player `Position` near pickable `Entity` with `Behavior.p.object.pickable == true` for X seconds. Player's view cone might also be checked. `PlayerGear.held_item` is `None`.
    *   *Holding and Failing Action:* `PlayerGear.held_item` is `Some`. Player attempts to use `[R]` or `[Tab]` (gear activation) OR player collides N times.
    *   `WalkiePlay.can_play(WalkieEvent::StrugglingWithGrabDrop, current_time)` returns true.
    *   This event would likely only trigger if the "Grab/Drop" mechanic has been "unlocked" or is part of the current tutorial chapter.
*   **Key Game Data/Resources Needed:** Player `Position`, `PlayerGear`, nearby entity `Behavior.p.object` properties, input state for `[R]`/`[Tab]`, collision events, `WalkiePlay`, current tutorial chapter/unlocked mechanics.
*   **`WalkieEvent` Enum Variant:** `StrugglingWithGrabDrop`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for initial encounters), `Guidance`, `PlayerStruggling`, `ReminderLow`.
*   **Repetition Strategy:** A few times per mission if player repeatedly struggles. Cooldown.
*   **Priority/Severity:** Medium (becomes more important once object interaction puzzles are introduced).

---

**6. `WalkieEventConceptEntry: StrugglingWithHideUnhide`** (Assuming this is introduced around Chapter 2)

*   **Scenario Description (Recap):**
    *   *Not Hiding During Hunt:* Ghost is actively hunting, player is near a valid `Behavior.p.object.hidingspot == true` but doesn't press and hold `[E]`.
    *   *Immediately Unhiding:* Player successfully hides, but presses `[E]` again very quickly while hunt is still active.
    *   *Trying to Hide While Carrying:* Player attempts to `[E]` interact with a hiding spot while `PlayerGear.held_item` is `Some`.
*   **Goal of Hint:** Guide player on how/when to hide and unhide effectively, and limitations.
*   **Trigger Logic (Conceptual):**
    *   *Not Hiding:* `GhostSprite.hunting > 0`, Player `Position` near `Entity` with `Behavior.p.object.hidingspot == true`, no `[E]` hold detected for X seconds.
    *   *Immediately Unhiding:* Player gains `Hiding` component, then loses it within Y (very short, e.g., < 2) seconds while `GhostSprite.hunting > 0`.
    *   *Trying to Hide While Carrying:* Player presses `[E]` near a hiding spot, `PlayerGear.held_item` is `Some`.
    *   `WalkiePlay.can_play(WalkieEvent::StrugglingWithHideUnhide, current_time)` returns true.
    *   Mechanic unlocked/current tutorial chapter.
*   **Key Game Data/Resources Needed:** `GhostSprite.hunting` state, Player `Position` & `PlayerGear.held_item` & input state for `[E]`, `Hiding` component presence, `Behavior.p.object.hidingspot`, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `StrugglingWithHideUnhide`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint`, `Guidance`, `PlayerStruggling`, `ConcernedWarning` (for not hiding during hunt).
*   **Repetition Strategy:** Can repeat if mistakes are made, with cooldowns.
*   **Priority/Severity:** High during hunts, Medium otherwise.









---






**RON File: `environmental_awareness.ron`**

**1. `WalkieEventConceptEntry: DarkRoomNoLightUsed`**

*   **Scenario Description (Recap):** Player is inside a location, in a dark room (low `BoardData.light_field` lux at player's `Position`), and hasn't activated their flashlight or a room light switch for X seconds.
*   **Goal of Hint:** Encourage the player to use a light source to improve visibility.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player is in a room (checked via `RoomDB`).
    *   `BoardData.light_field[player_bpos].lux < DarkThreshold` (e.g., < 0.1 lux).
    *   Player's equipped Flashlight `Flashlight.status == Off` (if flashlight is equipped in `PlayerGear`).
    *   Current room's `RoomDB.room_state` indicates lights are off (or query `Interactive` light switches in the room, ensuring they are `TileState::Off`).
    *   Timer `DarkRoomNoLightTimer` exceeds X seconds (e.g., 10-15s). This timer resets if player enters a lit area or activates a light.
    *   `WalkiePlay.can_play(WalkieEvent::DarkRoomNoLightUsed, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `PlayerGear`, `Flashlight.status`, `BoardData.light_field`, `RoomDB`, state of `Interactive` light switches in the current room, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `DarkRoomNoLightUsed`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint`, `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** Can play once or twice per mission if player repeatedly enters dark areas without using light. Increasing cooldown.
*   **Priority/Severity:** Medium-High. Important for basic gameplay and evidence spotting.

---

**2. `WalkieEventConceptEntry: IgnoredObviousBreach`**

*   **Scenario Description (Recap):** Player is in a lit room (or has flashlight on), the `GhostBreach` entity is visible and relatively close, but the player's view cone (derived from `PlayerSprite.direction`) hasn't significantly intersected with the breach's screen position for X seconds.
*   **Goal of Hint:** Draw the player's attention to the visual manifestation of the ghost's breach.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostBreach` entity exists and is "visible" (not occluded by walls from player's perspective, `VisibilityData` for breach position > threshold).
    *   Player `Position` is within Y distance of `GhostBreach` `Position` (e.g., < 5-7 units).
    *   Player's calculated view cone has not aimed at/near the `GhostBreach` for X seconds (e.g., 15-20s). This requires comparing player direction with direction to breach.
    *   `WalkiePlay.can_play(WalkieEvent::IgnoredObviousBreach, current_time)` returns true.
    *   This hint might be disabled for the very first tutorial mission where the breach concept might not be formally introduced yet.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `Direction`, `GhostBreach` entity `Position` & `Visibility`, `VisibilityData`, `WalkiePlay`, current mission/tutorial stage.
*   **`WalkieEvent` Enum Variant:** `IgnoredObviousBreach`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for first time seeing it), `Guidance`, `ContextualHint`, `NeutralObservation`.
*   **Repetition Strategy:** Once per mission, or until player clearly interacts with/looks at the breach.
*   **Priority/Severity:** Medium. Important for understanding a core game element.

---

**3. `WalkieEventConceptEntry: IgnoredVisibleGhost`**

*   **Scenario Description (Recap):** The `GhostSprite` becomes visible (e.g., during a manifestation event, not necessarily a hunt), is relatively close to the player, but the player doesn't react (e.g., turn towards it, use gear, or move away) for X seconds.
*   **Goal of Hint:** Alert the player to the ghost's presence and encourage observation or appropriate reaction.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `GhostSprite` entity exists and its `Sprite.color.alpha > VisibleThreshold` (or a dedicated `is_manifested` flag on `GhostSprite`).
    *   Player `Position` is within Y distance of `GhostSprite` `Position` (e.g., < 6-8 units).
    *   Player's `Direction` has not significantly changed towards the ghost, AND no gear has been activated for X seconds (e.g., 5-7s) since ghost became visible.
    *   `GhostSprite.hunting == 0.0` (i.e., not a hunt).
    *   `WalkiePlay.can_play(WalkieEvent::IgnoredVisibleGhost, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `Direction` & `PlayerGear` (to check for gear activation), `GhostSprite` `Position` & visibility state & `hunting` state, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `IgnoredVisibleGhost`
*   **Primary `WalkieTag`(s) for Line Selection:** `NeutralObservation`, `Questioning`, `Guidance`, `ImmediateResponse`.
*   **Repetition Strategy:** Can play if the scenario repeats, but with a cooldown.
*   **Priority/Severity:** Medium. Ghost sightings are key moments.

---

**4. `WalkieEventConceptEntry: WildFlashlightSweeping`**

*   **Scenario Description (Recap):** Player has the Flashlight active but is changing their `PlayerSprite.direction` very rapidly and across wide arcs for X seconds, indicating they are not effectively illuminating or searching.
*   **Goal of Hint:** Advise the player to use slower, more deliberate flashlight movements for better observation.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player has Flashlight equipped and `Flashlight.status != Off`.
    *   A system tracks the angular velocity or frequency of large changes in `PlayerSprite.direction`. If this exceeds a threshold for X seconds (e.g., 5-7s).
    *   `WalkiePlay.can_play(WalkieEvent::WildFlashlightSweeping, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` & `Flashlight.status`, `PlayerSprite.direction` (and its rate of change), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `WildFlashlightSweeping`
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `PlayerStruggling`, `Humorous` (or `SnarkyHumor` for later reminders).
*   **Repetition Strategy:** Once or twice per mission if behavior persists.
*   **Priority/Severity:** Low. A minor gameplay refinement hint.

---

**5. `WalkieEventConceptEntry: RoomLightsOnGearNeedsDark`**

*   **Scenario Description (Recap):** Player activates a piece of gear that requires darkness for optimal use (e.g., Video Camera for Orbs, UV Torch for certain evidence types if that becomes a mechanic) while the current room's lights are ON.
*   **Goal of Hint:** Inform the player that the current room lighting might interfere with their gear's effectiveness.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player activates specific gear (e.g., `PlayerGear` changes to `GearKind::Videocam` and `Videocam.enabled == true`).
    *   The activated `GearKind` is known to require darkness (internal list/property).
    *   Current room's `RoomDB.room_state` indicates lights are ON (or query `Interactive` light switches in the room).
    *   `WalkiePlay.can_play(WalkieEvent::RoomLightsOnGearNeedsDark, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` (and specific gear states), `RoomDB` (or `Interactive` light switch states), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `RoomLightsOnGearNeedsDark`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for the specific gear), `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** Can play if player repeats this with different "darkness-required" gear types. Cooldown per gear type or globally.
*   **Priority/Severity:** Medium. Important for finding certain evidence types.

---

**6. `WalkieEventConceptEntry: FlashlightOnInLitRoom`**

*   **Scenario Description (Recap):** Player turns on their flashlight in a room where the main room lights (via light switch) are already on and providing sufficient illumination.
*   **Goal of Hint:** Gently suggest conserving flashlight battery when room lights are adequate.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player activates Flashlight (`Flashlight.status` changes to not `Off`).
    *   Current room's `RoomDB.room_state` indicates lights are ON.
    *   `BoardData.light_field[player_bpos].lux > BrightThreshold` (e.g., > 1.0 lux from non-player sources).
    *   `WalkiePlay.can_play(WalkieEvent::FlashlightOnInLitRoom, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `PlayerGear` & `Flashlight.status`, `RoomDB`, `BoardData.light_field`, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `FlashlightOnInLitRoom`
*   **Primary `WalkieTag`(s) for Line Selection:** `FriendlyReminder`, `Guidance`, `NeutralObservation`.
*   **Repetition Strategy:** Can play occasionally if the player frequently does this.
*   **Priority/Severity:** Low. Minor optimization hint.

---






---

**RON File: `basic_gear_usage.ron`**

**1. `WalkieEventConceptEntry: GearSelectedNotActivated`**

*   **Scenario Description (Recap):** Player has a piece of starting/basic gear (e.g., Thermometer, EMF Meter, Flashlight) selected in their active hand but hasn't pressed the activate button (`[R]` or `[Tab]`) for X seconds, especially when in a relevant context (e.g., in a potentially haunted room, or it's dark for the flashlight).
*   **Goal of Hint:** Remind the player that most gear needs to be actively turned on to function.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player's `PlayerGear.right_hand.kind` OR `PlayerGear.left_hand.kind` is one of {`Thermometer`, `EMFMeter`, `Flashlight`, etc. - any gear that has an on/off state}.
    *   The corresponding gear's internal state (e.g., `Thermometer.enabled == false`, `Flashlight.status == Off`) indicates it's off.
    *   A timer specific to "inactive selected gear" exceeds X seconds (e.g., 7-10s). This timer might reset if player activates gear or switches gear.
    *   *Optional Context:* For tools like Thermometer/EMF, this might trigger more readily if player is in a room with known cold spots or near the breach. For Flashlight, if `BoardData.light_field[player_bpos].lux < DarkThreshold`.
    *   `WalkiePlay.can_play(WalkieEvent::GearSelectedNotActivated, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and internal states of specific gear like `Thermometer.enabled`, `Flashlight.status`), local timer, `WalkiePlay`, potentially `BoardData.light_field`, `RoomDB`.
*   **`WalkieEvent` Enum Variant:** `GearSelectedNotActivated`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint`, `Guidance`, `ReminderLow`.
*   **Repetition Strategy:** Can play a couple of times per mission if player repeatedly forgets. Cooldown.
*   **Priority/Severity:** Medium-High for core tools.

---

**2. `WalkieEventConceptEntry: ThermometerNonFreezingFixation`**

*   **Scenario Description (Recap):** Player is using the Thermometer, it's showing cold temperatures (e.g., 1-10°C) but not sub-zero (Freezing Temps evidence). Player lingers in the area for X seconds or sweeps multiple nearby tiles with the Thermometer without getting definitive Freezing evidence and without switching to other gear like EMF.
*   **Goal of Hint:** Guide the player to understand that "cold" isn't "freezing" for evidence, and suggest trying other tools or recognizing this might not be the evidence type.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerGear.right_hand.kind == GearKind::Thermometer` (or left hand) AND `Thermometer.enabled == true`.
    *   `Thermometer.temp` is consistently > 0°C but < (e.g., 10°C) for X seconds (e.g., 20-30s) in a suspected ghost room or near breach.
    *   Player has not switched active gear or marked other evidence during this period.
    *   `WalkiePlay.can_play(WalkieEvent::ThermometerNonFreezingFixation, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, `Thermometer.enabled` & `Thermometer.temp`, `RoomDB` (for ghost room context), local timer for this state, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `ThermometerNonFreezingFixation`
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `NeutralObservation`, `PlayerStruggling`, `SuggestStopThermometer` (potentially `SuggestUseEMFMeter` if that's the logical next step).
*   **Repetition Strategy:** Once or twice per "fixation" period.
*   **Priority/Severity:** Medium. Helps prevent players from getting stuck on ambiguous readings.

---

**3. `WalkieEventConceptEntry: EMFNonEMF5Fixation`**

*   **Scenario Description (Recap):** Player is using the EMF Meter, it's showing activity (EMF 2-4) but not EMF Level 5 evidence. Player lingers or sweeps with EMF for X seconds without getting definitive EMF5 and without switching gear.
*   **Goal of Hint:** Guide the player that while activity is noted, EMF5 is the specific evidence, and suggest trying other tools or recognizing this might not be the evidence type.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   `PlayerGear.right_hand.kind == GearKind::EMFMeter` (or left hand) AND `EMFMeter.enabled == true`.
    *   `EMFMeter.emf_level` is consistently one of {`EMF2`, `EMF3`, `EMF4`} (but not `EMF5` or `None`) for X seconds (e.g., 20-30s) in a suspected ghost room or near breach.
    *   Player has not switched active gear or marked other evidence during this period.
    *   `WalkiePlay.can_play(WalkieEvent::EMFNonEMF5Fixation, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, `EMFMeter.enabled` & `EMFMeter.emf_level`, `RoomDB`, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `EMFNonEMF5Fixation`
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `NeutralObservation`, `PlayerStruggling`, `SuggestStopEMFMeter` (potentially `SuggestUseThermometer`).
*   **Repetition Strategy:** Once or twice per "fixation" period.
*   **Priority/Severity:** Medium.

---

**4. `WalkieEventConceptEntry: DidNotSwitchStartingGearInHotspot`**

*   **Scenario Description (Recap):** Player gets a *promising but not definitive* reading with one starting tool (e.g., Thermometer shows 5°C, or EMF shows Level 3) in a "hotspot" (near breach, in ghost room) but doesn't switch to the *other primary starting tool* (EMF if using Thermometer, or vice-versa) within X seconds to cross-reference.
*   **Goal of Hint:** Encourage the player to use their *other* basic tool in an area that's already showing some activity.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player is in ghost room/near breach.
    *   *Condition A:* `PlayerGear` has active `Thermometer` with non-freezing cold reading for X seconds, AND `EMFMeter` is available (in hand or inventory) but not used in this spot recently.
    *   *OR Condition B:* `PlayerGear` has active `EMFMeter` with EMF 2-4 reading for X seconds, AND `Thermometer` is available but not used in this spot recently.
    *   `WalkiePlay.can_play(WalkieEvent::DidNotSwitchStartingGearInHotspot, current_time)` returns true.
    *   The specific event variant sent might include which tool is currently active, to allow for more specific tagged lines (e.g., `SuggestUseEMFMeter` if Thermometer is active).
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and states of Thermometer/EMF), `RoomDB`, breach `Position`, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `DidNotSwitchStartingGearInHotspot` (Could potentially be split into `HotspotThermometerSuggestEMF` and `HotspotEMFSuggestThermometer` if finer control is needed, but primary tags can handle this).
*   **Primary `WalkieTag`(s) for Line Selection:** `Guidance`, `ContextualHint`, `Encouraging`, `SuggestUseThermometer`, `SuggestUseEMFMeter`.
*   **Repetition Strategy:** Once per distinct hotspot encounter where this occurs.
*   **Priority/Severity:** Medium. Promotes good investigation habits.

---

**5. `WalkieEventConceptEntry: DidNotCycleToOtherGear`**

*   **Scenario Description (Recap):** Player has been actively using one or two specific pieces of gear for an extended period (e.g., 1-2 minutes) in the location without pressing `[Q]` to cycle to other available gear in their inventory (especially if they have more than just two items, or if one hand is empty and they have backpack items).
*   **Goal of Hint:** Remind the player they have other tools in their inventory accessible via the cycle key.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Timer `TimeSinceLastGearCycleOrSwap` > X minutes (e.g., 1.5-2 mins). This timer resets when `[Q]` or `[T]` is pressed.
    *   Player has >2 usable items in `PlayerGear` (across hands and inventory), OR one hand is `GearKind::None` and inventory has items.
    *   Player has been actively using their current gear (e.g., `enabled` flags are true, or there's recent activation).
    *   `WalkiePlay.can_play(WalkieEvent::DidNotCycleToOtherGear, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear`, local timer for gear cycling, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `DidNotCycleToOtherGear`
*   **Primary `WalkieTag`(s) for Line Selection:** `ReminderLow`, `Guidance`.
*   **Repetition Strategy:** Infrequently per mission if player seems to forget about cycling.
*   **Priority/Severity:** Low-Medium.

---

**6. `WalkieEventConceptEntry: AdvancedGearUnusedLongTime`** (This is a placeholder for now, as "advanced gear" is relative to the chapter. The *actual* events would be more specific, e.g., `RecorderUnusedInChapter3`, `SaltUnusedInChapter5`).

*   **Scenario Description (Recap):** Player has unlocked/been given a new piece of "advanced" gear relevant to the current chapter/difficulty, has it in their inventory, but has been in the location for a significant time (e.g., X minutes) without equipping or using it, especially if current conditions might call for it.
*   **Goal of Hint:** Gently remind the player about their new tool and suggest trying it out.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Current `Difficulty.tutorial_chapter` indicates a certain set of "new" gear.
    *   Player has one of these "new" `GearKind` in `PlayerGear.inventory` (or even hands, but `enabled == false`).
    *   Timer `TimeSinceNewGearAcquiredAndUnusedInLocation` > X minutes.
    *   *Optional Context:* Specific game conditions might make the hint more relevant (e.g., if Spirit Box is new and player is in a dark room near breach).
    *   `WalkiePlay.can_play(WalkieEvent::AdvancedGearUnused[SpecificGearName], current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `Difficulty`, `PlayerGear` (and specific gear states), local timers per new gear type, `WalkiePlay`, potentially `RoomDB`/`BoardData` for context.
*   **`WalkieEvent` Enum Variant:** Would be specific, e.g., `RecorderUnused`, `SpiritBoxUnused`. For now, `AdvancedGearUnusedLongTime` is a conceptual placeholder.
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for that gear), `ReminderLow`, `Guidance`, `Encouraging`.
*   **Repetition Strategy:** Once or twice for each *new* piece of gear if unused for a long time after being introduced.
*   **Priority/Severity:** Medium, as it relates to learning new core mechanics.

---





---

**RON File: `evidence_gathering_and_logic.ron`**

**1. `WalkieEventConceptEntry: ClearEvidenceFoundNoActionCKey`**

*   **Scenario Description (Recap):** Player's active gear clearly indicates a piece of evidence (e.g., Thermometer shows <0°C, EMF Meter shows EMF5, Recorder shows "EVP RECORDED", etc.), but the player does not press the `[C]` key (Change Evidence/Log Quick Evidence) within X seconds.
*   **Goal of Hint:** Remind the player they can log evidence directly from their gear for convenience.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player's active `PlayerGear` (right or left hand) is in a state that clearly signifies a valid piece of evidence (e.g., `Thermometer.temp < 0.0`, `EMFMeter.emf_level == EMFLevel::EMF5`, `Recorder.evp_recorded_display == true`, etc.).
    *   Timer `TimeSinceDefinitiveEvidenceOnGear` > X seconds (e.g., 7-10s). This timer resets if `[C]` is pressed or gear state changes away from definitive evidence.
    *   The specific evidence type has *not* yet been marked as "Found" in the `TruckUI`'s internal evidence state for this mission attempt (to avoid redundant hints if already logged via truck).
    *   `WalkiePlay.can_play(WalkieEvent::ClearEvidenceFoundNoActionCKey, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (and specific gear states like `Thermometer.temp`, `EMFMeter.emf_level`, `Recorder.evp_recorded_display`), local timer, current mission's evidence state (from `TruckUI` or a shared resource), `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `ClearEvidenceFoundNoActionCKey`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for using `[C]`), `Guidance`, `PositiveReinforcement`.
*   **Repetition Strategy:** Once or twice per mission if player repeatedly finds evidence and doesn't use `[C]`.
*   **Priority/Severity:** Medium. `[C]` key is a convenience.

---

**2. `WalkieEventConceptEntry: ClearEvidenceFoundNoActionTruck`**

*   **Scenario Description (Recap):** Player has found one or more pieces of clear evidence (either logged with `[C]` or just observed), has been in the location for a significant time afterwards (e.g., X minutes), and has not returned to the truck to use the journal.
*   **Goal of Hint:** Encourage the player to return to the truck to log evidence in the journal, which helps in ghost identification.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player is inside the location (checked via `RoomDB`).
    *   A flag `NewEvidenceFoundSinceLastTruckVisit` is true (set when evidence is found/logged via `[C]`).
    *   Timer `TimeSinceLastTruckVisitWithNewEvidence` > X minutes (e.g., 2-3 mins). This timer resets when player enters truck.
    *   `WalkiePlay.can_play(WalkieEvent::ClearEvidenceFoundNoActionTruck, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position`, `RoomDB`, internal flag for new evidence, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `ClearEvidenceFoundNoActionTruck`
*   **Primary `WalkieTag`(s) for Line Selection:** `ReminderMedium`, `Guidance`, `DelayedObservation`.
*   **Repetition Strategy:** Once per "batch" of unlogged evidence if player stays out too long.
*   **Priority/Severity:** Medium. Journal use is key.

---

**3. `WalkieEventConceptEntry: InTruckWithEvidenceNoJournal`**

*   **Scenario Description (Recap):** Player is in the truck (`GameState::Truck`), has found at least one piece of evidence during the current mission (internal flag/counter is > 0), but has not switched to or interacted with the Journal tab in the `TruckUI` for X seconds.
*   **Goal of Hint:** Guide the player to use the Journal tab in the truck to log their findings and identify the ghost.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::Truck`.
    *   Flag `NewEvidenceFoundSinceLastTruckVisit` is true OR total evidence found in mission > 0.
    *   Current active `TruckUI.TabContents` is *not* `TabContents::Journal`.
    *   Timer `TimeInTruckWithoutJournalInteraction` > X seconds (e.g., 15-20s). Timer resets if Journal tab is selected.
    *   `WalkiePlay.can_play(WalkieEvent::InTruckWithEvidenceNoJournal, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab state, internal count/flag of evidence found, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `InTruckWithEvidenceNoJournal`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for journal usage), `Guidance`, `ContextualHint`.
*   **Repetition Strategy:** Once per truck visit if conditions met.
*   **Priority/Severity:** Medium-High. Journal is crucial.

---

**4. `WalkieEventConceptEntry: JournalConflictingEvidence`**

*   **Scenario Description (Recap):** Player has marked a combination of "Found" and/or "Discarded" evidence in the `TruckUI` Journal such that the list of possible ghosts becomes zero.
*   **Goal of Hint:** Alert the player that their current evidence selection is contradictory and they need to re-evaluate.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::Truck`.
    *   Current active `TruckUI.TabContents` is `TabContents::Journal`.
    *   The internal logic of the journal (after player clicks an evidence button) results in `possible_ghosts.len() == 0`.
    *   This event should fire immediately after the action that causes the conflict.
    *   `WalkiePlay.can_play(WalkieEvent::JournalConflictingEvidence, current_time)` returns true (might have a short immediate cooldown to prevent spam if player clicks rapidly).
*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab & evidence states, journal's internal ghost filtering logic result, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `JournalConflictingEvidence`
*   **Primary `WalkieTag`(s) for Line Selection:** `ConcernedWarning`, `PlayerStruggling`, `Guidance`, `ImmediateResponse`.
*   **Repetition Strategy:** Can play each time a conflict is created.
*   **Priority/Severity:** High. Player is logically stuck.

---

**5. `WalkieEventConceptEntry: JournalPointsToOneGhostNoCraft`**

*   **Scenario Description (Recap):** Player is in the `TruckUI` Journal tab, and the selected/discarded evidence has narrowed the possible ghost types down to exactly one, but the player doesn't click the "Craft Repellent" button for X seconds.
*   **Goal of Hint:** Prompt the player to proceed with crafting the repellent now that a single ghost type is identified.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::Truck`.
    *   Current active `TruckUI.TabContents` is `TabContents::Journal`.
    *   Journal's internal ghost filtering logic results in `possible_ghosts.len() == 1` (and `GhostGuess.ghost_type` is updated to this single ghost).
    *   Timer `TimeSinceSingleGhostIdentifiedNoCraft` > X seconds (e.g., 10-15s). Timer resets if "Craft Repellent" is clicked or evidence changes.
    *   `WalkiePlay.can_play(WalkieEvent::JournalPointsToOneGhostNoCraft, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `TruckUI` active tab & evidence states, `GhostGuess` resource, journal's internal ghost filtering logic result, local timer, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `JournalPointsToOneGhostNoCraft`
*   **Primary `WalkieTag`(s) for Line Selection:** `PositiveReinforcement`, `DirectHint`, `Guidance`.
*   **Repetition Strategy:** Once per identification if player hesitates.
*   **Priority/Severity:** Medium-High. Next step in progression.

---

**(Placeholders for specific evidence types - these would ideally be covered by a more generic `ClearEvidenceFoundNoActionCKey` or specific events if needed, but included for completeness of your earlier list):**

**6. `WalkieEventConceptEntry: SpiritBoxResponseNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` if SpiritBox sets a clear "EVP detected" state that can be queried)

**7. `WalkieEventConceptEntry: RLPresenceFoundNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` if Red Torch interaction sets a clear "RL Presence detected" state)

**8. `WalkieEventConceptEntry: UVTraceFoundNoFollow`** (This is more about *acting* on UV, fits better in `environmental_awareness.ron` or a new `tracking_and_observation.ron`)
    *   **Scenario Description:** Player illuminates UV traces (salt footprints, handprints) with UV Torch but doesn't move along the indicated path for X seconds.
    *   **Goal of Hint:** Encourage player to follow the UV trail.
    *   **Trigger Logic:** Player has `UVTorch` active, `UVReactive` entities are visible nearby, player's `Position` doesn't change significantly along the trail's apparent direction for X seconds.
    *   **`WalkieEvent` Enum Variant:** `UVTraceFoundNoFollow`
    *   **Primary Tags:** `Guidance`, `ContextualHint`, `PlayerStruggling`.
    *   *Decision: This feels distinct enough to keep separate from general C-key logging. It's about *acting* on visual info.*

**9. `WalkieEventConceptEntry: EVPRecordedNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` when `Recorder.evp_recorded_display == true`)

**10. `WalkieEventConceptEntry: CPM500FoundNoAction`**
    *   (Covered by `ClearEvidenceFoundNoActionCKey` when `GeigerCounter.sound_display > 500`)
















---

**RON File: `repellent_and_expulsion.ron`**

**1. `WalkieEventConceptEntry: HasRepellentEntersLocation`**

*   **Scenario Description (Recap):** Player has successfully crafted a ghost-specific repellent in the truck and then enters the haunted location with it equipped or in inventory.
*   **Goal of Hint:** Remind the player about the repellent's purpose and the need to get close to the ghost/breach for it to be effective.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player `PlayerGear` contains a `GearKind::RepellentFlask` where `RepellentFlask.liquid_content` is `Some(GhostType)`.
    *   Player crosses a threshold from "VanArea" into the "HauntedLocation" (based on `RoomDB` or coordinate boundaries).
    *   This is the first time player enters the location *after crafting this specific repellent batch*.
    *   `WalkiePlay.can_play(WalkieEvent::HasRepellentEntersLocation, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, `PlayerGear` (specifically `RepellentFlask` state), Player `Position`, `RoomDB` (or area definitions), flag/timestamp for "last repellent crafted," `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `HasRepellentEntersLocation`
*   **Primary `WalkieTag`(s) for Line Selection:** `FirstTimeHint` (for this repellent batch), `Guidance`, `Encouraging`.
*   **Repetition Strategy:** Once per repellent crafting.
*   **Priority/Severity:** Medium. Reinforces correct usage.

---

**2. `WalkieEventConceptEntry: RepellentUsedTooFar`**

*   **Scenario Description (Recap):** Player activates the `RepellentFlask` (`RepellentFlask.active == true`), but their `Position` is too far from the `GhostSprite.Position` or `GhostBreach.Position` for the repellent particles to likely have an effect.
*   **Goal of Hint:** Advise the player to get closer to the target for the repellent to be effective.
*   **Trigger Logic (Conceptual):**
    *   `GameState` is `GameState::None`.
    *   Player's active `RepellentFlask.active == true` (and `qty > 0`).
    *   `Player.Position.distance(Ghost.Position) > MaxEffectiveRepellentRange` (e.g., > 3-4 units) AND/OR `Player.Position.distance(Breach.Position) > MaxEffectiveRepellentRange`.
    *   This check is performed shortly after repellent activation or if player moves away while it's active.
    *   No significant `GhostSprite.repellent_hits_frame` is being registered (indicating misses).
    *   `WalkiePlay.can_play(WalkieEvent::RepellentUsedTooFar, current_time)` returns true.
*   **Key Game Data/Resources Needed:** `GameState`, Player `Position` & `PlayerGear` (`RepellentFlask` state), `GhostSprite.Position` & `repellent_hits_frame`, `GhostBreach.Position`, `WalkiePlay`.
*   **`WalkieEvent` Enum Variant:** `RepellentUsedTooFar`
*   **Primary `WalkieTag`(s) for Line Selection:** `PlayerStruggling`, `Guidance`, `DirectHint`, `ContextualHint`.
*   **Repetition Strategy:** Can play if player repeatedly uses it from too far. Cooldown.
*   **Priority/Severity:** Medium-High. Prevents wasting repellent.

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
