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

