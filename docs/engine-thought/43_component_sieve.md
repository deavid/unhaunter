# Component Sieve: Abstract Inventory for Unhaunter

This document consolidates all abstract component concepts from docs 14, 40, 41, and 42, then sieves them into:

- **✅ Definitely Needed** — Core to Unhaunter, validated by high-fit games
- **❓ In Doubt** — Useful but uncertain if essential
- **❌ Not Needed** — Outside scope or too niche

---

## Sources Consolidated

| Document                      | What it contributed                     |
| ----------------------------- | --------------------------------------- |
| 14_first_principles.md        | Nouns/verbs from gameplay narrative     |
| 40_game_analysis_synthesis.md | Cross-game capability validation        |
| 41_component_definition.md    | Rigorous definition, example components |
| 42_comparative_engine.md      | Struct-defined component library        |

---

## High-Fit Games (Our North Star)

These games are closest to Unhaunter's vision. Components they need should be prioritized:

| Game                   | Fit | Key Mechanics                                           |
| ---------------------- | --- | ------------------------------------------------------- |
| **Phasmophobia**       | 95% | EMF, temp, ghost room, evidence, hunting, sanity, photo |
| **Demonologist**       | 90% | EMF, temp, rituals, exorcism, render layers             |
| **Ghost Watchers**     | 90% | EMF, temp, countermeasures, aggression as info          |
| **Ghost Exorcism Inc** | 85% | Pathfinding control, banishment progress                |
| **Forewarned**         | 80% | Procedural maps, distinct AI profiles                   |
| **Mortuary Assistant** | 70% | Task-based horror, entity anchoring                     |
| **Devour**             | 70% | Fetch objectives, item encumbrance                      |

---

## ENGINE LAYER — Genre-Agnostic Simulation

### ✅ Definitely Needed

#### `GridTransform`

**Struct:** `{ coords: IVec3, orientation: Direction }`

**What it does:** Stores an entity's position on the game board as discrete tile coordinates (x, y, z) plus which way
it's facing. This is the fundamental "where is this thing?" component.

**What it enables:**

- Player and ghost positions in the house
- Spatial queries ("what's in this room?", "what's near me?")
- Movement systems can update coords
- Rendering system uses coords to place sprites

**Why needed:** Every spatial game requires positions. All 25 analyzed games have this concept.

---

#### `ThermalMass`

**Struct:** `{ current_temp_c: f32, conductivity: f32, specific_heat: f32 }`

**What it does:** Stores how much heat a tile or object holds, how quickly it transfers heat to neighbors
(conductivity), and how resistant it is to temperature change (specific_heat). A metal radiator has high conductivity
but high specific_heat. Air has low specific_heat so it changes temperature quickly.

**What it enables:**

- Temperature field propagation — ghost makes one spot cold, cold spreads outward
- Thermometer readings — tool queries nearby ThermalMass values
- Environmental storytelling — cold spots indicate paranormal activity
- Gradient detection — player can "follow the cold" to find the ghost

**Why needed:** Freezing temperatures are primary evidence in Phasmophobia, Demonologist, Ghost Watchers. The narrative
in doc 14: "we pick up a trail of cold spots with the thermometer."

---

#### `AcousticEmitter`

**Struct:** `{ volume_db: f32, frequency_range: Range<f32>, is_active: bool }`

**What it does:** Marks an entity as producing sound. The volume_db is how loud, frequency_range is what kind of sound
(low rumble vs high-pitched), is_active toggles it on/off.

**What it enables:**

- Footsteps alert the ghost — player walking activates their AcousticEmitter
- Environmental sounds — creaky floorboards, breaking glass
- Ghost detection — ghost's footsteps can be heard
- Sound propagation system reads these to calculate what's audible where

**Why needed:** Sound is critical for atmosphere and gameplay. The ghost hears you; you hear it. Lethal Company's
proximity chat horror relies entirely on sound physics.

---

#### `FieldAttenuator`

**Struct:** `{ light_block_factor: f32, sound_dampening_db: f32 }`

**What it does:** Attached to walls, doors, furniture. Defines how much this object blocks light (0.0 = transparent, 1.0
= opaque) and reduces sound (in decibels).

**What it enables:**

- Light doesn't pass through walls
- Closed doors muffle sound — ghost in another room is quieter
- Open doors let light and sound through
- Different materials block differently (glass vs brick)

**Why needed:** Without attenuation, light and sound would pass through everything. Every spatial game with rooms needs
this.

---

#### `SensoryProfile`

**Struct:** `{ vision_range: f32, vision_cone_deg: f32, hearing_threshold_db: f32, thermal_vision: bool }`

**What it does:** Defines what an AI entity can perceive. How far can it see? How wide is its vision cone? How quiet
does a sound need to be before it can't hear it? Can it see heat signatures through walls?

**What it enables:**

- Different ghost types have different senses — a Banshee might have better hearing, a Demon might see further
- Stealth gameplay — stay outside vision cone, move quietly
- Thermal_vision lets some ghosts detect flashlight heat through walls
- Forewarned's mummies each have distinct sensory weaknesses

**Why needed:** AI perception is fundamental to horror games. Doc 40: "Forewarned — AI needs distinct sensory profiles."
This is generic enough to be engine-level.

---

#### `Tether`

**Struct:** `{ anchor_entity: Entity, max_radius: f32, elasticity: f32 }`

**What it does:** Physically constrains an entity to stay within max_radius of another entity (the anchor). Elasticity
determines how "hard" the boundary is — high elasticity means it snaps back, low means it can stretch slightly.

**What it enables:**

- Ghost anchored to ghost room — can roam nearby but pulled back
- Haunted object — ghost tethered to a specific item
- Mortuary Assistant: demon attached to a specific body
- Escape mode: ghost protects ritual items by being tethered to them

**Why needed:** Doc 40 identifies "entity binding modes" as high priority. Games need ghosts that stay in certain areas
rather than wandering randomly everywhere.

---

#### `AreaDenial`

**Struct:** `{ nav_cost_modifier: f32, shape: ColliderShape, affects_tags: BitMask }`

**What it does:** Creates a zone that modifies pathfinding costs. A salt line with nav_cost_modifier: 100.0 makes the
ghost's pathfinding see that tile as 100x more expensive to cross. The affects_tags lets it only affect certain entity
types (ghosts but not players).

**What it enables:**

- Salt lines — ghost avoids crossing unless necessary
- Smudge sticks — temporary area denial
- Safe zones — church/blessed areas ghosts avoid
- Tower-defense-style indirect ghost control (Ghost Exorcism Inc)

**Why needed:** Every high-fit game has some form of "create a barrier the ghost respects." This is the player's main
way to manipulate ghost pathing.

---

#### `ContinuousInteractable`

**Struct:** `{ progress: f32, required_time: f32, decay_rate: f32 }`

**What it does:** An interaction that requires holding a button for time. Progress fills up while holding, decays if you
let go (based on decay_rate). Interaction completes when progress >= required_time.

**What it enables:**

- Hold E to open door (can't just spam-click)
- Repair the fuse box — requires sustained attention
- Place items carefully — can't rush
- Creates vulnerability — you're committed while holding

**Why needed:** Mortuary Assistant insight: "Horror works best when the player is busy with a precise task." The
hold-to-complete pattern appears in most horror games for tension.

---

#### `LocalAtmosphere`

**Struct:** `{ fog_density: f32, fog_color: Color, grain_intensity: f32, color_grade: Handle<Lut> }`

**What it does:** Overrides visual post-processing in a zone. A room can have its own fog density, color tint, film
grain level, and color grading lookup table.

**What it enables:**

- Ghost room feels different — denser fog, colder color grade
- Miasma zones — thick particulate, reduced visibility
- Horror atmosphere — grain increases tension
- LIMINAL SHROUD: entire horror comes from atmosphere alone

**Why needed:** Doc 40: "Atmosphere alone creates horror." Silent Hill 2's fog is legendary. This enables per-room mood
without changing the whole map.

---

#### `TrailEmitter`

**Struct:** `{ particle_type: ParticleId, emission_rate: f32, lifetime: f32 }`

**What it does:** Entity leaves behind particles as it moves. Particle_type defines what kind (footprints, heat residue,
ectoplasm). Emission_rate is how often. Lifetime is how long particles persist.

**What it enables:**

- UV footprints — ghost leaves invisible prints, UV light reveals them
- Heat trails — thermal camera shows where ghost walked
- Ectoplasm drips — forensic evidence
- Tracking gameplay — follow the trail to find the ghost

**Why needed:** Phasmophobia's UV footprints are key evidence. White Noise 2 uses trails for tracking. This is core
forensic investigation.

---

#### `NoiseTrap`

**Struct:** `{ activation_velocity: f32, sound_on_trigger: Handle<AudioSource> }`

**What it does:** A surface or object that makes noise when something moves over/into it fast enough.
Activation_velocity is the speed threshold. Sound_on_trigger is what audio plays.

**What it enables:**

- Broken glass — run over it, crunch alerts ghost
- Creaky floorboards — walk slowly or risk noise
- Tin cans — knocked over, clatter
- Player must move carefully, creating tension

**Why needed:** Doc 14 narrative: "The floor creeks as we walk." Outlast Trials, Lethal Company use noisy surfaces as
hazards. Creates meaningful movement choices.

---

### ❓ In Doubt

#### `Destructible`

**Struct:** `{ integrity: f32, fracture_threshold: f32, broken_mesh: Handle<Mesh> }`

**What it does:** Object can be damaged. When integrity drops below fracture_threshold, swap to broken_mesh visual.
Enables breaking things.

**What it enables:**

- Poltergeist throws vase, it shatters
- Environmental damage from ghost activity
- Evidence of violence — broken furniture tells a story
- Midnight Ghost Hunt: ghosts possess and throw objects

**Why in doubt:** Cool for atmosphere but not core gameplay. High-fit games (Phasmo, Demo) don't emphasize destruction.
Probably v2 feature.

---

#### `FluidDrag`

**Struct:** `{ viscosity: f32, flow_vector: Vec3 }`

**What it does:** Zone that slows movement. Viscosity 0.5 means 50% speed. Flow_vector can push entities in a direction.

**What it enables:**

- Miasma zones — thick air slows you down
- Water areas — wading is slow
- Supernatural mud — ghost can create slow zones
- Pools (the game) uses water for pacing

**Why in doubt:** Unhaunter's miasma design wants this, but high-fit games don't use it. Atmospheric but not
evidence-gathering.

---

#### `TeleportLink`

**Struct:** `{ destination: Entity, trigger_bounds: Aabb, seamless: bool }`

**What it does:** Walking into trigger_bounds teleports you to destination. If seamless=true, no loading screen — looks
like continuous space.

**What it enables:**

- P.T.-style looping corridors
- Non-Euclidean geometry — house is bigger inside
- Secret passages
- Horror from impossible spaces

**Why in doubt:** Very cool but niche. P.T. is only 35% fit. Most ghost-hunting games have normal geometry.

---

#### `PuppetTarget`

**Struct:** `{ is_possessed: bool, control_strength: f32 }`

**What it does:** Marks a physics object as possessable. When possessed, AI can "drive" it — move it, throw it.

**What it enables:**

- Poltergeist possesses chair, slides it across room
- Objects fly during ghost activity
- Midnight Ghost Hunt: ghosts hide by possessing props

**Why in doubt:** Not in Phasmophobia or Demonologist. Midnight Ghost Hunt (15% fit) uses it. Cool but different vibe.

---

#### `PerceptionFilter`

**Struct:** `{ hallucination_intensity: f32, color_shift: Color, ghost_entities: Vec<Entity> }`

**What it does:** Per-player visual modifications. Hallucination_intensity spawns fake ghost sightings. Color_shift
tints vision. Ghost_entities are fake ghosts only this player sees.

**What it enables:**

- Low sanity causes hallucinations — see ghosts that aren't there
- Private horror — you see something, teammates don't
- Outlast Trials: players disagree on what's real

**Why in doubt:** Great for multiplayer confusion but complex to implement. Might belong in Shared Mod layer, not
Engine. Phasmo's hallucinations are simpler.

---

#### `RenderLayerMask`

**Struct:** `{ visible_layers: BitMask }`

**What it does:** Controls which render layers an entity can see. Different layers for normal vision vs UV vs spectral.

**What it enables:**

- UV flashlight reveals hidden writing (SPECTRAL layer)
- Ecto-glass shows ghost traces
- Different tools reveal different things
- Demonologist uses this heavily

**Why in doubt:** Probably needed for UV evidence, but might be simpler than full layer system. Could be just a bool
"UV_visible" per object.

---

#### `ConditionalVisibility`

**Struct:** `{ condition: VisibilityCondition, target_entity: Entity }`

**What it does:** Object only renders when condition is met. Conditions like "sanity below 30" or "phase == hunting" or
"UV light active."

**What it enables:**

- Bloody handprints appear at low sanity
- Ghost writing only visible under UV
- P.T.: things appear/disappear based on loop count

**Why in doubt:** Overlaps with RenderLayerMask. P.T. uses it but is low fit. Might be too complex for what we need.

---

#### `VolumetricObscurance`

**Struct:** `{ density: f32, scatter_color: Color, height_falloff: f32 }`

**What it does:** Volumetric fog that reduces visibility. Density is thickness. Scatter_color is fog color.
Height_falloff makes fog thicker at ground level.

**What it enables:**

- Silent Hill 2 fog — can't see far
- Ground mist in graveyards
- Atmosphere that limits sightlines

**Why in doubt:** Overlaps with LocalAtmosphere's fog_density. Do we need both? LocalAtmosphere might be sufficient.

---

#### `ItemSocket`

**Struct:** `{ accepted_tags: BitMask, held_item: Option<Entity>, lock_on_insert: bool }`

**What it does:** A receptacle that accepts specific item types. Ritual bowl accepts candles, bones, etc. Lock_on_insert
means you can't remove it once placed.

**What it enables:**

- Ritual mechanics — place 3 items in correct spots
- Demonologist exorcism — put crucifix in holder
- Puzzle elements — key goes in lock

**Why in doubt:** Essential for Escape mode rituals but might be Shared Mod not Engine. Is "item placement for rituals"
genre-specific?

---

#### `RemoteSignalSource`

**Struct:** `{ channel_id: u8, signal_strength: f32, data_type: SignalType }`

**What it does:** Broadcasts data to a channel. Video cameras broadcast their feed. Sensors broadcast readings. Paired
with RemoteSignalReceiver.

**What it enables:**

- Video camera in house → monitor in truck
- EMF sensor placed in room → readings on PDA
- Doc 14: "we take a look at the sensors" from outside

**Why in doubt:** Core to Unhaunter's truck-based gameplay, but not all ghost games have remote monitoring. How many
games really use this pattern?

---

#### `WorldInteractableUI`

**Struct:** `{ canvas_size: Vec2, interaction_bounds: Aabb }`

**What it does:** A surface in the 3D world that acts as a 2D UI. Player can interact with it like a screen — click
buttons, move cursor.

**What it enables:**

- Computer terminals in the house
- Truck's monitoring station
- Stories Untold: entire game is diegetic screens

**Why in doubt:** Stories Untold is only 20% fit. Phasmo doesn't have interactive in-world screens. Cool but niche.

---

#### `FocusInteraction`

**Struct:** (undefined — from Dead by Daylight analysis)

**What it does:** Task that requires player attention, blocking peripheral awareness. Like Dead by Daylight's generator
repair skill checks.

**What it enables:**

- Busy with task, can't see ghost approaching
- Tension from vulnerability
- Mini-game interactions

**Why in doubt:** Dead by Daylight is 20% fit, different genre. Phasmo doesn't have attention-demanding minigames.

---

### ❌ Not Needed (Engine Layer)

| Concept                 | Why Not Needed                                                                                                      |
| ----------------------- | ------------------------------------------------------------------------------------------------------------------- |
| **Combat system**       | Doc 40: "Combat — breaks the genre." Control, Alan Wake 2, Silent Hill 2 — combat is what makes them NOT Unhaunter. |
| **Branching narrative** | Doc 40: "Branching narrative — different architecture." Stanley Parable is outside scope.                           |
| **Voice recognition**   | Doc 40: "Voice interaction — low priority, niche." Spirit box voice recognition is cool but not essential.          |

---

## SHARED MOD LAYER — Paranormal Investigation Genre

These components define "ghost hunting" as a genre but are agnostic to specific game modes (Classic vs Escape).

### ✅ Definitely Needed

#### `Sanity`

**Struct:** `{ current: f32, max: f32, drain_rate_modifiers: Vec<f32> }`

**What it does:** Tracks player's mental health. Current value decreases over time based on conditions (darkness, ghost
proximity, witnessing events). Modifiers stack — darkness might be 1.5x, ghost nearby 2.0x.

**What it enables:**

- Psychological pressure — staying in dark drains sanity faster
- Risk/reward — push deeper into house vs retreat to restore
- Gameplay consequences — low sanity triggers hallucinations, attracts ghost attention
- Doc 14: "We ran a sanity check... 45%... trying to be easy on the entity also drained our sanity"

**Why needed:** Every high-fit game has sanity or equivalent. It's THE core resource that creates tension in ghost
hunting games. Without it, there's no pressure to leave.

---

#### `FieldEmitter`

**Struct:** `{ field_type: FieldTypeId, intensity: f32, range: f32 }`

**What it does:** Marks an entity as emitting a detectable field. The ghost might emit EMF (electromagnetic), cold
(thermal), or radiation (Geiger). Intensity is how strong, range is how far it reaches.

**What it enables:**

- Ghost causes EMF spikes — EMF reader detects it
- Ghost causes temperature drops — thermometer reads cold
- Ghost causes Geiger clicks — radiation detector responds
- Tools work by detecting FieldEmitter values nearby

**Why needed:** This IS evidence. Doc 14: "EMF, Temperature, and Geiger counter." Every high-fit game has the ghost emit
detectable fields that tools pick up.

---

#### `EvidenceState`

**Struct:** `{ evidence_type: EvidenceId, discovery_progress: f32, is_confirmed: bool }`

**What it does:** Tracks discovery of a specific evidence type. Progress fills as player gathers data (thermometer in
cold room over time). Once progress hits 100%, is_confirmed becomes true and evidence is "found."

**What it enables:**

- Phasmophobia's core loop — gather 3 evidence types to identify ghost
- Thermometer needs to stay in cold zone long enough to confirm "Freezing"
- EMF reader must show level 5 for a few seconds to confirm "EMF 5"
- Journal shows confirmed evidence

**Why needed:** Classic mode's entire goal is confirming evidence types. This tracks the "did we get it?" state per
evidence.

---

#### `IdentityTag`

**Struct:** `{ true_name: String, death_cause: DeathCause, age: u16 }`

**What it does:** The ghost's hidden identity that the player is trying to discover. True_name is what you speak for
spirit box. Death_cause might affect behavior. Age might determine ghost type.

**What it enables:**

- Classic mode goal — discover the ghost's name
- Speaking the name might anger the ghost
- Obituary/records in the house hint at identity
- "Who was this person?" investigation

**Why needed:** Investigation is about discovering who the ghost was. Phasmo, Demonologist — the name matters.

---

#### `CountermeasureReaction`

**Struct:** `{ item_tag: ItemTag, reaction: ReactionType, intensity: f32 }`

**What it does:** Defines how an entity reacts to specific item types. Salt might cause reaction: Burn with intensity
0.5. Crucifix might cause reaction: Repel. Smudge stick might cause reaction: Calm.

**What it enables:**

- Salt hurts certain ghosts — "Chemistry is a combat mechanic"
- Crucifix prevents hunts in an area
- Smudge stick calms ghost temporarily
- Different ghosts have different weaknesses
- Ghost Watchers: learning reactions IS the investigation

**Why needed:** Player agency against the ghost. Every high-fit game lets you fight back somehow without actual combat.

---

#### `SpiritualStability`

**Struct:** `{ current: f32, max: f32 }`

**What it does:** The ghost's "health bar" in a non-combat sense. Completing ritual steps, banishing actions, or
exorcism progress reduces current. At 0, the ghost is banished/defeated.

**What it enables:**

- Escape mode — complete rituals to weaken the ghost
- Ghost Exorcism Inc — banishment progress bar
- Demonologist exorcism — ritual completion
- Non-violent win condition

**Why needed:** Escape mode needs a goal beyond "identify." This is "defeat the ghost" without combat — through ritual,
holy items, completing objectives.

---

### ❓ In Doubt

#### `InterferenceReceiver`

**Struct:** `{ threshold: f32, current_noise: f32 }`

**What it does:** Attached to electronic equipment. When near FieldEmitter (ghost), current_noise increases. Visual
static, audio distortion scales with noise level.

**What it enables:**

- Video camera gets static near ghost
- Spirit box has more interference
- Walkie-talkies crackle
- VIDEO NASTY: glitches are evidence

**Why in doubt:** Atmospheric and cool, but not core evidence in Phasmo. More of a "something's wrong" indicator than
diagnostic tool.

---

#### `PhotoTarget`

**Struct:** `{ tags: Vec<PhotoTag>, reward_value: u32, max_distance: f32 }`

**What it does:** Makes an entity photographable for bonus objectives. Tags describe what it is (ghost, bone, ghost
writing). Reward_value is money earned. Max_distance is how close camera must be.

**What it enables:**

- Phasmo's photo rewards — ghost photo, bone photo
- Secondary objectives
- Evidence through photography

**Why in doubt:** Nice for secondary objectives but not essential for core identify loop. Phasmo has it, but you can win
without photos.

---

#### `Lure`

**Struct:** (undefined — from Ghost Watchers)

**What it does:** Attracts AI pathfinding toward this location. Opposite of AreaDenial — instead of repelling, it pulls.

**What it enables:**

- Bait items — place something ghost wants
- Trap setups — lure ghost into area
- Aggressive investigation tactics

**Why in doubt:** Could be done with AreaDenial (negative cost) or AI behavior. Is a separate component needed?

---

### ❌ Not Needed (Shared Mod Layer)

| Concept                | Why Not Needed                                                                        |
| ---------------------- | ------------------------------------------------------------------------------------- |
| **SearchableDatabase** | Her Story mechanic. Resource, not component. Doc 42 correctly identifies as Resource. |
| **DeductionBoard**     | Alan Wake 2 "Mind Place." Entity with components, not a component itself.             |
| **NarratorTrigger**    | Stanley Parable. This is a System, not a component.                                   |

---

## GAME MODE LAYER — Specific Win/Loss Loops

These components define specific game modes like Classic (identify) or Escape (survive/banish).

### ✅ Definitely Needed

#### `ZoneOwnership`

**Struct:** `{ favored_zone_ids: Vec<ZoneId>, roam_chance: f32 }`

**What it does:** AI preference for specific zones. The ghost "owns" the kitchen and bedroom — it prefers to stay there.
Roam_chance is probability of wandering outside owned zones.

**What it enables:**

- Ghost room mechanic — ghost has a "home base"
- Investigation focus — find which room is the ghost room
- Predictable but not deterministic ghost location
- Classic mode: narrow down ghost room through evidence

**Why needed:** Phasmophobia's entire early-game is "find the ghost room." This is what makes that work.

---

#### `AggressionState`

**Struct:** `{ current_level: f32, escalation_rate: f32, hunt_threshold: f32 }`

**What it does:** Tracks ghost's anger. Current_level rises over time (escalation_rate) and from player actions
(speaking name, using equipment). When current_level > hunt_threshold, ghost starts hunting.

**What it enables:**

- Tension ramp — ghost gets more dangerous over time
- Player actions have consequences — saying name makes it angry
- Hunt mechanic — ghost actively chases when threshold crossed
- Different ghost types have different thresholds

**Why needed:** Hunts are core to Phasmophobia's horror. Without escalation, there's no reason to hurry or be careful.

---

### ❓ In Doubt

#### `PowerDependency`

**Struct:** `{ power_source: Entity, behavior_when_off: BehaviorId }`

**What it does:** Links entity behavior to power state. Ghost might become aggressive (behavior_when_off: Aggressive)
when breaker is off.

**What it enables:**

- Pacify: ghost is calm when lights on, dangerous when dark
- Breaker manipulation as strategy
- Environmental puzzle — fix/break power to affect ghost

**Why in doubt:** Pacify (60% fit) uses this heavily. Phasmophobia doesn't — breaker affects sanity but not ghost
behavior directly. Interesting but not core.

---

#### `LightReactive`

**Struct:** `{ light_threshold_lux: f32, reaction: ReactionType }`

**What it does:** Ghost behavior changes based on ambient light level. Below threshold_lux, one behavior; above it,
another.

**What it enables:**

- Ghost retreats from bright light
- UV light triggers reactions
- Flashlight as weak "weapon"
- White Noise 2: flashlight stuns ghost

**Why in doubt:** Overlaps with SensoryProfile (can ghost see light?) and CountermeasureReaction (flashlight item
reaction). Might be redundant.

---

### ❌ Not Needed (Game Mode Layer)

| Concept                        | Why Not Needed                                                     |
| ------------------------------ | ------------------------------------------------------------------ |
| **Asymmetric player-as-ghost** | White Noise 2, Dead by Daylight. Outside Unhaunter's co-op design. |
| **PvP scoring**                | Midnight Ghost Hunt. Different genre.                              |

---

## GAPS IDENTIFIED

Components NOT in the docs but suggested by high-fit games:

#### `Inventory`

**Struct:** `{ slots: Vec<Option<Entity>>, max_slots: u8 }`

**What it does:** Player's carrying capacity. Slots hold item entities. Max_slots limits how much you can carry.

**What it enables:**

- Backpack system — doc 14: "carrying everything, a backpack with plenty of stuff"
- Item selection — cycle through held items
- Loadout choices — bring EMF or thermometer?
- Drop/swap items

**Why missing:** Every game has inventory. Doc 14 clearly describes it. Should have been in component list.

---

#### `Battery`

**Struct:** `{ current: f32, max: f32, drain_rate: f32 }`

**What it does:** Equipment power level. Drains over time when active. At 0, tool stops working.

**What it enables:**

- Flashlight runs out — major Phasmo tension point
- EMF reader needs new batteries
- Resource management layer
- "My flashlight died" panic

**Why missing:** Core to Phasmophobia. Tools running out is huge tension. Doc 41 lists Battery as valid component.

---

#### `Equipped`

**Struct:** `{ in_hand: bool }` or marker component

**What it does:** Tracks whether item is currently being held/used by player.

**What it enables:**

- Only equipped item is active
- Switching between tools
- Visual — render item in hand
- Some items work only when equipped

**Why missing:** Basic gameplay — "which item am I holding?" Doc 41 lists Equipped as valid.

---

#### `RoomMembership`

**Struct:** `{ room_id: RoomId }`

**What it does:** Tags an entity or tile as belonging to a specific room.

**What it enables:**

- "Ghost is in the Kitchen" detection
- Room-based evidence — freezing in ONE room
- Spatial queries — "what's in this room?"
- Map board shows room names

**Why missing:** Phasmo ghost room mechanic requires knowing what room things are in. Doc 41 lists RoomID as valid.

---

#### `Visibility`

**Struct:** `{ is_visible: bool, last_seen_by: Vec<Entity> }`

**What it does:** Tracks whether entity can currently be seen and by whom.

**What it enables:**

- Hiding mechanics — ghost can't see you in closet
- Stealth — stay out of line of sight
- "Did the ghost see me?" queries
- AI perception uses this

**Why missing:** Core to hiding gameplay. Doc 40: "Hiding mechanics — exist in Unhaunter." Doc 41 lists Visibility as
valid.

---

#### `HidingSpot`

**Struct:** `{ capacity: u8, current_occupants: Vec<Entity> }`

**What it does:** Marks a location as a hiding spot (closet, under bed). Capacity is how many can hide there.

**What it enables:**

- Hiding during hunts — Phasmo core survival mechanic
- Limited spots — can't all hide in same closet
- Finding hiding spots is early-game priority

**Why missing:** Hiding from hunts is essential. Without hiding spots, hunts are instant death.

---

#### `Health`

**Struct:** `{ current: f32, max: f32 }`

**What it does:** Physical health, separate from sanity.

**What it enables:**

- Ghost attacks reduce health, not just sanity
- Death condition — health hits 0
- Damage from environment (falling, etc.)

**Why missing/in doubt:** Phasmo doesn't have health — ghost touch = instant death. Some modes might want health as a
buffer. Medium priority.

---

#### `ObjectiveProgress`

**Struct:** `{ objective_id: ObjectiveId, progress: f32, is_complete: bool }`

**What it does:** Generic objective tracker. Can be "find 3 evidence types" or "collect 5 ritual items" or "survive 10
minutes."

**What it enables:**

- Secondary objectives in Phasmo
- Main objectives in Escape mode
- Generic goal system
- Devour: "bring N items to location X"

**Why missing:** Doc 40 identifies "Objectives framework" as engine-level. This is how you track goals.

---

## SUMMARY TABLE

### Engine Layer

| Component              | Status | Description                                          |
| ---------------------- | ------ | ---------------------------------------------------- |
| GridTransform          | ✅     | Position and facing on the game board                |
| ThermalMass            | ✅     | Heat storage for temperature propagation             |
| AcousticEmitter        | ✅     | Produces sound that propagates and AI can hear       |
| FieldAttenuator        | ✅     | Walls/doors block light and muffle sound             |
| SensoryProfile         | ✅     | AI perception: vision range, hearing threshold       |
| Tether                 | ✅     | Constrains entity to stay near an anchor             |
| AreaDenial             | ✅     | Zones that repel AI pathfinding (salt lines)         |
| ContinuousInteractable | ✅     | Hold-to-complete interactions                        |
| LocalAtmosphere        | ✅     | Per-zone fog, grain, color grading                   |
| TrailEmitter           | ✅     | Leaves particles behind (UV footprints)              |
| NoiseTrap              | ✅     | Surfaces that make sound when disturbed              |
| Destructible           | ❓     | Objects that can break (poltergeist activity)        |
| FluidDrag              | ❓     | Zones that slow movement (miasma)                    |
| TeleportLink           | ❓     | Non-Euclidean connections (P.T. loops)               |
| PuppetTarget           | ❓     | AI can possess and control physics objects           |
| PerceptionFilter       | ❓     | Per-player visual hallucinations                     |
| RenderLayerMask        | ❓     | Spectral/UV visibility layers                        |
| ConditionalVisibility  | ❓     | Objects only visible under certain conditions        |
| VolumetricObscurance   | ❓     | Volumetric fog (overlaps with LocalAtmosphere?)      |
| ItemSocket             | ❓     | Receptacles for ritual item placement                |
| RemoteSignalSource     | ❓     | Broadcast sensor data to remote displays             |
| WorldInteractableUI    | ❓     | In-world interactive screens                         |
| FocusInteraction       | ❓     | Tasks that demand attention, block peripheral vision |

### Shared Mod Layer

| Component              | Status | Description                                             |
| ---------------------- | ------ | ------------------------------------------------------- |
| Sanity                 | ✅     | Mental health, drains in darkness/near ghost            |
| FieldEmitter           | ✅     | Ghost emits EMF, cold, radiation for tools to detect    |
| EvidenceState          | ✅     | Tracks discovery progress per evidence type             |
| IdentityTag            | ✅     | Ghost's hidden name and identity to discover            |
| CountermeasureReaction | ✅     | How ghost reacts to items (salt burns, crucifix repels) |
| SpiritualStability     | ✅     | "Boss health" for non-combat banishment progress        |
| InterferenceReceiver   | ❓     | Equipment gets static/noise near ghost                  |
| PhotoTarget            | ❓     | Photographable for bonus objectives                     |
| Lure                   | ❓     | Attracts AI pathfinding (opposite of AreaDenial)        |

### Game Mode Layer

| Component       | Status | Description                                        |
| --------------- | ------ | -------------------------------------------------- |
| ZoneOwnership   | ✅     | Ghost prefers certain rooms (ghost room mechanic)  |
| AggressionState | ✅     | Anger level that triggers hunts when threshold hit |
| PowerDependency | ❓     | Behavior changes based on power/lights state       |
| LightReactive   | ❓     | Behavior changes based on ambient light level      |

### Gaps (Need to Add)

| Component         | Layer  | Priority  | Description                           |
| ----------------- | ------ | --------- | ------------------------------------- |
| Inventory         | Engine | ✅ High   | Player's item slots                   |
| Battery           | Engine | ✅ High   | Equipment power that drains over time |
| Equipped          | Engine | ✅ High   | Which item is currently in hand       |
| RoomMembership    | Engine | ✅ High   | What room an entity/tile belongs to   |
| Visibility        | Engine | ✅ High   | Is entity currently visible to others |
| HidingSpot        | Engine | ✅ High   | Locations where player can hide       |
| Health            | Engine | ❓ Medium | Physical health (if modes need it)    |
| ObjectiveProgress | Shared | ✅ High   | Generic goal/objective tracking       |

---

## NEXT STEPS

1. **Finalize ✅ Definitely Needed** — These form the abstract "must have" list
2. **Decide ❓ In Doubt** — For each, ask: "Does Classic or Escape mode need this?"
3. **Document Gaps** — Add missing components to the canonical list
4. **Later: Compare to Reality** — Map abstract components to existing Rust code
