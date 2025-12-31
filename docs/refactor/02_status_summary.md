# Refactoring Status Summary

**Date:** December 31, 2025

## Completed Work

### Phase 1: Leaf Plugin Renames
Most leaf plugins have been successfully renamed to `*-plugin`.
- `uncampaign` -> `uncampaign-plugin`
- `unfog` -> `unfog-plugin`
- `ungame` -> `ungame-plugin`
- `ungearitems` -> `ungearitems-plugin`
- `unlight` -> `unlight-plugin`
- `unmanual` -> `unmanual-plugin`
- `unmapload` -> `unmapload-plugin`
- `unmenu` -> `unmainmenu-plugin`
- `unmenusettings` -> `unmenusettings-plugin`
- `unnpc` -> `unnpc-plugin`
- `unprofile` -> `unprofile-plugin`
- `unroot` -> `unroot-plugin`
- `untmxmap` -> `untmxmap-plugin`
- `untruck` -> `untruck-plugin`
- `unwalkie` -> `unwalkie-plugin`

### Phase 2: Core Data Renames
Several core data crates have been renamed/created:
- `undifficulty-core`
- `uninteraction-core`
- `unnavigation-core`
- `unnoise-core`
- `untags-core`
- `untiled-core`
- `unui-core`
- `unwalkie-core`

### Phase 3: Dissolving God Crates
- `uncore-components`: **DISSOLVED**. Contents distributed to `ungear-core`, `unrender-std`, `uninteraction-core`.
- `uncore-resources`: **DISSOLVED**. Contents distributed to `unsummary-core`, `untypes-core`, `unmenu-core`, `unui-core`, `unsettings-core`.

### Phase 4: Splits
- `unmetrics` split into `unmetrics-core` and `unmetrics-plugin`.
- `unsettings` split into `unsettings-core` and `unsettings-plugin`.
- `unghost` split into `unghost-core` and `unghost-plugin`.
- `unplayer` split into `unplayer-core` and `unplayer-plugin`.
- `unpicking` split into `unpicking-core` and `unpicking-plugin`.
- `unmaphub` split into `unmaphub-core` and `unmaphub-plugin`.
- `uncoremenu` split into `unmenu-core` and `unmenu-plugin`.
- `ungear` split into `ungear-core` and `ungear-plugin`.
- `unrender` split into `unrender-std` and `unrender-plugin`.
- `unsummary` split into `unsummary-core` and `unsummary-plugin`.

## Pending Work

### Phase 2: Remaining Core Renames
The following `uncore-*` crates still need to be renamed/migrated:
- `uncore-assets` -> `unassets-core`
- `uncore-board` -> `unboard-core`
- `uncore-events` -> `unevents-core`
- `uncore-foundation` -> `unfoundation-core`
- `uncore-types` -> `untypes-core`

## Next Immediate Steps
1.  Rename the remaining `uncore-*` crates to `un*-core`.
