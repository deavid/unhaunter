# FastText Explorer

This is a utility tool for exploring FastText word embeddings in Rust.
It's designed to help with the development of the ghost communication system in Unhaunter.

## Usage

1. **Install FastText:**

   Make sure you have the FastText library installed on your system.
   You can install it using `apt` on Debian-based systems:

   ```bash
   sudo apt install fasttext
   ```

2. **Download the Model:**

   Download the pre-trained FastText model (`crawl-300d-2M-subword.bin`) from the FastText website.

   Place the downloaded model file in the `assets/phrasebooks/vectors/` directory.

3. **Run the Tool:**

   From the Unhaunter project root directory, run the following command:

   ```bash
   cargo run -p fasttext_explorer -- <subcommand> [options]
   ```

## Subcommands

### `generate-embeddings`

Generates embeddings for phrases in the Unhaunter phrasebook and stores them in JSONL files.

**Example:**

```bash
cargo run -p fasttext_explorer -- generate-embeddings --phrasebook-type player
```

This command will process all YAML files in the `assets/phrasebooks/player`
directory and generate corresponding JSONL files containing the phrase embeddings
in the `assets/phrasebooks/vectors/player` directory.

**Options:**

* `--phrasebook-type <type>`: 
  The phrasebook type to process (e.g., "player", "ghost"). Required.
* `--no-overwrite`: 
  Don't overwrite existing embedding files.
* `--process-newer`: 
  Only process files where the source YAML file is newer than the destination JSONL file. (Default: true)

### `query-embeddings`

Loads embeddings from the specified phrasebook type and allows
interactive querying for similar phrases.

**Example:**

```bash
cargo run -p fasttext_explorer -- query-embeddings --phrasebook-type player
```

This command will load all embeddings from the `assets/phrasebooks/vectors/player`
directory and enter an interactive loop where you can enter phrases and see
the closest matches from the phrasebook. 

**Options:**

* `--phrasebook-type <type>`: The phrasebook type to load (e.g., "player", "ghost"). Required.

**Interactive Loop:**

1. The tool will prompt you to enter a ghost metadata file. (note: we removed this and defaulted to the shade in source code)
2. Enter the filename (e.g., `sample_ghosts/shade.yaml`), note that the code already looks in `./assets`.
3. Enter the distance from the ghost in tiles (1, 5, 10, 20, 50).
4. Enter a phrase to query. 
5. The tool will display the closest matching phrases and a simulated ghost response based on the specified ghost and distance. 
6. The ghost's mood will be updated based on the interaction, influencing subsequent responses. 
7. Press Enter without typing a phrase to change the ghost metadata or distance. 
8. Press Ctrl-C to exit the tool.

### `simulate-response`

Simulates a ghost response to a given player phrase, considering the ghost's type, mood, and distance.

**Example:**

```bash
cargo run -p fasttext_explorer -- simulate-response --ghost-metadata-file "assets/sample_ghosts/shade.yaml" --distance 5 
```

This command will load the ghost metadata from the `assets/sample_ghosts/shade.yaml` file,
set the distance to 5 tiles, and then prompt you to enter a player phrase.
It will then generate a simulated ghost response based on the provided context.

**Options:**

* `--ghost-metadata-file <file>`: Path to the ghost metadata YAML file. Required.
* `--distance <distance>`: Distance from the ghost in tiles (1, 5, 10, 20, 50). Required.

## Plans

This tool is continuously being developed to:

* Generate embeddings for phrases in the Unhaunter phrasebook.
* Experiment with different approaches to ghost response generation.
* Analyze the semantic similarity of phrases and ghost moods.

The goal is to create a dynamic and emergent ghost communication system that
enhances player agency and creates a more immersive and engaging gameplay experience.
