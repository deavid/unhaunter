# Crate Audit — Stale Entries (Old Audit)

> Part of the Unhaunter DDD audit. Index: [02_crate_audit.md](02_crate_audit.md)

---

## STALE ENTRIES (Old Audit — 2026-03-16)

> The entries below were written before the current tier model and rules were established. They used a different 4-tier
> numbering (0, 1, 2, 3) that does not match the current model (T0–T4 with subgroups). Tier assignments and compliance
> verdicts must be re-verified. Prose observations may still be useful context.
>
> When an entry is re-audited under current rules, remove it from this section and add a proper entry in the correct
> tier section above.

---

## Old Tier Model Reference (for reading stale entries only)

```text
Old Tier 0: Foundation & Primitives — pure math, spatial, core types.
Old Tier 1: Infrastructure & Adapters — networking, file I/O, asset parsing.
Old Tier 2: Domains & Features — ghost AI, temperature, walkie, items.
Old Tier 3: Presentation & App Glue — UI, rendering, audio, game modes.
```

This does **not** match the current model. Do not use it for new work.

---

## tools/\* (out of scope)

These are technically not part of the Bevy app and therefore are out of scope of this analysis.

---

## [STALE] unaudiobg-core + unaudiobg-plugin

> ⚠️ Written 2026-03-16 under old rules. Old tier: "Presentation (Tier 3)" ≈ current T4. Tier assignment direction
> appears correct but must be formally re-verified.

Status (old): Green — Textbook Vertical Slice.

What this is: The domain responsible for all continuous background audio tracks (Menu music, House Ambient, Street
Ambient, Heartbeat, Insanity). It owns the volume calculations, decibel smoothing math, and muting effects.

Architecture Notes:

- **Clean Core:** `unaudiobg-core` holds pure math (`smooth.rs`), state definitions (`mute.rs`), and the
  `AmbientSoundMuteEvent`. It has zero game dependencies, relying only on Bevy.
- **Data-Driven Lifecycle:** Instead of violently spawning and despawning background tracks during map loads (which
  previously polluted the Orchestrator and Render plugins), this domain spawns tracks once at `Startup` and smoothly
  transitions their volume to `0.0` when not in use.
- **Proper presentation sizing:** This plugin correctly acts as a consumer. It reads `VisibilityData`, `RoomTopology`,
  and Player Health/Sanity to calculate volumes, without forcing any lower-level systems to know about audio sinks.
- **Major Decoupling Win:** By creating this, we successfully removed audio components/events from `unrender-std`,
  `unfoundation-core`, `unevents-core`, and `unmainmenu-plugin`.

Conclusion: A well-scoped presentation domain. Keep as is.

NOTE: This could be even better and act like an external plugin, where everything is ECS driven. Just imagine: Entities
are the tracks we want to have, components would carry the intended state, the current state and expose all the API, and
the other crates would interact with it.

---

## [STALE] unbehavior

> ⚠️ Written 2026-03-16 under old rules. Old tier: "Unknown??, Tier 0" — **this was wrong**. Under the current model
> `unbehavior` is T3 · 3a (Map Pipeline ACL). It depends on `unspatial-core` and the external `tiled` crate, which is
> definitional T3 behavior.

Status (old): Lime — with caveats.

What is this: The central final stop for all tile behavior for Unhaunter — the translation layer between raw TMX and
actual roles and properties.

This is a very critical, very core crate of what Unhaunter is. Could be foundation but technically it isn't.

The main problem is that it bundles a lot of features and stuff together — it might be mixing domains.

It is a single stop for all behavior and this means that a lot of stuff will go through here. However that's a very
minor problem compared to the amount of good work it does.

Weird: it's not a -core nor a -plugin (technically a -core without a -plugin).

Conclusion: Leave as is for the time being.

---

## [STALE] unboard-core + unboard-plugin

> ⚠️ Written 2026-03-16 under old rules. Old tier: "Foundation, Tier 0" — **this was wrong**. `unboard-core` depends on
> `unspatial-core`, which by definition puts it in T1 or higher. Correct placement under current rules is **T1 · 1a (The
> Stage)**.

Status (old): Lime — with caveats.

What is this: A collection of stuff that defines the space where the map loads in, the board.

Overall quite clean cut. There are mainly 2 small issues here:

- The core side has components and stuff that might not really be part of the domain... but they do compile well
  together so it seems... fine.
- The plugin side has no systems and this points towards systems that might belong here but are elsewhere.

---

## [STALE] uncampaign-plugin

> ⚠️ Written 2026-03-16 under old rules. Old tier: "Leaf, UI, Tier 3" ≈ current T4 · 4c. Tier assignment direction
> appears correct but must be formally re-verified.

Status (old): Yellow — suspicious.

What this is: UI stuff related to the campaign mission selection screen.

Problem: it is too thin, it lacks a -core counterpart. It's not clear that this is a domain on its own.

I am suspicious that this is a bigger domain, spread across more crates. But all dependency analysis I could do shows
that this is just leaf code, very well isolated.

---

## [STALE] unclassic-mode-\* plugins

> ⚠️ Written 2026-03-16 under old rules. Old tier: "Leaf, UI + render + simulation, Tier 3" ≈ current T2
> (gameplay/orchestrator) and T4 (render/ui). The split across T2 and T4 needs formal verification.

Status (old): Orange — Work needed.

What this is: Was a single crate that contained everything for running the mission — split into 4 sub-crates. It is a
dumping ground for stuff related to the game core loop.

At some point they need a revisit. A game mode plugin should look more like configuration than a dumping ground for
everything. The functionality needs to better split into what's engine and what is a game mode configuration.

Good thing: they are 100% isolated.
