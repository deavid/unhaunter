# Concept design

This is the **Unhaunter Architectural Manifesto**. It is a synthesis of our deep-dive re-analysis of the audits,
stripping away the enterprise "Java-isms" and replacing them with a blueprint optimized for **Context Isolation,
Emergent Simulation, and Low Cognitive Load.**

The core philosophy: **Dwarf Fortress for Ghosts.** The world is a set of interacting mathematical fields; actors are
modular "spherical cows" floating through them; and the UI/Audio are independent observers that never touch the math.

---

## Original suggestion

### Phase 1: The Bedrock (Physics & Fields)

The "Stage" is no longer a collection of sprites. It is a set of interacting 3D matrices.

- **`unworld-ontology` (T1):** The "Dictionary of Reality." Defines components like `Class(Door)`, `ThermalEmitter`,
  `Opaque`, and `LightSource`.
  - _Strategy:_ Pull work IN from `unbehavior` and `unmapload`. It owns the _definitions_, not the files.
- **`unworld-physics-grids` (T1):** A set of crates (Thermal, Sound, EMF, Visibility, Miasma) that manage raw
  `Array3<f32>` matrices.
  - _Rules:_ They are blind. They compute diffusion/propagation math exclusively. They observe `unworld-ontology`
    components to determine boundaries and sources.
- **`unworld-navigation` (T1):** Pathfinding as a service. It reads the collision matrix and provides A\* waypoints to
  anyone (Player, Ghost, Rat) who asks via a `RequestPath` event.

### Phase 2: The Agent Slices (Breaking the Nouns)

We obliterate `unplayer` and `unghost`. Actors are now Entity IDs composed of modular vertical feature-backpacks.

- **`unfeature-vitals` (T1):** The sole arbiter of biological math (Health, Sanity, Stamina).
  - _Fire & Forget:_ It integrate stimulus from the Physics Grids (Fear, Temp). It does not "call" audio; it simply
    writes to a local `TargetVolume` component on its own entities.
- **`unfeature-logistics` (T1):** The slot manager. It owns `HandSlot`, `Backpack`, and `VanShelf`.
  - _Mechanics:_ It only handles the _transfer of IDs_. It moves Entity(EMF_Reader) from `World` to `Hand`. It doesn't
    know what an EMF reader does.
- **`unactor-intent-queue` (T1):** The "Brains."
  - Translates clicks (Human) or Wander-Targets (AI) into a `CommandQueue`.
  - _Logic:_ It decides: "I'll walk there, then interact." It handles the sequencing, then passes the velocity to the
    Kinematics.
- **`unactor-kinematics` (T1):** The "Meat Driver." Reads a desired vector, resolves collision with the `unboard` grid,
  and applies the delta. It has no idea what a door is.

### Phase 3: The Metagame (Agency vs. Field)

We separate the "Job" (Mission) from the "Career" (Agency).

- **`unfeature-agency` (T2):** The Metagame Engine. It lives in memory.
  - _Responsibilities:_ Payout calculations, XP curves, map unlocking, insurance deposits.
  - _Trigger:_ It listens for `MissionResult` events. It does all the math and then updates the save-file record.
- **`unfeature-mission` (T2):** The Conductor. It owns the rules of the current match (Classic mode, Tutorial, etc.).
  - It uses the `MissionEvaluator` trait to decide when a win/loss occurs and emits the police report to the Agency.
- **`unadapter-persistence` (T3):** The Dumb Floppy Disk. It has zero game logic. It simply takes snapshots of the
  Agency's ledger or the User's Settings and writes them to RON/JSON.

### Phase 4: The Observers (Protected Optics & Audio)

Presentation never controls logic. It purely observes and represents.

- **`bevy_smooth_mixer` (Portable / T4):** A game-agnostic audio controller.
  - _Logic:_ It manages `ManagedLoop` entities. Gameplay systems "turn the knobs" by writing to `TargetVolume`. The
    mixer handles the lerps and Bevy-API calls. This protects the game from the upcoming Bevy 0.21 audio breakage.
- **`unadapter-world-renderer` (T4):** The Skin of the Matrix.
  - _Rules:_ T1 Physics does not spawn sprites. This crate observes Entity changes.
    `"Oh, a Wall was added? I'll build the mesh."` `"Oh, sanity dropped to 10? I'll paint the vignette."`
  - _Purity:_ This allows a Headless Server to compile with 0% rendering code by simply not including this crate.

---

### The Operational Refactor Strategy: "The Gravity Well"

Do not try to "move stuff out." That creates the **Label-on-a-Carrot** problem. Instead, use this prompt structure with
LLMs:

1. **Define the Vacuum:** Create the empty destination crate (e.g., `unfeature-vitals`).
2. **State the Goal:** "I want to centralize all X math here. It must be fire-and-forget. It only takes these
   mathematical inputs and produces these specific output states."
3. **Pull Work IN:** "Comb `unplayer-plugin`, `unvitals-plugin`, and `unghost-plugin`. Suck in every sanity and health
   variable. Strip all audio-playing and UI-formatting logic during the move. Leave behind a 'Hole' in the form of a
   Bevy Event if a side-effect is needed."
4. **Seal the Bulkhead:** Verify that the new crate does not import any higher-tier concepts.

**Result:** Cognitive load drops to near zero. To fix the heartbeat sound, you look at `vitals`. To fix the flashlight
battery, you look at `equipment`. To fix the reward payout, you look at `agency`.

**The Unhaunter North Star:** You are building a simulation of interacting data-silos. Each silo is "Deletable"—if you
delete the `logistics` folder, the ghost still haunts and the fog still flows; the players just lose their hands.

---

## Re: About the Mission Domain

The "Mission" (or Contract) is the **Conductor** of your game's orchestra.

Throughout our discussion, we realized that the Mission currently feels messy because its brain is scattered across
three different places: the map loader, the post-game UI, and the player save file.

Here is the collected blueprint for the **Mission Domain** as we’ve defined it through our VSA/Actor re-analysis:

### 1. What a Mission IS (The "Contract" Folder)

Think of a Mission as a physical clipboard on a van dashboard. It contains the "Terms of Engagement."

- **The Target:** Which map are we currently simulating?
- **The Terms (Difficulty + Modifiers):** You brilliantly defined Difficulty as a **Template for Modifiers**.
  - A "Hard" difficulty is just a preset bucket of modifiers (e.g., _LowVisibility, FastGhost, BreakerBroken_).
  - The Mission can also take **Additive Modifiers** (e.g., a _Red Moon_ event adds _HighAgression_ on top of whatever
    the difficulty already set).
- **The Stake:** The insurance deposit (the money you stand to lose).
- **The Evaluation Strategy:** A generic `MissionEvaluator` trait. This is how the Mission knows how to score you (e.g.,
  "Classic Mode Evaluator" vs. "Tutorial Evaluator").

### 2. The Mission Lifecycle (The Conductor)

We identified that "Map Loading" (T3) and "The Mission" (T2) are currently too entangled. The Mission should be the one
who owns the flow of time.

- **Step 1: The Inflation (T3):** The Map Pipeline inflates the hard drive bytes into 3D grids. It shouts:
  `LevelMathematicalDataReady`.
- **Step 2: The Warm-up (T2):** The Mission Conductor hears that shout. It initializes the "Mission State." It spawns
  the players and the ghost.
- **Step 3: The Starting Pistol:** Instead of physics systems waiting for a Tiled file to load, they wait for the
  Mission to transition to `MissionState::Active`. The Conductor fires the `MissionReady` event, and the clock starts.
- **Step 4: The Final Curtain:** The Conductor decides the mission is over (all players dead, or van doors closed). It
  captures the "State of the Universe" and stops the simulation.

### 3. The Ledger (Telemetry vs. Truth)

One of the biggest "Aha!" moments was realizing that `unsummary-core` (the post-game screen) is actually the **Mission's
Ledger.**

- **The Passive Tracker:** During the game, a system in the Mission Domain holds a notepad. It records: "How long were
  they inside?", "Did they find EMF 5?", "Who died?".
- **Client vs. Server Asymmetry:**
  - **The Server** tracks only the "Arbiter Truth" (Win/Loss conditions).
  - **The Client** tracks "Telemetry" (the juicy details for the Summary screen).
- **The Debrief:** When the mission ends, the Ledger produces a `MissionSummary` parcel. This is just pure data.

### 4. The Separation of Church and State (Mission vs. Agency)

We realized your economy was "squatting" in your save-file code. We separated them into two distinct world-views:

- **The Mission (The Field Work):** This domain is only about what happens inside the house. Its final output is the
  `MissionSummary` report. It has no idea what a "Bank Account" is.
- **The Agency (The Career):** This domain (T2) lives _back at the Hub_. It waits for the `MissionSummary` report.
  - The Agency reads the report and says: _"Oh, you got a Grade A? Here is $500 and 1000 XP."_
  - _The Agency_ updates the Save File.
  - _The Agency_ unlocks new map contracts for the next Mission.

### The "Mission" North Star Summary

In the ideal world, you should be able to open the `unfeature-mission` folder and see:

1. **The State Machine:** (Loading -> Ready -> Active -> Success/Failure).
2. **The Modifiers:** The logic that applies the "Red Moon" or "Freezing Weather" properties to the world.
3. **The Evaluator:** The D&D-style DM logic that says "The ghost is a Poltergeist, you guessed Wraith, you lose."
4. **The Handoff:** The event that sends the final report to the Agency.

**The Mission kills the players; the Agency writes the paycheck.** By keeping them separate, you can rewrite your entire
economy without ever touching your Ghost-hunting logic.

NOTE from user: "The mission kills the players" doesn't sound that correct. I mean, they're KIA in the mission, yes. But
this needs to be read with a pinch of salt... or two.

## Appendix: A Clarification on "Obliterating" Player and Ghost

> _This section captures a follow-up discussion (2026-03-22). None of this is a plan or a directive. It is speculative
> thinking about what "ideal" domains might look like if we ever had a clean slate. Take with appropriate amounts of
> salt._

The phrase "obliterate `unplayer` and `unghost`" in Phase 2 is easy to misread as "delete them." That is not the intent.
The intent is: **drain them until they are mostly empty shells**, by pulling their actual concerns out into proper
domain crates. At the end of that process, `unplayer` and `unghost` probably still exist — because "player" and "ghost"
are genuinely important concepts in this game, and there will always be glue that is specific to one or the other. But
they would be thin. The interesting logic would live elsewhere.

### Why the current names are the problem

The real issue with `unplayer` and `unghost` as they stand today is that they are organized by **noun** (the thing)
rather than by **concern** (what happens). A domain organized around a noun becomes a dumping ground: if it touches the
player, it goes in `unplayer`. The result is that audio logic, rendering logic, inventory logic, vitals math, and
network routing all end up in the same crate, because they all "touch the player."

Domains organized around concerns answer a cleaner question: _what problem do I solve?_ And crucially, a concern-domain
does not know who is asking. Navigation doesn't know it's serving a player. Vitals doesn't know it's attached to a
ghost. That ignorance is what makes them reusable and independently deletable.

### What the ideal concern-domains might look like

The following is **purely hypothetical** — an attempt to name the actual concerns hiding inside the current crates.
Names are placeholders. This is not a migration plan.

- **World geometry** — static topology; walls know they block; rooms have identity. Does not know about actors.
- **Physics fields** — temperature, sound, light, fog, miasma each diffuse through the geometry. Each is a number that
  spreads. Each is blind to who caused it and who reads it. _(These already exist as separate crates; the concern is
  keeping them blind.)_
- **Navigation** — pathfinding as a service. Reads the collision field, answers `RequestPath` events. Serves players,
  ghosts, rats — does not distinguish between them.
- **Vitals** — health, sanity, stamina math. Responds to stimuli from fields and events. Does not know it is attached to
  a "player." Does not call audio.
- **Logistics** — who holds what. The handoff protocol between hand, backpack, shelf, world. Moves entity IDs between
  slots. Does not know what an EMF reader does.
- **Interaction** — the verb layer. Pick up, open, activate. Mediated by proximity and intent; agnostic to actor type.
- **Sensing / Equipment** — entities that read a field and produce a measurement. Not "gear" (that is logistics) but
  specifically the act of sampling a field value and surfacing a reading.
- **Supernatural profile** — the part that is genuinely, specifically _ghost_: behavioral profile, ghost type, hunting
  triggers, manifestation logic. This is the irreducible ghost concern. It cannot be generalized away because the whole
  game is about this being unknowable and dangerous.
- **Mission** — a run has a purpose and an end condition. The conductor of the current match.
- **Career / Agency** — what persists between runs: bank, reputation, map unlocks, XP.

The key observation: **player and ghost do not appear on this list as concerns.** They are compositions of concerns from
this list. A player entity is: vitals + logistics + interaction + navigation-consumer. A ghost entity is: supernatural
profile + navigation-consumer + field-emitter. The remaining shells of `unplayer` and `unghost` would own whatever is
genuinely irreducible about each — and after the drain, it becomes clearer what that actually is.

---

## Appendix: Mapping Current Crates Against Ideal Concerns

> _This section captures a follow-up discussion (2026-03-22). Speculative. Not a plan._

After sketching the ideal concern-domain list above, we compared it against the actual `Cargo.toml` crate list. The
result is neither encouraging nor discouraging — it is _specific_, which is the useful thing.

### What is already close to ideal

- **Physics fields** (`unthermal`, `unsoundfield`, `unfog`, `unlight`) — already exist as separate crates with separate
  concerns. The only outstanding issue is keeping them genuinely blind, which is the stated intent and the direction the
  Physics Field Rule in the audit protocol enforces.
- **Vitals** — `unvitals-core/plugin` already exists as a standalone concern. Its description says "Player health and
  sanity state," which reveals a residual noun-bias in the label, but the domain boundary is largely correct.
- **Interaction** — `uninteraction-core/plugin` exists and is reasonably scoped.
- **Navigation** — `unnavigation-core/plugin` exists, though the audit showed the A\* algorithm lives in `-core` (wrong
  crate type) and the plugin is correspondingly hollow.

### The wrong axis: gear

The gear split is the most instructive mismatch. The codebase has:

- `ungear` — the framework layer: slot traits, `PlayerGear`, `GearSpawnerRegistry`
- `ungearitems` — concrete item state: `Thermometer`, `SpiritBox`, `Flashlight`, etc.

That is an **implementation split** (abstract vs. concrete). The concern-domain split would instead be:

- **Logistics** — who holds what; the handoff protocol between hand, backpack, shelf, world
- **Sensing** — entities that read a field and produce a measurement

An EMF reader has both: a slot identity (logistics) and field-sampling behavior (sensing). Right now both aspects live
inside `ungearitems`. The wrong question was asked when creating that split.

### Missing domains

- **Career / Agency** (see naming note below) — there is no domain for the metagame economy. `unprofile` is shaped
  around the save file (persistence), not around the business logic of XP curves, payouts, and map unlocking. That math
  has no home; it is probably smeared between `unmission-plugin` and `unprofile-plugin`.
- **Sensing** as a distinct concern — nothing in the current crate list is named for "entities that read a field and
  produce a measurement." It is buried inside gear items.

### Already-acknowledged problems in the codebase

These appear verbatim in `Cargo.toml` comments — the problems are known:

- `unreplicon-core` — self-described as "God Crate — being drained"
- `uninput-core` — has "TIER EXCEPTION" in its comment; known to be in the wrong tier, tolerated for now
- `unsummary-core` — declared T4·4c (UI) but contains mission outcome data (see Mission Domain section above)

### The shell question

`unplayer` and `unghost` are the candidates to become thin shells after the drain. Looking at what they currently hold:

- `unplayer-core`: "Player: state, stats, asset handles" — stats → vitals, asset handles → rendering, identity stays.
  What remains is: _this entity is a player, and here is what makes it player-specific._
- `unghost-core`: "Ghost: evidence, enrage state, AI data" — evidence → mission mechanics, enrage state → supernatural
  profile, AI data → navigation-consumer behavior. What remains is the irreducible ghost identity and its behavioral
  profile.

Both of those shells are legitimate. The noun-domains are not wrong to exist; they are wrong to be as _full_ as they
currently are.

---

## Appendix: Naming — "Career" vs. "Agency"

> _Captured from discussion (2026-03-22). Just a naming note, not a plan._

The document above uses the term "Agency" for the metagame economic domain. This was flagged as probably the wrong word.
In English, "agency" has two common meanings:

1. **A company or bureau** (e.g., a detective agency, a talent agency) — the intended reading
2. **The capacity to act freely** (e.g., "player agency," "moral agency") — the unintended reading

In a game context, the second meaning is heavily dominant. A developer or AI reading `unagency` is likely to assume it
relates to player autonomy, not to a metagame company.

**"Career"** avoids this entirely. `uncareer` reads clearly: this is the domain of what persists between runs — the
player's professional track record, earnings, reputation, and progression. It maps naturally to the job metaphor already
running through the design ("The Mission is the fieldwork; the Career tracks the result").

### Mission as a sub-domain of Career

This raises an interesting structural question: Mission might not be a peer of Career at T2 — it might be a _sub-domain_
of Career. The relationship is one of delegation:

> Career says: "Go do this job." Mission executes it. Mission reports back. Career updates the ledger.

If that is the correct reading, then `unmission` and `uncareer` could be sister domains under the same parent concern,
with Mission being the ephemeral per-run state and Career being the persistent across-run ledger. The boundary between
them is the `MissionSummary` event — the handoff document.

This does not necessarily mean they should be merged. Keeping them separate is probably still correct — the separation
is exactly what lets you delete Mission logic without touching Career logic. But the naming and conceptual relationship
matter for understanding _why_ they are separate.

### `unsummary-core` as a symptom

`unsummary-core` is currently declared T4·4c (UI). It lives there because the post-mission _screen_ needed somewhere to
put its data. But the data it holds — `SummaryData`, `MissionEvaluator`, `ActiveMissionEvaluator` — is mission outcome
data evaluated at T2 time. It is the Mission's Ledger (as described in the Mission Domain section above).

The reason it ended up in T4 is a classic case of **storage location driven by presentation need**: the summary screen
needed to display something, so somewhere was found to put the data, and that somewhere was close to the screen. The
correct owner was never asked.

In the ideal model, `unsummary-core` is two things that got glued together:

- **The Ledger** (mission outcome data, grade, score) — belongs in `unmission-core` at T2
- **The Display** (which summary screen widget shows which field) — belongs in T4 UI code

Separating them would let the server track mission outcomes without compiling any UI code.

---

## Appendix: The Horizontal-Cut Problem — `unui-core` and `unrender-std`

> _Captured from design discussion (2026-03-22). This is a problem statement and directional intent, not a migration
> plan. A planning stage will follow to derive specific codebase changes._

### The Core Diagnosis: Implementation Slices vs. Feature Stacks

Two crates in the current codebase exhibit the same structural mistake in different layers:

- `unui-core` — described as "shared UI types." Functions as a horizontal bag of UI components that multiple features
  borrow from.
- `unrender-std` — described as "shared rendering support." Functions as a horizontal bag of sprite/rendering components
  that multiple domain crates import.

Both are **implementation slices**: they were created by asking "what do multiple crates need?" (a horizontal question)
instead of "what feature does this belong to?" (a vertical question).

The result is that features do not own their full vertical stack. Instead, they reach sideways into a shared pool to
borrow the face they need. This creates the tier-violation pattern visible in the audit: T1 domain crates import T4
crates because that is where the pieces they wrote were placed.

The fix is not to create a smaller shared bag at a lower tier to make the tier check pass. That is a label change, not a
structural change — it relocates the gravitational problem without solving it. The fix is to give each feature ownership
of its own types, up and down the stack.

---

### `unui-core`: Every Marker Has a Natural Owner

`unui-core` exists because features did not own their own T4 face. They borrowed from a shared pool. Looking at what it
actually holds, each component already has a natural home in a feature that exists:

| Component(s)                                                | Natural owner                                      | Reason                                                                                                                            |
| ----------------------------------------------------------- | -------------------------------------------------- | --------------------------------------------------------------------------------------------------------------------------------- |
| `WalkieTextUIRoot` / `WalkieText` / `OnScreenHintEvent`     | walkie/hint feature (`unwalkie-plugin` or similar) | These are the walkie-talkie system's own UI face. The marker lives where the feature's presentation lives.                        |
| `DamageBackground`                                          | vitals presentation, in-game HUD                   | Visual consequence of taking damage — owned by the vitals-HUD vertical, already logically in `unclassic-mode-ui-plugin` territory |
| `EvidenceUI` / `RightSideGearUI` / `HeldObjectUI`           | game-mode HUD                                      | These are classic-mode mission HUD widgets — belong in `unclassic-mode-ui-plugin`                                                 |
| `HintBoxUIRoot` / `HintBoxText`                             | hint system, game-mode HUD                         | Same HUD vertical                                                                                                                 |
| `SummaryUIType` (18-variant enum) / `SCamera` / `SummaryUI` | `unsummary-plugin`                                 | These are entirely the summary screen's concern                                                                                   |

What survives after the drain: genuine infrastructure that serves all features without belonging to any one of them.
Primarily: `UiAssets` (the font/image handle collection used everywhere) and `MouseVisibility` (a cross-cutting input
state concern). These two are legitimately shared — not because they resist classification but because they are truly
primitive artifacts (asset handle bags, cursor state). Everything else has a home.

The creation of another shared bag at T0/T1 for "common HUD markers" would reproduce the same mistake at a lower tier.
The question must always be: what feature-silo does this marker announce? That feature owns it.

---

### `unrender-std`: Simulation State Mis-filed as Presentation

`unrender-std` is named as a rendering crate and lives at T4. Yet much of what it contains is not rendering — it is
**simulation state that rendering happens to observe**.

The analogy: `Position` has a visual counterpart (a sprite appears somewhere on screen), but `Position` is not a
rendering type. It is a simulation type. The same logic applies here.

| Component                                                 | Actual domain                      | Why it is simulation, not rendering                                                                                                                                                                |
| --------------------------------------------------------- | ---------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `CharacterAnimationState` / `CharacterAnimationDirection` | `unlocomotion-core`                | Locomotion computes which direction and movement state an entity is in. Rendering reads this to pick the sprite frame. The state belongs to locomotion; the sprite selection belongs to rendering. |
| `SpriteLayer`                                             | `unboard-core` or `unspatial-core` | Draw-ordering of entities is a function of their position and depth in the spatial simulation. It is a simulation consequence, not a rendering decision.                                           |
| `LightSensitive`                                          | `unlight-core`                     | This is an ontology input that the light diffusion field reads — exactly as `ThermalEmitter` is for the thermal field. It belongs next to the field it annotates.                                  |
| `UltravioletSensitive` / `InfraredSensitive`              | sensing / supernatural domain      | These mark what a UV lamp and IR camera actually measure. They are sensing-field ontology, not rendering.                                                                                          |
| `SpectralClarity`                                         | supernatural profile               | Describes ghost visibility across spectra — a property of the ghost's supernatural state, observed by rendering.                                                                                   |
| `GameSprite` / `MapTileSprite`                            | entity identity / `unboard-core`   | These tag what kind of entity something is for animation resolution. They are identity markers in the simulation, not rendering state.                                                             |

What legitimately belongs in a rendering crate: shader material types (`CustomMaterial1/2`, `UIPanelMaterial`),
`SpriteDB` (the rendering asset database), quad mesh utilities, `GearAssets` (asset handle collection). These are
genuinely pipeline-coupled — they reference GPU-facing types or Bevy rendering APIs that have no meaning in a headless
context.

The irony of the current state: the T1→T4 import violations visible in the audit are **correct dependency
relationships** — those domain crates genuinely need those simulation components. The components were just filed in the
wrong place. The domain crates import `unrender-std` not because they want to render anything, but because that is where
their simulation types happen to live.

After the drain, the domain crates that currently import `unrender-std` would instead import only their own `-core`
crates — correctly downward. `unrender-std` would become what its name implies: a library of rendering-specific
infrastructure.

---

### The Shared Thread

Both `unui-core` and `unrender-std` are symptoms of the same root cause: when a feature needed something and there was
no obvious home for it, it was placed in a horizontal shared crate. Over time these shared crates accumulated components
from many features. The gravitational result is widespread sideways importing across tiers.

The directional fix for both:

1. For each component in the shared crate, ask: **what feature-silo does this component describe?**
2. Move the component to that feature's `-core` crate.
3. After the drain, what remains in the original shared crate is what was genuinely primitive and shared from the start.

No new shared bags should be created during this process. If a component resists classification, that is a signal to
look harder at what it describes — not a signal to create another bag.
