#!/bin/bash

# Script to get the duration of an OGG audio file in seconds.
# It uses ffprobe, which is part of the ffmpeg suite.

# Arguments:
#   $1: Path to the OGG file (string) - The audio file whose duration needs to be determined.

set -e # Exit immediately if a command exits with a non-zero status.

# --- Argument Parsing ---
OGG_FILE_PATH="$1"

# --- Input Validation ---
# Check if the provided file path actually points to a file.
if [ ! -f "${OGG_FILE_PATH}" ]; then
    echo "Error: OGG file not found at ${OGG_FILE_PATH}" >&2
    exit 1 # Exit with an error code if the file doesn't exist.
fi

# --- Duration Extraction ---
# Use ffprobe to extract the duration:
#   -v error: Suppress all ffprobe output except for errors.
#   -show_entries format=duration: Only show the duration from the format section.
#   -of default=noprint_wrappers=1:nokey=1: Output in a simple format (value only, no key or wrappers).
DURATION=$(ffprobe -v error -show_entries format=duration -of default=noprint_wrappers=1:nokey=1 "${OGG_FILE_PATH}")

# --- Output Validation and Return ---
# Check if ffprobe successfully returned a duration.
if [ -z "${DURATION}" ]; then
    echo "Error: Could not determine duration for ${OGG_FILE_PATH}" >&2
    exit 1 # Exit with an error code if duration couldn't be found.
fi

# Print the duration to standard output.
# The calling Rust program will capture this.
echo "${DURATION}"
