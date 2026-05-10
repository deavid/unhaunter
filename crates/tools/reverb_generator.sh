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

    # Check if we should skip processing
    if [ -f "$OUTPUT_PATH" ] && [ "$OUTPUT_PATH" -nt "$f" ] && [ "$OUTPUT_PATH" -nt "$IR_PATH" ]; then
        continue
    fi

    echo "Processing: $FILENAME"

    # Apply afir filter:
    # [0:a]apad=pad_dur=4[padded]: Add 4 seconds of silence to the input to allow the reverb tail to ring out
    # afir=...: Apply the impulse response (100% wet)
    # lowpass=...: Tone shaping
    # volume=... / asoftclip=...: Boost and softly distort
    # agate=threshold=0.0316:release=100: Noise gate to smoothly mute the tail below -30dB (0.0316 amplitude) with 100ms release
    # silenceremove=...: Trim the file once the gate drops it below -60dB
    # -ac 1: Convert to mono
    ffmpeg -y -i "$f" -i "$IR_PATH" -filter_complex "[0:a]apad=pad_dur=4[padded];[padded][1:a]afir=dry=0dB:wet=0dB,lowpass=f=100:poles=1,volume=50dB,asoftclip=type=tanh,agate=threshold=0.0316:release=100,silenceremove=stop_periods=1:stop_duration=0.1:stop_threshold=-60dB" -ac 1 -c:a libvorbis "$OUTPUT_PATH" > /dev/null 2>&1
done

echo "Success! Reverb layers generated in $REVERBS_DIR."
echo "Next step: Run 'cargo run -p unassetidx-updater' to register the new files."
