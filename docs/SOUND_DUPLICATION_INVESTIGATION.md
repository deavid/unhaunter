# Investigation: Audio Duplication in Multiplayer

## Issue Description

In multiplayer (tested with 1 player + 1 dedicated server), environmental sounds caused by interactables such as doors
and switches appear to duplicate or multiply exponentially on successive triggers.

- 1st interaction: 1 sound
- 2nd interaction: 2 sounds
- 3rd interaction: 4 sounds (subjective observation, might be additive or multiplicative)

## Facts & Observations

1. `trigger_interactive_sounds` was synthesizing audio from `Changed<Behavior>` on all nodes instead of consuming an
   authoritative server push.
2. `play_interaction_sounds` in `unghost-presentation` was synthesizing audio from `Changed<GhostInteractionSoundCue>`
   on all nodes.
3. `play_vocalization_sounds` in `unghost-presentation` was synthesizing audio from `Changed<GhostVocalization>` on all
   nodes.
4. The earlier assumption that `app.add_message::<SoundEvent>()` replicated audio over the network was wrong.
   `add_message` is the repo's local Bevy message path. Actual network fan-out in this codebase uses
   `app.add_server_message::<T>(Channel::Ordered)` plus `MessageWriter<ToClients<T>>`.
5. The real architectural problem was that audio was being derived reactively from replicated state/components instead
   of being emitted as an authoritative server decision and then played locally on receiving nodes.
6. A second multiplayer edge case was verified during implementation: writing `GhostAudioMessage` to
   `MessageWriter<ToClients<GhostAudioMessage>>` did not make the host hear the sound locally, so the listen server was
   deaf to ghost interaction sounds and roar broadcasts until explicit local loopback was added.
7. Implementation has now started on that standardized server-push design.

## Revised Understanding

### What Was Incorrect In The Original Theory

The original write-up incorrectly stated that `SoundEvent` itself was being network-replicated. That part is false.
`SoundEvent` is currently on the local Bevy message path.

The corrected understanding is:

1. The server changes authoritative gameplay state such as `Behavior`.
2. Clients receive that replicated state.
3. Presentation/audio systems were then inferring playback from `Changed<T>` locally.
4. In the ghost case, two different replicated-state paths existed at once: `Behavior` and `GhostInteractionSoundCue`.
5. That meant there were multiple independent local audio-synthesis paths instead of one standardized authoritative
   server push.

### What Is Proven

1. Ghost interaction audio had a real duplicate-by-design hazard because it used two separate replicated-state signals
   for the same semantic action.
2. Interaction audio was architecturally wrong even if the exact multiplication factor still needs runtime confirmation,
   because it was being generated from replicated `Behavior` changes rather than an authoritative server audio command.
3. The correct network transport pattern for this repo is not `add_message`, but `add_server_message` plus
   `ToClients<T>`.

## Implementation Status

The redesign has started and the following pieces are now implemented:

1. `unaudiospatial` now has a separate local-only audio path via `LocalSoundEvent` and `LocalAudioEmitter`.
2. The clearly local-only caller cluster now uses `LocalAudioEmitter` explicitly instead of the shared-world
   `AudioEmitter` path. This includes handheld detector/device sounds, hide rustle, and handheld toggle clicks.
3. Interaction audio no longer comes from `Changed<Behavior>`.
4. The interaction domain now emits `PlayInteractionAudioMessage` from authoritative interaction execution and room
   synchronization paths.
5. Pure clients receive `PlayInteractionAudioMessage` and convert it into local playback.
6. Ghost interaction sounds, standard ghost roars, and the death-vocalization sequence now all use `GhostAudioMessage`
   as an authoritative server broadcast.
7. Pure clients play `GhostAudioMessage` from the replicated server-message path, and the host now explicitly plays the
   same ghost sounds locally at the authority emit point so listen-server audio matches join clients.
8. `GhostInteractionSoundCue` and `GhostVocalization` have both been removed; ghost presentation now plays only the
   explicit authoritative broadcasts.
9. Pickup/drop audio no longer fires optimistically in `uninventory-plugin`. The authoritative accept points in
   `ungear-plugin` now emit `PlayerGearAudioMessage`, and local playback happens only after server acceptance.
10. Van-entry audio no longer fires from the local locomotion intent path. The truck domain now emits
    `TruckAudioMessage` from the authority-side `Added<InTruck>` transition and clients play that broadcast locally.
11. The short authority audit found that `InTruck` itself is still client-authored today: the owning client inserts the
    marker locally and the authority mirrors it via `ExportPlayerMarkersMessage`. The audio path is now authoritative
    relative to that mirrored transition, but the truck-entry state transition itself is not yet server-validated.

## The Standardized Design Fix (Centralized Push)

To achieve a **clear and clean flow from A->Z with no chance of duplication by design**, we need a unified approach for
handling audio over the network. Unification and standardization simplify the architecture, meaning developers do not
have to guess between "state-reactive" or "event-pushed" audio.

All audio triggers will be **Server-Authoritative Ephemeral Audio (Centralized Push)**.

Make the Server the sole authority on when an audio effect should play, pushing ephemeral audio events to the clients.

1. **Keep `SoundEvent` local:** `SoundEvent` is a local playback primitive, not the cross-network transport.
2. **Dedicated Audio Commands:** The domain (e.g. Interaction domain, Ghost domain) on the **Server** emits a specific
   replicated message (e.g. `PlayInteractionAudioMessage { position, sound_file }`).
3. **Targeted Listeners:** The clients receive `PlayInteractionAudioMessage` and pipe it into a local `SoundEvent`.
4. **Resulting Flow:** Cause (Interaction) -> Server Emits Replicated Message -> Clients Receive -> Clients Translate to
   Local SoundEvent -> Speakers Play once. No state-change double-dipping.

**Why this is the chosen design:**

- **Standardization:** Every single audio event follows the exact same path. There is no need to make subjective choices
  about whether a sound is "stateful" or "ephemeral".
- **Server Authority:** The server has complete control over chance, RNG, and timing.
- **Simplicity:** Client logic becomes "dumb." It only plays a sound when it receives a network message explicitly
  telling it to do so. It doesn't need to try and infer sound triggers from component changes.

## Pending Research & Open Questions

- Migrate the remaining legacy `AudioEmitter` and `SoundEvent` callers to either explicit authoritative server messages
  or explicitly local-only playback, depending on the feature.
- Decide whether truck entry/exit should remain client-authored marker state or be promoted to a proper validated server
  intent path. The audio migration no longer depends on that decision, but the gameplay authority model still does.
- Re-test the original door/switch reproduction case after the new interaction path is in place and record the exact
  before/after behavior.

## Investigation Log

- Initial creation of this document.
- Subagent `Explore` investigated `door` and `switch` sound trigger logic.
- Found `trigger_interactive_sounds` in `uninteraction-plugin` and `GhostInteractionSoundCue` in `unghost-presentation`.
- Confirmed during implementation that `app.add_message::<T>()` is local-only in this codebase; real network transport
  uses `add_server_message` with `ToClients<T>`.
- Added a local-only audio emitter path in `unaudiospatial` for non-network playback.
- Reclassified the clearly local-only caller cluster onto `LocalAudioEmitter`.
- Migrated interaction audio to an authoritative `PlayInteractionAudioMessage` broadcast instead of `Changed<Behavior>`
  synthesis.
- Migrated ghost interaction audio and standard roars to authoritative `GhostAudioMessage` broadcasts.
- Fixed the listen-server ghost-audio loopback bug by pairing authoritative `GhostAudioMessage` broadcasts with local
  host playback at the ghost-domain emit points.
- Deleted the now-unused `GhostInteractionSoundCue` definition after confirming there were no remaining references.
- Migrated the death-vocalization chain to explicit authoritative `GhostAudioMessage` broadcasts and deleted
  `GhostVocalization`.
- Migrated pickup/drop audio to authoritative `PlayerGearAudioMessage` broadcasts emitted from the validated server
  accept points.
- Audited the `InTruck` ownership path and confirmed it is currently client-authored then mirrored by the authority via
  `ExportPlayerMarkersMessage`.
- Migrated van-entry audio to authoritative `TruckAudioMessage` broadcasts emitted from the authority-side
  `Added<InTruck>` transition in the truck domain.
