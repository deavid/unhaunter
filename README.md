# Unhaunter

A paranormal investigation game in an isometric perspective.

## How to Play

Your task is to expel the ghost(s) from a location (currently 1 ghost only).

To be able to do this, first you need to identify the ghost among 44 different
possible ghost types. Each ghost type interacts with your equipment in a
different way. Each piece of equipment is responsible for detecting a type of
evidence.

There are 8 types of evidence. A ghost has 5 evidences out of those 8.

Locate the ghost, test the different equipment, note down in the van which
evidences you found, create the "Unhaunter Ghost Repellent" and use it to expel
the ghost.

Once you're done, you can click "End Mission" on the van and you'll get the
mission score.

Press [E] on the van to access the van UI for the journal and other useful
stuff. The same key is used to open doors or actuate switches and lamps.

Press [R] to activate the gear on your right hand, [T] for the gear in the left
hand.

Press [Q] to cycle the inventory on your right hand. [TAB] to swap your left
and right hands.

## Evidences

* Freezing temps: Thermometer. The room that the ghost frequents can go below
  zero. Be warned that lights heat up the room and open doors will leak air
  outside. These factors limit your ability to read freezing temps.

* Floating Orbs: Night Vision IR Camera. The ghost's breach (grey translucent
  dust) can illuminate under Night Vision (IR). The room has to be dark for this
  effect to be perceptible.

* UV Ectoplasm: UV Torch. The ghost might glow green under UV light. Other light
  sources might make this very hard to see.

* EMF Level 5: EMF Meter. The EMF Meter might read EMF5 for some ghosts.

* EVP Recording: Recorder. The recorder might show up a "EVP Recorded" message.

* Spirit Box: Spirit Box. Screams and other paranormal sounds might be heard
  through the static.

* RL Presence: Red Light Torch. The ghost might glow orange under this light.
  Other light sources might need to be off for the effect to be evident.

* 500+ cpm: Geiger Counter. The device might read above 500cpm for some ghosts.

## Basic Strategy

First, locate where the ghost is - what room it tends to roam. For this, it is
best to turn on all the lights in the house and open all doors.

Press [T] to enable the torch; it has several power settings, but be warned
that it might turn itself off by overheating.

The ghost has a spawn point, which also sets its favorite room. This spawn point
is known as the ghost's breach, which can be seen as a form of white,
semi-transparent dust. This is better seen using the location's lights; a
regular torch will not show it.

The ghost always roams around the breach. So most of the analysis and tests
should be done in this room.

Once this is located, turn off the lights of that room and the rooms contiguous
to it. Also, close the doors of the room.

Try the different equipment and note which ones gave positive results.

Go to the van to note these down in the journal. It is not possible to note
these down outside of the van, so if you cannot memorize them, you'll need to
make more trips to the van to write them down.

As you set these on the van, the list of possible ghosts will narrow. Once
you're sure which one it is, select it and you'll be able to click 
"Craft Unhaunter Ghost Repellent".

This will fill a vial in your inventory with the repellent for that particular
ghost type. Go to the ghost room (breach) and wait for the ghost to be there,
then activate the vial, which will spread the substance.

If successful, the ghost should disappear. Be warned, there's no cue indicating
if it worked. You need to double-check that the ghost is gone.
If needed, you can refill the vial in the van.

Once you're sure there are no more ghosts, proceed to the van
and click "End Mission".

## Building and Installing

There are no binary, packages or installers for Unhaunter. So far there are no
plans for them, the only way to run the game is to build it from sources.

Download the repository as usual via your favorite method, for example:

$ git clone https://github.com/deavid/unhaunter.git

You'll need to have Rust, follow the steps to install it at:

https://www.rust-lang.org/tools/install

You'll need also to install dependencies for Bevy, follow the instructions for
your operating system at:

https://bevyengine.org/learn/quick-start/getting-started/setup/#installing-os-dependencies

With this, you should be able to run the game by simply running:

$ cargo run

The command must be run from the game source folder.

NOTE: The game is currently being developed and built on a single developer 
  machine using Debian GNU/Linux in some Testing/Sid version in an AMD machine
  using a NVIDIA 3080. The game should run in a wide range of computers and
  configurations, but it has not been tested.

## Profiling

Unhaunter, as any other game, will perform wildly different depending on where
it is executed. If there are performance issues on your system, you can help
by profiling the problem yourself.

WARN: Profiling creates gigabytes worth of data. It is imperative that you know
what do you want to test and do it as quickly as possible. A minute worth of
data could be over 3 GiB.

To run a profiling session, run:

  $ cargo run --release --features bevy/trace_chrome

This will create a file named `trace-1999999999999999.json` in the same folder
from where you executed `cargo run` (numbers will be different).

Be warned that the trace might also contain some private information about your
system. The trace can be opened by others, but only send it to trusted people.
(It shows the paths of where do you have Bevy and other libraries installed,
which is just a minor concern)

Being a JSON file, it is likely that it can be compressed really well. You can
use 7-Zip, or other tools. ZSTD, if you have it, will probably yield good
results in a short amount of time.

For example, compressing a sample trace allows us to get 1.3 GiB
compressed into 33 MiB:

```
$ zstd -9kv trace-1709882754518652.json 
*** Zstandard CLI (64-bit) v1.5.4, by Yann Collet ***
trace-1709882754518652.json :  2.52%   (  1.30 GiB =>   33.5 MiB, trace-1709882754518652.json.zst) 
```

This could be useful to share via email or other methods. ZSTD gives very good
compression ratios at quite fast speed.

If you want to inspect the file yourself, you can use https://ui.perfetto.dev

But usually it won't fit in the browser WASM limit (2 GiB), so you might need 
to follow instructions here:

https://perfetto.dev/docs/quickstart/trace-analysis#trace-processor

Sample session:

```
$ curl -LO https://get.perfetto.dev/trace_processor
  % Total    % Received % Xferd  Average Speed   Time    Time     Time  Current
                                 Dload  Upload   Total   Spent    Left  Speed
100  9759  100  9759    0     0  10107      0 --:--:-- --:--:-- --:--:-- 10102
$ chmod +x ./trace_processor
$ ./trace_processor ~/git/rust/unhaunter/trace-1709880857003805.json --httpd
[865.803] processor_shell.cc:1636 Trace loaded: 2644.45 MB in 63.29s (41.8 MB/s)
[865.804]             httpd.cc:99 [HTTP] Starting RPC server on localhost:9001
[865.804]            httpd.cc:104 [HTTP] This server can be used by reloading https://ui.perfetto.dev and clicking on YES on the "Trace Processor native acceleration" dialog or through the Python API (see https://perfetto.dev/docs/analysis/trace-processor#python-api).
```

Once you have the trace up, you can zoom in/out and pan left/right using the
WASD keys.

In there, zoom in on the timeframe you want, usually it would be on the last
part (3/4th to the right) and look for a single frame to inspect.

To be exact, we are looking for:

* Process: main 0
  * bevy_app
    * winit event_handler
      * update:
        * main_app (for CPU bound problems, for GPU: sub app: name=RenderExtractApp)
          * schedule: name=Main
            * schedule: name=Update

Take a look on that are and see what are the main culprits of the time spent.

NOTE: bevy_framepace::framerate_limiter is intended to take the majority of the
  time. This is because its task is to add a sleep/delay so we keep a constant
  FPS and we don't burn CPU/GPU resources without need.

There's additional info on profiling Bevy here:

https://github.com/bevyengine/bevy/blob/main/docs/profiling.md


## WASM Support

You can test this game in WASM by navigating to:

https://deavid.github.io/unhaunter/

However, Google Chrome is recommended.

Known issues: (Some seem to be because we enabled Bevy's trace by default)

* Firefox seems to have serious performance problems.
* Noticeable audio crackling.
* Map names appear by filenames not by their internal name.
* Map data is pre-backed in, does not react to new maps added into the folder.
* University/School map is very slow.

NOTE: Overall this is provided as a "demo" that is easy to access for those that
cannot build the game themselves. Unhaunter is not targeting WASM, and the
support will be minimal.

Building WASM locally:

https://bevy-cheatbook.github.io/platforms/wasm.html

Install deps

  rustup target install wasm32-unknown-unknown
  cargo install wasm-server-runner

Run with:

  cargo run --target wasm32-unknown-unknown --release

Wasm bindgen:

  wasm-pack build --release --target web

This will build in pkg/

And to test:

  python3 -m http.server

## Faster compile times:

Dynamic linking for small changes reduces from 20s to 5s:

  cargo run --features bevy/dynamic_linking  

This is only useful for debug incremental builds for small changes. If you're 
building from scratch, the difference is negligible.

Profiling the build step can be done with: 

  RUSTFLAGS="-Zself-profile" cargo +nightly run --features bevy/dynamic_linking

Be aware that this requires a nightly toolchain to work, and probably you'll
need to build twice to get the proper timing for incremental.

This shows that the most time is spent in the linker anyway.


