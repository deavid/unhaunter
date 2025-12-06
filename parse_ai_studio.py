#!/usr/bin/env python3
"""
Parser for Google AI Studio conversation exports.
Converts JSON format to clean markdown with role labels.
"""

import json
import sys


def parse_conversation(json_file):
    """Parse Google AI Studio JSON export and convert to markdown."""

    with open(json_file, "r", encoding="utf-8") as f:
        data = json.load(f)

    markdown_output = []

    # Process chunks (user messages)
    if "chunkedPrompt" in data and "chunks" in data["chunkedPrompt"]:
        for chunk in data["chunkedPrompt"]["chunks"]:
            if "text" in chunk:
                role = chunk.get("role", "user")
                text = chunk["text"]

                # Skip if this is a thinking part
                if chunk.get("isThought", False):
                    continue

                # Also skip if it's in a parts array with thought=true
                if "parts" in chunk:
                    # Filter out thinking parts
                    text_parts = []
                    for part in chunk["parts"]:
                        if not part.get("thought", False) and "text" in part:
                            text_parts.append(part["text"])

                    if not text_parts:
                        continue
                    text = "".join(text_parts)

                # Format based on role with clear separators
                if role == "user":
                    markdown_output.append(
                        f"\n\n{'='*80}\n## USER\n{'='*80}\n\n{text}\n"
                    )
                elif role == "model":
                    markdown_output.append(
                        f"\n\n{'='*80}\n## MODEL\n{'='*80}\n\n{text}\n"
                    )
                else:
                    markdown_output.append(
                        f"\n\n{'='*80}\n## {role.upper()}\n{'='*80}\n\n{text}\n"
                    )

    return "\n".join(markdown_output)


def main():
    if len(sys.argv) != 2:
        print("Usage: python parse_ai_studio.py <input_json_file>")
        print("Output will be saved as <input_file>.md")
        sys.exit(1)

    input_file = sys.argv[1]
    output_file = input_file.rsplit(".", 1)[0] + ".md"

    print(f"Parsing {input_file}...")

    try:
        markdown_content = parse_conversation(input_file)

        with open(output_file, "w", encoding="utf-8") as f:
            f.write(markdown_content)

        print(f"Successfully converted to {output_file}")

    except Exception as e:
        print(f"Error: {e}")
        sys.exit(1)


if __name__ == "__main__":
    main()
