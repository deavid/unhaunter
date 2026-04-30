#!/bin/bash
# Unhaunter Reverb Generation Script
#
# This script takes all .ogg files in assets/sounds/ and processes them
# with an Impulse Response (IR) to create a "100% Wet" reverb version
# in assets/reverbs/.

set -e

# Configuration
IR_PATH="bundled/impulse_response/423803__johnnyguitar01__ir-estonia-nordea-hall-loud.wav"
SOUNDS_DIR="assets/sounds"
REVERBS_DIR="assets/reverbs"

# Ensure the reverb directory exists
mkdir -p "$REVERBS_DIR"

# Check if IR exists
if [ ! -f "$IR_PATH" ]; then
    echo "Error: Impulse response not found at $IR_PATH" >&2
    exit 1
fi

echo "Generating spooky echoes..."

for f in "$SOUNDS_DIR"/*.ogg; do
    # Skip if no .ogg files found
    [ -e "$f" ] || continue

    FILENAME=$(basename "$f")
    OUTPUT_PATH="$REVERBS_DIR/$FILENAME"

    echo "Processing: $FILENAME"

    # Apply afir filter:
    # [0:a]apad=pad_dur=4[padded]: Add 4 seconds of silence to the input to allow the reverb tail to ring out
    # dry=-100dB: Remove the original sound entirely
    # wet=0dB: Keep the full reverb tail
    # volume=15dB: Boost the signal into the soft-clipper to drive the distortion
    # asoftclip=type=tanh: Apply a smooth hyperbolic tangent (tanh) soft distortion
    # lowpass=f=1000:poles=1: Apply a single-pole lowpass filter at 1kHz
    # -ac 1: Convert to mono (required for spatial audio)
    # -ar 44100: Ensure consistent sample rate
    ffmpeg -y -i "$f" -i "$IR_PATH" -filter_complex "[0:a]apad=pad_dur=4[padded];[padded][1:a]afir=dry=0dB:wet=0dB,lowpass=f=100:poles=1,volume=30dB,asoftclip=type=tanh" -ac 1 -c:a libvorbis "$OUTPUT_PATH" > /dev/null 2>&1
done

echo "Success! Reverb layers generated in $REVERBS_DIR."
echo "Next step: Run 'cargo run -p unassetidx-updater' to register the new files."
