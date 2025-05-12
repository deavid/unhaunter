TTS using Kokoro
===================

As a possible feature for later on, it could be nice to have a buddy in the radio
that says some lines in certain scenarios. This could add some inmersion on the
game, and provide helpful hints for new players.

This is just a thought and some tests done that are saved here for the posterity,
so they can be referenced later. I'm not sure when this would be in the game
or if it will be discarded later on.

[Kokoro](https://kokorotts.net/) is an opensource AI Text To Speech tool using
Apache 2.0 license, fairly modern and powerful. It sounds fairly realistic,
and it could be enough to replace some cases where a human voice is needed.

The [HexGrad Kokoro GitHub](https://github.com/hexgrad/kokoro) contains the
basic tooling to make this work.

However, this is not something we'd ever want to run on the player end. This is
only for offline voice generation. However note that the model is 82M parameters,
and that could work out if someone wanted to run this inside a game, since it
doesn't seem that big. But it's not the case for Unhaunter.

For Unhaunter I strictly want this for offline generation, to create the voice
lines in my computer and then add the results as part of the source distribution.

This folder will remain as notes on how to do this in a repeatabla manner. The
point being that if new lines are added, or changed, we can easily update them
and ensure all voice lines sound exactly the same.

The process will consist on roughly 3 steps:

1) Generate the voice lines with Kokoro
2) Process the voice with ffmpeg to apply a walkie-talkie effect that gives a
   sensible mood for the game. This step also compresses to OGG at a very low
   quality.
3) Copy the resulting files into the assets folder for regular distribution.

Note that this might require Nvidia's CUDA. And this might not work on all computers.


Setup / Preparing the environment
----------------------------------

If you cloned this repo and you're doing this for the first time, there's a lot
of stuff to get in place before this works.

1. Create a Python Virtual Environment here. I will assume it is named `myenv`.

This assumes you're in the `tools/text_to_speech` folder.

```
apt install python3-venv
python3 -m venv myenv
```

2. Install kokoro via pip in the venv:

```
source ./myenv/bin/activate
pip3 install kokoro
```

3. Ensure ffmpeg is installed on your system:

```
apt install ffmpeg
```

4. For hearing the voice after generation we need `paplay`:

```
apt install pulseaudio-utils
```

Running Kokoro to generate voices
-----------------------------------

At this moment all I have is just a demo to test stuff, this is in `test_kokoro.sh`.

Just run:

```
bash test_kokoro.sh
```

And the voice should be hearable. Some sample lines can be changed in the `sh`
file, and other voices can be tested manually.

The file `reverb-clap-room-2.wav` is a recording of me clapping once on my own
room, this is used for making ffmpeg create a soft reverb.

I haven't found how to ask for the voice list, but
[app.py](https://github.com/hexgrad/kokoro/blob/main/demo/app.py) does contain
a good list inside the CHOICES dict.

The program `test_kokoro.sh` outputs two files:

1) `speech.wav` that is the clean kokoro voice.
2) `speech-eq-noise.ogg` that is after processing with ffmpeg to give it the walkie-talkie effect.

Nothing else here! for now, this is all I have.