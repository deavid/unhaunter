#!/bin/bash

# This script bundles parts of the codebase into text files in ~/Downloads/
# for easy input to AI models, with granular control over directory grouping.

OUTPUT_DIR="$HOME/Downloads/unhaunter"
mkdir -p "$OUTPUT_DIR"

# Helper function to bundle a set of files into an output file
bundle_files() {
    local target_name="$1"
    local file_list="$2"
    local output_file="$OUTPUT_DIR/unhaunter_${target_name}.txt"

    # Don't create empty files
    if [ -z "$file_list" ]; then return; fi

    echo "Bundling '$target_name' into $(basename "$output_file")..."
    {
        for f in $file_list; do
            # Skip files larger than 100KB (102400 bytes)
            file_size=$(stat -c%s "$f" 2>/dev/null || echo 0)
            if [ "$file_size" -gt 102400 ]; then
                echo "--- SKIPPED FILE \`$f\` (size: $file_size bytes, > 100KB) ---"
                echo
                continue
            fi

            echo "--- BEGIN FILE \`$f\` ---"
            echo
            cat "$f" 2>/dev/null
            echo
            echo "--- END FILE \`$f\` ---"
            echo
        done
    } > "$output_file"
}

# 1. Root and unhaunter/ files
# Select root files and unhaunter/ files, excluding other major top-level dirs
root_and_unhaunter_files=$(git ls-files | grep -v '/' | grep -E '\.(rs|toml|md|ron|sh|py|txt)$')
root_and_unhaunter_files="$root_and_unhaunter_files $(git ls-files -- "unhaunter/" | grep -E '\.(rs|toml|md|ron)$')"
bundle_files "root_and_unhaunter" "$root_and_unhaunter_files"

# 2. Crates (crates/*) - excluding tools/
for crate_path in crates/*/ ; do
    crate_dir=$(basename "$crate_path")
    if [ "$crate_dir" == "tools" ]; then continue; fi

    name="crates_${crate_dir}"
    files=$(git ls-files -- "$crate_path" | grep -E '\.(rs|toml|md|ron)$')
    bundle_files "$name" "$files"
done

# 3. Crate Tools (crates/tools/*)
for tool_path in crates/tools/*/ ; do
    tool_dir=$(basename "$tool_path")
    name="crates_tools_${tool_dir}"
    files=$(git ls-files -- "$tool_path" | grep -E '\.(rs|toml|md|ron|sh|py)$')
    bundle_files "$name" "$files"
done

# 4. Docs (root docs)
doc_root_files=$(git ls-files -- "docs/" | grep -E '^docs/[^/]+\.md$' || true)
bundle_files "docs_root" "$doc_root_files"

# 5. Docs Subdirectories (docs/*/)
for doc_sub in docs/*/ ; do
    [ -d "$doc_sub" ] || continue
    sub_dir=$(basename "$doc_sub")
    name="docs_${sub_dir}"
    files=$(git ls-files -- "$doc_sub" | grep -E '\.md$')
    bundle_files "$name" "$files"
done

echo "Done! Files exported to $OUTPUT_DIR"
