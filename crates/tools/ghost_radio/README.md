# Ghost Radio

This is a console-based demo tool for experimenting with a simplified ghost
communication system using scored responses. 

## Prerequisites

* **Rust:**  Make sure you have Rust and Cargo installed.
  If not, you can download them from [https://www.rust-lang.org/](https://www.rust-lang.org/).

## Running the Demo

1. **Navigate to the Project Directory:**
   ```bash
   cd tools/ghost_radio/ 
   ```

2. **Build and Run:**
   ```bash
   cargo run
   ```

## Usage

1.  **Ghost Selection:**  The tool will display a list of available ghosts.
    Choose a ghost by entering the corresponding number.

2.  **Enter Player Phrases:**  Enter a phrase that you want the player to say to the ghost.  

3. **View Ghost Responses:**  The tool will simulate a ghost response based on:
    * The player's phrase.
    * The ghost's current emotional state.
    * The ghost's response pool.

4. **Quit the Demo:**  Type "quit" and press Enter to exit the demo.

## Data Files

*  **`assets/phrasebooks/player.yaml`:**  Contains a list of player phrases with
   their corresponding semantic tags and emotional signatures.
*  **`assets/phrasebooks/ghost.yaml`:**  Contains a list of ghost responses with 
   their corresponding metadata (speech act, emotional signature, response type, etc.).
*  **`assets/ghost_metadata.yaml`:**  Defines the ghost's name, type, and initial emotional state. 

## Notes

* The emotional signature and response selection 
  are currently simplified for demonstration purposes. 

* The console interface is basic and will be 
  refined further in future development. 

Enjoy experimenting with Ghost Radio! 
