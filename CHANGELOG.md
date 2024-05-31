### Version 0.2.2 - 2024-06-??

**Features:**

* **Item Grab & Drop:** 
    * Items are now dropped to the center of the tile and it
      checks for other items to prevent dropping items in occupied tiles.
* **Deployable Gear:**
    * Players can now press `G` to drop the gear to the ground.
    * `F` will pick up the gear if there's space for it.
    * Gear continues to work while on the ground (except torches)

**Fixes:**

  * Increase overall brightness of the game
  * Ensure the torch iluminates the player itself
  * Fix player sprite to follow flashlight ilumination
  * Lower difficulty by making ghost a bit friendlier


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

