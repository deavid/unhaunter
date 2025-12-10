import re
import sys

def extract_walkie_event_stats(file_path):
    try:
        with open(file_path, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: File not found at {file_path}", file=sys.stderr)
        return {}

    # First, try to isolate the walkie_event_stats map
    walkie_stats_match = re.search(r"walkie_event_stats:\s*{([^}]*)}", content, re.DOTALL)
    if not walkie_stats_match:
        print("Error: Could not find walkie_event_stats map in the file.", file=sys.stderr)
        return {}

    stats_content = walkie_stats_match.group(1)

    # Regex to find conceptual_id and its play_count
    # This regex looks for:
    #   "ConceptualId": WalkieEventStats(
    #       play_count: 12,
    #       ...
    #   ),
    # It captures "ConceptualId" and 12
    entry_regex = r'"([^"]+)":\s*WalkieEventStats\s*\(\s*play_count:\s*(\d+),'

    play_counts = {}
    for match in re.finditer(entry_regex, stats_content):
        conceptual_id = match.group(1)
        play_count = int(match.group(2))
        play_counts[conceptual_id] = play_count

    return play_counts

if __name__ == "__main__":
    if len(sys.argv) > 1:
        file_path = sys.argv[1]
        stats = extract_walkie_event_stats(file_path)
        if stats:
            print(f"Successfully extracted {len(stats)} walkie_event_stats from {file_path}.")
            # Outputting in a format that can be parsed as a dictionary string
            print(str(stats))
        else:
            print(f"No walkie_event_stats found or error reading file {file_path}.", file=sys.stderr)
    else:
        print("Usage: python extract_play_counts.py <path_to_profile.ron>", file=sys.stderr)
