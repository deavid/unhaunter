
#!/bin/bash

# This script bundles the project's source code into a single text file
# for easy input to AI models. Run: bash catall.sh [-p path_filter ...] > output_file.txt

# This program is intended to be run as this:
# $ bash catall.sh [-p path_filter] >~/Downloads/project_name.txt
#
# The purpose of this is to easily bundle the project into a single TXT file
# that any AI can read it, so that we can get help from them and make them
# code this.

# Parse options
declare -a path_filters
while getopts "p:" opt; do
    case $opt in
        p) path_filters+=("$OPTARG") ;;
        *) echo "Usage: $0 [-p path_filter ...]" >&2; exit 1 ;;
    esac
done
shift $((OPTIND-1))

# Get files
files=$(git ls-files -- "*.rs" "*.toml" "*.md" "*.ron")
if [[ ${#path_filters[@]} -gt 0 ]]; then
    filter_pattern=$(printf "|%s" "${path_filters[@]}")
    filter_pattern=${filter_pattern:1}
    files=$(echo "$files" | grep -E "$filter_pattern")
fi

echo; echo;
echo "--- INCLUDED FILE LIST ---";
echo; echo;
echo $files | xargs -n1 echo
echo; echo;

echo "PROJECT: ";
pwd;
echo; echo;
echo "--- PROJECT FILE LIST ---";
echo; echo;

git ls-files

echo; echo;
echo "--- END PROJECT FILE LIST ---";
echo; echo;

echo; echo;
echo "--- LAST COMMITS ---";
echo; echo;

git log -5


echo; echo;
echo "--- PROJECT SOURCE CODE ---";
echo; echo;
echo; echo;

# for f in $(git ls-files -- "*.rs" "*.yaml" "*.md" "*.sh" "*.toml" "*.html" "*.wgsl"); do
for f in $files; do
    # Skip files larger than 100KB (102400 bytes)
    file_size=$(stat -c%s "$f" 2>/dev/null || echo 0)
    if [ "$file_size" -gt 102400 ]; then
        echo "--- SKIPPED FILE \`$f\` (size: $file_size bytes, > 100KB) ---";
        echo; echo;
        continue
    fi

    echo "--- BEGIN FILE \`$f\` ---";
    echo; echo;
    # echo "// $f";
    # echo; echo;
    cat $f;
    echo; echo;
    # echo "// end of: $f";
    # echo; echo;
    echo "--- END FILE \`$f\` ---";
    echo; echo;

done