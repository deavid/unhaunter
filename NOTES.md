# NOTES

This is just a scratchpad for things that I am researching.


# Storing state

It is becoming really clear as of v0.2.4 that Unhaunter needs right now a way to
persist data across game sessions.

There are two main things here that I already detected:

1. Storing configuration, options: for example I like using my own music when
   coding and testing, but the menu music pops in. So ideally this should be
   possible to disable for me without removing it from the builds. This means
   that users should be able to disable music too.

2. Gameplay progress. The game will never feel like a real progression can be
   made if we cannot store scores, money, or unlock things.

But also it is becoming very clear that we need full WASM support. Most of the
playerbase comes from WASM builds. We cannot ignore this.

We need some crate that allows us to do this properly, it must work in new
Bevy builds and also in WASM by storing in the browser.

Candidate crates of interest:

- https://crates.io/crates/bevy-persistent
- https://crates.io/crates/bevy_pkv


# Stereo sound management for isometric games

Ideally we would like to be able to pick where a sound is coming from but we
face the problem that the game is not 3D fist person, but isometric. It is 
unclear how to map the stereo channel balance of the sounds to make it feel
natural.

If we always map left and right of the screen to the stereo balance, this feels
natural, but we lack then the up and down.

Following the character as if it was a 1st person shooter it gives full 360º
coverage, but it might be very hard to grasp.

I'm leaning towards FPS style. But probably we want to set this as an option
for the player to change.

# Reverbs and filters

Also it would be interesting to have some reverbs and low/high pass filtering
to the sounds depending on where they come from, so they can sound muffled, or
distant. These are good sound cues.

Bevy 0.15 at least supports reverb, low pass and high pass:

https://docs.rs/bevy/latest/bevy/audio/trait.Source.html

There's also a delay method that could be potentially used to create phase shift
between the channels. (this is to give better audio positioning with less balance
which is optimal for headphones, 0.9ms delay max, low frequencies <1500hz only)

Balance and phase shift might be dizzening to some players so that might need
an option to configure them. As well to set back to Mono.

# Movement keys

Some players prefer arrows (although this is a bit strange). but mainly the
problem here is that there are players that prefer that WASD map to screen
space instead of map space.

This probably means another two options to add.
