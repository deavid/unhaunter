# Crate Audit — PL · Portable Libraries

> Part of the Unhaunter DDD audit. Index: [02_crate_audit.md](02_crate_audit.md)

---

## Portable Libraries

Game-agnostic Bevy plugins with zero Unhaunter-specific knowledge. No `un*` deps. These crates sit **outside** the
hexagonal architecture — they are the foundation the tier stack rests on, equivalent in status to external crates from
crates.io. They have no tier number, no binding-rule obligations, and are strong candidates for extraction and
publication.

**What belongs here:** A crate qualifies as a Portable Library if and only if it has zero `un*` imports. If it needs any
internal game crate, it belongs in the tier system. This constraint is self-enforcing — the membership criterion is
trivially verifiable from `Cargo.toml`.

**The `-core`/`-plugin` split does not apply.** That split exists for the DDD headless-safe domain requirement. A
game-agnostic library has no such constraint — it exposes a single `Plugin`, done.

Crates: `unfps-core`, `unfps-plugin`

---

### unfps-core + unfps-plugin

- **Status:** CURRENT
- Audit date: 2026-03-21 (moved from T2 · Application Layer on 2026-03-21; detailed analysis in [02_t2.md](02_t2.md))
- Declared section: Portable Libraries
- Pair: `unfps-core` / `unfps-plugin` (split unjustified — no other crate needs `unfps-core` independently)
- Internal deps (`unfps-core`): none
- Internal deps (`unfps-plugin`): `unfps-core` only
- Highest tier imported: none (`un*` imports: zero)
- Tier compliance: N/A — Portable Libraries are outside the tier model

**What it does:**

`unfps-core` defines one resource (`TargetFps`) and one system function (`fps_limiter`). `unfps-plugin` registers that
resource and adds that system. The plugin limits frame delivery by calling `std::thread::sleep` to hold the frame until
the target interval has elapsed. Three timing resources communicate the limiter state to the rest of the app.

**Why it is here:**

The crate has no `un*` imports. It knows nothing about the game's states, coordinate systems, entities, or domain types.
It could be published to crates.io today without any source changes — only a rename (removing the `un` prefix) would be
needed.

In T2, it was a tier-compliance pass but a semantic mismatch: T2 implies the crate coordinates game flow or knows about
sessions. `unfps-plugin` does neither. Moving it here correctly signals that it is a generic utility, not an application
layer participant.

**Remaining actions:**

1. Collapsing `unfps-core` and `unfps-plugin` into a single crate — deferred (code change required).
2. Optionally rename to remove the `un` prefix — deferred.
3. `Cargo.toml` placement is now correct.

---
