# Crate Audit for DDD + Hexagonal Architecture

- Author: @deavid

This document audits every crate in the workspace against the tier model defined in `copilot-instructions.md` and
encoded as comments in `Cargo.toml [workspace.members]`.

---

## How to Read This Document

Entries are organized **by tier, bottom-up** (T0 first). This is the only order that makes sense: you cannot verify that
a T2 crate is clean until you know what T0 and T1 contain. Alphabetical order is wrong for a dependency audit.

Each crate entry answers these questions mechanically, then optionally adds prose notes:

```markdown
### crate-name

- Audit date: YYYY-MM-DD
- Declared tier: T?? · group name
- Pair: crate-name-core / crate-name-plugin (same tier? YES / NO)
- Internal deps: [list every un* crate from this crate's Cargo.toml]
- Highest tier imported: T?
- Tier compliance: PASS / FAIL
- If FAIL: which import, which tier it is, what violation it causes
- Notes: (optional)
```

**Tier compliance is binary.** A crate either only imports from its own tier or below, or it doesn't. "A bit suspicious"
belongs in Notes, not in the compliance verdict.

---

## Status Legend

Each entry carries one of these statuses, distinct from tier compliance:

- **CURRENT** — audited under the current rules, findings are up to date.
- **STALE** — audited under old rules or before recent code changes. The prose may still be useful but the tier
  assignment and compliance verdict must be re-verified before trusting them.
- **PENDING** — not yet audited.

---

## Appendix: Background and Motivation

### Why We Do This

Part of the feeling of "can't find things on this repo — who did what?" seems to be because we are still not following
ECS properly, and adding `bevy_replicon` (which assumes full ECS architecture) breaks all previous assumptions. Anything
that slightly relates to multiplayer becomes impacted if it wasn't done in a proper ECS way.

Components need to be split properly. Plugins need to be vertical domain slices, not horizontal layers.

So in this audit, we review the crates. As we go, we fix. However there is stuff that we can't fix — either because it's
just suspicious, or because it's not the right moment to fix. In these cases we note down here what's the story with
each crate.

### The Strangler Fig Approach

The approach used here is pragmatic: a perfect DDD rewrite would stall the project for too long. The chosen path is the
**Strangler Fig** refactoring pattern. By splitting `unclassic-mode-plugin` into `gameplay`, `orchestrator`, `render`,
and `ui`, the mess is put into neatly labeled boxes. Later, when `unclassic_mode_render` gets merged into the main
rendering pipeline, it will be a simple drag-and-drop instead of a surgical extraction.

This is the correct call for right now.

### Beyond the Hexagon: Portable Libraries

Game development over time converges on generic utilities: a frame rate limiter, a sprite animation timer, a
screen-shake system, a transition fade manager. These solve technical problems that have nothing to do with the game's
domain. They do not belong in any tier of the hexagonal model — they are the foundation the whole stack rests on, the
same as external crates from crates.io.

**The distinction from T0:**

T0 (Domain Kernel) exists to give other crates a shared vocabulary of primitives: coordinate types, state enums, marker
components. T0 types are consumed by multiple tiers. They are domain-aware — `BoardPosition`, `AppState`, `Direction`
all exist because of this specific game's design decisions.

A portable library is different: it is domain-agnostic. It knows nothing about the game. Another developer could use it
in their own project without changing a line. It has no opinion about game states, entity types, or coordinate systems.
Where T0 is the game's vocabulary, a portable library is a general-purpose tool.

**Structural implications:**

A portable library should not use the `-core`/`-plugin` split. That split exists because of the DDD headless-safe domain
requirement. A game-agnostic library has no such constraint — it is one crate, it exposes a `Plugin`, done. This is the
standard pattern in the Bevy ecosystem.

A portable library should not carry the `un` prefix. The `un*` prefix is a workspace namespace convention. On extraction
to crates.io, it would become `bevy_fps_limiter`, `bevy_frame_pace`, etc. Keeping `un` while in the workspace is
acceptable for consistency, but it should be the first thing to drop on publication.

**Proposed workspace organisation:**

A new section above T0 in `[workspace.members]` and `[workspace.dependencies]`:

```toml
# ── Portable Libraries ──────────────────────────────────────────────────────
# Game-agnostic Bevy plugins. Zero Unhaunter-specific knowledge.
# Strong candidates for extraction to crates.io. No -core/-plugin split.
# These are the foundation the tier stack rests on — outside the hexagon.
```

This signals intent clearly: anything placed here must have zero game-specific knowledge. That requirement is a useful
forcing function — if a candidate needs any `un*` dep, it stays in the tier system. The category enforces its own
membership criteria automatically.

**Current candidate in this workspace:**

`unfps-core` + `unfps-plugin` — see the T2 section for the full audit. The frame rate limiter is the clearest example:
`std::thread::sleep()` to throttle frame delivery, three resource types, timing metric output. No game knowledge
whatsoever. Could be published to crates.io as-is with only a rename.

**Long-term vision:**

This is the natural output of maturing ECS fluency: over time, generic Bevy patterns crystallise out of game-specific
code and the portable library section grows. These crystallised patterns are then publishable, making the project a
contributor to the ecosystem rather than only a consumer.

---

## The Tier Model (Summary)

Full definition lives in `copilot-instructions.md`. Reproduced here for quick reference:

```text
T0  Domain Kernel      — pure types, math, markers. No internal game deps.
T1  Domain Core        — the game's world and rules. Headless-safe.
      1a  The Stage    — board, physics fields (thermal, sound, fog, light, nav)
      1b  The Actors   — ghost, player, NPC, locomotion
      1c  The Mechanics— gear, interaction, inventory, vitals, difficulty, truck, walkie
T2  Application Layer  — mission lifecycle, game-mode orchestration, input, networking
T3  Infrastructure     — external I/O and persistence (Tiled ACL, settings, profiles, hub)
      3a  Map Pipeline — Tiled ACL (untiled, untmxmap, unmapload, unbehavior)
      3b  Persistence  — settings, profile
      3c  Net Clients  — unhub
T4  Presentation       — rendering, audio, UI. Client-side only.
      4a  Rendering    — unrender, unpicking
      4b  Audio        — unaudiobg, unaudiospatial
      4c  UI           — unui, menus, screens, HUD
```

Dependency direction is **downward only**. A crate may depend on its own tier or lower. Never higher.

---

## Audit Files by Tier

| File                       | Section                        | Status              |
| -------------------------- | ------------------------------ | ------------------- |
| [02_t0.md](02_t0.md)       | T0 · Domain Kernel             | 6 entries — CURRENT |
| [02_t1a.md](02_t1a.md)     | T1·1a · The Stage              | 7 entries — CURRENT |
| [02_t1b.md](02_t1b.md)     | T1·1b · The Actors             | 4 entries — CURRENT |
| [02_t1c.md](02_t1c.md)     | T1·1c · The Mechanics          | 8 entries — CURRENT |
| [02_t2.md](02_t2.md)       | T2 · Application Layer         | 6 entries — CURRENT |
| [02_t3a.md](02_t3a.md)     | T3·3a · Map Pipeline           | PENDING             |
| [02_t3b.md](02_t3b.md)     | T3·3b · Persistence            | PENDING             |
| [02_t3c.md](02_t3c.md)     | T3·3c · Net Clients            | PENDING             |
| [02_t4a.md](02_t4a.md)     | T4·4a · Rendering              | PENDING             |
| [02_t4b.md](02_t4b.md)     | T4·4b · Audio                  | PENDING             |
| [02_t4c.md](02_t4c.md)     | T4·4c · UI                     | PENDING             |
| [02_stale.md](02_stale.md) | Stale Entries (pre-2026-03-20) | STALE               |

Pure primitives: types, math, markers. Must have **zero internal game dependencies**. Any `un*` import in a T0 crate is
a violation by definition.

---
