# Playtesting 01 with Multiplayer

After tons of fixes and refactors multiplayer with bevy_replicon starts to be somewhat usable (spoiler: it isn't).

Let's take a look overall on the user experience. For these tests, we are running 1 dedicated server and 1 client with
the --join flag.

## On the menu

**[A.1]: Clicking "Start Mission" is confusing:** The server starts, the client swaps to "Join mission" but the player
is utterly confused that nothing happened. This was done on purpose because there are bugs if the client joins the
mission before the server spins up and all entities are replicated on the client. And we should wait for the server, but
we also need some way to signal the user that we are in a loading state, and auto-join them eventually without them
doing anything. We need probably some way of telling the status, something that moves, animates, to signal the load.
Something rotating could do. Even just timer based would work 99.9999% of the time if we do like a 2 second wait before
auto-hitting "Join Mission".

## On mission - World Interactions

**[B.1] Cannot enter the van:** The van has a special entity that when interacting with it opens the van. Currently for
some strange reason we are sending the event to the server, and the server is failing to find an interactive entity
there:

```text
2026-03-15T08:08:55.640803Z  WARN unreplicon_plugin::systems::players: SERVER: Failed to find interactive entity at BoardPosition { x: 22, y: 10, z: 0 }
```

While this also points to the server not loading the map properly (maybe??). The important thing is that this operation
of entering the van is client-local, not even predictive. There should be no difference here with a single player mode.

We click [E] for opening the Van (on the van, of course), and it shoud just open. The server doesn't need to gate this.

Of course, that grants us - the player, the hiding status and that should be replicated as with all replicated
components of the player - the client should send their player component changes to the server so that they're re-sent
to other players.

In short, currently it doesn't work at all because it is trying to do something with the server that is not needed.

**[B.2] Light switches do not work:** There seems to be some issue with the RoomStateMap (refactored from the original
RoomDB), where clicking the switch does not change the RoomStateMap. I believe that RoomStateMap is to be replicated so
this means that the probable way of operation for this is, we interact with a switch, the server updates RoomStateMap,
we receive in the client the update of RoomStateMap, then we recompute the lights. Uh no no. Wait. This is probably
incorrect because the server would be in control of the light statuses as well - so it means that the server upon
updating RoomStateMap has to update all lights of the room - as a single player game would - then these new light states
have to be replicated in the client. And I'm aware that we are not replicating on the client the Behavior component on
the excuse of "it's too big", which is making things more complicated. In the end - switches don't work. They should.

## On mission - Grab & Drop

**[C.1] Cannot grab at all:** No idea why. The key just doesn't seem to do anything on multiplayer.

**[C.2] Cannot drop at all:** No idea why. The key just doesn't seem to do anything on multiplayer.

## Setting Evidence

**[D.1] Can't set evidence, seems read-only:** Both in-truck clicking the buttons and in-mission pressing [C] do not
seem to do anything at all. The player can't change the evidence. I believe this is replicated in the server.

## On the truck

**[E.1] End mission most of the time is not enabled:** In theory, we need all alive+connected+non-AFK players in the
truck. During these tests there is only 1 player connected, in the truck and we don't see the button enabled.

## Gear

**[F.1] Pressing [R] to activate sometimes fails or feels bouncy:** Activating the gear sometimes doesn't work,
sometimes seems to fail or be bouncy. This should be a local component, a local action - we shouldn't need the server
here and the server should not override these.
