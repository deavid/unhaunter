# Unhaunter: The Comparative Engine Analysis Report

## 1. Executive Summary: The Identity of Unhaunter

**Unhaunter** is defined as a **Nightmare Simulator** and **Occult Forensics** game. It rejects the arcade loops of its
competitors in favor of deep simulation.

- **The Fantasy:** You are a **Trespasser** in a hostile ecosystem, not a hero.
- **The Loop:** Infiltrate → Diagnose (Abductive Reasoning) → Manipulate (Engineering) → Ritual → Escape.
- **The Key Differentiator:** "Illiterate Tools." The game provides physical symptoms (Entropy, Temperature, Sound); the
  player must provide the diagnosis.

This report documents the architectural deconstruction of over 25 reference games to isolate the "True Components"
required for the **Unhaunter Engine**.

---

## 2. The Architectural Standard

To prevent "Feature Soup," all proposed ideas are subjected to two rigorous tests.

### 2.1 The Component Definition

> **A Component is a discrete, measurable property that can be attached to an entity, has a representable state, and can
> be queried by systems.**

| Test                 | Question                                      |
| :------------------- | :-------------------------------------------- |
| **1. Attachable**    | Can it belong to a specific entity?           |
| **2. Representable** | Can it be encoded as data (Struct/Enum)?      |
| **3. Queryable**     | Can a system ask "Who has this?"              |
| **4. Separable**     | Can it be removed without killing the entity? |
| **5. Observable**    | Does it have a current value?                 |

### 2.2 The Struct Test

> **"Show me the struct. What fields does it have?"**

If a concept cannot be expressed as a struct with typed fields, it is NOT a component. It may be a System, Resource,
Entity, or just a vague idea.

### 2.3 The Gameplay Narrative Test

Every component must answer:

> **"When the player does X, the [Component].field changes from Y to Z, which causes System S to do W."**

If you cannot write this sentence, the component is not well-defined.

---

## 3. Layer Assignment Rules

To stop arbitrary classification, each layer has a strict rule:

| Layer          | Rule                                                                                    | Scope                                                            |
| :------------- | :-------------------------------------------------------------------------------------- | :--------------------------------------------------------------- |
| **ENGINE**     | Must be **genre-agnostic**. Could make a Sci-Fi Stealth game without changing the code. | Space, Physics (Light/Sound/Heat), Interaction, I/O, Networking. |
| **SHARED MOD** | Essential to **"Paranormal Investigation"** but agnostic to specific mission type.      | Ghost concepts, Sanity, Evidence types, Tool feedback logic.     |
| **GAME MODE**  | Specific to the **win/loss loop** of a single game type.                                | AI State Machines, Ritual Steps, Scoring, Specific UI flows.     |

---

## 4. What Is NOT a Component

These were incorrectly classified in earlier drafts:

| Name                 | Claimed As | Actually Is               | Why                                         |
| :------------------- | :--------- | :------------------------ | :------------------------------------------ |
| `MapGenerator`       | Component  | **System (Engine)**       | Logic that spawns entities                  |
| `SearchableDatabase` | Component  | **Resource (Shared Mod)** | Global `HashMap<String, LoreEntry>`         |
| `AudioProcessor`     | Component  | **System (Engine)**       | Processes the audio buffer                  |
| `NarratorTrigger`    | Component  | **System (Shared Mod)**   | Queries state and queues audio              |
| `DeductionBoard`     | Component  | **Entity**                | A mesh with `Interactable` + `WorldSpaceUI` |

---

## 5. Thematic Cluster Analysis

We analyzed reference games by grouping them into functional layers to determine what belongs in the **Engine**
(Universal), **Shared Mod** (Unhaunter Standard), or **Game Mode** (Specific Rules).

### Cluster A: Interface, Investigation & Database

_Focus: How the player acquires and processes information._

| Game                        | Key Insight                                                                       | Derived Concept                               | Layer          |
| :-------------------------- | :-------------------------------------------------------------------------------- | :-------------------------------------------- | :------------- |
| **Stories Untold**          | The UI should be the world (Diegetic). Low-fidelity data creates high engagement. | `WorldInteractableUI`                         | Engine         |
| **Home Safety Hotline**     | "Reading the Manual" is gameplay. Diagnosing remote descriptions is fun.          | `RemoteSignalSource` / `RemoteSignalReceiver` | Engine         |
| **Her Story**               | Searching a database is a mechanic. Non-linear lore is realistic.                 | SearchableDatabase (Resource)                 | Shared Mod     |
| **Return of the Obra Dinn** | Information is the reward. Death is a puzzle, not just a fail state.              | `ForensicSnapshot`, `EvidenceLink`            | Engine, Shared |
| **Alan Wake 2**             | Deduction requires visualization (The Mind Place / Corkboard).                    | DeductionBoard (Entity)                       | Shared Mod     |
| **VIDEO NASTY**             | Media is a sensor. Glitches are evidence.                                         | `MediaAsset`, `InterferenceReceiver`          | Engine, Shared |

### Cluster B: Physicality, Rituals & Interaction

_Focus: How the player touches and manipulates the world._

| Game                       | Key Insight                                                      | Derived Concept                    | Layer          |
| :------------------------- | :--------------------------------------------------------------- | :--------------------------------- | :------------- |
| **The Mortuary Assistant** | Horror works best when the player is busy with a precise task.   | `ContinuousInteractable`, `Tether` | Engine         |
| **Demonologist**           | Rituals are "Combo Locks." Visuals (Render Layers) create magic. | `ItemSocket`, `RenderLayerMask`    | Shared, Engine |
| **Ghost Watchers**         | Aggression is information. Chemistry is a combat mechanic.       | `CountermeasureReaction`, `Lure`   | Shared Mod     |
| **Devour**                 | Carrying items should be a burden (Encumbrance).                 | `ItemDebuff`                       | Shared Mod     |
| **Pacify**                 | Players enjoy negotiating/toggling the monster's state.          | `PowerDependency`                  | Shared Mod     |

### Cluster C: Environment, Atmosphere & Space

_Focus: The "Liminal" feel and dynamic geometry._

| Game                     | Key Insight                                                                           | Derived Concept                         | Layer      |
| :----------------------- | :------------------------------------------------------------------------------------ | :-------------------------------------- | :--------- |
| **Liminal Shroud**       | Post-processing _is_ gameplay. The "Health Bar" of the room is the visual distortion. | `LocalAtmosphere`                       | Engine     |
| **Pools / Anemoiapolis** | Audio is the monster. Slowness is texture.                                            | `FluidDrag`, `ReverbZone`               | Engine     |
| **P.T.**                 | Reusing geometry is scarier than new geometry. Spatial mutation.                      | `ConditionalVisibility`, `TeleportLink` | Engine     |
| **Control**              | The house is alive. Bureaucracy adds tone.                                            | `WorldModifier`                         | Engine     |
| **The Stanley Parable**  | The Narrator (Audio) guides the player based on triggers.                             | NarratorSystem (System)                 | Shared Mod |

### Cluster D: Simulation, AI & Systems

_Focus: The "Brain" of the game._

| Game                   | Key Insight                                                          | Derived Concept                                | Layer          |
| :--------------------- | :------------------------------------------------------------------- | :--------------------------------------------- | :------------- |
| **Phasmophobia**       | The baseline. Evidence thresholds and "Zone Anchoring" are critical. | `ZoneOwnership`, `EvidenceState`               | Game, Shared   |
| **Forewarned**         | Loot incentivizes risk. AI needs distinct sensory profiles.          | `SensoryProfile`, MapGenerator (System)        | Engine         |
| **Ghost Exorcism Inc** | Indirect control via pathfinding costs (Tower Defense).              | `AreaDenial`, `SpiritualStability`             | Engine, Shared |
| **Silent Hill 2**      | Audio is a radar (Radio Static). Fog denies vision.                  | `InterferenceReceiver`, `VolumetricObscurance` | Shared, Engine |

### Cluster E: Multiplayer, Physics & Social

_Focus: Co-op dynamics and physical interactions._

| Game                    | Key Insight                                               | Derived Concept                              | Layer          |
| :---------------------- | :-------------------------------------------------------- | :------------------------------------------- | :------------- |
| **Lethal Company**      | Voice chat physics are mandatory. Radar blips are scary.  | AudioProcessingSystem (System), `SensorBlip` | Engine, Shared |
| **The Outlast Trials**  | Private horror (Hallucinations) creates social confusion. | `PerceptionFilter`, `NoiseTrap`              | Engine         |
| **White Noise 2**       | Synergy: One player tracks, one stuns.                    | `TrailEmitter`, `LightReactive`              | Engine, Shared |
| **Midnight Ghost Hunt** | Physics possession (Poltergeist).                         | `PuppetTarget`, `Destructible`               | Engine         |
| **Dead by Daylight**    | Tasks should blind the player (Tunnel Vision).            | `FocusInteraction`                           | Engine         |

---

## 6. The Unhaunter Component Library

Every component has a **struct definition** and a **gameplay narrative** proving its necessity.

### 6.1 Physics & Space (Engine)

These are genre-agnostic. A sci-fi stealth game could use them unchanged.

| Component             | Struct                                                                                   | Gameplay Narrative                                                                                                                                                   |
| :-------------------- | :--------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| **`GridTransform`**   | `{ coords: IVec3, orientation: Direction }`                                              | "The player moves from (5,3,0) to (6,3,0). The Movement System updates `coords`, and the Render System repositions the sprite."                                      |
| **`ThermalMass`**     | `{ current_temp_c: f32, conductivity: f32, specific_heat: f32 }`                         | "When the Ghost enters the room, the `current_temp_c` of air tiles drops; the radiator's high `specific_heat` makes it drop slower, creating a detectable gradient." |
| **`AcousticEmitter`** | `{ volume_db: f32, frequency_range: Range<f32>, is_active: bool }`                       | "When the player steps on 'Broken Glass', it enables its `AcousticEmitter`, causing the Sound System to propagate noise."                                            |
| **`FieldAttenuator`** | `{ light_block_factor: f32, sound_dampening_db: f32 }`                                   | "The closed door has `sound_dampening_db: 20.0`. The Sound System reduces propagated volume by 20dB through this tile."                                              |
| **`FluidDrag`**       | `{ viscosity: f32, flow_vector: Vec3 }`                                                  | "When the player enters the Miasma, `viscosity: 0.8` reduces their Character Controller velocity by 80%."                                                            |
| **`Destructible`**    | `{ integrity: f32, fracture_threshold: f32, broken_mesh: Handle<Mesh> }`                 | "When the Poltergeist applies 50N of force to the Vase, its `integrity` drops to 0, triggering swap to `broken_mesh`."                                               |
| **`Tether`**          | `{ anchor_entity: Entity, max_radius: f32, elasticity: f32 }`                            | "The Haunted Doll tries to physics-throw itself, but the `Tether` constrains it to within 2.0m of the Ritual Circle."                                                |
| **`TeleportLink`**    | `{ destination: Entity, trigger_bounds: Aabb, seamless: bool }`                          | "The player walks through the hallway end. The `TeleportLink` seamlessly moves them to the other end without a loading screen."                                      |
| **`LocalAtmosphere`** | `{ fog_density: f32, fog_color: Color, grain_intensity: f32, color_grade: Handle<Lut> }` | "The player enters the Miasma zone. The PostProcess System blends to `fog_density: 0.8`, obscuring vision."                                                          |

### 6.2 Interaction & Data (Engine)

Generic interaction patterns usable by any game type.

| Component                    | Struct                                                                        | Gameplay Narrative                                                                                                                |
| :--------------------------- | :---------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------------- |
| **`ContinuousInteractable`** | `{ progress: f32, required_time: f32, decay_rate: f32 }`                      | "The player holds 'E' on the Fuse Box. `progress` increases. Releasing causes `decay_rate` to drop it back, forcing commitment."  |
| **`ItemSocket`**             | `{ accepted_tags: BitMask, held_item: Option<Entity>, lock_on_insert: bool }` | "The Player places the 'Finger' item into the 'Ritual Bowl'. The `ItemSocket` detects the matching tag and captures the entity."  |
| **`RemoteSignalSource`**     | `{ channel_id: u8, signal_strength: f32, data_type: SignalType }`             | "The Video Camera broadcasts its Render Target to Channel 1, which the Truck Monitor's `RemoteSignalReceiver` displays."          |
| **`WorldInteractableUI`**    | `{ canvas_size: Vec2, interaction_bounds: Aabb }`                             | "The player looks at the PC terminal. Their mouse input is raycasted to the `WorldInteractableUI` surface."                       |
| **`NoiseTrap`**              | `{ activation_velocity: f32, sound_on_trigger: Handle<AudioSource> }`         | "The player runs over the glass. Impact velocity exceeds `activation_velocity`, playing the crunch sound and alerting the Ghost." |
| **`TrailEmitter`**           | `{ particle_type: ParticleId, emission_rate: f32, lifetime: f32 }`            | "The Ghost walks through the room. `TrailEmitter` spawns UV-visible footprint particles every 0.5 seconds."                       |
| **`AreaDenial`**             | `{ nav_cost_modifier: f32, shape: ColliderShape, affects_tags: BitMask }`     | "The salt line has `nav_cost_modifier: 100.0`. The Ghost's pathfinding avoids crossing it unless desperate."                      |

### 6.3 Perception & Rendering (Engine)

How information reaches the player's senses.

| Component                   | Struct                                                                              | Gameplay Narrative                                                                                                          |
| :-------------------------- | :---------------------------------------------------------------------------------- | :-------------------------------------------------------------------------------------------------------------------------- |
| **`PerceptionFilter`**      | `{ hallucination_intensity: f32, color_shift: Color, ghost_entities: Vec<Entity> }` | "The player's Sanity drops below 20. `hallucination_intensity` rises, and the Render System spawns fake ghost silhouettes." |
| **`RenderLayerMask`**       | `{ visible_layers: BitMask }`                                                       | "The Ecto-Glass tool sets the player's camera to `visible_layers: SPECTRAL`. Ghost writing becomes visible."                |
| **`ConditionalVisibility`** | `{ condition: VisibilityCondition, target_entity: Entity }`                         | "The bloody handprint has `condition: SanityBelow(30)`. It only renders when the player's Sanity is critically low."        |
| **`VolumetricObscurance`**  | `{ density: f32, scatter_color: Color, height_falloff: f32 }`                       | "The Fog System reads `density` to determine how far the player can see. High density = Silent Hill."                       |

### 6.4 AI Perception (Engine)

Generic AI sensing — usable for any enemy type, not just ghosts.

| Component            | Struct                                                                                         | Gameplay Narrative                                                                                                  |
| :------------------- | :--------------------------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------ |
| **`SensoryProfile`** | `{ vision_range: f32, vision_cone_deg: f32, hearing_threshold_db: f32, thermal_vision: bool }` | "The player turns on a flashlight. The Ghost's `thermal_vision: true` detects the heat signature through the wall." |
| **`PuppetTarget`**   | `{ is_possessed: bool, control_strength: f32 }`                                                | "The Poltergeist possesses a chair. `is_possessed: true` lets the AI drive the physics object directly."            |

### 6.5 Investigation & Status (Shared Mod)

These define "Paranormal Investigation" as a genre but not a specific game mode.

| Component                    | Struct                                                                       | Gameplay Narrative                                                                                                                     |
| :--------------------------- | :--------------------------------------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------- |
| **`Sanity`**                 | `{ current: f32, max: f32, drain_rate_modifiers: Vec<f32> }`                 | "The player stands in darkness. The Sanity System subtracts from `current`. Below 20, Hallucination System activates."                 |
| **`FieldEmitter`**           | `{ field_type: FieldTypeId, intensity: f32, range: f32 }`                    | "The Ghost has `field_type: EMF, intensity: 5.0`. The EMF Reader within range displays Level 5."                                       |
| **`InterferenceReceiver`**   | `{ threshold: f32, current_noise: f32 }`                                     | "The Video Camera's `current_noise` increases near the Ghost. The Rendering System blends in static based on this value."              |
| **`EvidenceState`**          | `{ evidence_type: EvidenceId, discovery_progress: f32, is_confirmed: bool }` | "The Thermometer sits in the cold room. Over 5 seconds, `discovery_progress` fills. Once full, the Journal marks 'Freezing' as found." |
| **`IdentityTag`**            | `{ true_name: String, death_cause: DeathCause, age: u16 }`                   | "The player finds a Medical Record. A system links it to the Ghost's `IdentityTag`, revealing `death_cause: Drowning`."                |
| **`PhotoTarget`**            | `{ tags: Vec<PhotoTag>, reward_value: u32, max_distance: f32 }`              | "The player takes a picture. The camera raycasts, hits the Ghost's `PhotoTarget`, and logs 'Ghost Photo' in the Journal."              |
| **`CountermeasureReaction`** | `{ item_tag: ItemTag, reaction: ReactionType, intensity: f32 }`              | "The player throws salt at the Ghost. Its `CountermeasureReaction` for Salt triggers `reaction: Burn`, dealing sanity damage to it."   |
| **`SpiritualStability`**     | `{ current: f32, max: f32 }`                                                 | "The player completes a ritual step. The Ghost's `SpiritualStability.current` decreases. At 0, banishment succeeds."                   |

### 6.6 AI Behavior (Game Mode)

Specific to the win/loss loops of Classic or Escape modes.

| Component             | Struct                                                              | Gameplay Narrative                                                                                               |
| :-------------------- | :------------------------------------------------------------------ | :--------------------------------------------------------------------------------------------------------------- |
| **`ZoneOwnership`**   | `{ favored_zone_ids: Vec<ZoneId>, roam_chance: f32 }`               | "The Ghost AI checks `ZoneOwnership`. It picks Zone 5 (Kitchen) as its target because it's in the favored list." |
| **`AggressionState`** | `{ current_level: f32, escalation_rate: f32, hunt_threshold: f32 }` | "The player speaks the Ghost's name. `current_level` spikes. It crosses `hunt_threshold`, triggering the Hunt."  |
| **`PowerDependency`** | `{ power_source: Entity, behavior_when_off: BehaviorId }`           | "The player cuts the breaker. The Ghost's `PowerDependency` triggers `behavior_when_off: Aggressive`."           |
| **`LightReactive`**   | `{ light_threshold_lux: f32, reaction: ReactionType }`              | "The player shines the UV light. The Ghost's `LightReactive` triggers at 500 lux, causing it to retreat."        |

---

## 7. De-duplication & Clarifications

Resolved conflicts from earlier drafts:

| Conflict                                     | Resolution                                                                                                                                              |
| :------------------------------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------ |
| **`Tether` vs `ZoneAnchor`**                 | `Tether` (Engine) = Physics constraint, physically stops movement. `ZoneOwnership` (Game Mode) = AI preference, ghost _prefers_ a zone but _can_ leave. |
| **`EntropySource` vs `Destructible`**        | `FieldEmitter` (Shared Mod) is the **Cause** — ghost emits cold/EMF. `Destructible` (Engine) is the **Effect** — vase breaks when force applied.        |
| **`DataTransmitter` vs `MediaSource`**       | `RemoteSignalSource` (Engine) **sends** data (camera broadcast). `MediaAsset` **is** data (a .mp4 file on disk).                                        |
| **`ProximityStatic` vs `SignalDegradation`** | Merged into `InterferenceReceiver` (Shared Mod). Threshold + current_noise, applied by rendering system.                                                |

---

## 8. Reconciliation with Existing Unhaunter

How proposed components map to what already exists:

| Existing System   | Proposed Component                   | Relationship                                                   |
| :---------------- | :----------------------------------- | :------------------------------------------------------------- |
| Board/Tile Grid   | `GridTransform`                      | New component wrapping coords for ECS compatibility            |
| Light Propagation | `FieldAttenuator`                    | Walls/doors have this; system reads it for propagation         |
| Sound Propagation | `FieldAttenuator`, `AcousticEmitter` | Emitters produce, attenuators block, tiles store result        |
| Temperature Field | `ThermalMass`, `FieldEmitter`        | Ghost's FieldEmitter causes ThermalMass to change              |
| Ghost Types       | `IdentityTag`                        | Moves strings into structured data for deduction               |
| Evidence Journal  | `EvidenceState`                      | Per-evidence-type progress tracking                            |
| Sanity            | `Sanity`                             | Direct mapping, adds drain_rate_modifiers                      |
| Gear/Tools        | Various                              | Tools query `FieldEmitter`, display via `InterferenceReceiver` |

---

## 9. Conclusion: The "True Unhaunter" Architecture

The analysis confirms that **Unhaunter** is a platform for **Systemic Horror**.

### The Three Layers

```
┌─────────────────────────────────────────────────────────────────┐
│                        GAME MODE                                │
│  Classic: Diagnose IdentityTag via EvidenceState                │
│  Escape: Reduce SpiritualStability via ItemSockets              │
│  (ZoneOwnership, AggressionState, PowerDependency, LightReactive)│
├─────────────────────────────────────────────────────────────────┤
│                        SHARED MOD                               │
│  "Paranormal Investigation" genre definitions                   │
│  (Sanity, FieldEmitter, EvidenceState, IdentityTag,             │
│   InterferenceReceiver, CountermeasureReaction, PhotoTarget)    │
├─────────────────────────────────────────────────────────────────┤
│                          ENGINE                                 │
│  Genre-agnostic simulation                                      │
│  (GridTransform, ThermalMass, AcousticEmitter, FieldAttenuator, │
│   FluidDrag, Destructible, Tether, TeleportLink, LocalAtmosphere,│
│   ContinuousInteractable, ItemSocket, RemoteSignalSource,       │
│   PerceptionFilter, SensoryProfile, AreaDenial, TrailEmitter)   │
└─────────────────────────────────────────────────────────────────┘
```

### What Each Layer Provides

1. **The Engine** provides a simulation of:

   - **Space** (Euclidean via GridTransform, Non-Euclidean via TeleportLink)
   - **Physics** (Light, Sound, Fluid, Heat — via Attenuators and Emitters)
   - **Perception** (Visual Distortion, Hallucinations — via PerceptionFilter)
   - **Data** (Transmission from Sensor → Screen via RemoteSignal\*)
   - **AI Sensing** (Generic perception via SensoryProfile)

2. **The Shared Mod** provides the "Ghost Hunting Standard":

   - Definitions of EMF, Temperature, and Sanity as meaningful concepts
   - The concept of a "Ghost" as an entity with FieldEmitter + IdentityTag
   - Tools that read Engine data and output feedback (InterferenceReceiver)
   - Evidence discovery mechanics (EvidenceState, PhotoTarget)

3. **The Game Mode** provides the Loop:
   - **Classic:** Diagnose the `IdentityTag` using `EvidenceState` accumulation
   - **Escape:** Reduce `SpiritualStability` via `ItemSocket` rituals while evading aggressive AI

This architecture allows _Unhaunter_ to support the mechanics of _Phasmophobia_, _P.T._, _Control_, and _Stories Untold_
simultaneously, by abstracting their specific gimmicks into generic, struct-defined components.
