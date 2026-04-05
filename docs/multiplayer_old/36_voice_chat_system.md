# Voice Chat System Design (2026-02-22)

**Status:** Proposed Design

**Goal:** Implement a dual-mode voice communication system (Local and Walkie) that leverages the game's existing
physical simulation (EMI, distance, walls) to create an immersive, tense, and diegetic audio experience.

## 1. Core Philosophy: Audio as a Physical Entity

Voice chat in _Unhaunter_ is not a meta-layer; it exists within the physical space of the game world. It is subject to
the same rules of attenuation, occlusion, and Electro Magnetic Interference (EMI) as any other sound or signal.

To maintain immersion, there is **no visual UI indicator** when someone is speaking. Players must rely entirely on
directional audio and the sound of their teammates' voices to locate them.

## 2. Dual-Mode Communication

Players have two methods of voice communication, each with distinct physical properties.

### 2.1 Local Voice (Press [V])

Local voice represents the player character speaking out loud in the environment.

- **Distance Attenuation:** The volume gradually fades out over distance, becoming completely inaudible beyond a
  realistic range (e.g., 15 meters).
- **Occlusion (Walls):** Physical walls and floors significantly muffle the sound. A player shouting from the basement
  will sound muffled and distant to a player on the ground floor.
- **Directional Audio:** The audio is fully spatialized (3D). Players can determine the direction of the speaker based
  on the sound.
- **EMI Effect:** Local voice is _not_ affected by EMI. It is a purely acoustic phenomenon.

### 2.2 Walkie Voice (Press [B])

Walkie voice represents a radio transmission. It is assumed that all players always have a Walkie-Talkie equipped as
part of their standard gear.

- **Universal Range:** Walkie voice can be heard by all players, regardless of distance or walls.
- **The Squelch:** Pressing and releasing [B] plays a short, immersive "ksssh" (squelch) sound effect to signal the
  start and end of a transmission.
- **Default Distortion:** Walkie audio is slightly distorted by default to sound like a low-fidelity radio speaker.
- **EMI Effect (Digital Corruption):** Walkie transmissions are highly susceptible to EMI.
  - As the sender or receiver enters areas of high EMI (near the ghost), the audio does _not_ get louder or add white
    noise.
  - Instead, it suffers from **digital corruption**: the bitrate drops, the voice becomes robotic, choppy, and packets
    are dropped, simulating a struggling digital radio signal.
  - This corruption is computed based on the sender's EMI, the receiver's EMI, and the EMI along the line-of-sight path
    between them (identical to the Text Chat logic).

## 3. The Ghost and the Dead

### 3.1 Ghost Hearing (Future Scope)

Currently, the ghost does not react to voice chat. However, the system is designed with the future intent that **the
ghost will be able to hear players talking**.

- Any electronic transmission (Walkie Voice, Text Chat) will generate an electronic signature that the ghost can detect.
- Local Voice will generate an acoustic signature.
- _Note:_ This is currently out of scope for the initial implementation, but the architecture (sending spatial data with
  audio packets) supports this future feature.

### 3.2 Dead Players (Ghostly Whispers)

When a player dies, they are not muted or relegated to a spectator-only channel.

- Dead players can still use Local and Walkie voice to speak to alive players.
- However, their voice is **heavily distorted** (e.g., extreme reverb, pitch shifting, or echoing).
- This allows dead players to still participate and provide vague hints, but their communication is inherently creepy
  and difficult to understand, maintaining the horror atmosphere.

## 4. Technical Implementation (TCP/WASM Architecture)

As established in the `guide_to_audio.md` and Hub Architecture (`hub_architecture_v2.md`), all voice traffic is routed
through the Host/Dedicated Server using a TCP/WebSocket relay to protect player IP addresses and support WASM clients.

1. **Client Send (Phase 1):** The speaking client captures audio via `cpal`, pushes it to a lock-free `ringbuf`, encodes
   it (e.g., Opus), and sends it to the Server along with metadata:
   - `mode`: Local or Walkie
   - `position`: `[f32; 3]`
2. **Server Relay:** The Server receives the audio packet and broadcasts it to all other clients. The Server does _not_
   process the audio.
3. **Client Receive & Compute (Phase 2 & 5):** The receiving client decodes the Opus packet, pushes it to a Jitter
   Buffer (`ringbuf`), and plays it via a custom `VoiceStreamAsset` (to bypass the Bevy Asset Server). It applies the
   physical effects locally using Iterator Decorators:
   - _If Local:_ Applies 3D spatialization (via Bevy's `PlaybackSettings::SPATIAL`), distance attenuation, and wall
     occlusion based on the sender's `position` and the local map geometry.
   - _If Walkie:_ Applies the default radio filter. Computes the EMI along the path between the sender and receiver. If
     EMI is high, applies the "robotic/choppy" digital corruption filter (e.g., using `fundsp` or `dasp`).
   - _If Sender is Dead:_ Applies the "Ghostly Whispers" filter.

## 5. Next Steps

- **Phase 1:** Implement the `cpal` to `ringbuf` microphone capture system.
- **Phase 2:** Implement the `VoiceStreamAsset` and Jitter Buffer for continuous playback.
- **Phase 3:** Integrate Opus encoding and TCP/WebSocket transmission into the `unnet` stack.
- **Phase 4:** Implement the Walkie Voice filters (default distortion, squelch sounds, and EMI-driven digital
  corruption) using Iterator Decorators.
- **Phase 5:** Implement the "Ghostly Whispers" filter for dead players.
