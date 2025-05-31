#!/bin/bash

# Script to extract a specific version's notes from CHANGELOG.md
# Usage: ./extract_changelog_section.sh <version_without_v_prefix> [changelog_file_path]
# Example: ./extract_changelog_section.sh 0.3.0 CHANGELOG.md

set -e # Exit immediately if a command exits with a non-zero status.

VERSION_TO_EXTRACT="$1"
CHANGELOG_FILE="${2:-CHANGELOG.md}" # Default to CHANGELOG.md if not provided

if [ -z "$VERSION_TO_EXTRACT" ]; then
  echo "Error: Version number not provided." >&2
  echo "Usage: $0 <version_without_v_prefix> [changelog_file_path]" >&2
  exit 1
fi

if [ ! -f "$CHANGELOG_FILE" ]; then
  echo "Error: Changelog file '$CHANGELOG_FILE' not found." >&2
  exit 1
fi

# Awk script:
# -v ver_pattern="### Version $VERSION_TO_EXTRACT": Passes the version pattern to awk.
#                                                  We expect headings like "### Version 0.3.0"
# BEGIN { p=0 }: Initializes a flag 'p' to 0 (not printing).
# $0 ~ ver_pattern { p=1; getline; next }: If the current line matches the version pattern:
#                                          - Set 'p' to 1 (start printing from the *next* line).
#                                          - 'getline' reads the next line (skipping the header).
#                                          - 'next' skips further processing for this header line.
# p && /^### Version / { exit }: If 'p' is 1 (we are in the desired section) AND
#                               the current line starts with "### Version " (next version header),
#                               then exit awk (we've reached the end of the section).
# p { print }: If 'p' is 1, print the current line.

awk -v ver_pattern="### Version ${VERSION_TO_EXTRACT}" '
  BEGIN { p=0 }
  $0 ~ ver_pattern { p=1; getline; next }
  p && /^### Version / { exit }
  p { print }
' "$CHANGELOG_FILE"

