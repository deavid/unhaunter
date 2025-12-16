# Retelling the story from first principles

**Tell me the story of a play session. Not the code. Not the design doc. The experience.**

> "A player loads a mission. They appear at [where?]. They can see [what?]. They decide to [do what?]. Something
> responds by [what?]. They use a tool that shows [what?]. Eventually [what happens] and the game ends because [why?]."

Walk through that narrative. Every noun that appears is a candidate concept. Every verb is a candidate interaction.

---

Or, if that feels too tied to the current game, try the **two-game test**:

**Classic mode:** "The player investigates a haunted house to identify what's haunting it. They win by..."

**Escape mode:** "The player performs a ritual and tries to escape. They win by..."

---

## Classic mode

Pitch black. It softly brightens up and we just arrived at our location. Behind us there's our transport, the truck. On
the left, the house we want to investigate. We're standing just outside of the location that has grass all around it
with a stone path leading to it.

We are carrying everything, a backpack with plenty of stuff, straps and other things to have equipment hanging and also
a Tablet / PDA that allows us to get critical data from anywhere.

From the outside nothing is out of the ordinary. Maybe it's us the ones that are out of the ordinary peeking into a
house in the middle of the night. We can hear distant noises of transport, wild life, the grass moving from the wind.

As we know nothing about what might be dwelling in there, the plan is to enter with caution, slowly - and keep track of
the EMF, Temperature, and Geiger counter. Maybe there's nothing, or a very angry Demon. So we open the door and proceed
at a snail walking pace. The floor creeks as we walk, the ambient is completely different from outside - it is
oppressive.

We turn on the flashlight and walk deeper in. So far everything is normal, temps are a bit cold, EMF chirps here and
there but just EMF1, maybe rarely EMF2. Nothing to be worried of.

We do not want to agitate whatever is inside, so it's imperative that we disturb the contents as little as possible,
that we make as little noise as possible.

The Geiger counter has been picking something for a bit, confirming that there's something inside. After examining for a
bit we pick up a trail of cold spots with the thermometer - temperature seems to get colder in a particular direction.

The EMF suddenly picks pace, we're very close. Some shadow seemed to cross our side vision, something moved.

This seems to be the spot, or close by, so we decide to plant a videocamera and throw to the ground the EMF and
Thermometer. We grab the UV Torch to look around for ectoplasm. The thermometer beeps, it's -0.5ºC, which means it's
freezing. This is the first piece of evidence. The EMF starts getting crazy, the flashlight starts going crazy -
something is going very wrong.

After hearing a growl we decide to make a run before it gets worse. We manage to get back outside, out of breath. It's
incredibly hard to breathe in the atmosphere of that house.

We ran a sanity check on our PDA, and after a few seconds it came out as 45%. Trying to be "easy" on the entity also
drained our sanity, as we were going way too much time in the dark.

As we rest outside our sanity will restore, and using this time we take a look at the sensors. All our gear communicates
with the PDA, so we can read them remotely. The EMF is still going off very frequently. Temperatures are oscillating
between -1ºC and 2ºC. The videocamera feed shows some distortions from time to time, and sometimes a shadow.

A minute in and we located that the shadow seems to go towards some weird direction. And also the activity seems to have
faded down. We do another sanity check and we're up to 80% now. So we decide to get in again, but this time we will
secure the path. We grab from our backpack a pack of candles, and we lit them in the ground in the path from the exit
door till the cold room. This will prevent the sanity drop and it's the least intrusive way with the entity.

## Escape mode

What's different:

Ghost behavior:

- In Classic: Ghost roams the ghost room (selected randomly) + 2 objects in the house are haunted and attract the ghost
  so it roams them.
- In Escape: ghost roams the whole location, but will be "asleep" on start. There are around 4 items needed to do a
  ritual and the ghost protects them. The ghost will be wary of anyone getting too close or aiming too fast in the right
  direction. If they're disturbed the ghost will rage.

Main goal:

- Classic: Identify the ghost by using tools, then once you get the name of the ghost, use a specifically crafted
  repellent to expel it.
- Escape: Identify what items in the house are powering this haunting - use the gear to locate them. Extract the items
  or perform a ritual. The location closes down: survive. Exiting the location alive after the goal is complete is what
  marks success.

---

## Analysis: Extracting Concepts

### Nouns from Classic narrative (candidate concepts)

| Noun                                                                 | Category            |
| -------------------------------------------------------------------- | ------------------- |
| Location, house, outside/inside                                      | Space               |
| Truck                                                                | Base/refuge         |
| Door, floor                                                          | Interactive objects |
| Backpack, equipment, straps                                          | Inventory system    |
| Tablet/PDA                                                           | Remote sensing      |
| EMF, Thermometer, Geiger, UV Torch, Videocamera, Candles, Flashlight | Tools/Gear          |
| Temperature, EMF readings, Radiation                                 | Field values        |
| Entity, Demon, Shadow                                                | The threat          |
| Sanity                                                               | Player state        |
| Evidence, Ectoplasm, Freezing temps                                  | Clues/diagnosis     |
| Noise, ambient, atmosphere                                           | Environmental mood  |

### Verbs from Classic narrative (candidate interactions)

| Verb             | What it operates on                    |
| ---------------- | -------------------------------------- |
| Walk (slow/fast) | Player → Space                         |
| Open             | Player → Door                          |
| Turn on/off      | Player → Flashlight                    |
| Plant/throw/grab | Player ↔ Gear                          |
| Track/read       | Gear → Field                           |
| Disturb/agitate  | Player actions → Entity state          |
| Run/flee         | Player escape                          |
| Check sanity     | Player → PDA → Sanity value            |
| Rest/restore     | Time → Sanity                          |
| Light candles    | Player → Space (prevents sanity drain) |

### Key observation

The player doesn't interact with the entity directly. They interact with **space**, **gear**, and **their own state**.
The entity is known only through **field readings** and **environmental changes**.

---

## The Split: What's Shared vs What's Different

### SHARED (Engine territory)

| Concept                            | Why shared                                                  |
| ---------------------------------- | ----------------------------------------------------------- |
| Location/Space/House               | Both modes have a place to explore                          |
| Ghost/Entity exists                | Both have something dwelling there                          |
| Ghost roams                        | Both have entity movement (different rules, same mechanism) |
| Objects can be "special"           | Haunted objects (Classic) or ritual items (Escape)          |
| Tools/Gear                         | Same equipment for sensing                                  |
| Fields (EMF, temp, etc.)           | Same sensing targets                                        |
| Player movement, sanity, inventory | Identical                                                   |
| Danger escalates                   | Both have a tension ramp                                    |
| Exits                              | Both need to leave                                          |

### DIFFERENT (Game Mode territory)

| Classic                                    | Escape                                            |
| ------------------------------------------ | ------------------------------------------------- |
| Ghost tied to ghost room + haunted objects | Ghost roams whole location, protects ritual items |
| Ghost is passive (you disturb it)          | Ghost is wary/guarding (it watches you)           |
| Goal: **identify** (diagnosis)             | Goal: **extract/ritual** (action)                 |
| Win: craft repellent, expel                | Win: complete ritual, escape alive                |
| "Evidence" concept                         | "Ritual items" concept                            |
| Ghost has a **name/type** to discover      | Ghost has **items** to steal                      |

---

## The Boundary

The **mechanisms** are identical:

- Entity exists and moves according to rules
- Entity reacts to player actions
- Player uses tools to read fields
- Player has a goal, completes it, exits

The **meaning** differs:

- Classic: readings → diagnosis → crafted solution
- Escape: readings → location → theft → survival

### Engine provides:

- Entity that roams according to _configurable_ rules
- Objects that can be _marked as special_ (the mode decides what "special" means)
- Goal completion hook (the mode defines what "complete" means)
- Exit/win condition hook

### Game mode provides:

- What roaming rules the entity uses
- What makes objects special (haunted vs ritual)
- What the player must do to "win"
- What "identifying" or "extracting" means

---

## Other Games

Here is a list of highly-regarded games that cover the **horror, unsettling, liminal, puzzle, investigative, and analog
horror** genres:

### 📼 Analog & Found Footage Horror

These games often utilize a VHS aesthetic, old computer interfaces, or found media to build their unique, unsettling
atmosphere.

- **Stories Untold:** A narrative horror game split into four episodes, each featuring a different experimental horror
  scenario centered around text-based adventures, retro computer terminals, and puzzle-solving. It's a prime example of
  analog horror aesthetics.
- **The Mortuary Assistant:** A horror game where you must perform embalming tasks while simultaneously investigating
  and banishing a malevolent entity. It uses a mix of procedural generation and environmental clues to create a deeply
  unsettling and investigative loop.
- **Home Safety Hotline:** A narrative puzzle/horror game where you work at a 90's call center, identifying and advising
  on threats (which can be mundane, paranormal, or outright terrifying) based on cryptic clues and analog media.
- **LIMINAL SHROUD:** An exploration game with a strong 1970s analog horror and dreamcore aesthetic. It focuses on pure
  atmospheric dread and wandering through unsettling, empty liminal spaces.
- **VIDEO NASTY:** A creepy analog horror Full Motion Video (FMV) puzzle game.

### 🏢 Liminal Space & Unsettling Exploration

These focus on the unnerving feeling of familiar but empty, out-of-place environments.

- **P.T. (Playable Teaser):** Though only a teaser, it's one of the most famous examples of an unsettling, looping
  liminal space combined with complex, disturbing puzzles.
- **Control:** While not strictly horror, it has heavy elements of cosmic horror and investigative mystery, taking place
  in a vast, shifting, brutalist building (the Oldest House) that is the definition of unsettling, abstract liminality.
- **Anemoiapolis** and **Pools:** Often cited as top-tier examples of liminal space horror that rely on atmosphere and
  exploration, often without traditional "monsters." _Pools_ is particularly praised for its terrifying lack of
  monsters.
- **The Stanley Parable:** Not a horror game, but its endless, shifting corporate office environment and existential
  themes perfectly capture the **unsettling/liminal** vibe.

### 🔎 Investigative & Puzzle Horror

These games require meticulous deduction and puzzle-solving, often to uncover a dark or terrifying truth.

- **Return of the Obra Dinn:** A unique investigative puzzle game where you must determine the fate and identity of the
  60 crew members of an abandoned ship using a magical pocket watch that lets you witness their final moments. It's
  deduction-heavy and has a macabre, unsettling feel.
- **Her Story / Telling Lies / Immortality (by Sam Barlow):** These are FMV-based, non-linear investigative games. You
  sort through a database of live-action video clips to piece together a story, with _Immortality_ being the most
  overtly **psychological horror** of the group.
- **Alan Wake 2:** A survival horror and investigative game that features two protagonists, with the writer, Alan Wake,
  experiencing a reality heavily influenced by his own writing in a dark, shifting reality that is both unsettling and
  deeply puzzling.
- **Silent Hill 2:** A classic survival horror masterpiece. Its fog-choked town is a massive **liminal space**, and the
  core narrative is an intense **investigative** journey to solve the mystery of James' deceased wife, filled with
  abstract puzzles and deep psychological horror.

### 👻 The Core Ghost-Hunting Experience (Direct Competitors)

These games are the most similar to _Phasmophobia_, focusing on a team of investigators using equipment to gather
evidence and identify a paranormal entity.

| Game Title              | Focus/Key Difference                                                                                                                              | Co-op Size      |
| :---------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------ | :-------------- |
| **Phasmophobia**        | The original psychological co-op investigation with realistic equipment and voice interaction.                                                    | 4 Players       |
| **Demonologist**        | A more challenging and graphically intense experience built on Unreal Engine 5. Often requires an exorcism ritual after identifying the ghost.    | 4 Players       |
| **Ghost Exorcism Inc.** | Focuses more heavily on the exorcism process and has larger team sizes and more varied locations.                                                 | Up to 6 Players |
| **Forewarned**          | Takes the investigation to Egyptian tombs, hunting mummies and uncovering ancient secrets. Features procedural generation for high replayability. | 4 Players       |
| **Ghost Watchers**      | Similar formula with a slightly more action-oriented feel and a wider variety of unique ghosts.                                                   | 4 Players       |

### 🏃 Intense Co-op Survival & Action (Survival-Horror Twist)

These games involve investigation but quickly ramp up the intensity into a relentless survival or objective-based chase.

| Game Title             | Focus/Key Difference                                                                                                                                                                                                                              | Co-op Size      |
| :--------------------- | :------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | :-------------- |
| **Devour**             | An objective-based co-op game where players gather ritual items to purify a map while a relentlessly fast and aggressive demon hunts them down.                                                                                                   | 4 Players       |
| **Pacify**             | Faster-paced, shorter rounds focused on a single, increasingly aggressive entity. Often described as a condensed, high-panic horror session.                                                                                                      | 4 Players       |
| **Lethal Company**     | A sci-fi twist on the formula. You and your crew scavenge scrap from dangerous, moon-based facilities to meet a corporate quota while surviving environmental and monster threats. Huge viral hit with a strong emphasis on proximity chat chaos. | 4 Players       |
| **The Outlast Trials** | While not ghost-hunting, this co-op entry in the _Outlast_ series features a team of "test subjects" completing objectives while avoiding horrific enemies.                                                                                       | Up to 4 Players |

### 🆚 Asymmetrical & Competitive Horror (Vs. Player)

These games feature one or more players controlling the entity hunting the rest of the team.

| Game Title              | Focus/Key Difference                                                                                                                                                                      | Co-op Size                                |
| :---------------------- | :---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | :---------------------------------------- |
| **White Noise 2**       | Asymmetrical 4v1 horror where four investigators collect evidence, and one player controls the monster hunting them down.                                                                 | 5 Players (4 Investigators vs. 1 Monster) |
| **Dead by Daylight**    | The dominant asymmetrical horror game. Four survivors attempt to fix generators to escape while one killer (often from iconic horror franchises) tries to sacrifice them.                 | 5 Players (4 Survivors vs. 1 Killer)      |
| **Midnight Ghost Hunt** | A PvP (Player vs. Player) "prop hunt" style game where ghosts hide by possessing objects, and ghost hunters must eliminate them before the clock runs out and the ghosts gain full power. | Up to 8 Players (4 Hunters vs. 4 Ghosts)  |

