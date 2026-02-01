# Multiplayer Breadth: The Missed Systems - Round 8

This document explores deeper gameplay and architectural systems that were previously overlooked, specifically focusing
on single-player assumptions in the codebase and shared world logic.

## 1. Single-Player Assumptions (The `.single()` Problem)

Many existing systems in [unplayer-plugin](crates/unplayer-plugin/src) and [unwalkie-plugin](crates/unwalkie-plugin/src)
use `query.single()` to access the player. In multiplayer, these will cause crashes.

### Examples:

- **Stuck Detection**: [locomotion_interaction.rs](crates/unwalkie-plugin/src/triggers/locomotion_interaction.rs#L42)
  uses `.single()` to check if "the" player is stuck.
- **Intro Triggers**: [tutorial_introductions.rs](crates/unwalkie-plugin/src/triggers/tutorial_introductions.rs)
  triggers based on global state, but needs to handle multiple potential listeners.

**Solution**: All systems targeting "the player" must be refactored to iterate over `Query<(Entity, &MainPlayer, ...)>`.

## 2. Shared Interaction Logic (`InteractiveStuff`)

The `InteractiveStuff` SystemParam
([crates/uninteraction-plugin/src/systems/interactivestuff.rs](crates/uninteraction-plugin/src/systems/interactivestuff.rs))
is the heart of world interactivity (doors, switches, van entry).

### Sync Challenges:

- **State Swapping**: It replaces the `Behavior` component on an entity. In multiplayer, the Host must perform the swap
  and the `Behavior` component (or a state index) must be replicated.
- **Van Entry**: Currently sets `GameState::Truck`. In multiplayer, "Everyone getting in the van" should be a
  synchronized event (e.g. 4/4 players inside -> Transition).

## 3. The Advisor & Walkie-Talkie Triggers

The Advisor (narrative voice) has dozens of triggers in
[crates/unwalkie-plugin/src/triggers/](crates/unwalkie-plugin/src/triggers/).

### Types of Notifications:

- **Universal**: "Chapter Intro" - Everyone should hear it simultaneously.
- **Personal**: "You are moving erratically" or "Your sanity is low" - Should only play for the affected player.
- **Evidence-Based**: "Solid proof found" - Usually shared, as evidence is a team effort.

**Network Strategy**:

- `WalkiePlay` resource must be tracked on the Host for universal events.
- Clients can calculate personal triggers locally, or the Host can send `PlayVoiceMessage(MsgId, TargetPlayer)` events.

## 4. Lobby & Map Hub

The game has a `MapHubState` (a 3D scene for map selection).

- **Social Space**: Players should see each other here before the mission.
- **Ready System**: Mission selection in the Map Hub must be synced so everyone sees the same "poster" or "map screen"
  update.

## 5. Gear & Battery Persistence

Items like Flashlights and Geiger Counters have a `Battery` component.

- **Drain**: The Host should track the battery drain over time.
- **Visibility**: If Player A's flashlight dies, Player B should see the light go out. This implies `Toggleable` and
  `Battery` must stay in sync across the network.

## 6. Hiding & Stealth

The `Hiding` and `HidingSpot` components are used for ghost evasion.

- **Concurrency**: Can two players hide in the same closet? If not, the `HidingSpot` needs an `occupied_by` field.
- **Detection**: The Ghost's AI (on Host) needs to check the `Hiding` state of all connected players.

## 7. Economy & Shared Rewards

### Persistent Data:

- Each player has a local `Persistent<PlayerProfileData>`.
- **Mission End**: The Host calculates `SummaryData` (Ghost correctly identified? Deposits returned?).
- **Broadcast**: Host sends the "Official Result" to all clients.
- **Local Save**: Clients update their own profile based on this result.

## 8. Performance: Large Field Syncing

`ThermalGrid` and `SoundGrid` are large.

- **Optimization**: Do not send the whole grid every frame.
- **Delta-Compression**: Only send changed cells.
- **Quantization**: Send lower-resolution field data to clients; clients interpolate the smooth field locally.

## 9. Conflict Resolution: Equipment Selection

In the truck [crates/untruck-plugin/src/loadoutui.rs](crates/untruck-plugin/src/loadoutui.rs):

- Players interact with a "Van Inventory".
- **Race Condition**: Two players clicking the same Flashlight.
- **Request-Verify-Grant**: Client clicks -> Host verifies "Flashlight 1 is available" -> Host assigns to Client ->
  Client receives update.
