#!/bin/bash
# Unhaunter Walkie-Talkie Voice Generation Script
#
# Arguments:
#   $1: Text to synthesize (passed to Kokoro)
#   $2: Temporary WAV output path (e.g., "temp_audio/filename.wav")
#   $3: Final OGG output path (e.g., "assets/walkie/generated/filename.ogg")

set -e

# Get the directory of this script
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"
VENV_ACTIVATE_PATH="$SCRIPT_DIR/../myenv/bin/activate"

# Activate Python virtual environment if it exists
if [ -f "$VENV_ACTIVATE_PATH" ]; then
  echo "Activating Python virtual environment: $VENV_ACTIVATE_PATH"
  source "$VENV_ACTIVATE_PATH"
else
  echo "Warning: Python virtual environment not found at $VENV_ACTIVATE_PATH. Kokoro might not work." >&2
fi

TTS_TEXT="$1"
TEMP_WAV_PATH="$2"
FINAL_OGG_PATH="$3"

KOKORO_VOICE_MODEL="bf_emma"
KOKORO_SPEED="0.9"
FFMPEG_REVERB_IMPULSE="tools/text_to_speech/reverb-clap-room-2.wav" # Relative to project root

# Ensure temp directory for WAV output exists
# mkdir -p "$(dirname "$TEMP_WAV_PATH")"

# Step 1: Generate speech using Kokoro
echo "Kokoro: Synthesizing '$TTS_TEXT' -> $TEMP_WAV_PATH"
kokoro -m "$KOKORO_VOICE_MODEL" --speed "$KOKORO_SPEED" -o "$TEMP_WAV_PATH" -t "$TTS_TEXT"
KOKORO_EXIT_CODE=$?
if [ $KOKORO_EXIT_CODE -ne 0 ]; then
  echo "Error: Kokoro TTS failed with exit code $KOKORO_EXIT_CODE for text: \"$TTS_TEXT\"" >&2
  exit $KOKORO_EXIT_CODE
fi

# Step 2: Apply audio effects using ffmpeg
echo "FFmpeg: Processing $TEMP_WAV_PATH -> $FINAL_OGG_PATH"
ffmpeg -y -i "$TEMP_WAV_PATH" -i "$FFMPEG_REVERB_IMPULSE" -f lavfi -i "anoisesrc=c=brown:a=1,highpass=f=200,lowpass=f=3000" -filter_complex "
  [0][2]amix=inputs=2:duration=shortest:weights=100 1[audio];
  [audio]highpass=f=600,highpass=f=700,alimiter=limit=0.063,asoftclip=type=atan:threshold=0.04:oversample=4,
  lowpass=f=1800,lowpass=f=1500,afir,volume=20dB,alimiter=limit=0.063:level_out=0.9
" -ac 1 -ar 11025 -c:a libvorbis -ab 32k "$FINAL_OGG_PATH"
FFMPEG_EXIT_CODE=$?
if [ $FFMPEG_EXIT_CODE -ne 0 ]; then
  echo "Error: FFmpeg processing failed with exit code $FFMPEG_EXIT_CODE for $TEMP_WAV_PATH" >&2
  unlink "$FINAL_OGG_PATH" # Attempt to clean up failed OGG
  exit $FFMPEG_EXIT_CODE
fi

# Step 3: Cleanup temporary WAV file
unlink "$TEMP_WAV_PATH"

echo "Success: Generated $FINAL_OGG_PATH"
exit 0