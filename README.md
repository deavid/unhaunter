# Unhaunter: Dare to Face the Unseen

Dare to enter a world where shadows whisper and every creak could be a ghostly presence. In Unhaunter, you're a paranormal investigator armed with cutting-edge gear, tasked with identifying and expelling restless spirits from haunted locations. 

This 2D isometric game seamlessly blends exploration, puzzle-solving, and strategic investigation, offering a unique blend of thrills and chills for those brave enough to confront the unknown.

## Gameplay

### Exploration 

Explore atmospheric, isometric environments and unravel their secrets.  Venture into dimly lit rooms, interact with objects like doors, switches, and lamps, and uncover clues to help you identify the ghost.

### Investigation 

Your ultimate goal is to banish the lingering spirits (currently one per location). 

To achieve this, you must first identify the ghost among 44 distinct possible ghost types. Each ghost type interacts with your equipment in a different way, leaving behind specific clues known as evidence.

There are 8 types of evidence. Each ghost exhibits 5 of these 8.

### Ghost Identification 

Locate the ghost, carefully test your equipment, and record your findings in your trusty van. Once you have enough evidence, synthesize a specialized "Unhaunter Ghost Repellent" to expel the ghost.

Once you're done, you can click "End Mission" on the van and you'll get the mission score.

### Controls

* **[WASD]:** Movement
* **[E]:** Interact (doors, switches, lamps)
* **[R]:** Activate right-hand gear
* **[T]:** Activate left-hand gear
* **[Q]:** Cycle right-hand inventory
* **[TAB]:** Trigger left-hand item
* **[F]:** Grab item
* **[G]:** Drop item
* **[C]:** Change evidence

### Ghost Hunting

Beware! Ghosts can enter a hunting phase, becoming more aggressive and directly pursuing players to inflict damage. The likelihood of a hunt increases as the ghost's rage grows, and its duration is determined by the ghost's "hunting" state.

## Evidence & Equipment

| Evidence       | Description                                                                                      |
| -------------- | ------------------------------------------------------------------------------------------------ |
| Freezing Temps | The room frequented by the ghost becomes unusually cold, sometimes dropping below 0°C.           |
| Floating Orbs  | The ghost's breach (a spectral dust cloud) might glow when viewed through a Night Vision camera. |
| UV Ectoplasm   | The ghost might emit a greenish glow under UV light.                                             |
| EMF Level 5    | The EMF Meter may spike to level 5 in the presence of certain ghosts.                            |
| EVP Recording  | The Recorder might capture ghostly voices (Electronic Voice Phenomena).                          |
| Spirit Box     | Screams, whispers, or other paranormal sounds may be heard through the Spirit Box.               |
| RL Presence    | The ghost might emit an orange glow under red light.                                             |
| 500+ cpm       | The Geiger Counter may detect elevated radiation levels near certain ghosts.                     |

## Basic Strategy

### Quick Tips:

* **Use your ears:** Pay close attention to audio cues from your equipment; they can provide valuable clues.
* **Control the environment:** Closing doors helps contain cold air for more accurate temperature readings. Lights also heat up the room, so turning them off can create a colder environment.
* **Sanity is key:** Manage your sanity by taking breaks in the truck.

### Finding the Ghost

Your first task is to locate the ghost and determine its preferred area.

The ghost's spawn point, known as its breach, appears as a subtle, semi-transparent dust cloud. It's most visible with the location's lights (not from your torch). 

### Gathering Evidence

Ghosts can move throughout the environment, but you might find more activity near their breach (spawn point). Investigate this area carefully.

For the best results, turn off lights near the breach and close the doors. This will help create a colder environment for more accurate temperature readings and might enhance the visibility of certain paranormal phenomena.

Use your equipment and take note of which ones yield positive results.

### Crafting the Repellent

Return to your van and record the evidence you've gathered in your journal. The crafting of the ghost repellent can only be performed inside the van.

As you record evidence, the list of possible ghosts in your journal will narrow down. Once you're confident in your identification, select the ghost and click "Craft Unhaunter Ghost Repellent".

### Expelling the Ghost

This will create a vial filled with the specific repellent needed to banish that ghost type. Return to the ghost's room (breach), wait for it to appear, and activate the vial. 

If successful, the ghost will simply vanish. There's no special animation or sound effect, so it's up to you to confirm that the ghost is truly gone. You can refill the vial automatically when crafting a new repellent in the van.

Once you're certain there are no more ghosts, go back to the van and click "End Mission".

## Building and Installing

There are no pre-built binaries or installers for Unhaunter. To play, you'll need to build it from source.

1. Clone the repository:

   ```bash
   $ git clone https://github.com/deavid/unhaunter.git
   ```

## Prerequisites

You'll need to have Rust and the necessary dependencies for Bevy installed.

2. Install Rust:

   [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)

3. Install Bevy dependencies:

   Follow the instructions for your operating system at:

   [https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies](https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies)

4. Run the game:

   ```bash
   $ cargo run
   ```

   Run this command from the game's source folder.

**Note:** Unhaunter is being actively developed and built using Bevy version `0.13.0`. While it should run on a wide range of computers, configurations haven't been extensively tested.

## Profiling

If you encounter performance issues, profiling can help identify the bottlenecks.
**Warning:** Profiling generates a large amount of data (potentially gigabytes). Be mindful of this and profile only for short durations.

To run a profiling session:

   ```bash
   $ cargo run --release --features bevy/trace_chrome
   ```

   This creates a file named `trace-1999999999999999.json` (the numbers will vary) in the same folder from where you ran `cargo run`.

   **Warning:** The trace file may contain private information about your system. Be cautious about sharing it.

### Inspecting the Trace

1.  **Compress the trace:** The JSON trace file can be compressed significantly. Use 7-Zip or a tool like ZSTD for efficient compression.
2.  **Open the trace:**  You can inspect the trace using [https://ui.perfetto.dev](https://ui.perfetto.dev). If the file is too large for the browser's WASM limit, follow the instructions at:
    [https://perfetto.dev/docs/quickstart/trace-analysis#trace-processor](https://perfetto.dev/docs/quickstart/trace-analysis#trace-processor)
3.  **Analyze the trace:**  Zoom in on the timeframe you want to analyze (typically the later portion) and look for the `bevy_app -> winit event_handler -> update -> main_app -> schedule: name=Main -> schedule: name=Update` section. This will reveal the main contributors to frame time.

**Note:**  `bevy_framepace::framerate_limiter` will likely take up most of the time, as its purpose is to introduce delays to maintain a consistent FPS.

For more information on profiling Bevy, see:

[https://github.com/bevyengine/bevy/blob/main/docs/profiling.md](https://github.com/bevyengine/bevy/blob/main/docs/profiling.md)

## WASM Support

You can play Unhaunter directly in your web browser:

[https://deavid.github.io/unhaunter/](https://deavid.github.io/unhaunter/)

**Note:**  Google Chrome is the recommended browser for the best experience.

### Current WASM Limitations:

*  Performance issues may occur in Firefox.
*  Map names are displayed by filename rather than internal name.
*  Map data is pre-baked and doesn't reflect newly added maps.

This WASM version is intended as a demo for those who cannot build the game locally. Unhaunter primarily targets native builds, so WASM support will be minimal for now. 

## Faster Compile Times

### Dynamic Linking

Using dynamic linking for incremental builds (small code changes) can significantly reduce compile times:

   ```bash
   cargo run --features bevy/dynamic_linking  
   ```

This is mainly beneficial for debug builds. For fresh builds, the difference is negligible.

You can profile the build process to identify further optimizations using:

   ```bash
   RUSTFLAGS="-Zself-profile" cargo +nightly run --features bevy/dynamic_linking
   ```

**Note:** This requires a nightly Rust toolchain.

## Community

Unhaunter has a Matrix room for discussion and collaboration. Access is by invitation only. To join, please contact deavid (@deavidsedice:matrix.org) on Matrix.

[Matrix Room](https://matrix.to/#/#unhaunter:matrix.org)

## Future Plans

Unhaunter is constantly evolving!  Currently, two ghost events are implemented: Door Slamming and Light Flickering. Look forward to more ghost types, expanded locations, additional equipment, new ghost events, and more challenging gameplay in future updates. 
