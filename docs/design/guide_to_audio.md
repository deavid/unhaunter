# The Ultimate Guide to Custom Audio & Real-Time Voice Chat in Bevy 0.18

## Table of Contents

1. [Project Context & Constraints](#1-project-context--constraints)
2. [Core Architecture: Audio Threads vs. Bevy ECS](#2-core-architecture-audio-threads-vs-bevy-ecs)
3. [Phase 1: Recording (Microphone to ECS)](#3-phase-1-recording-microphone-to-ecs)
4. [Phase 2: Continuous Playback (ECS to Speakers)](#4-phase-2-continuous-playback-ecs-to-speakers)
5. [Phase 3: Network, Encoding & Jitter Buffers](#5-phase-3-network-encoding--jitter-buffers)
6. [Phase 4: WASM Gotchas (The Danger Zone)](#6-phase-4-wasm-gotchas-the-danger-zone)
7. [Phase 5: Real-Time Streaming Audio Effects](#7-phase-5-real-time-streaming-audio-effects)
8. [Phase 6: Dynamic Game SFX (Precomputed PCM Audio)](#8-phase-6-dynamic-game-sfx-precomputed-pcm-audio)
9. [Development Roadmap & Plugin Blueprint](#9-development-roadmap--plugin-blueprint)

---

## 1. Project Context & Constraints

**The Goal:** Implement real-time voice chat (Global and 3D Spatial) and dynamic audio effects in Rust using Bevy 0.18.
**The Platforms:** Windows, Linux, and WebAssembly (Browser). **The Network:** Audio is relayed via a central server
using raw TCP (Native) and WebSockets (WASM). _Disclaimer: While UDP/WebRTC is standard for voice, this proof-of-concept
strictly uses TCP and accepts the latency/head-of-line blocking trade-offs._ **The Audio Engine:** Strict adherence to
native `bevy_audio` (powered by `rodio`). No massive third-party engines like `kira` unless absolutely necessary.

---

## 2. Core Architecture: Audio Threads vs. Bevy ECS

Bridging native audio I/O with Bevy’s ECS requires strict thread isolation. `cpal` (the underlying Rust audio crate)
relies on a background stream with a high-frequency callback.

**The Golden Rule:** You cannot interact with Bevy's ECS directly from the audio callback. Attempting to lock a `Mutex`
inside the audio callback will cause audio stuttering (glitches) and anger the borrow checker. Furthermore, pushing
single `f32` samples 48,000 times a second through an MPSC channel (like `crossbeam_channel`) will burn CPU cycles
unnecessarily via atomic overhead.

**The Solution:** Use **`ringbuf`** (a lock-free, zero-allocation SPSC ring buffer) for both directions:

- **Mic to Network:** `cpal` pushes to `ringbuf_A`, Bevy pops from `ringbuf_A`.
- **Network to Speaker:** Bevy pushes to `ringbuf_B`, `rodio::Source` pops from `ringbuf_B`.

---

## 3. Phase 1: Recording (Microphone to ECS)

Because `bevy_audio` lacks built-in microphone support, we must use `cpal` directly to capture audio, spin up an input
stream, and send it to the server without blocking the main game loop.

### Recommended Crates

- `cpal`: Standard audio I/O (handles Native and WASM Web Audio API).
- `ringbuf`: Lock-free queues.

### The Implementation Snippet

```rust
use bevy::prelude::*;
use ringbuf::{traits::*, HeapRb};
use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};

// Resource to keep the stream alive (NonSend because streams aren't Send/Sync everywhere)
#[derive(Resource)]
struct MicStream(cpal::Stream);

// Resource to pull data into the ECS
#[derive(Resource)]
struct MicConsumer(ringbuf::Consumer<f32, std::sync::Arc<ringbuf::SharedRb<f32, Vec<std::mem::MaybeUninit<f32>>>>>);

fn setup_microphone(mut commands: Commands) {
    let host = cpal::default_host();
    let device = host.default_input_device().expect("No mic found");
    let config = device.default_input_config().unwrap().into();

    // Create a lock-free ring buffer (e.g., 1 second of audio at 48kHz)
    let rb = HeapRb::<f32>::new(48000).into_arc();
    let (mut prod, cons) = rb.split();

    let stream = device.build_input_stream(
        &config,
        move |data: &[f32], _| {
            // Push raw f32 audio to the ring buffer (non-blocking)
            prod.push_slice(data);
        },
        |err| eprintln!("Mic error: {}", err),
        None,
    ).unwrap();

    stream.play().unwrap();

    // Store in Bevy
    commands.insert_resource(MicStream(stream));
    commands.insert_resource(MicConsumer(cons));
}

fn process_mic_audio(mut mic: ResMut<MicConsumer> /* inject networking resource here */) {
    let mut buffer = Vec::new();

    // Pull available samples (non-blocking) out of the ring buffer
    mic.0.pop_iter().for_each(|sample| buffer.push(sample));

    if !buffer.is_empty() {
        // 1. Encode buffer with Opus
        // 2. Send over TCP/WebSockets
    }
}
```

---

## 4. Phase 2: Continuous Playback (ECS to Speakers)

When receiving decoded Opus packets from the server, we cannot use a static `AssetServer` `.ogg` file. We must bypass
the asset server by tricking Bevy into playing a continuous, dynamically generated stream.

We achieve this by implementing a custom `Asset` (`VoiceStreamAsset`) that implements Bevy's `Decodable` trait, which
returns an Iterator that implements `rodio::Source`.

### The "Silence" Problem (Critical Pitfall)

If your `ringbuf` is empty, you must return `Some(0.0)` (silence) rather than `None`. If you return `None`, `rodio`
assumes the audio is finished, stops the `AudioPlayer`, and despawns the internal sink. Returning `0.0` keeps the stream
open while the player is quiet.

### The Implementation Snippet 2

```rust
use bevy::prelude::*;
use bevy::audio::Decodable;
use bevy::reflect::TypePath;
use rodio::Source;
use ringbuf::traits::*;

// --- STEP A: The Custom Source ---
// The struct that rodio will continuously call .next() on
pub struct VoiceStreamDecoder {
    // Reading from the playback ringbuffer
    pub consumer: ringbuf::Consumer<f32, std::sync::Arc<ringbuf::SharedRb<f32, Vec<std::mem::MaybeUninit<f32>>>>>,
    pub sample_rate: u32,
    pub channels: u16,
}

impl Iterator for VoiceStreamDecoder {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        // If data is available, return it.
        // If empty, return 0.0 (silence) to keep the stream alive!
        match self.consumer.try_pop() {
            Some(sample) => Some(sample),
            None => Some(0.0),
        }
    }
}

impl Source for VoiceStreamDecoder {
    fn current_frame_len(&self) -> Option<usize> { None }
    fn channels(&self) -> u16 { self.channels }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<std::time::Duration> { None } // Infinite duration
}

// --- STEP B: The Bevy Asset ---
#[derive(Asset, TypePath)]
pub struct VoiceStreamAsset {
    // We hold the consumer to pass it to the decoder when played
    pub consumer: ringbuf::Consumer<f32, std::sync::Arc<ringbuf::SharedRb<f32, Vec<std::mem::MaybeUninit<f32>>>>>,
}

impl Decodable for VoiceStreamAsset {
    type DecoderItem = f32;
    type Decoder = VoiceStreamDecoder;

    fn decoder(&self) -> Self::Decoder {
        // Unfortunately, ringbuf consumers aren't cloneable, so you usually
        // have to wrap the consumer in an Option and .take() it here, or use Arc<Mutex>
        // strictly for the initialization phase (not the audio loop).
        // For brevity, assuming we pass the consumer in.
        VoiceStreamDecoder {
            // Note: In actual implementation, manage ownership so .decoder()
            // can take ownership of the consumer.
            consumer: self.consumer.clone(), // Conceptual
            sample_rate: 48000,
            channels: 1, // Mono is required for spatial panning to work!
        }
    }
}

// --- STEP C: Hooking it up with Spatial Audio ---
// In your app setup: app.add_audio_source::<VoiceStreamAsset>();

fn spawn_remote_player(mut commands: Commands, mut assets: ResMut<Assets<VoiceStreamAsset>>) {
    // Create a new playback ringbuffer for this specific player
    let rb = ringbuf::HeapRb::<f32>::new(48000).into_arc();
    let (prod, cons) = rb.split();

    // Save `prod` to a Bevy Component on the remote player entity
    // so we can push Opus-decoded network packets into it later.

    let stream_asset = VoiceStreamAsset { consumer: cons };
    let handle = assets.add(stream_asset);

    commands.spawn((
        AudioPlayer(handle),
        PlaybackSettings::SPATIAL, // Bevy automatically handles 3D positional audio!
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}
```

---

## 5. Phase 3: Network, Encoding & Jitter Buffers

### A. Encoding (Opus)

You must encode audio to avoid flooding your TCP/WebSocket connection.

- **Native:** Use C-binding crates like `audiopus` or `opus`.
- **WASM (Pure Rust):** Compiling C-code for WASM leads to "Linker Hell." For WASM, use pure-Rust implementations like
  `opus-wa`, `concentrate`, or the `symphonia` ecosystem.

To prevent memory leaks, tie the Opus Decoder lifecycle to the Bevy entity representing the remote player.

```rust
use opus::Decoder;

#[derive(Component)]
pub struct VoicePeer {
    pub decoder: Decoder,
    // The producer end of the playback ringbuffer
    pub audio_producer: ringbuf::Producer<f32, std::sync::Arc<ringbuf::SharedRb<f32, Vec<std::mem::MaybeUninit<f32>>>>>,
}
```

### B. TCP Clumping & The Jitter Buffer

Because TCP guarantees delivery but _not_ timing, packets will arrive in "clumps." If `rodio` plays them instantly, it
will starve in between clumps, sounding like a robot.

- **The Fix:** Implement a Jitter Buffer. When a player starts talking, push decoded Opus frames into your `ringbuf`,
  but _do not_ start playing the `VoiceStreamAsset` (or output silence) until the `ringbuf` has built up roughly 60ms to
  100ms of data.

### C. Sample Rate Conversion

Ensure your `cpal` input (Mic) and your `rodio` output (Speakers) use the exact same sample rate (e.g., 48,000Hz). If
hardware differs, use a resampler crate like `rubato` before encoding, or voices will sound pitch-shifted (like
chipmunks).

---

## 6. Phase 4: WASM Gotchas (The Danger Zone)

If targeting WebAssembly, three massive pitfalls exist:

1. **The Autoplay Policy:** Browsers completely block the Web Audio API until the user interacts with the page
   (click/keypress). If you initialize `cpal` or Bevy's audio plugin on startup, it will silently fail.
   - _Fix:_ Defer audio initialization. Start silently, show a "Click to Enter" button, and only spin up `cpal` after
     the interaction.
2. **WASM TCP Sockets:** Browsers do not support pure `std::net::TcpStream`. You _must_ use WebSockets (e.g., using the
   `ewebsock` crate) for browser targets.
3. **Single Threading:** WASM defaults to a single thread. Avoid blocking calls (`recv()`, `lock()`) at all costs, as
   they will freeze the entire browser tab. `try_recv()` and `ringbuf` are safe.

---

## 7. Phase 5: Real-Time Streaming Audio Effects

To apply effects to the live voice chat stream (e.g., "radio" distortion or noise) without allocating memory, wrap your
`rodio::Source` in an Iterator "Effect Decorator."

### Crates to Consider

- `fundsp`: The powerhouse. Mathematical DSL, WASM-compatible, highly performant.
- `dasp`: Low-level "standard library" for audio DSP.
- `usfx`: Lightweight, good for game-like procedural crunch.

### Manual Streaming Wrapper Example (Soft-Clip Distortion)

```rust
pub struct DistortionEffect<S> {
    pub input: S,
    pub drive: f32, // Strength of distortion
}

impl<S: Iterator<Item = f32>> Iterator for DistortionEffect<S> {
    type Item = f32;

    fn next(&mut self) -> Option<Self::Item> {
        let sample = self.input.next()?;

        // 1. Simple Soft-Clip Distortion (tanh approximation)
        let distorted = (sample * self.drive).tanh();

        // 2. Mix in static Noise
        // Note: Avoid rand() on WASM audio threads. Use a precomputed noise array.
        let noise = (rand::random::<f32>() - 0.5) * 0.01;

        Some(distorted + noise)
    }
}

// Pass through rodio Source traits
impl<S: rodio::Source<Item = f32>> rodio::Source for DistortionEffect<S> {
    fn current_frame_len(&self) -> Option<usize> { self.input.current_frame_len() }
    fn channels(&self) -> u16 { self.input.channels() }
    fn sample_rate(&self) -> u32 { self.input.sample_rate() }
    fn total_duration(&self) -> Option<std::time::Duration> { self.input.total_duration() }
}
```

_To use with `fundsp`, you would build an effect chain (e.g., `highpass_hz(500.0) >> softclip()`) and apply
`effect_chain.tick(sample)` inside the `next()` loop._

---

## 8. Phase 6: Dynamic Game SFX (Precomputed PCM Audio)

**The Trap:** Playing two audio assets simultaneously (e.g., a "dry" footstep and a "wet" reverb layer) and mixing their
volumes will cause "comb filtering" (phase cancellation) because Bevy's ECS cannot guarantee sample-accurate
synchronization across multiple entity spawns.

**The Solution:** Precompute the effects. Load the sound, process the raw data offline during the loading screen, and
store it as a lightweight in-memory asset backed by `Arc<Vec<f32>>`.

- **No RAM Bloat:** A 1-second sound at 48kHz mono is 192KB. `Arc` ensures cloning uses zero extra RAM.
- **Zero CPU Cost:** Reverb/EQ is calculated on loading; playback takes zero CPU (crucial for WASM).

### Implementation Snippet: In-Memory PCM Asset

```rust
use bevy::prelude::*;
use bevy::audio::Decodable;
use bevy::reflect::TypePath;
use rodio::{Decoder, Source};
use std::io::Cursor;
use std::sync::Arc;

// --- 1. The Custom Asset ---
#[derive(Asset, TypePath, Clone)]
pub struct PcmAudio {
    pub samples: Arc<Vec<f32>>,
    pub sample_rate: u32,
    pub channels: u16,
}

// --- 2. The Iterator ---
pub struct PcmDecoder {
    samples: Arc<Vec<f32>>,
    index: usize,
    sample_rate: u32,
    channels: u16,
}

impl Iterator for PcmDecoder {
    type Item = f32;
    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.samples.len() {
            let sample = self.samples[self.index];
            self.index += 1;
            Some(sample)
        } else {
            None // Audio finished naturally
        }
    }
}

impl rodio::Source for PcmDecoder {
    fn current_frame_len(&self) -> Option<usize> { Some(self.samples.len() - self.index) }
    fn channels(&self) -> u16 { self.channels }
    fn sample_rate(&self) -> u32 { self.sample_rate }
    fn total_duration(&self) -> Option<std::time::Duration> {
        Some(std::time::Duration::from_secs_f32(
            self.samples.len() as f32 / self.channels as f32 / self.sample_rate as f32,
        ))
    }
}

impl Decodable for PcmAudio {
    type DecoderItem = f32;
    type Decoder = PcmDecoder;
    fn decoder(&self) -> Self::Decoder {
        PcmDecoder {
            samples: self.samples.clone(),
            index: 0,
            sample_rate: self.sample_rate,
            channels: self.channels,
        }
    }
}

// --- 3. Offline Precomputation System ---
fn precompute_sfx(mut pcm_assets: ResMut<Assets<PcmAudio>>) {
    // Load and decode raw OGG
    let ogg_bytes = include_bytes!("assets/footstep.ogg");
    let decoder = Decoder::new(Cursor::new(ogg_bytes)).unwrap();
    let sample_rate = decoder.sample_rate();
    let channels = decoder.channels();
    let raw_samples: Vec<f32> = decoder.convert_samples().collect();

    // APPLY YOUR DSP HERE (e.g., apply_cave_reverb)
    let processed_samples = raw_samples; // Placeholder

    // Store in Bevy Assets
    let custom_asset = PcmAudio {
        samples: Arc::new(processed_samples),
        sample_rate,
        channels,
    };
    let handle = pcm_assets.add(custom_asset);
    // commands.insert_resource(MySfx { cave_footstep: handle });
}

// --- 4. Spawning in Game ---
// Spawn like standard Bevy audio!
// commands.spawn((AudioPlayer(handle), PlaybackSettings::SPATIAL, Transform::from_xyz(10.0, 0.0, 0.0)));
```

---

## 9. Development Roadmap & Plugin Blueprint

### The Step-By-Step Roadmap

1. **Phase 1: The Echo Chamber (Local Native):** Connect `cpal` to `ringbuf_A`, pipe directly to `ringbuf_B`, and play
   locally via your custom `VoiceStreamAsset` to verify the ECS/Audio bridge.
2. **Phase 2: Network & Encoding (Native):** Add `opus` encoding, transmit over TCP, decode, implement Jitter buffer,
   and play via the speaker ringbuffer.
3. **Phase 3: The WASM Port:** Switch TCP to WebSockets (`ewebsock`), migrate to a pure-Rust Opus codec, and implement
   the browser Autoplay "Click to Start" screen.

### The Bevy Plugin Blueprint

Keep all audio I/O entirely separate from game logic so that when the Bevy Team eventually replaces `rodio` with a new
internal audio engine (slated for later versions), you only have to rewrite your Audio Plugins, not your entire game.

- **`VoiceChatPlugin` (Real-time Streaming):** Handles `cpal` mic recording, network bridging, Opus encode/decode,
  Jitter buffering, and custom `VoiceStreamAsset` management.
- **`VoiceFxPlugin` (Live Stream Decorators):** Wraps iterators inside `VoiceStreamAsset` to apply zero-cost math
  (distortion, filters) before hitting speakers.
- **`DynamicSfxPlugin` (Precomputed Game Audio):** Decodes `.ogg` files during loading screens, runs DSP math on
  vectors, saves `Arc<Vec<f32>>` to `PcmAudio` assets, and exposes handles to gameplay systems.
