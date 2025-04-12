# This is a simple test to compare how much time it takes for Python to read
# and parse the *.tsx and *.tmx as XML, placing them onto memory.
# In Rust, the Tiled crate takes 225ms to parse everything (although Tiled crate
# does much, much more than this).

import os
import time
import xml.etree.ElementTree as ET


def parse_xml_file(file_path):
    start_time = time.time()
    try:
        tree = ET.parse(file_path)
        root = tree.getroot()
    except ET.ParseError:
        print(f"Error parsing file: {file_path}")
    end_time = time.time()
    return (end_time - start_time, root)


def main():
    total_parse_time = 0
    files = os.listdir(".")
    xmldata = []
    for file_name in files:
        if file_name.endswith(".tsx") or file_name.endswith(".tmx"):
            (parse_time, xmlroot) = parse_xml_file(file_name)
            total_parse_time += parse_time
            print(f"Parsed file: {file_name} in {parse_time:.6f} seconds")
            xmldata.append((file_name, xmlroot))

    # To showcase that the XML files are indeed loaded:
    tags = []
    for name, root in xmldata:
        tags.append(name)
        for child in root:
            tags.append(child.tag)
    print(tags)

    print(f"\nTotal time spent parsing files: {total_parse_time:.6f} seconds")


if __name__ == "__main__":
    main()
