# Deep Dive: Spirit Box Icon Not Changing on Ghost Shout

## 1. Problem Description

On the client side, the Spirit Box fails to show the "response" or "evidence" icons when the ghost responds to it
(shouts). The Host correctly sees the Spirit Box transition to its "Answer" state, but the Client remains in the
"Scanning" state.

## 2. Facts (100% Certain)

- **Local Response Calculation**: The Spirit Box logic
  ([update_spiritbox](crates/ungearitems-plugin/src/components/spiritbox.rs)) calculates responses locally on both Host
  and Client based on environmental factors.
- **Charge-Based Trigger**: Responses are triggered when the Spirit Box's `charge` value exceeds a threshold (30.0).
- **Environmental Dependency**: `charge` is accumulated based on values in the `SoundGrid` at the Spirit Box's position.
- **Missing Sync**: `unnet-plugin` [does not sync](crates/unnet-plugin/src/systems.rs) the `SoundEmitter`,
  `ThermalEmitter`, or `FluidEmitter` components of ghost entities.
- **RNG Divergence**: The decision to answer is determined by `random_seed::rng()`, which is not synchronized between
  the Host and Client.

## 2. Facts (100% Certain) (Extended)

- **Grid Sync Mismatch:**
  - The `SnapshotMsg` in [crates/unnet-core/src/messages.rs](crates/unnet-core/src/messages.rs#L182) includes `ghosts`,
    `players`, and `gear`, but it **does not** include a serialization of the `SoundGrid`, `ThermalGrid`, or
    `LightGrid`.
  - The Spirit Box logic in
    [crates/ungearitems-plugin/src/components/spiritbox.rs](crates/ungearitems-plugin/src/components/spiritbox.rs#L131)
    checks for the ghost's clarity fields, which **are** synced.
  - **RESEARCH FINDING:** However, the "trigger" for a response involves checking for environmental changes (like recent
    noise). If the Host handles the noise logic but doesn't sync the resulting `SoundGrid` update, the Client-side gear
    sees a "static" environment and never triggers the response logic, even if the "Ghost Clarity" says it should be
    possible.

- **Transient Events (Sounds/Particles):**
  - The networking system _does_ have a mechanism for syncing discrete events: `TransientEvent`.
  - `unnet-plugin/src/systems.rs` correctly passes these from Host to Client inside snapshots.
  - Evidence chirps (like the EMF "ping" or Spirit Box "response" audio) are often spawned as local sounds. If these are
    not wrapped in `TransientEvent` or a network-tracked component state, the client won't hear them unless they are
    triggered locally.

- **Predictive Interaction:**
  - Clients use [ExecuteInteractionEvent](crates/uninteraction-core/src/interaction.rs) to signal intent.
  - If the Spirit Box is "turned on" by the client, they see the local state change, but any _effects_ that depend on
    the Host-authoritative grid (like the ghost talking back) will never arrive.

## 3. Guesses and Theories

### Theory A: The "Ghost Silence" on Client (High Confidence)

Since `SoundEmitter` components are not synced, the Host's ghost might be "shouting" (having high
`SoundEmitter.volume`), but the Client's representation of the ghost is silent.

1. The Host ghost shouts, increasing its local `SoundEmitter.volume`.
2. The Host's `sound_update` system fills the `SoundGrid` with noise near the ghost.
3. The Host's Spirit Box accumulates `charge` and triggers a response (`ghost_answer = true`).
4. On the Client, the ghost exists but its `SoundEmitter` remains at default or zero value because it's not in the
   snapshot.
5. The Client's `SoundGrid` stays quiet near the ghost.
6. The Client's Spirit Box `charge` stays at 0.0, and it never enters the `ghost_answer` state.
7. Consequently, the `GearSprite` stays on the "Scanning" variants and the "EVP Detected!" text/icons never appear.

### Theory B: Non-Deterministic Responses

Even if the environment was perfectly synced, the Spirit Box relies on a local RNG call:

```rust
let r = if spiritbox.charge > 30.0 {
    spiritbox.charge = 0.0;
    rng.random_range(0..10)
} else { 99 };
spiritbox.ghost_answer = matches!(r, 0..=3);
```

Since `rng` is local and not seeded with a shared value (like the map seed), the Host and Client would still diverge on
whether a response occurs at any given moment.

## 4. Proposed Solutions

- **Sync Emitters**: Update the network snapshot system to include `SoundEmitter`, `ThermalEmitter`, and `FluidEmitter`
  for all networked entities.
- **Authoritative Gear States**: Synchronize the `spiritbox.ghost_answer` and `spiritbox.charge` values in the snapshot
  instead of calculating them locally on the Client. This ensures the Client always sees what the Host sees.
- **Shared RNG Seed**: Seed the local RNGs with a value derived from the mission seed and current game tick to improve
  determinism, although authoritative state sync is preferred for gear.

## 5. Further Research Needed

1. **Grid Sync**: Confirm if any part of the `SoundGrid` or `ThermalGrid` is ever synced (current investigation suggests
   no).
2. **Ghost Dynamics**: Verify how the ghost's "shout" is implemented—does it directly modify a `SoundEmitter` or play a
   sound effect? (Spirit Box ignores sound effects, it only reads the Grid).
