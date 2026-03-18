# Crate Audit for DDD + Hexagonal Architecture

- Date: 2026-03-16 ... ???
- Author: @deavid

## Basis

part of the feeling of "can't find things on this repo who did what?" seems because we are still not following ECS
properly, and adding bevy_replicon, which it is architected in a way that does assume full ECS architecture, it breaks
all previous assumptions and anything that slightly relates to multiplayer becomes impacted if it wasn't done in a
proper ECS way.

Components need to be splitted properly, Plugins need to be vertical domain slices, not horizontal layers.

So in this audit, we are reviewing the crates. As we go, we fix. However there is stuff that we can't fix, either
because it's just suspicious, or because it's not the right moment to fix. In these cases we note down here what's the
story with each crate.

I think your approach is incredibly pragmatic, and the Audit Log is a masterclass in managing technical debt without
paralyzing your development.

You recognized a massive problem, realized a perfect DDD rewrite would stall your project for a month, and chose the
**"Strangler Fig" refactoring pattern**. By splitting `unclassic-mode-plugin` into `gameplay`, `orchestrator`, `render`,
and `ui`, you are putting the mess into neatly labeled boxes. Later, when you want to merge `unclassic_mode_render` into
your main rendering pipeline, it will be a simple drag-and-drop instead of a surgical extraction.

It is the absolute right call for right now.

To answer your specific question, here is exactly how **The Tier System** is defined in a Bevy/ECS architecture. You can
literally use this as a rubric for your Audit Log:

## The ECS Tier System

**Tier 0:** Foundation & Primitives (The Laws of Physics)

- **What it is:** Pure math, spatial reasoning, and the absolute core data types.
- **The Rule:** Cannot depend on _any_ other game crates. Only depends on Bevy and standard libraries.
- **Examples:** `unspatial-core`, `unboard-core`, `untypes-core`.

**Tier 1:** Infrastructure & Adapters (The Translators)

- **What it is:** Code that talks to the "outside world" or translates external data into your game's format.
  Networking, File I/O, Asset parsing.
- **The Rule:** Depends on Tier 0 to know what data structs to fill out, but knows nothing about gameplay rules.
- **Examples:** `untmxmap-plugin` (reads XML, outputs Tier 0 structs), `unbehavior` (your ACL that translates Tiled
  strings into ECS properties).

**Tier 2**: Domains & Features (The Game Rules)

- **What it is:** The actual mechanics. Ghost AI, Temperature simulation, Walkie-Talkie logic, Item usage.
- **The Rule:** Depends on Tier 0 and Tier 1. _Domains should not depend on other Domains_ (e.g., Thermal shouldn't
  depend on Walkie-Talkies). If they must interact, they do it via Events or Tier 0 shared components.
- **Examples:** `unghost`, `unthermal`, `ungear`.

**Tier 3**: Presentation & App Glue (The User Experience)

- **What it is:** UI, Rendering pipelines, Audio playback, and "Game Modes" (Orchestrators that stitch Domains together
  to define winning/losing).
- **The Rule:** Can depend on _everything_ below it. This is where the user actually sees the game.
- **Examples:** `uncampaign-plugin`, `unui-core`, and your new `unclassic_mode_*` crates.

## Status Colors

- Green: Perfect, all done.
- Lime: Good, but with caveats.
- Yellow: Okay, but suspicious.
- Orange: Poor, work needed.
- Red:

## Crates

### tools/\*

These are technically not part of the Bevy app and therefore are out of scope of this analysis

### unaudiobg-core + unaudiobg-plugin

Status: Green - Textbook Vertical Slice.

Category: Presentation (Tier 3).

What this is: The domain responsible for all continuous background audio tracks (Menu music, House Ambient, Street
Ambient, Heartbeat, Insanity). It owns the volume calculations, decibel smoothing math, and muting effects.

Architecture Notes:

- **Clean Core:** `unaudiobg-core` holds pure math (`smooth.rs`), state definitions (`mute.rs`), and the
  `AmbientSoundMuteEvent`. It has zero game dependencies, relying only on Bevy.
- **Data-Driven Lifecycle:** Instead of violently spawning and despawning background tracks during map loads (which
  previously polluted the Orchestrator and Render plugins), this domain spawns tracks once at `Startup` and smoothly
  transitions their volume to `0.0` when not in use.
- **Proper Tier 3 Sizing:** This plugin correctly acts as a consumer. It reads `VisibilityData`, `RoomTopology`, and
  Player Health/Sanity to calculate volumes, without forcing any lower-level systems to know about audio sinks.
- **Major Decoupling Win:** By creating this, we successfully removed audio components/events from `unrender-std`,
  `unfoundation-core`, `unevents-core`, and `unmainmenu-plugin`.

Conclusion: A perfectly scoped presentation domain. Keep as is.

NOTE: This could be even better and act like an external plugin, where everything is ECS driven. Just imagine: Entities
are the tracks we want to have, components would carry the intended state, the current state and expose all the API, and
the other crates would interact with it.

### unbehavior

Status: Lime - with caveats.

Category: Unknown??, Tier 0.

What is this: The central final stop for all tile behavior for Unhaunter - it is the translation layer between raw TMX
and actual roles and properties.

This is a very critical, very core crate of what Unhaunter is. Could be foundation but technically it isn't.

The main problem on this crate is that it bundles a lot of features and stuff together - it might be mixing domains.

It is a single stop for all behavior and this means that a lot of stuff will go through here.

However that's a very minor problem compared on the amount of good work it does.

Weird: it's not a -core nor a -plugin (technically a -core without -plugin)

Conclusion: We leave as is for the time being.

### unboard-core + unboard-plugin

Status: Lime - with caveats.

Category: Foundation, Tier 0.

What is this: A collection of stuff that defines the space where the map loads in, the board.

Overall quite clean cut. There are mainly 2 small issues here:

- The core side has components and stuff that might not be really part of the domain... but they do compile well
  together so it seems... fine.
- The plugin side has no systems and this points towards that there are systems that might belong here but are
  elsewhere.

### uncampaign-plugin

Status: Yellow - suspicious.

Category: Leaf, UI, Tier 3.

What this is: UI stuff related on how to make the UI show the campaign mission selection.

Problem, it is too thin, it lacks a -core counter part. It's not clear that this is a domain on its own.

I am suspicious that this is a bigger domain, spread across more crates. But all dependency analysis I could do show
that this is just leaf code, very well isolated.

### unclassic-mode-\*-plugin

Status: Orange - Work needed.

Category: Leaf, UI + render + simulation, Tier 3 (Game Mode / Presentation)

What this is: Was a single crate that contained everything for running the mission - we split it in 4 sub-crates. It is
a dumping ground for stuff related on the game core loop.

At some point they need a revisit. That can't be a game mode plugin. These need to split functionality better into
what's engine and what is a game mode configuration. A game mode crate should look like more as configuration than just
dumping there everything.

Good thing: they are 100% isolated though.

### undifficulty-core + undifficulty-plugin

Status: Green.

Category: Tier 1 (Game Primitives).

What this is: The core `Difficulty` enum and mathematical multipliers for game rules.

The plugin is very thin, too thin - as it only registers a single resource.

The giant DifficultyStruct probably should be removed eventually, and use just traits to get the static values.

These traits could live on the respective places of the code, ghost, gear, etc.

But for now, this crate is self contained and in good shape.
