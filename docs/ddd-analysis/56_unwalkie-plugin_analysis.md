# DDD Analysis: `unwalkie-plugin`

## 1. Bounded Context

**Walkie-Talkie Execution & Narration Shell**. This crate implements the systems that bridge the high-level guidance
rules in `unwalkie-core` with the Bevy engine's audio and visual systems.

## 2. Responsibility

- **Audio Playback**: Monitors the `WalkiePlay` resource and triggers the actual audio playback of voice lines.
- **Trigger Monitoring**: Contains systems that observe the game state (e.g., player inactivity, ghost behavior) and
  push events to the `WalkiePlay` queue.
- **Visual Feedback**: Implements the "Focus Ring" system that highlights entities on the map when the walkie-talkie
  mentions them.
- **Event Handling**: Manages the `WalkieTalkingEvent` and other communication-related events.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `unwalkie-core`, `unghost-core`, `unrender-std`.
- **Appropriateness**: High as a logic plugin. It connects the "Intent" (from core) with the "Representation"
  (audio/visual).

## 4. Encapsulation (Data/Logic Split)

- **Data**: Uses the data models from `unwalkie-core`.
- **Logic**: Contains the Bevy systems for audio playback orchestration and trigger detection.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It implements specific triggers for "ghost presence" or "player activity", which
  couples it to those domains.
- **Infrastructure Leakage**: High. Deeply integrated with Bevy's `AssetServer`, `Audio`, and `Time`.

---

## Technical Debt & Strategic Notes

- **Decoupled Triggers**: Currently, many triggers are implemented directly within this plugin. In a more modular
  future, other plugins (like `unghost-plugin`) would publish events that the walkie-talkie simply consumes, rather than
  the walkie-talkie plugin querying the ghost's state.
- **Focus Ring Logic**: The inclusion of focus rings here is a good example of "Visual Feedback" being owned by the
  narration system. It uses `unrender-std` components to achieve this.
