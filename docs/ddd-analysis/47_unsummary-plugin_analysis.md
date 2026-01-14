# DDD Analysis: `unsummary-plugin`

## 1. Bounded Context

**Mission Summary Lifecycle & UI**. This crate implements the logic that bridges gameplay outcomes with persistent
player progression and visual feedback.

## 2. Responsibility

- **Metric Observation**: Monitors the `InGame` state to capture real-time data such as elapsed time, average player
  sanity, and survival counts.
- **State Transition**: Orchestrates the move to `AppState::Summary` when mission-end conditions are met (e.g., all
  players dead or mission aborted).
- **UI Orchestration**: Spawns and manages the summary screen UI, including the animated score counting and reward
  display.
- **Profile Integration**: Triggers the final update to the `PlayerProfileData` (money earned, progression) after score
  calculation is complete.

## 3. Dependencies and Appropriateness

- **Dependencies**: `bevy`, `undifficulty-core`, `unassets-core`, `unfoundation-core`, `unboard-core`, `untypes-core`,
  `unsummary-core`, `unui-core`, `bevy-persistent`, `unprofile-core`, `unplayer-core`.
- **Appropriateness**: High as a high-level orchestration plugin. It acts as the "glue" that connects the gameplay
  domain to the persistence and UI domains.

## 4. Encapsulation (Data/Logic Split)

- **Data**: Uses `SummaryData` from `unsummary-core` and persistent profile data.
- **Logic**: Contains the imperative systems for UI management, input handling (Enter to continue), and the
  `finalize_profile_update` logic.

## 5. Semantic & Infrastructure Leakage

- **Semantic Leakage**: Moderate. It has direct knowledge of `PlayerSprite` internals to calculate sanity. In a more
  decoupled world, the player domain might publish a "Mission Metrics" event that the summary plugin consumes.
- **Infrastructure Leakage**: High (by design). It is a UI-heavy plugin that uses Bevy's camera, UI, and input
  subsystems.

---

## Technical Debt & Strategic Notes

- **Metric Decoupling**: Currently, `update_time` queries for `PlayerSprite` to calculate sanity. This couples the
  summary plugin to the player plugin. Moving to an event-based system where the player domain reports its own metrics
  would improve isolation.
- **Profile Persistence**: It directly modifies the `Persistent<PlayerProfileData>`. This is consistent with how other
  plugins handle persistence.
