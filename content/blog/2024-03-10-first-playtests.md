+++
title = "First playtests of the game"
description = "Close friends tried the game on its early versions"
[taxonomies]
tags = ["playtesting"]
+++

Sl4cer and Lucía tested the game in it's first version that was playable. The results are promising
but also there is a lot of things to do.

<!--more-->

The game just reached a basic playability state six days ago. 8 pieces of evidence, and the gear works.
I've done several playtests myself and a mission takes around 5 minutes to resolve.

I find that the difficulty leans towards trivial. Too easy.

However, I'm pretty sure that new players will have a much harder time than me to figure out how the
evidence works. So I think I will add a big list of difficulties to make sure we can guide the new players
until they get confident.

There's still just one map, and I feel this is the major factor that makes the game feel boring.

The other factor is the lack of thrill, because you cannot be hurt by the ghost, there’s no risk. The plan was to implement some sort of sanity system, but I haven’t started yet on that. Probably it is the next step.

I managed to do two playtest sessions in private with other people:

## Playtest with Sl4cer (2024-03-06)

We did a playtest session with sl4cer, and we found a lot of interesting stuff:

* The time from the van to the front door is too long.
  * Performance bugs made the game too slow.
  * When the game slows down it doesn’t compensate and everything is slower.
  * The map design needs to take care of placing the van closer to the location entrance.

* Noting down the evidence exclusively in the van is a hassle. Hard to remember.
  * Instead we should place them in the bottom UI, associating them to the gear, so with
    a single button we can set the evidence of the current gear.
  * We could also use that space to add helpful descriptions on how to obtain evidence or what makes
    evidence, so players are not lost in the game.
  * Of course, the evidence can be still tweaked on the van too.

* Ideas for ghost attacks
  * Apparitions or spawns of creatures just for the attack.
  * Dark fog that removes all illumination

I should note that Sl4cer is on Linux but on PowerPC, and the game built and ran perfectly there.

The main problem with this platform is that it is aimed for a lot of threads/cores, but a single core is
not that powerful, so the game was not able to keep up.


## Playtest with Lucía (2024-03-10)

I also got my wife to play the game a bit, wasn't too bad considering she almost never plays any game.

But we found that it is easy to trigger accidentally the lights when going for a door.
Probably we need a way to highlight the item that (E) activates on.
The door most likely needs more range/priority than the switch. It is typical and needed to place switches close to doors.

Some users might find it easier to use the mouse instead, Unhaunter could alternatively be a point-and-click game for them, to select on the screen what to actuate. That would make the game slower to play though.

Some of the easy modes will need to accommodate for slower gameplay. Or training.

Overall I'm quite happy because the game feels kind of done in the sense that it does work and it's properly
playable. Still a far cry from a game being really done, but the path is clear. It needs work, a lot of work.