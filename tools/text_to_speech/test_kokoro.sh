#!/bin/bash
# filepath: /home/deavid/git/tts/test_kokoro.sh

# Activate the virtual environment
source ./myenv/bin/activate || exit 1

# Voices can be seen at: https://github.com/hexgrad/kokoro/blob/main/demo/app.py
TEXT01A="The ghost was expelled from the haunted location. 
Congratulations! 
Mission is now completed.

Get to the truck and click on the End Mission button.
"
TEXT01B="The ghost is gone. The haunted location is now safe. 
Congratulations!
You have completed the mission.

Remember to push the End Mission button on the van.
"
TEXT02A="No ghost likes being sprayed with repellent. We will need to observe to confirm if it works."
TEXT03A="Well... it seems the repellent did not work. The ghost is still there. We will need to try something else."
TEXT04A="You went in without any gear?... That's brave... But you will need to go back, and get some from the van."
TEXT05A="Remember to use the flashlight on dark areas. It will help you see better."
TEXT06A="If you keep exploring in the dark you'll get insane. Turn on the lights of the location to preserve your sanity."
TEXT07A="Hey!... Get out NOW!... The ghost wants to harm you."
TEXT08A="Hey!... you seem a bit lost, do you need help? 
Maybe you could use the thermometer to find the ghost's room.
The cold typically spreads throughout the haunted location so you should be able to follow the trail.
"
TEXT09A="The ghost is not happy. It's throwing objects around. Be careful."
TEXT10A="The ghost is angry. It's hunting you. You need to hide."
TEXT11A="Uh... Aren't you forgetting something? The van is full of gear. And your hands are empty."
TEXT12A="Bingo! We have a ghost room! The thermometer is showing a cold spot. 
Let's set up the gear."

# VOICE="bf_emma" # quite good.
# VOICE="bm_fable" # acceptable
VOICE="bf_emma"
TEXT="$TEXT12A"

# Run kokoro to generate speech
echo "Kokoro is generating speech..."
kokoro -m "$VOICE" -o speech.wav -t "$TEXT" || exit 1

# Apply audio effects using ffmpeg
echo "FFMPEG is applying audio effects..."
ffmpeg -y -i speech.wav -i reverb-clap-room-2.wav -f lavfi -i "anoisesrc=c=brown:a=1,highpass=f=200,lowpass=f=3000" -filter_complex "
  [0][2]amix=inputs=2:duration=shortest:weights=100 1[audio];
  [audio]highpass=f=600,highpass=f=700,alimiter=limit=0.063,asoftclip=type=atan:threshold=0.02:oversample=4,
  lowpass=f=2000,lowpass=f=1500,afir,volume=20dB,alimiter=limit=0.063:level_out=0.9
" -c:a libvorbis -ab 32k -ar 11025 speech-eq-noise.ogg || exit 1

# Play the processed audio
paplay speech-eq-noise.ogg || exit 1