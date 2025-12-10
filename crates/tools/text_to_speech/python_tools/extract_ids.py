import re
import sys

def extract_conceptual_ids(file_path):
    try:
        with open(file_path, 'r') as f:
            content = f.read()
    except FileNotFoundError:
        print(f"Error: File not found at {file_path}", file=sys.stderr)
        return []

    regex = r'conceptual_id:\s*"([^"]*)"'
    matches = re.findall(regex, content)
    unique_ids = sorted(list(set(matches)))
    return unique_ids

if __name__ == "__main__":
    if len(sys.argv) > 1:
        file_path = sys.argv[1]
        ids = extract_conceptual_ids(file_path)
        if ids:
            print(f"Successfully extracted {len(ids)} unique conceptual_ids from {file_path}.")
            print("The extracted conceptual_ids are:")
            for id_val in ids:
                print(id_val)
        else:
            print(f"No conceptual_ids found or error reading file {file_path}.", file=sys.stderr)
    else:
        print("Usage: python extract_ids.py <path_to_manifest.ron>", file=sys.stderr)
