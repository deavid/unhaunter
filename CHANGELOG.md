### Version 0.3.1-beta1 - 2025-06-29

**Mouse Aiming**

* Now you can hover with the mouse around and the flashlight will follow.
* Flashlight visual enhanced for long range scan.
* Clicking now will make the character walk towards the cursor.
* Clicking nearby something activable/interactive, will also interact with it, as if it were the [E] key.
* Right click will enable/disable right hand equipment, like the [R] key.
* Scrollwheel on the mouse will cycle through inventory like [Q].
* Walking around with mouse has pathfinding and indicators for waypoints. Works on stairs too.

**Journal UI**

* Discard evidence hidden away, now the user has to Shift+Click.
* Simpler, better Ghost filtering logic.
* If only one ghost is possible, it auto-selects. If the ghost selected is no longer possible, it de-selects.
* Improved UI element visibility with adjusted border thickness and padding.
* Implemented repellent crafting limits and refund system based on difficulty level.
* Player automatically exits truck after crafting repellent.
* Added GrayMan to TmpEMFUVOrbs ghost set - now we have 6 ghosts for missions 4, 5 and 6.

**Gameplay Improvements:**

* Miasma can now spread through half-walls and other see-through obstacles.
* Temperature spreads more realistically through walls and obstacles, including through stairs.
* Repellent particles now change color upon hitting ghosts (electric blue for correct, bright red for incorrect).
* Smart hint system provides feedback for incorrect repellent usage.
* Adjusted Temps/Geiger to be easier and more reliable to get.

**Walkie-Talkie System:**

* Ghost hunt warnings are now only triggered if the player is inside the location and the ghost is sufficiently healthy.
* Walkie-talkie hints for player sanity and wellbeing are now more sensitive and frequent.
* Increased repeat frequency for hints about crafting repellents when enough evidence is gathered.
* Walkie event priority is reduced if messages have been played before to avoid repetition.
* Adjusted timing and delays for better hint delivery.

**UI/UX Enhancements:**

* Increased visible duration for hint UI messages.
* Better camera reference point for improved player centering.
* Adjusted flashlight lighting effect.

**Other:**

* Bevy upgraded to version 0.16
* Complete overhaul of the `ghost_list` CLI tool.

**Fixes:**

* Fixed error on negative miasma pressure that got NaN into player position and direction, making the game randomly unplayable.
* Fix for random crash when expelling an entity (sound file format error)
* Fix collision bug when a door is closed on top of the player.
* WASM: Prevent accidental closing of the tab via Ctrl+W.
* Fixed bug where ghost guess auto-selection wasn't always working properly.
* Fixed ghost guess resource to use difficulty-specific ghost sets.
* Fixed log spam when music volume setting was zero.


### Version 0.3.0 - 2025-05-31

* New campaign with 15 new maps!
* Progression system with money, experience and leveling up.
* Walkie Talkie buddy extended aggressively with an additional Hint UI.
* Now missions are graded from A to F.
* New design for the menu.
* The menu now works with the mouse as well as keyboard.
* Multi-floor support and stairs. Maps can now have multiple floors.
* Controls are now fully configurable via config file. (User must edit file manually)
* Flashlight and lighting rework to make it look moodier.
* Additional spritesheets.
* New contributed map `Tarin Library` (thanks!!)
* New website! https://www.unhaunter.com

### Version 0.2.7 - 2025-03-26

**Features:**

*   **Walkie Talkie Buddy:** Introduced an NPC companion who provides hints and commentary
  via walkie-talkie, primarily aimed at assisting players on easier difficulties. #94
    *   Includes UI text display accompanying the audio messages.
    *   Provides contextual messages for events like:
        *   starting a mission
        *   forgetting gear
        *   pre-hunt warnings
    *   Added more message variety and tuned activation conditions.
*   **Hold-to-Activate Truck Buttons :** Implemented a hold duration requirement for critical
  truck UI buttons (Craft Repellent, End Mission) to prevent accidental activation.
  Includes visual progress bar and audio feedback during the hold. #32
*   **Auto-Hiding Mouse Cursor:** The mouse cursor now automatically hides after a short
  period of inactivity during gameplay, enhancing immersion. #74
*   **Basic Spatial Audio:** Implemented initial spatial audio (volume-based positioning)
  for sound effects, providing better directional awareness. #54
*   **Look at left hand:** `Left Control` now allows to focus and see the left hand gear,
  and set the evidence on the left hand. #27

**Changes:**

*   **Music volume:** Increased music volume, which allows for a very loud music at 100%. #92
*   **Snappier Camera:** Adjusted for a faster, snappier feel and improved responsiveness. #93
*   **WASM now allows for resizing:** Improved WASM CSS handling so in-browser gaming uses the full browser window. #63

**Fixes:**

*   **UI:** Corrected visibility issues with the in-game control key legend.
*   **Gear - Recorder:** Removed an experimental false reading mechanic that could be confusing.
*   **Spirit Box:** Limited the effective range of the Spirit Box to require closer proximity to the ghost. #68
*   **Replaced duplicate ghosts:** Domovoi and Wisp were duplicates so they have been renamed to new unique ghosts. #87

**Other:**

*   **Tools:** Added an internal `ghost_list` developer tool for viewing ghost/evidence statistics. #86
*   **Kokoro TTS:** Added notes and setup instructions for Text-to-Speech tooling experimentation.
*   **Documentation:** Updated README and internal developer notes. #37
*   **Wiki:** Added the basis for the wiki on GitHub with pages for evidence, ghosts, etc. #38 #40

### Version 0.2.6 - 2025-03-09

**Features:**

*   **Miasma System:** Introduced a dynamic miasma (fog) system that affects gameplay.
    The miasma's density and movement are simulated, and it interacts with the environment and player.
*   **Electromagnetic Interference (EMI):** Added a new mechanic where the ghost,
    particularly during its pre-hunt warning phase, emits electromagnetic interference.
    This affects electronic gear (Flashlight, EMF Meter, Recorder, Red Torch, Videocam),
    causing glitches, malfunctions, and false readings. This adds a layer of challenge
    and realism to using electronic equipment.
*   **Stamina System Integration:** The miasma now directly impacts the player's stamina.
    Higher miasma density in a location increases the rate at which stamina depletes while sprinting.
    This encourages players to avoid or quickly traverse miasma-filled areas.

**Changes:**

*   **Lighting System Refactoring:**
    * The lighting system has been significantly refactored for improved performance and organization.
    * Light propagation now uses a pre-baked data structure along with wave edges, allowing for dynamic light calculation with closed doors and windows.
    * The van's light is sampled by an offset to avoid darkening.

*   **Ghost Behavior:**
    * Ghost rage mechanics were refined, now considering player presence in the location overall.
    * Added a pre-hunt warning state for the ghost, including visual and auditory cues.

*   **Difficulty Adjustments:**
    * Updated hunt provocation radius values across different difficulty levels to improve game balance.

*   **Gear Adjustments:**
    * Added Electromagnetic interference to electronic gear.


### Version 0.2.5 - 2025-02-08

**Features:**

*   **Screenspace orthogonal movement**: Now players can choose to move the
    character relative to the screen or relative to the map.
*   **Settings**: Added gameplay + audio settings which are saved to disk or
    local storage for WASM.

**Changes:**

*   Bumped Bevy version to 0.15

**Other:**

*   Migrated core features and systems to separate crates for better organization and maintainability.

### Version 0.2.4 - 2024-12-30

**Features:**

* **Manual with 5 chapters!**
  * Created a whole manual with 5 chapters to ease in new people into the game.
  * Each chapter is linked to a difficulty to help people read the critical things on the game.
* **Ghost Rage adjusted:**
    * Ghosts now get angrier with more aggressive actions.
    * Ghost hunting duration is no longer increased by the anger.
    * A new cool-down logic for hunting has been implemented on top of the ghost rage.

**Changes:**

* **Gear:**
  * Added support to use different gears in the truck depending on the chosen difficulty.
  * Adjusted `EMF` meter sensibility to adapt to the difficulty.
  * Improved `Red Torch` and `Video Cam` lighting.
* **UI:**
  * Refactored code to support tutorial mode and normal user manual.
  * Minor UI fixes.
* **Upgraded:**
  * Bevy 0.14
  * Tiled 0.12

**Fixes:**
* Fixed a small bug that was making the light levels to be too low.
* Code has been cleaned and commented, improving overall quality.

### Version 0.2.3 - 2024-06-23

**Features:**

* **New Consumable Items:**
    * Salt: You can drop salt on the ground, and if the ghost walks over it,
      it will leave a trace of salt where it goes. These traces are only visible
      under UV light.
    * Quartz Stone: It absorbs the ghost's hunting energy, effectively shortening
      hunts and protecting the player. The stone gradually cracks and eventually
      breaks after repeated uses.
    * Sage: Burn it and smoke the ghost, and it will calm it down for 30 seconds.
      During hunts, it will confuse the ghost and make it lose track of the player.
* **New Menu: Map Hub:**
    * Provides a clear and dedicated menu for selecting the map and difficulty
      level before starting a new game.
    * Displays all available maps in a list, allowing for easy browsing and selection.
    * Presents a separate screen for choosing from a range of 16 difficulty levels,
      each with a description of its unique challenges.
* **Expanded Difficulty System:**
    * Offers 16 distinct difficulty levels, ranging from "Novice Investigator"
      to "Master Guardian", providing a wide range of challenges for players
      of all skill levels.
    * Each difficulty level affects various aspects of the game, including
      ghost behavior, environment conditions, player attributes, and scoring.
    * Difficulties now customize the available equipment for the player.
* **Improved Ghost Expulsion Feedback:**
    * When a ghost is successfully expelled, it now fades out over 5 seconds
      while emitting smoke particles, creating a more noticeable and satisfying
      visual effect.
    * The ghost's breach also fades out, indicating its permanent
      departure from the location.
    * Two distinct roar sounds now play during the expulsion, adding to the
      dramatic effect.

**Fixes:**

* **Lighting:** Several adjustments have been made to the lighting system to
  create a more visually appealing and atmospheric environment.
* **Quartz Stone:** The Quartz Stone's energy absorption rate is now adjusted
  based on the selected difficulty level, making it more effective at higher difficulty levels.
* **Repellent Flask:** The behavior of the Repellent Flask particles has been
  improved. They now spread more realistically, preventing them from clumping
  together or moving in an unnatural way.
* **Hiding Mechanics:**
    * Players can no longer hide while carrying any items,
      preventing conflicts with other actions.
    * The player's sprite now changes color when hiding,
      providing a visual indicator of their hidden state.
    * Hiding now requires the player to hold down the "E" key
      for a short duration, reducing accidental triggers.
* **Truck Journal:** The Truck Journal UI now filters possible evidence more
  effectively based on the selected ghost type, streamlining the evidence selection process.
* **Windows:** Fixed a bug that prevented map tileset images from loading correctly
  in the Windows build, ensuring compatibility and a consistent experience across platforms.
* **General:**
    * Addressed several warnings and clippy lints to improve
      code quality and maintainability.
    * Fixed minor UI issues and typos in the game.

### Version 0.2.2 - 2024-06-02

**Features:**

* **Item Grab & Drop:**
    * Items are now dropped to the center of the tile and it
      checks for other items to prevent dropping items in occupied tiles.
* **Deployable Gear:**
    * Players can now press `G` to drop the gear to the ground.
    * `F` will pick up the gear if there's space for it.
    * Gear continues to work while on the ground
* **New Sound effects:**
    * The ghost will snore, roar and more on different scenarios.
    * The player will hear a loud heardbeat when the health is low.
    * The backgroud music will fade out and be replaced by unsettling sounds
      when the sanity is low.
* **Basic Spatial sound:**
    * Now the sounds will have a volume (not panning) depending on the distance.

**Fixes:**
  * Increased overall brightness of the game.
  * Ensure the torch iluminates the player itself.
  * Fix player sprite to follow flashlight ilumination.
  * Lower difficulty by making ghost a bit friendlier.
  * Now torches have occlusion too.
  * Prevent the ghost from re-hunting when the player re-enters location.


### Version 0.2.1 - 2024-05-26

**Features:**

* **Ghost Events:**
    * The ghost can now slam a door closed.
    * The ghost can also flicker lightly the lights of the room.
* **Hiding Mechanics:**
    * Player can now press `E` near tables, beds, and other objects to hide itself.
    * The ghost will be confused when hunting for the player.
    * The player will need to press `E` again to be able to move.
* **Grab & Drop Items:**
    * Players can now pick up light items with `F`
    * While carrying an item they're slower, they can drop the item with `G`
* **Object-Ghost Interaction System:**
    * Introduced a new system that allows players to influence the ghost's
      behavior by manipulating objects in the environment.
    * Objects are assigned hidden properties (`Attractive`, `Repulsive`) that
      affect how the ghost interacts with them.
    * The ghost's roaming behavior is now influenced by the proximity
      and charge levels of objects.
    * Players can provoke hunts by placing `Repulsive` objects near
      the ghost's breach and by removing `Attractive` objects from the location.
    * Added visual glow effects to objects based on their properties
      and the active light source, revealing their influence on the ghost.

**Maps:**

* **Updated University map:**
    * As we now depend on having objects, it was time to get an update to the
      big university map to add all sorts of objects. It looks pretty cool!
* **New Tutorial: Glass House:**
    * A simple map where all walls are transparent so that it is easy to see,
      and players can learn the ghost evidences at their own pace.

**Fixes:**

* **Lighting:** Improved lighting system and see in the dark.
* **Reduced difficulty:** With the new object interaction system it was becoming
  much more difficult as there's more need to explore, so sanity loss and ghost rage
  was reduced. Temperatures now update in a more aggressive way to simplify tracking
  the ghost as it now roams much more.

**Tools:**

* **Ghost Radio Tool:**  Added a console-based demo tool for testing and
  experimenting with simplified ghost communication.


### Version 0.2.0 - 2024-04-02

Features:
  - New tutorial for basic movement, doors, light switches and torch.
  - Improved Pixel-perfect shader.
  - Improved see in the dark view with additional exposure range and bluish
    tones to convey the sensation of a dark place.
  - Added "ambient light" mixing mode for the shader for improved visuals.
  - Sanity now slowly increases when outside or when the ghost is very far.
  - Added NPC dialogs for use in tutorials.
  - Auto-trigger NPC dialogs by proximity.
  - New sprites and utilities to be used mainly by tutorials.

Fixes:
  - Sort maps by filename in the main menu.
  - Prevent van from auto-opening if far away.
  - Do not animate player walking while the game is paused.

### Version 0.1.9 - 2024-03-24

Features:
  - Add Equipment description in Loadout UI on hover, also tracks the evidence
    status.
  - Rearrange key help legend to fit the key info closer to the items that the
    keys act upon.

Fixes:
  - Found and fixed the issue with bad WASM performance and stutter. It was
    related to Bevy's trace feature. It has been disabled by default. Now works
    on Firefox too.
  - Prevent the player inventory in Loadout UI to wrap (WASM)


### Version 0.1.8 - 2024-03-23

Features:
- Make WASM window smaller

Other:
- Fix UI issues after `cargo update` breaks the interface

### Version 0.1.7 - 2024-03-21

Features:
- WASM version autodeployed from `main` branch


### Version 0.1.6 - 2024-03-20

Features
- New Tab in Truck for Loadout that is now the default
- Able to select the gear the player wants to carry
- Limit the maximum amount of gear of the player to 2 hands + 2 extra inventory items
- Skip empty inventory slots when cycling
- Truck UI is shown when starting the game so player can begin by choosing gear
- Two extra disabled tabs added in Truck UI for future use

Other:
- Refactor Truck UI code

### Version 0.1.5 - 2024-03-15

Features:
- Game UI rework to show clearly gear on left and right hand
- Next item to be grabbed by [Q] key is shown on screen
- [TAB] now toggles the left hand, [T] swaps left and right
- Vignetting added for health display, better visibilty
- Vignetting now also displays sanity to encourage players to go to the van
- Prevent flashlight from turning off when overheating
- Breach and ghost now slowly pulsate to make them more obvious
- Ghost can now warp long distances

Fixes:
- Typo on help for Ghost Orbs evidence
- Prevent Ghost range/hunt from being stuck high while outside
- Sanity now is no longer lost while outside of the location
- Added feedback on the "Craft Repellent" button

### Version 0.1.4 - 2024-03-14

Changes:
- WASM support (with problems)
- Deployed WASM to https://deavid.github.io/unhaunter/

### Version 0.1.3 - 2024-03-12

Features:
- Added main menu song
- Added two new maps (small house and school)
- Added map selector in main menu screen
- Added sanity system + added it to summary scoring
- Added ghost hunting mechanic + summary scoring
- Added evidence quick selector
- Record evidence outside of the truck
- Instructions visible on screen all-time, gear tied to evidence.

Performance fixes:
- Don’t draw invisible tiles
- Avoid updating UI / Texts if they don’t change
- Decoupled walking speed from FPS
- Optimized Temperature field update to avoid considering cells with no heat sources
- Optimized realtime light rendering to constrain checks to tiles nearby to the player

Fixes:
- Added sprite to prevent occlusion leak from top-left corner of maps
- Player sprite was being “colored” from old & new code which caused flickering. Removed old code.
- Camera / player sprite stutter fix

Other:
- Migrated from Bevy 0.12 to 0.13
- Added instructions to profile Unhaunter
- Lots of refactor 🙂


### Version 0.1.0 - 2024-03-06

Initial MVP Release.

