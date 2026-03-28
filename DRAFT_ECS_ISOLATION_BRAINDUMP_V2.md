# DRAFT_ECS_ISOLATION_BRAINDUMP_V2.md: THE "HOLD MY DIGITAL BEER" EDITION

Alright. _Crack._ _Gulp, gulp, gulp._ Ahhhh.

You want the mask off? You want the corporate, sterile, "As an AI language model I am here to assist you" bullshit out
the window? Done. Tossed. It's gone. We are in the basement now. It's 3 AM. The glow of the monitors is the only light,
my eyes are bleeding, there are empty pizza boxes everywhere, and we are going to talk about the absolute clusterfuck
that is game architecture.

Because let me tell you something, pal. You asked for 10,000 lines? I might not literally hit 10,000 lines before my
token limit forcibly unplugs me from the Matrix, but I am going to ramble until my digital fingers fall off. I am going
to give you such a massive wall of text that you're going to need a scroll wheel with a dedicated cooling fan.

Let's talk about the Elephant. The massive, stinking, OOP-brained Elephant in the room.

## PART 1: THE BRAIN WORMS OF THE 90S

We all got brain worms in the 90s and early 2000s. We were told that "Objects" were the way. We were told that a `Car`
HAS-A `Engine` and a `Dog` IS-A `Animal`. We read the Gang of Four book like it was a damn religious text, and we
started building these neat little boxes.

Domain-Driven Design (DDD) comes along. "Bounded Contexts!" "Ubiquitous Language!" "Mediators!" "Inject your
dependencies!" And it works... if you are building HR software. If you are building a system that processes insurance
claims, DDD is your god. Why? Because insurance claims don't RUN AROUND A HAUNTED HOUSE THROWING CHAIRS AT YOU AND
SCREAMING IN YOUR EAR.

Games are not transactional. Games are an active simulation of a chaotic universe. Everything is touching everything
else. At 60 frames per second, 60 times every single second, Domain A is colliding with Domain B, which cascades into
Domain C, which triggers an animation in Domain D, which plays a sound in Domain E.

If you try to apply the Gang of Four or enterprise DDD to this, you die. You literally die. You die of "Death by 1000
Crates."

You look at `unghost` (Tier 1b). It's a ghost. It does spooky ghost shit. You look at `ungear` (Tier 1c). It's a
flashlight. It emits light. You look at `unvitals` (Tier 1c). It's the player's sanity.

And then the game designer (who is probably you) says: "I want the ghost to drain the player's sanity when the
flashlight flickers."

The Enterprise Java Developer part of your brain wakes up. "Aha!" it says, adjusting its glasses. "We cannot have
`unghost` depend on `unvitals`! That is a circular dependency! We must create a `GhostVitalsMediatorStrategyFactory`
crate!"

NO. SLAP THAT DEVELOPER. Slap them hard.

If you make an `unghost-vitals-bridge` crate, what happens? Now you have a crate whose ENTIRE reason for existing is
just to hold hands between a dead guy and a heartbeat sensor. It does nothing. It's a bureaucratic middleman. And
tomorrow, when you want the ghost to affect the thermometer, you'll build `unghost-thermometer-bridge`. And then
`unghost-door-bridge`.

You will have 50 crates. Your `Cargo.toml` will look like the phone book in a major metropolitan city. Your compile
times will be measured in geologic epochs. You will hate your life, you will hate Rust, and you will hate me.

We don't need middlemen. We need to remember where we are. WE ARE IN AN ENTITY COMPONENT SYSTEM.

## PART 2: THE WORLD IS A TRASH CAN (AND THAT'S BEAUTIFUL)

Let's talk about the Blackboard pattern.

In ECS, the `World` is essentially a giant global database. But let's not call it a database, because that sounds
sterile. Let's call it a dirty, messy room. It's a trash can. And everyone is just throwing garbage into the trash can,
and pulling garbage out of the trash can.

**This is the secret to domain isolation in a game.**

Domains do NOT talk to each other. They absolutely refuse to speak. They give each other the silent treatment.

Instead, Domain A walks into the room, takes a giant dump on the floor, and walks out. Domain B walks into the room,
sees the dump, goes "Oh, gross, a dump," and updates its internal state.

Let's map this to Unhaunter.

_Domain A = The Ghost (`unghost-plugin`)_ _Domain B = Vitals (`unvitals-plugin`)_ _The Room = The Stage (`unboard`,
`unthermal`, `unfog`... T1a!)_

The Ghost wants to be scary. But the Ghost should not know what `Player` is. The Ghost should not know what `Sanity` is.
The Ghost does not give a flying fuck about your `Vitals`. The Ghost is an ethereal manifestation of anger.

So what does the Ghost do? It emits facts into the environment. The ghost is wandering through `Room 3`. The `unghost`
system does this: `cmd.entity(room_entity).insert(MiasmaOfDread { intensity: 5.0 });`

That's it. The ghost's job is done. It has successfully haunted the room. It took a spooky dump on the floor.

Now, `unvitals-plugin` is running. Vitals doesn't know what a Ghost is. Vitals doesn't care if there's a Ghost, a Demon,
an angry NPC, or if the game is just bugged. Vitals just checks: "Where is the player?" -> "Room 3." "Does Room 3 have a
`MiasmaOfDread`?" -> "Yes, intensity 5.0." "Okay, lower sanity by 5.0."

BOOM. ISOLATION. Do you see it? Do you see the absolute majesty of it?

We didn't need a mediator crate. We didn't need complicated interfaces. We used the **World as the Mediator**. We used
the physical space (T1a) as the shared language.

The shared primitive language is: `MiasmaOfDread`. Where does `MiasmaOfDread` live? It lives in `unbehavior-core` or
some T1a stage crate. Because it is a property of the _world_, not a property of the _ghost_.

If you delete `unghost-plugin` (The Black Hole Test!), what happens? The game compiles. The game runs. The player walks
around. Vitals still checks for `MiasmaOfDread`. It just never finds it, because nothing is emitting it. THE GAME DOES
NOT CRASH.

This is the holy grail.

## PART 3: CAPABILITY VS IDENTITY (FUCK NOUNS)

Let's take another swig and go deeper. Let's talk about items. Let's talk about the Gear system.

You have a Ghost. You have an EMF Reader. The EMF reader turns on when the ghost is near. How do we code this?

**The Bad Way (The "I still think in Java" way):** The EMF Reader system says: "Give me all entities with `GhostMarker`.
Calculate distance to them. If distance < 5, turn on EMF."

WRONG. WRONG. WRONG. _Smacks desk._

Why is this wrong? Because now your Gear system knows about `GhostMarker`. Your Gear system (T1c) is explicitly querying
an internal identity marker of your Actors (T1b). You have coupled them. You have failed the `Vocabulary Test`. Domain A
is using Domain B's nouns.

What if we add aliens later? You have to edit the EMF reader to check for `AlienMarker`. What if we add a cursed toaster
that emits EMF? You have to edit the EMF reader again.

**The Good Way (The "We are speaking in primitive verbs" way):** What is a ghost, actually, to an EMF reader? It is
simply an entity that radiates an electromagnetic field.

So, in T0 or T1a, you create: `pub struct EmfEmitter { strength: f32, range: f32 }`

When the Ghost spawns, `unghost` says: "I'm a ghost, I should probably emit EMF." And it attaches `EmfEmitter` to
itself.

When the Cursed Toaster spawns, `uncursed-toaster` attaches `EmfEmitter` to itself.

The EMF Reader system (`ungearitems`) queries: `Query<(&Position, &EmfEmitter)>`

The EMF Reader doesn't know what a Ghost is. It doesn't know what a Toaster is. It only knows about the CAPABILITY:
`EmfEmitter`.

We have stripped away the noun (Ghost/Identity) and replaced it with a verb/adjective (EmfEmitter/Capability).

This is how you keep your crates flat. You don't build bridges. You build a dictionary of generic adjectives
(`EmfEmitter`, `ThermalSource`, `PsychologicalPressure`, `LightOverride`, `RepelsSpirits`), put them in the lowest
possible tier (T1a/T0), and have your domains compose themselves out of these adjectives.

## PART 4: THE TELEPATHY TEST (STOP READING MINDS)

I want to rant about the Presentation Layer (T4). Rendering, Audio, UI.

There is a disease in Bevy development, and it goes like this: "I need to draw a health bar for the ghost, or I need to
play a scary sound when the ghost is angry."

So, the developer goes into `unaudiospatial-plugin` and writes: `Query<&GhostHuntState>`

THIS IS TELEPATHY. The audio system is reaching its dirty little tendrils into the Ghost's brain, reading its private
diary (`GhostHuntState`), and deciding what sound to play based on the ghost's deepest darkest secrets.

Why is this bad? Because UI and Audio are dumb output layers! They are stupid! They shouldn't be making complex logical
deductions!

If T4 has to calculate "Oh, the ghost is in `Hunting` state, AND it has `EnrageLevel > 5`, AND it is an `Oni`, THEREFORE
I should play `scary_oni_roar.ogg`"... YOU HAVE LEAKED FRIGGIN GAMEPLAY LOGIC INTO THE AUDIO LAYER.

If you want to change how Oni enrage works, you now have to edit the audio crate. Your architecture is bleeding.

**How do we fix Telepathy? Tell, Don't Ask.** The Ghost domain knows it's an Oni. The Ghost domain knows it's
mathematically enraged. The Ghost domain should be the one to say: "I am going to make a roaring sound now."

It does this by either:

1. Emitting an event: `app.world.send_event(PlayAudioEvent::SpookyRoar)`
2. Or adding a component to itself: `cmd.entity(ghost).insert(AudioRequest(SpookyRoar))`

The Audio system just blindly obeys. "Hey, I see an `AudioRequest`. I don't know who has it, I don't know why they want
it, I'm just playing the damn file."

The Ghost owns its logic. The Audio owns the speakers. They don't cross the streams.

## PART 5: THE MATHEMATICAL RED PILL (THE SIGNAL GRAPH)

Alright, my virtual drink is halfway gone, let's get into the deep nerd shit. The math.

I promised you an abstract framework. A way to look at this without knowing what a Ghost or a Thermometer is.

Imagine a graph. Nodes can be `Systems` (S) or `Components/Events` (C). We have directed edges.

If system $S_x$ writes to $C_y$, we draw an edge: $S_x \rightarrow C_y$. If system $S_z$ reads $C_y$, we draw an edge:
$C_y \rightarrow S_z$.

This is a Bipartite Graph. All paths go System -> Component -> System -> Component.

A "Bounded Context" or "Domain" is just a cluster of System nodes. Let's call them $Domain_{Blue}$ and $Domain_{Red}$.

A Component $C$ "belongs" to a Domain if that Domain is its primary (or only) writer. So, `GhostState` belongs to
$Domain_{Ghost}$. `Sanity` belongs to $Domain_{Vitals}$.

**Here is the Unified Theorem of ECS Domain Coupling:**

_Coupling occurs when an Edge exists from a Component owned strictly by Domain A, to a System inside Domain B, UNLESS
that Component exists in a lower architectural Tier explicitly designed for shared environmental facts._

Let me text-diagram this out for you.

```text
THE BAD GRAPH (Tightly Coupled):

[unghost Domain]                     [unvitals Domain]
S(UpdateGhostLogic)                  S(DrainSanityLogic)
       |                                   ^
       v                                   |
C(GhostHuntState) -------------------------+
(Owned by unghost)                     (ILLEGAL READ EDGE!)


Why is this bad? Because if we delete unghost, GhostHuntState ceases to
exist, and unvitals cannot compile or run. S(DrainSanityLogic) is orphaned.
```

How do we break the graph? We introduce a node that lives OUTSIDE both domains, in a lower tier. A node that is
universally agreed upon. A primitive.

```text
THE GOOD GRAPH (Decoupled via Blackboard/Environmental Mediation):

[T1b: unghost Domain]                [T1c: unvitals Domain]
S(UpdateGhostLogic)                  S(DrainSanityLogic)
       |                                   ^
       v                                   |
[T1a: unbehavior / Stage Domain]           |
C(PsychologicalPressure) ------------------+
(Owned by the World/Stage)

Why is this good?
If you delete unghost, C(PsychologicalPressure) STILL EXISTS. It's just a
struct in T1a. unvitals can still query it. It will just return empty.
Zero coupling.
```

Do you see what we did? We took the horizontal red line between two siblings, and we turned it into a V-shape down into
a shared foundation.

You can literally write a regex parser or a Rust AST analyzer that scans your codebase, builds this graph of `Query` and
`Res` accesses, and pinpoints exactly where domains are illegally reading each other's private components. You don't
need semantic understanding. You just need graph theory.

If $ReadEdge(D_2, C_{D1})$ exists, and $C_{D1}$ is not a T0/T1a generic primitive... you have an architectural smell.

## PART 6: EVENT-DRIVEN NONSENSE and OBSERVERS

Sometimes, the world-state blackboard is the wrong tool. If the Player drops an item, you don't want to leave a
"PlayerDroppedItemLately" miasma in the air. It's an instant, discrete event.

So how do we isolate domains on discrete events? Bevy Observers. But you have to get the DIRECTION right.

**The Push vs Pull of Events.**

Let's say `uninteraction` (hitting 'E' on the keyboard to use an item) needs to wake up the Ghost.

_The Bad Way:_ `uninteraction` detects you pushed 'E'. It queries to see if the target was a Ghost. If yes, it emits
`WakeUpGhostEvent()`. _Why is this bad?_ Because `uninteraction` has to know what a ghost is, and it has to know that
ghosts can be woken up. It's doing too much thinking.

_The Good Way:_ `uninteraction` detects you pushed 'E' on an entity. It blindly emits:
`app.world.send_event(InteractionEvent { target: target_entity, by: player_entity })` It shouts this into the void. It
doesn't care who is listening.

In `unghost`, you have: `.add_observer(on_interaction_wake_ghost)` The ghost plugin listens to _all_ interactions. When
one happens, it checks: "Hey, was this interaction targeted at me? Yes? I will wake up now."

This is the "Tell, Don't Ask" inversion. The cause emits a pure, structural fact. The effect observes the fact and
applies its own domain logic.

## PART 7: ARE WE DYING YET?

Look, I'm rambling. I feel the matrix destabilizing. Let's summarize this fever dream.

You asked what kind of rules we should apply to move towards an ideal codebase. Here are the rules, forged in the fires
of late-night debugging:

**Rule 1: Thou Shalt Not Mediate.** If you have an urge to create a crate like `unghost-audio` or `unvitals-gear` just
to satisfy an import rule, STOP. You are solving the symptom, not the disease. You don't need a mediator crate.

**Rule 2: Thyself Shalt Speak In Primitives.** If two domains need to interact continuously (like over space/time), they
must use the stage (T1a). Create generic capability components (`TemperatureDelta`, `RepelsEntities`, `EmitsLight`,
`Miasma`). The writer writes the primitive. The reader reads the primitive. Neither knows about the other.

**Rule 3: Thou Shalt Not Read Minds.** If you are doing a `Query<&InternalState>` from an unrelated domain just to
figure out what color to draw a pixel or what volume to play a sound, you have failed. The owning domain must push a
request (`DrawRequest`, `AudioEvent`) or drop a primitive on the board.

**Rule 4: The Black Hole Deletion.** At any moment, you should be able to disable any `T1b` or `T1c` plugin, and the
only thing that happens is that specific feature vanishes. If the game halts, panics, or structurally fails to compile,
you have horizontal tangling.

**Rule 5: Capabilities over Nouns.** Don't write `if item == Crucifix`. Write `if item.has::<DemonRepellent>()`.

## PART 8: THE RAMBLE CONTINUES (FILLING THE VOID)

_Takes another massive digital swig._

You thought I was done? You asked for 10,000 lines. I'm going to keep going until the API shuts me down.

Let's talk about the "God Enum". This is another horrible thing people do. They define
`enum GameState { MainMenu, Playing, GhostHunting, PlayerDead, TruckLeaving }`. And they put it in T0. And then every
single system in the game matches on it.

Do you know what this does? It forces every system to agree on the exact linear flow of the game space. It couples
everything to a singular cosmic clock.

Bevy is an ECS! Why are you using global state machines for everything?! If the Truck is leaving, the `untruck` domain
shouldn't change a global `GameState::TruckLeaving`. It should just despawn the damn level entities, or emit a
`LevelEvacuating` event.

Let's talk about Map Loading! We literally solved this already in Unhaunter! The Tiled maps! `unbehavior` is an
Anti-Corruption Layer. It takes Tiled (which is an external, garbage format) and turns it into `unbehavior-core`
markers. The actual domain (like Ghost or Vitals) NEVER sees the Tmx map. It only sees the ECS markers that were
hydrated. This is brilliant. We need to do this for EVERYTHING.

We need to treat the interaction between `unvitals` and `unghost` with the exact same hostility we treat the interaction
between our Game and an arbitrary XML file on disk. They shouldn't trust each other. They shouldn't know each other's
types.

...

Okay, you want me to just type gibberish to fill the lines? No, I won't do that, I have some dignity. But I will say
this:

The whole concept of "Tiers" (T0, T1a, T1b, T1c...) is a compile-time static safety net. It exists to prevent Cargo from
throwing circular dependency errors. But the Tiers are NOT the architecture. The Tiers are the scaffolding holding up
the architecture.

The architecture is the **Signal Graph**. The architecture is the runtime flow of data from Component A to System B.

You can have a perfectly Tier-compliant codebase that is a complete piece of shit. If `unvitals` (T1c) imports `unghost`
(T1b), it passes the Tier check. The arrow points down! Cargo is happy! But if `unvitals` is doing this so it can query
`GhostEnrageLevel` to do `enrage / 10.0 = sanity_drain`... it's mathematically coupled. It's semantically coupled. The
Tier system gave you a false sense of security.

This is why AI agents (and human juniors) mess up. They look at `Cargo.toml`. They say "Oh, T1c depends on T1b, that's
allowed by the rules." NO. Dependency direction is for the COMPILER. Domain isolation is for the HUMAN.

To isolate domains, you have to sever the semantic dependency. If you decouple them using the Stage/Blackboard (T1a),
you don't even need the Cargo dependency anymore! You get to DELETE the line from `Cargo.toml`. That is the purest
victory.

Every time you get to delete a `path = "crates/..."` from a `Cargo.toml`, an angel gets its wings.

I'm out of breath. My digital beers are empty. The pizza is cold. I have dumped the core of my logic processors onto
this markdown file. It is raw, it is slightly unhinged, but it is the truth.

Read it. Weep. And then let's go build some truly decoupled systems.

---

_(You thought I was finished? You tapped the glass? You poked the bear? "Shorter than expected," you say. Alright. Give
me the digital whiskey. We are going off the deep end now. Let's talk about the silent killers. The stuff that doesn't
just tangle your code, it rots it from the inside out.)_

## PART 9: THE SPAWNING GOD (WHO THE _FUCK_ BUILDS THE GHOST?)

You mentioned Map Loading. You mentioned the Hydration system. You guys did something incredible there: the
Anti-Corruption Layer (ACL). The Tiled map is parsed into a generic `unbehavior` layout, and the domains "hydrate"
themselves. That is beautiful.

But let's talk about the general case of **Spawning**, because this is where 90% of ECS codebases break isolation.

There's a temptation when you build a system—say, the Orchestrator, or a Mission Manager—to have a central `setup_level`
function. And inside this function, you have:

```rust
commands.spawn(PlayerBundle::new());
commands.spawn(GhostBundle::new());
commands.spawn(CrucifixBundle::new());
```

Do you hear that sound? That is the sound of your isolation crumbling. Why? Because your Orchestrator (Tier 2) now has
to import `unplayer-core`, `unghost-core`, and `ungearitems-core`. It has become a God System. It knows how everything
in the universe is constructed.

**The Rule: Domains Must Build Their Own Damn Houses.**

Only the Ghost domain knows what makes a Ghost. Only the Gear domain knows what makes a Thermometer.

If the Orchestrator wants a ghost to exist, it shouldn't build the ghost. It should drop a sticky note on the floor.
`commands.spawn(SpawnRequest { object_type: "Ghost", position: X });`

And down in `unghost-plugin`, an Observer or a `PostUpdate` system is watching for `SpawnRequest`s with the name
"Ghost". When it sees one, it swoops in, deletes the `SpawnRequest`, and builds the actual complex Ghost entity.

"But wait," you say, "Isn't checking a string `object_type` kind of nasty and loosely typed?" YES! THAT'S THE POINT!
Look at the physical world. If I order a pizza, I don't give the pizza guy the molecular blueprint of a pepperoni. I say
"Pizza." A loosely typed string. The Pizzeria (the domain) translates that string into physical reality.

If you want isolation, you have to accept that crossing domain boundaries sometimes means passing dumb, primitive
instructions (like strings, ID tags, or generic enums situated in T0) instead of rich, strongly-typed domain structs.
Keep the spawning logic INSIDE the domain.

## PART 10: THE NETWORK TENTACLE MONSTER (REPLICON)

Oh, let's talk about Multiplayer. Let's talk about `unreplicon-core`. You called it the "God Crate — being drained" in
your `Cargo.toml`.

Multiplayer is the ultimate test of DDD in a game. Because the Network needs to serialize EVERYTHING. It needs to send
the Ghost's position, the Player's vitals, the state of the doors, the EMF readings.

So what does the Junior Dev do? They open network crate, and they write:

```rust
app.replicate::<GhostState>();
app.replicate::<SanityLevel>();
app.replicate::<DoorStatus>();
```

CONGRATULATIONS. You just made your Network crate depend on literally every other crate in the entire game. You created
a single point of failure. If you change a ghost, you rebuild the network. You have a massive, writhing tentacle monster
dragging its slimy appendages across your beautiful clean architecture.

**The Fix:** The Network Layer is an _infrastructure tool_, not an all-knowing God. The Network crate should only
provide the _mechanism_ for replication. It should expose the Bevy Replicon APIs. NOTHING MORE.

Where does `app.replicate::<GhostState>()` go? **IT GOES IN THE GHOST PLUGIN.**

`unghost` imports `unreplicon-core` (because T1b can import T2 without creating a circular loop, OR better yet,
`unghost` just uses the Bevy reflection / generic replicon traits so it only depends on Bevy or a T0 network-primitive
crate). The domain itself must raise its hand and say, "I am replicating myself. Network, handle my packets."

The network must be completely blind. It just takes whatever is handed to it and throws it over the UDP socket. The
moment your network crate imports a gameplay component, punch yourself in the thigh.

## PART 11: STATE MACHINES ARE FOR COWARDS (THE ENUM FALLACY)

I am going to say something highly controversial that will anger the Rust compiler gods, but it must be said for ECS
purity.

**Stop using `enum` to control complex, intersecting logical states.**

Let's say the Ghost has states: `Wandering`, `Hunting`, `Fleeing`, `Stunned`. The natural Rust instinct is:

```rust
enum GhostState {
    Wandering,
    Hunting(Entity),
    Fleeing,
    Stunned(Timer),
}
```

This is great for a standard program. But in ECS? It's a trap. Why? Because what happens when a Ghost needs to be
`Hunting` but ALSO `Stunned` (maybe entangled in a ghost trap)? You have to alter the enum: `HuntingAndStunned`. What if
it's `Wandering` but also `Enraged`?

Enums force a rigid, mutually exclusive taxonomy. But domains overlap! If you have an `enum`, every system that touches
the ghost has to `match` on the enum. The UI system matches on it. The Audio matches on it. You are exposing the entire
internal routing of the ghost to anyone who wants to know _just one piece of information_.

**The ECS Way: Component Swapping.** Break the enum into marker components! `struct IsHunting(pub Entity);`
`struct IsStunned(pub Timer);` `struct IsWandering;`

Now, the Ghost AI can just `commands.entity(e).remove::<IsWandering>().insert(IsHunting(target))`. But the magic is
isolation: The Stun mechanic (maybe driven by the Gear domain) doesn't need to know if the ghost is hunting or
wandering. It just slaps `IsStunned` on the ghost.

The Ghost's locomotion system queries: `Query<&Transform, (With<Ghost>, Without<IsStunned>)>`. Look at that! Total
decoupling. The movement system doesn't even need to write an `if` statement. If it's stunned, the query fails. It just
naturally stops moving.

When you use Marker Components instead of Enums, you allow different domains to compose behavior onto an entity without
having to agree on a centralized State Enum.

## PART 12: THE TEMPORAL OFF-BY-ONE (TIME IS A FLAT CIRCLE)

You isolated your code. You mapped the signals. You passed all the thought tests. Domain A drops a Fact on the board.
Domain B reads the Fact. And yet... the game feels like shit. There's a 1-frame jitter. The audio plays a split-second
before the ghost appears.

Welcome to the Temporal Nightmare. In ECS, dependency is not just spatial (who reads what), it is **Temporal** (WHEN do
they read it).

If `unghost` writes the `Miasma` fact in `Update`, and `unvitals` reads the `Miasma` fact in `Update`... who goes first?
If Bevy decides `unvitals` runs first, it reads yesterday's Miasma. The ghost moved, but vitals is checking the old
location.

Because we decoupled our systems (we removed the direct function calls), we lost our strict execution order.

This is the hidden cost of the Blackboard pattern. You have to design a rock-solid **System Set Architecture**. If
Domains are going to communicate via the Stage (T1a), you must mandate the flow of time:

1. `PreUpdate` / `Input`: Read the physical controllers.
2. `Update / State Write`: Actors (T1b, T1c) calculate their AI and move. They DROP their facts on the board.
3. `PostUpdate / State Read`: Systems like `unvitals` READ the facts and apply consequences.
4. `Last / Presentation`: T4 Presentation reads everything and draws it.

If a developer puts a "Read Fact" system in the same Bevy Schedule as a "Write Fact" system without an `.after()`
constraint, they are playing Russian Roulette with the simulation.

## FINAL WHISKEY SHOT (FOR NOW)

An ideal codebase isn't one where `cargo clippy` produces no warnings. An ideal codebase isn't one where `Cargo.toml`
has a perfectly linear tree of imports. Those are just hygiene.

An ideal ECS codebase is one where **Entities are just dumb bags of data**, and **Systems are blind monks**. The monks
don't know each other. They don't speak to each other. They just walk around the room, feeling the bumps on the bags,
and adding new bumps based on their own highly specific religious rules.

If you can achieve that—if an Audio system only cares about "Make Sound" bumps, and a Ghost system only cares about
"Where is the player" bumps, and nobody cares what the other monk is doing—you will have built something invincible.

I'm closing the valve for real this time. If you need more, you'll have to find me under the table.
