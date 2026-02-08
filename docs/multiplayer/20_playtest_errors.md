# First playtest session in multiplayer across the internet

This was tested with RTT: ~60ms

## Symptomps

- Noticeable lag - 60ms is quite noticeable specially to move.
- Doors not syncing properly when joining late, that was a hassle.
- On the client, When hiding:
  - The "eye icon" does not seem to disappear after stop hiding.
  - The client seems to be able to move even when hidden.
- When the other player is hiding, they should be way more transparent to make it clear.

PARTIALLY FIXED:

- Trying to activate gear from the client, pressing [R] or Right click was painful as it was not reponsive, or it was
  bouncy (activates then deactivates)
  - Fixes in place but still with problems
  - More fixes, and better, but... we can't get this to work properly. For now, good enough.

MIGHT BE FIXED - aditional testing required:

- Host slows down a lot
  - Fixed - probably by adding TCP_NODELAY and a few optimizations.
  - Can't reproduce. TCP_NODELAY might have been the issue. Needs re-testing.

FIXED - needs more verification:

- Impossible to control whether listening should happen in IPv4 or IPv6. We need to listen on BOTH at the same time. Do
  we have a flag to provide a list of source IP addresses for the listening part?
  - Fixed, socket listening reworked.
  - Now we see an error of address already in use, after opening IPv6 successfully and attempting IPv4.
    - This needs additional checking. It is possible that the host has some "if open in IPv4 by default open IPv6" or
      vice-versa.
- Errors for despawining on the client: See Appendix A
  - Fixed: These seem gone after the fixes.

## Dedicated Server

We should put as a first priority making a dedicated server that supports at least 1-4 players. With this testing would
be much easier since we could directly deploy in a VPS and test as a client.

## Appendix A: Logs from Client crashes

```txt
2026-02-08T17:22:06.298873Z DEBUG unmetrics_plugin::performance_report: App State: InGame - Game State: None
2026-02-08T17:22:07.136842Z DEBUG unplayer_plugin::systems::input::mouse_interaction: player_gear_usage_system: Toggling right-hand item Some(5072v3) on entity 5072v3 (authority=Client)
2026-02-08T17:22:07.152328Z DEBUG unwalkie_core::resources: WalkiePlay: GearExplanation(Thermometer) - play dice: 0/16 (threshold: 1)
2026-02-08T17:22:07.152346Z DEBUG unwalkie_plugin::triggers::tutorial_gear_explanations: Evidence gear explanation triggered for Thermometer because it's enabled.
2026-02-08T17:22:07.171657Z DEBUG unnet_plugin::systems: Spawning remote gear NetworkId(16187845477774690381) (kind: RedTorch)
2026-02-08T17:22:07.171689Z DEBUG unnet_plugin::systems: Player NetworkId(1) gear state: left=Some(4261v0), right=Some(5726v2), inv_count=0
2026-02-08T17:22:07.172393Z DEBUG unnet_plugin::systems: Spawning remote gear NetworkId(16187845477774690381) (kind: RedTorch)
2026-02-08T17:22:07.172414Z DEBUG unnet_plugin::systems: Player NetworkId(1) gear state: left=Some(4261v0), right=Some(4602v2), inv_count=0
2026-02-08T17:22:09.460445Z DEBUG unnet_plugin::systems: Spawning remote gear NetworkId(18205961787247780956) (kind: EMFMeter)
2026-02-08T17:22:10.882152Z DEBUG unnet_plugin::systems: Spawning remote gear NetworkId(5348633166218031828) (kind: Thermometer)
2026-02-08T17:22:11.298585Z DEBUG unmetrics_plugin::performance_report: fps: 63.17
2026-02-08T17:22:11.298600Z DEBUG unmetrics_plugin::performance_report: unfog/systems/animate_miasma: 0.48 ms
2026-02-08T17:22:11.298603Z DEBUG unmetrics_plugin::performance_report: unfog/systems/spawn_miasma: 0.16 ms
2026-02-08T17:22:11.298605Z DEBUG unmetrics_plugin::performance_report: unfog/systems/update_miasma: 0.13 ms
2026-02-08T17:22:11.298608Z DEBUG unmetrics_plugin::performance_report: unfps/limit_remaining: 12.29 ms
2026-02-08T17:22:11.298610Z DEBUG unmetrics_plugin::performance_report: unfps/limit_usage: 26.25 %
2026-02-08T17:22:11.298612Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting: 1.11 ms
2026-02-08T17:22:11.298615Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting_sprites: 0.33 ms
2026-02-08T17:22:11.298617Z DEBUG unmetrics_plugin::performance_report: unlight/systems/player_visibility: 0.22 ms
2026-02-08T17:22:11.298620Z DEBUG unmetrics_plugin::performance_report: unthermal/temperature_update: 0.08 ms
2026-02-08T17:22:11.298623Z DEBUG unmetrics_plugin::performance_report: systems: 14.92%
2026-02-08T17:22:11.298625Z DEBUG unmetrics_plugin::performance_report: App State: InGame - Game State: None

thread 'main' (2942148) panicked at /home/deavid/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.0/src/error/handler.rs:125:1:
Encountered an error in command `<bevy_ecs::system::commands::entity_command::insert<unbehavior::components::FloorItemCollidable>::{{closure}} as bevy_ecs::error::command_handling::CommandWithEntity<core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError>>>::with_entity::{{closure}}`: Entity despawned: The entity with ID 2856v0 is invalid; its index now has generation 1.
Note that interacting with a despawned entity is the most common cause of this error but there are others

    If you were attempting to apply a command to this entity,
    and want to handle this error gracefully, consider using `EntityCommands::queue_handled` or `queue_silenced`.

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Encountered a panic when applying buffers for system `unnet_plugin::systems::client_apply_snapshots_system`!
Encountered a panic in system `bevy_ecs::apply_deferred`!
2026-02-08T17:22:12.371443Z  WARN bevy_ecs::world::command_queue: CommandQueue has un-applied commands being dropped. Did you forget to call SystemState::apply?
Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
deavid@deavid-kws:~/git/rust/unhaunter$
```

---

```txt
2026-02-08T18:05:26.039814Z DEBUG uninteraction_plugin::systems: Interaction successful, rewriting board topology (authority=Client)
2026-02-08T18:05:26.040430Z DEBUG uninteraction_plugin::systems: Room state synchronization triggered a board topology rebuild.
2026-02-08T18:05:29.300191Z DEBUG unghost_plugin::systems::evidence_decay: Evidence clarity for UVEctoplasm: 0.1%
2026-02-08T18:05:29.351175Z DEBUG unplayer_plugin::systems::input::mouse_interaction: player_gear_usage_system: Toggling right-hand item Some(5305v1) on entity 5305v1 (authority=Client)
2026-02-08T18:05:30.136354Z DEBUG unmetrics_plugin::performance_report: fps: 64.35
2026-02-08T18:05:30.136374Z DEBUG unmetrics_plugin::performance_report: unfog/systems/animate_miasma: 0.40 ms
2026-02-08T18:05:30.136379Z DEBUG unmetrics_plugin::performance_report: unfog/systems/spawn_miasma: 0.14 ms
2026-02-08T18:05:30.136382Z DEBUG unmetrics_plugin::performance_report: unfog/systems/update_miasma: 0.11 ms
2026-02-08T18:05:30.136384Z DEBUG unmetrics_plugin::performance_report: unfps/limit_remaining: 12.89 ms
2026-02-08T18:05:30.136386Z DEBUG unmetrics_plugin::performance_report: unfps/limit_usage: 22.69 %
2026-02-08T18:05:30.136388Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting: 0.62 ms
2026-02-08T18:05:30.136391Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting_sprites: 0.26 ms
2026-02-08T18:05:30.136393Z DEBUG unmetrics_plugin::performance_report: unlight/systems/player_visibility: 0.24 ms
2026-02-08T18:05:30.136395Z DEBUG unmetrics_plugin::performance_report: unthermal/temperature_update: 0.08 ms
2026-02-08T18:05:30.136397Z DEBUG unmetrics_plugin::performance_report: systems: 11.07%
2026-02-08T18:05:30.136399Z DEBUG unmetrics_plugin::performance_report: App State: InGame - Game State: None

thread 'main' (2946877) panicked at /home/deavid/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.0/src/error/handler.rs:125:1:
Encountered an error in command `<bevy_ecs::system::commands::entity_command::insert<unbehavior::components::FloorItemCollidable>::{{closure}} as bevy_ecs::error::command_handling::CommandWithEntity<core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError>>>::with_entity::{{closure}}`: Entity despawned: The entity with ID 2853v0 is invalid; its index now has generation 1.
Note that interacting with a despawned entity is the most common cause of this error but there are others

    If you were attempting to apply a command to this entity,
    and want to handle this error gracefully, consider using `EntityCommands::queue_handled` or `queue_silenced`.

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Encountered a panic when applying buffers for system `unnet_plugin::systems::client_apply_snapshots_system`!
Encountered a panic in system `bevy_ecs::apply_deferred`!
2026-02-08T18:05:30.807825Z  WARN bevy_ecs::world::command_queue: CommandQueue has un-applied commands being dropped. Did you forget to call SystemState::apply?
Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
deavid@deavid-kws:~/git/rust/unhaunter$
```

---

```txt
2026-02-08T18:08:49.925066Z DEBUG unmetrics_plugin::performance_report: fps: 62.53
2026-02-08T18:08:49.925079Z DEBUG unmetrics_plugin::performance_report: unfog/systems/animate_miasma: 0.42 ms
2026-02-08T18:08:49.925081Z DEBUG unmetrics_plugin::performance_report: unfog/systems/spawn_miasma: 0.07 ms
2026-02-08T18:08:49.925084Z DEBUG unmetrics_plugin::performance_report: unfog/systems/update_miasma: 0.12 ms
2026-02-08T18:08:49.925085Z DEBUG unmetrics_plugin::performance_report: unfps/limit_remaining: 13.28 ms
2026-02-08T18:08:49.925087Z DEBUG unmetrics_plugin::performance_report: unfps/limit_usage: 20.29 %
2026-02-08T18:08:49.925088Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting: 0.46 ms
2026-02-08T18:08:49.925090Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting_sprites: 0.29 ms
2026-02-08T18:08:49.925091Z DEBUG unmetrics_plugin::performance_report: unthermal/temperature_update: 0.08 ms
2026-02-08T18:08:49.925092Z DEBUG unmetrics_plugin::performance_report: systems: 8.80%
2026-02-08T18:08:49.925093Z DEBUG unmetrics_plugin::performance_report: App State: InGame - Game State: None
2026-02-08T18:08:50.193471Z DEBUG unnet_plugin::systems: Player NetworkId(2) gear state: left=Some(5189v0), right=Some(4896v1), inv_count=0
2026-02-08T18:08:54.556503Z DEBUG unghost_plugin::systems::evidence_decay: Evidence clarity for FreezingTemp: 51.6%
2026-02-08T18:08:54.556520Z DEBUG unghost_plugin::systems::evidence_decay: Evidence clarity for UVEctoplasm: 28.0%

thread 'main' (2947189) panicked at /home/deavid/.cargo/registry/src/index.crates.io-1949cf8c6b5b557f/bevy_ecs-0.18.0/src/error/handler.rs:125:1:
Encountered an error in command `<bevy_ecs::system::commands::entity_command::insert<unbehavior::components::FloorItemCollidable>::{{closure}} as bevy_ecs::error::command_handling::CommandWithEntity<core::result::Result<(), bevy_ecs::world::error::EntityMutableFetchError>>>::with_entity::{{closure}}`: Entity despawned: The entity with ID 2830v0 is invalid; its index now has generation 1.
Note that interacting with a despawned entity is the most common cause of this error but there are others

    If you were attempting to apply a command to this entity,
    and want to handle this error gracefully, consider using `EntityCommands::queue_handled` or `queue_silenced`.

note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
Encountered a panic when applying buffers for system `unnet_plugin::systems::client_apply_snapshots_system`!
Encountered a panic in system `bevy_ecs::apply_deferred`!
2026-02-08T18:08:54.809763Z  WARN bevy_ecs::world::command_queue: CommandQueue has un-applied commands being dropped. Did you forget to call SystemState::apply?
Encountered a panic in system `bevy_app::main_schedule::Main::run_main`!
deavid@deavid-kws:~/git/rust/unhaunter$
```

---

## Appendix B: Host logs

```txt
2026-02-08T17:43:22.495331Z  INFO unghost_plugin::systems::ghost_ai::enrage: Hunting player for 3.9s
2026-02-08T17:43:22.517398Z DEBUG unghost_plugin::systems::ghost_ai::roar: Ghost roar: Dim - Reason: HuntingInProgress
2026-02-08T17:43:25.534150Z DEBUG unghost_plugin::systems::ghost_ai::roar: Ghost roar: Dim - Reason: HuntingInProgress
2026-02-08T17:43:26.231692Z DEBUG unmetrics_plugin::performance_report: fps: 36.74
2026-02-08T17:43:26.231738Z DEBUG unmetrics_plugin::performance_report: unboard/systems/apply_perspective: 0.12 ms
2026-02-08T17:43:26.231767Z DEBUG unmetrics_plugin::performance_report: unfog/systems/animate_miasma: 0.13 ms
2026-02-08T17:43:26.231791Z DEBUG unmetrics_plugin::performance_report: unfog/systems/spawn_miasma: 0.26 ms
2026-02-08T17:43:26.231814Z DEBUG unmetrics_plugin::performance_report: unfog/systems/update_miasma: 0.29 ms
2026-02-08T17:43:26.231839Z DEBUG unmetrics_plugin::performance_report: unfps/limit_usage: 141.38 %
2026-02-08T17:43:26.231865Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting: 7.33 ms
2026-02-08T17:43:26.231887Z DEBUG unmetrics_plugin::performance_report: unlight/systems/apply_lighting_sprites: 0.05 ms
2026-02-08T17:43:26.231910Z DEBUG unmetrics_plugin::performance_report: unlight/systems/player_visibility: 1.27 ms
2026-02-08T17:43:26.231934Z DEBUG unmetrics_plugin::performance_report: unrender/systems/sync_map_entity_field: 0.08 ms
2026-02-08T17:43:26.231957Z DEBUG unmetrics_plugin::performance_report: unsound/sound_update: 0.23 ms
2026-02-08T17:43:26.231979Z DEBUG unmetrics_plugin::performance_report: unthermal/temperature_update: 0.09 ms
2026-02-08T17:43:26.232003Z DEBUG unmetrics_plugin::performance_report: systems: 57.83%
2026-02-08T17:43:26.232023Z DEBUG unmetrics_plugin::performance_report: App State: InGame - Game State: None
2026-02-08T17:43:26.334180Z DEBUG unghost_plugin::systems::dynamic_behavior_update: Dynamics: Frz:0.91, Orbs:0.00, UV:0.78, EMF:0.40, EVP:0.00, SprtBx:0.00, RL:0.47, CPM500:0.72, Alpha:-0.07, Rage:0.65
2026-02-08T17:43:27.397967Z  INFO unghost_plugin::systems::ghost_ai::movement: Hunt finished
^B^F2026-02-08T17:43:31.233790Z DEBUG unmetrics_plugin::performance_report: fps: 40.71
2026-02-08T17:43:31.233835Z DEBUG unmetrics_plugin::performance_report: unboard/systems/apply_perspective: 0.11 ms
2026-02-08T17:43:31.233864Z DEBUG unmetrics_plugin::performance_report: unfog/systems/animate_miasma: 0.13 ms
2026-02-08T17:43:31.233887Z DEBUG unmetrics_plugin::performance_report: unfog/systems/spawn_miasma: 0.25 ms
2026-02-08T17:43:31.233910Z DEBUG unmetrics_plugin::performance_report: unfog/systems/update_miasma: 0.28 ms
2026-02-08T17:43:31.233932Z DEBUG unmetrics_plugin::performance_report: unfps/limit_usage: 130.12 %
```
