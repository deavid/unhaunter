# NOTES

This is just a scratchpad for things that I am researching.


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