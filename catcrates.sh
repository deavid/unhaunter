#!/bin/bash

# This script bundles each folder in crates/ into its own text file in ~/Downloads/
# for easy input to AI models.
# Usage: bash catcrates.sh

OUTPUT_DIR="$HOME/Downloads"
mkdir -p "$OUTPUT_DIR"

# Loop through each directory in crates/
for crate_path in crates/*/; do
    # Remove trailing slash and get the folder name
    crate_name=$(basename "$crate_path")
    output_file="$OUTPUT_DIR/unhaunter_crate_${crate_name}.txt"

    echo "Bundling crate '$crate_name' into $output_file..."

    # Start the bundle
    {
        # Get files using git ls-files restricted to this crate path
        # and filter for extensions used in catall.sh
        files=$(git ls-files -- "$crate_path" | grep -E '\.(rs|toml|md|ron)$')

        # Check if we have any files to process
        if [ -n "$files" ]; then
            for f in $files; do
                # Skip files larger than 100KB (102400 bytes)
                file_size=$(stat -c%s "$f" 2>/dev/null || echo 0)
                if [ "$file_size" -gt 102400 ]; then
                    echo "--- SKIPPED FILE \`$f\` (size: $file_size bytes, > 100KB) ---"
                    echo
                    continue
                fi

                echo "--- BEGIN FILE \`$f\` ---"
                echo
                cat "$f"
                echo
                echo "--- END FILE \`$f\` ---"
                echo
            done
        fi
    } > "$output_file"
done

echo "Done! Files exported to $OUTPUT_DIR"
