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
   cargo run -p fasttext_explorer
   ```

## Plans

This tool will be used to:

* **Generate embeddings for phrases in the Unhaunter phrasebook.**
* **Experiment with different approaches to ghost response generation.**
* **Analyze the semantic similarity of phrases and ghost moods.**

The goal is to create a dynamic and emergent ghost communication system that enhances
player agency and creates a more immersive and engaging gameplay experience.
