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
TEXT12A="Bingo! We have a ghost room! The thermometer is showing a cold spot. 
Let's set up the gear."

# Gear In Van - player went in and forgot to get gear
GEAR_IN_VAN_1="Your gear is in the van. Are you sure you want to go in without it?"
GEAR_IN_VAN_2="Uh... Aren't you forgetting something? The van is full of gear. And your hands are empty."
GEAR_IN_VAN_3="Wait a minute, you left your kit in the truck! You might want to go back and collect your gear."
GEAR_IN_VAN_4="Wait! Where's your equipment? You need to grab it from the van before going in."
GEAR_IN_VAN_5="Hold on a second! Did you remember to pick up your gear from the truck?"
GEAR_IN_VAN_6="You might want to double-check the van. It seems like you're missing some essential tools."
GEAR_IN_VAN_7="Don't forget to equip yourself from the van before heading inside!"

# Ghost near hunt - rage limit about to be reached
GHOST_NEAR_HUNT_1="Activity levels are off the charts! Something is about to happen. You should get out of there."
GHOST_NEAR_HUNT_2="The energy levels are surging... it's getting angry... I don't think you're safe there."
GHOST_NEAR_HUNT_3="Static's spiking on my end... that's never good... you might want to leave before it's too late."
GHOST_NEAR_HUNT_4="Uh... you're not alone in there. And whatever it is, it's not happy. I'd look for an exit path if I were you."

# Mission start - flavor text to welcome the player to the mission and set the scene. 

# .. easy variant - these are for easy missions which we need to tell the player to enter the location somehow.
MISSION_START_EASY_1="Alright, you're on site. Reports indicate significant paranormal activity. Standard procedure: locate, identify, and neutralize the entity."
MISSION_START_EASY_2="Unhaunter, this is base. Seems you've arrived. We've got multiple reports of disturbances. Get in there; assess the situation, and deal with the problem."
MISSION_START_EASY_3="Looks like you made it. This should be a good one to warm you up. Get inside and find what's causing all that ruckus."
MISSION_START_EASY_4="Okay, you're at the location. Shouldn't be anything too crazy in there... just, you know, the usual ghostly stuff. Head in when you're ready."
MISSION_START_EASY_5="Welcome to the job, Unhaunter. This is a pretty standard haunting, so it's a good place to start. Get inside and do your thing."
MISSION_START_EASY_6="Alright... Rookie... This is it. Don't mess it up! Explore the location and take measurements. Find the ghost and expel it."
MISSION_START_EASY_7="This is a good opportunity to practice, the entity should be easy to deal with. Use the thermometer to find the ghost room."
MISSION_START_EASY_8="Base to Unhaunter, we are getting some readings here... Go inside and see what is that about."
MISSION_START_EASY_9="Hello there. I'm picking up some faint activity, so it's not a total waste of time. See if you can find the source."

# ..medium variant - these are for medium difficulty, a more serious tone. The player knows the drill.
MISSION_START_MEDIUM_1="Mission start, Unhaunter. Preliminary scans show elevated EMF and thermal anomalies. Proceed with caution."
MISSION_START_MEDIUM_2="Base to Unhaunter. You're clear to enter. Remember your training, and keep your wits about you."
MISSION_START_MEDIUM_3="We've got you patched in, Unhaunter. Reports are... unsettling. Let's get this done quickly and cleanly."
MISSION_START_MEDIUM_4="Another day, another haunted location. At least the commute was short. Get in there and do your thing."
MISSION_START_MEDIUM_5="Readings are unstable, Unhaunter. Keep your guard up in there."
MISSION_START_MEDIUM_6="We're detecting some unusual energy signatures. Be careful, this might be more than a routine haunting."
MISSION_START_MEDIUM_7="Alright, Unhaunter, you know the drill... Get in, get the evidence, get out...  And try to stay sane."
MISSION_START_MEDIUM_8="This place has a history, Unhaunter. Don't underestimate it."
MISSION_START_MEDIUM_9="We've got multiple reports on this one.  Could be a strong presence."

# ..hard variant - these are for hard difficulty, let's try to scare the player a bit.
MISSION_START_HARD_1="Hope you brought your    [A](/ɛɪ/)    game...   And maybe a spare pair of pants."
MISSION_START_HARD_2="I've got a bad feeling about this one, but hey, that's your job ...Right?"
MISSION_START_HARD_3="Try not to get too spooked, okay? I hate paperwork."
MISSION_START_HARD_4="Okay, time to earn your paycheck. Don't say I didn't warn you."
MISSION_START_HARD_5="Good luck... you're gonna need it."
MISSION_START_HARD_6="This place... I don't like it.  Get in, get it done, and get out.  Fast."
MISSION_START_HARD_7="The reports on this one are... disturbing.  Don't take any chances."
MISSION_START_HARD_8="I'm getting chills just looking at the readings.  You're on your own in there, Unhaunter."
MISSION_START_HARD_9="This is a bad one, Unhaunter.  A real bad one.  Just... try to come back in one piece."

# VOICE="bf_emma" # quite good.
# VOICE="bm_fable" # acceptable
VOICE="bf_emma"
TEXT="$MISSION_START_EASY_9"

# Run kokoro to generate speech
echo "Kokoro is generating speech..."
kokoro -m "$VOICE" --speed 0.9 -o speech.wav -t "$TEXT" || exit 1

# Apply audio effects using ffmpeg
echo "FFMPEG is applying audio effects..."
ffmpeg -y -i speech.wav -i reverb-clap-room-2.wav -f lavfi -i "anoisesrc=c=brown:a=1,highpass=f=200,lowpass=f=3000" -filter_complex "
  [0][2]amix=inputs=2:duration=shortest:weights=100 1[audio];
  [audio]highpass=f=600,highpass=f=700,alimiter=limit=0.063,asoftclip=type=atan:threshold=0.04:oversample=4,
  lowpass=f=1800,lowpass=f=1500,afir,volume=20dB,alimiter=limit=0.063:level_out=0.9
" -c:a libvorbis -ab 32k -ar 11025 speech-eq-noise.ogg || exit 1

# Play the processed audio
paplay speech-eq-noise.ogg || exit 1