# DRAFT: ECS DOMAIN ISOLATION & THE SIGNAL GRAPH BRAINDUMP

Alright, let's open the valve and just let this pour out. We are talking about the soul of ECS, the fundamental
difference between building enterprise software where a User sends an Invoice to a Billing System, and a video game
where a Ghost stalks a Player, freezing the air, shorting out the flashlight, and driving the poor bastard insane.

In traditional Domain-Driven Design (DDD), you have neat little boxes. You have APIs. If Domain A wants to effect Domain
B, it calls `DomainB.DoSomething()`. Or, if we are being "decoupled," we introduce `DomainC_Mediator` so A and B don't
know about each other. It works in enterprise because the interactions are transactional and sparse.

But game development? Game development is a constant, chaotic collision of domains. Everything is touching everything
all the time. The Ghost (Domain A) affects the Player (Domain B), changes the Temperature (Domain C), alters the Audio
(Domain D), triggers the EMF Reader (Domain E), and updates the UI (Domain F).

If you apply strict OOP-style DDD to this, you get what I call **Death by a Thousand Crates**. You end up writing
`unghost-audio-bridge`, `unghost-vitals-bridge`, `unemf-ghost-bridge`. You invent abstract concepts that have no meaning
other than "I needed a place to put the code where both plugins could see it." It’s an architectural nightmare. It
compiles, but it has no soul. It's brittle.

What we need to embrace—what fundamentally changes the game—is that we are operating in an Entity Component System. We
need to stop building telephone wires between our domains, and start viewing the **ECS World as a Blackboard**.

## 1. The Blackboard and Environmental Mediation

Think about reality. If I walk into a snowy forest, the forest doesn't "call" my `ReduceBodyHeat()` method. The forest
simply _is cold_. It projects a temperature field. My body, executing its own biological systems, reads the temperature
of the environment and reacts.

This is the key to preventing the 50-crate explosion. You don't need domains to talk to each other. You need them to
talk to the **Space between them**. We already have this in Unhaunter! T1a: The Stage (`unboard`, `unthermal`, `unfog`).
This isn't just physical geometry; it's the ultimate decoupling mediator.

Instead of the `unvitals` system querying `GhostMarker` and checking `Distance(player, ghost)` to drain sanity, the
Ghost should never even know what sanity is. The Ghost is an actor. It moves. And as it moves, it leaves "Facts" on the
board. It drops a `PsychologicalPressure` component on the Tiles it occupies. Or it emits a `Miasma` field.

The Vitals system, completely oblivious to the existence of ghosts, demons, or generic spooks, simply asks the Board:
_"Hey, the Player is standing on Tile X. Does Tile X have PsychologicalPressure?"_ If yes, sanity goes down.

No new crates. No tight coupling. Complete isolation. The Ghost domain is responsible ONLY for being a ghost and making
its surroundings spooky. The Vitals domain is responsible ONLY for keeping humans alive (or failing to). The Board (T1a)
holds the shared, primitive language.

## 2. Capability vs. Identity (Nouns vs. Verbs)

We run into this "smell" all the time, particularly with gear. Let's say a Crucifix repels a ghost. The bad way to code
this is to have the Ghost's AI system query `Query<&GearType, With<Equipped>>` and say
`if item.type == GearType::Crucifix { run_away(); }`.

Why is this bad? It's the **Vocabulary Test**. Domain A (Ghost) is using Domain B's (Gear) nouns (`Crucifix`). You have
essentially welded the ghost brains to the inventory catalog.

The fix is pure ECS capability tags. Nouns are for UI. We care about _Capabilities_. Down in T0 or T1a, we define an
opaque struct: `RepelsGhost(pub f32)`. When the `ungearitems-plugin` spawns a Crucifix, it slaps `RepelsGhost(10.0)` on
it. Because that's what a crucifix does. The Ghost AI queries `Query<(&Position, &RepelsGhost)>`. It doesn't know it's
running from a Crucifix. It might be running from a smudge stick, a holy relic, or a developer debug tool. It just knows
"this entity repels me."

By stripping away the identity (the noun) and replacing it with a capability tag (the verb/fact), you maintain total
domain isolation without needing intermediary mediator crates.

## 3. The Telepathy Test & The Black Hole Test

How do we know if we've screwed up? We ask two absurd questions.

**The Telepathy Test:** Is my system reading another actor's mind? If a UI system or an audio system queries another
domain's deep state (`GhostHuntState`, `EnrageLevel`) just to map it to a volume curve or a visual effect, it is
committing psychic violence. It's a "Tell, Don't Ask" violation. The Ghost should _Push_ its state changes into generic
audio/visual requests, or drop them as localized facts. The UI shouldn't be rummaging around in the ghost's brain.

**The Black Hole Test (Deletion):** If I go into `app.rs` and comment out `.add_plugin(GhostPlugin)`, what happens? In a
perfectly isolated codebase, the game compiles and runs perfectly. The player walks around an empty, boring, entirely
safe house. But if deleting the Ghost breaks the EMF reader, or makes the sanity system crash, or causes the audio
engine to panic because it can't find a target... your domains are bleeding into each other. They are not bounded
contexts.

## 4. The Mathematical Graph (Formalizing the Madness)

Now, let’s get abstract. Because what I really want is to be able to evaluate an architecture mechanically, like solving
`2 + 2 = 4`, regardless of whether we are talking about ghosts or racing cars.

Bevy forces us to declare our dependencies explicitly via system signatures. Every system function takes `Query`, `Res`,
`EventReader`, etc. This means the entire architecture of an ECS game is fundamentally a **Bipartite Directed Graph**.

**Let's build the graph in our minds:**

- Set of Nodes $S$: Our Systems (the verbs, logic).
- Set of Nodes $C$: Our Components, Resources, Events (the data, state).
- Edges are accesses:
  - $Write$ Edge: System $S_1$ mutates/inserts Component $C_1$. ($S \rightarrow C$)
  - $Read$ Edge: System $S_2$ queries Component $C_1$. ($C \rightarrow S$)

**Rules of the Ideal Graph:**

1.  **Domain Ownership:** A domain $D$ is a cluster of Systems. A domain _owns_ a Component $C$ if it is the primary
    writer of $C$. (e.g., `unghost` owns `GhostData` because it's the only one with a write edge to it).
2.  **The Law of Isolation:** A System in Domain B should NEVER have a read edge from a Component specifically owned by
    Domain A, IF that component defines A's internal identity.
3.  **The primitive bridge:** If B needs data heavily influenced by A, A must write to a _Shared Primitive Component_
    $C_{shared}$ (owned by a much lower tier like T1a/T0). Then, domains A, B, C, D can all have read edges from
    $C_{shared}$.

You don't need to read the code to know the architecture is drifting. You can just look at the Bipartite Edge Matrix. If
`unvitals/systems.rs` (Domain B) has a system taking `Query<&GhostTarget>` (Domain A Component), there is a glowing red
edge on your graph. It is mathematically, provably coupled.

We can literally trace the "Span" of a leak. If A writes to B, and B reads it to write to C, the conceptual
contamination spans across those nodes.

## 5. Upward Signal Observation

Sometimes the interaction is a discrete moment, not continuous. You flick a light switch. You drop an item. The ghost
opens a door. How do we do this without Domain A knowing about Domain B?

Bevy 0.14+ Observers and Events are the answer, but the _direction_ is critical. Lower Tiers emit pure structural facts.
Higher Tiers figure out what it means.

The `uninteraction` crate (T1c) shouldn't be triggering a `GhostStartledEvent`. It just emits an
`EntityInteractedWith { target: Entity, user: Entity }` event. It’s a blind structural event. The ghost AI (T1b), using
an Observer, listens to _all_ interactions. It simply filters: "Did someone interact with my target?" The cause emits.
The effect listens. No middleman crate. No circular dependency.

## Summary of the Ramble

We don't need more boxes. We don't need more "Core" packages acting as dumb data bags just to satisfy Cargo's dependency
checker (The Horizontal-Cut Trap).

We need a tighter, richer, denser physical environment (T1a / The Stage) that acts as the ultimate medium of exchange.
The game objects leave fingerprints on the world, and other game objects read those fingerprints.

We need opaque capability tags so systems manipulate _behaviors_ instead of identifying _nouns_.

And we need to evaluate our coupling not by looking at `Cargo.toml` lines, but by graphing out exactly which Systems are
Reading which Components. If the data belongs to another feature, we're doing it wrong.

End of brain dump. For now.
