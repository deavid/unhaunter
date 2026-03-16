#!/bin/bash

# Usage: ./upscale_assets.sh [-f] <zoom_level|auto>
# Example: ./upscale_assets.sh 3
# Example: ./upscale_assets.sh auto
# This will upscale assets using xbrzscale and place them in assets/upscaled/ with the correct prefix.

ZOOM=""
FORCE=false
SINGLE_FILE=""

while [[ "$#" -gt 0 ]]; do
    case "$1" in
        -f|--force) FORCE=true ;;
        --file) shift; SINGLE_FILE="$1" ;;
        *) [ -z "$ZOOM" ] && ZOOM="$1" ;;
    esac
    shift
done

if [ -z "$ZOOM" ]; then
    echo "Usage: $0 [-f] <zoom_level|auto>"
    echo "Example: $0 3"
    echo "Example: $0 auto"
    exit 1
fi

# Ensure xbrzscale is available
if ! command -v xbrzscale &> /dev/null; then
    echo "Error: xbrzscale could not be found. Please install it first."
    exit 1
fi

# Ensure ImageMagick is available
if command -v magick &> /dev/null; then
    IM_CONVERT="magick"
elif command -v convert &> /dev/null; then
    IM_CONVERT="convert"
else
    echo "Error: ImageMagick (magick or convert) could not be found. Please install it first. On Debian systems: sudo apt install imagemagick;"
    exit 1
fi

# Ensure bc is available
if ! command -v bc &> /dev/null; then
    echo "Error: bc could not be found. Please install it first."
    exit 1
fi

# Handle "auto" zoom level
if [ "$ZOOM" = "auto" ] && [ -z "$SINGLE_FILE" ]; then
    echo "Running automatic upscale for zoom levels 2, 3, 4, 6..."
    FORCE_ARG=""
    [ "$FORCE" = true ] && FORCE_ARG="-f"
    SCRIPT_PATH=$(realpath "$0")
    for z in 2 3 4 6; do
        "$SCRIPT_PATH" $FORCE_ARG "$z"
    done
    exit 0
fi

# Target directory
TARGET_DIR="assets/upscaled"
mkdir -p "$TARGET_DIR"

PREFIX=$(printf "zoom%02dx_" "$ZOOM")

# --- Single File Processing Mode ---
if [ -n "$SINGLE_FILE" ]; then
    REL_PATH="$SINGLE_FILE"
    FILE="assets/${REL_PATH}"

    if [ ! -f "$FILE" ]; then
        if [ -f "assets/img/${REL_PATH}" ]; then
            FILE="assets/img/${REL_PATH}"
            REL_PATH="img/${REL_PATH}"
        else
            exit 0 # Skip silently in parallel mode
        fi
    fi

    OUT_FILE="${TARGET_DIR}/${PREFIX}${REL_PATH}"

    if [ "$FORCE" = false ] && [ -f "$OUT_FILE" ] && [ "$OUT_FILE" -nt "$FILE" ]; then
        exit 0
    fi

    # Pre-processing
    PRE_FILE=$(mktemp --suffix=".png")
    $IM_CONVERT "$FILE" \( +clone -blur 0x0.25 \) +swap -background none -layers Flatten "$PRE_FILE"

    echo "Scaling $REL_PATH -> ${PREFIX}${REL_PATH} ..."
    mkdir -p "$(dirname "$OUT_FILE")"
    xbrzscale "$ZOOM" "$PRE_FILE" "$OUT_FILE" >/dev/null
    unlink "$PRE_FILE"

    # Post-processing
    SIGMA=$(echo "scale=2; 0.20 * $ZOOM" | bc)
    BLOOM_SIGMA=$(echo "scale=2; 0.80 * $ZOOM" | bc)
    SHARP_SIGMA=$(echo "scale=2; 1.50 * $ZOOM" | bc)

    $IM_CONVERT "$OUT_FILE" \
        -blur 0x${SIGMA} \
        -blur 0x${SIGMA} \
        -unsharp 0x${SIGMA}+0.7+0 \
        -unsharp 0x${SHARP_SIGMA}+0.3+0 \
        \( +clone -morphology Convolve Blur:${BLOOM_SIGMA},0 -channel A -evaluate multiply 0.6 +channel \) -compose Over -composite \
        "$OUT_FILE"
    exit 0
fi

# --- Main Multi-file Discovery Mode ---
echo "Discovering assets to upscale..."

# Discovery from maps
MAP_ASSETS=$(grep -rhPo '(?<=source=")[^"]+\.png' assets/maps/ | sed 's|^../||' | sort -u)

# Discovery from code (look for upscale_idx.resolve calls and capture the path in quotes)
# This handles the case where the path might be on the next line
CODE_ASSETS=$(git grep -A1 "upscale_idx.resolve(" | grep -oE 'img/[^"]+\.png' | sort -u)

# Manual list for important world sprites that might not be discovered
MANUAL_ASSETS=(
    "img/ghost.png"
    "img/characters-model1-demo.png"
    "img/gear_spritesheetA_48x48.png"
    "img/spritesheetA_3x3x3.png"
    "img/spritesheetA_3x3x3.png"
    "img/spritesheetA_6x6x10.png"
    "img/spritesheetB_3x3x3.png"
    "img/spritesheetB_6x6x10.png"
    "img/van.png"
)

# Combined unique list of assets relative to assets/
ALL_ASSETS=$(echo -e "${MAP_ASSETS}\n${CODE_ASSETS}\n${MANUAL_ASSETS[*]}" | tr ' ' '\n' | sort -u | grep . )

echo "Upscaling $(echo "$ALL_ASSETS" | wc -l) assets to level ${ZOOM}x (Prefix: ${PREFIX})..."

# Parallel processing using xargs
SCRIPT_PATH=$(realpath "$0")
FORCE_ARG=""
[ "$FORCE" = true ] && FORCE_ARG="-f"

echo "$ALL_ASSETS" | xargs -d '\n' -I {} -P "$(nproc)" "$SCRIPT_PATH" $FORCE_ARG "$ZOOM" --file "{}"

echo "Done. Remember to run the game to update the asset index!"

