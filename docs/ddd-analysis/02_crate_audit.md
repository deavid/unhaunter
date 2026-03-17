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

## Crates

### tools/\*

These are technically not part of the Bevy app and therefore are out of scope of this analysis

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

Category: Leaf, UI, Tier 9.

What this is: UI stuff related on how to make the UI show the campaign mission selection.

Problem, it is too thin, it lacks a -core counter part. It's not clear that this is a domain on its own.

I am suspicious that this is a bigger domain, spread across more crates. But all dependency analysis I could do show
that this is just leaf code, very well isolated.
