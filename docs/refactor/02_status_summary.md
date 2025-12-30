# Refactoring Status Summary

**Date:** December 30, 2025

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

### Phase 4: Splits
- `unmetrics` split into `unmetrics-core` and `unmetrics-plugin`.
- `unsettings` split into `unsettings-core` and `unsettings-plugin`.
- `unghost` split into `unghost-core` and `unghost-plugin`.
- `unplayer` split into `unplayer-core` and `unplayer-plugin`.
- `unpicking` split into `unpicking-core` and `unpicking-plugin`.
- `unmaphub` split into `unmaphub-core` and `unmaphub-plugin`.
- `uncoremenu` split into `unmenu-core` and `unmenu-plugin`.

## Pending Work

### Phase 2: Remaining Core Renames
The following `uncore-*` crates still need to be renamed/migrated:
- `uncore-assets` -> `unassets-core`
- `uncore-board` -> `unboard-core`
- `uncore-events` -> `unevents-core`
- `uncore-foundation` -> `unfoundation-core`
- `uncore-types` -> `untypes-core`

### Phase 3: Dissolving God Crates
- `uncore-components`: Needs to be dissolved into feature-specific core crates.
- `uncore-resources`: Needs to be dissolved into feature-specific core crates.

### Phase 4: Remaining Splits
- `ungear`: Needs to be split into `ungear-core` and `ungear-plugin`.
- `unrender`: Needs to be split into `unrender-std` and `unrender-plugin`.

## Next Immediate Steps
1.  Rename the remaining `uncore-*` crates to `un*-core`.
2.  Begin dissolving `uncore-components` and `uncore-resources`.
