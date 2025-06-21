Migrating Bevy Audio to Bevy Kira Audio Crate
=================================================


Reasons for doing this:

* Most important one: WASM audio crackles and it's unbearable. Kira might help here.
* Kira might have support for mixing and mastering the audio better, to be able to do more interesting stuff later on.


Base Migration Guide
-----------------------

We asked Gemini AI to do a deep research onto how, in generic term, does one migrate an existing game from bevy_audio to bevy_kira_audio. The result report is this:

---

Migrating Bevy Games from bevy_audio to bevy_kira_audio

Executive Summary

This report provides a comprehensive guide for migrating existing Bevy game projects from the default bevy_audio plugin to the more feature-rich bevy_kira_audio crate. The migration process, while enhancing audio capabilities with advanced controls like channels, tweens, and improved spatial audio, necessitates careful attention to dependency management, plugin replacement, and adaptation of audio playback APIs. The need for such a detailed migration guide for a seemingly straightforward plugin swap underscores a characteristic of the Bevy ecosystem: its rapid evolution. This dynamic environment frequently introduces breaking changes, even in core functionalities or their interactions with external plugins. Consequently, developers must often navigate specific, non-obvious configuration steps, such as disabling default engine features, and manage potential naming conflicts arising from common type aliases. This report outlines the precise steps and considerations required to successfully transition a Bevy project's audio system, illustrating how to adapt to the framework's ongoing development.

Introduction: Why Migrate to bevy_kira_audio?

The Bevy game engine provides a built-in audio solution, bevy_audio, which serves as its default sound system. This plugin is based on the rodio library  and is typically included by default when

DefaultPlugins are added to a Bevy application, unless explicitly disabled.

bevy_audio offers fundamental sound support, enabling the playback of sound effects and background music. It supports common audio file formats, including

.wav, .ogg, .flac, and .mp3. For playback control,

bevy_audio utilizes the AudioSink component, which is automatically attached to entities spawned with an AudioPlayer. This

AudioSink provides methods for basic operations such as play, pause, stop, mute, set_speed, and try_seek. While it includes a rudimentary "spatial audio" implementation that pans sounds left or right in stereo based on entity transforms, and allows global spatial audio settings to be configured , it is often described by the community as "barebones," lacking more advanced features like sound synthesis, filtering nodes, or richer playback controls.

In contrast, bevy_kira_audio is a third-party plugin specifically designed to integrate the Kira audio library into the Bevy game engine. Its primary goal is to offer a more robust and feature-rich alternative to, or potential replacement for,

bevy_audio. This plugin provides a richer set of features and playback controls , supporting the same common audio formats—

.ogg, .mp3, .flac, and .wav —and additionally supports streaming of generated audio.

bevy_kira_audio offers an API to control game audio through Bevy's Entity Component System (ECS), primarily through a resource-based approach for global audio control, complemented by instance-specific and channel-specific controls.

The following table provides a side-by-side comparison of the core features and API paradigms of bevy_audio and bevy_kira_audio, highlighting the key differences and the enhanced capabilities offered by the latter. This comparison serves as a quick reference for understanding the scope of the migration and the benefits gained.

Feature Category


bevy_audio (Default)


bevy_kira_audio (Third-Party)

Underlying Library


rodio



Kira

Primary API Style


Component-based (AudioPlayer/AudioSink on entities)




Resource-based (Audio resource with fluent API) ,




AudioInstance/AudioChannel for granular control

Supported Formats


ogg, mp3, wav, flac




ogg, mp3, wav, flac

Basic Playback (Play/Pause/Stop)


Yes (via AudioSink methods)




Yes (via Audio resource, AudioInstance methods)

Looping


Yes (via PlaybackSettings::LOOP on AudioPlayer entity)




Yes (.looped(), loop_from, loop_until methods)

Volume Control


Global and per-AudioSink




Per-sound/channel (with_volume, set_volume)

Playback Speed/Pitch


Yes (set_speed)




Yes (with_playback_rate, set_playback_rate; pitch control separate)

Panning


Rudimentary (left/right pan based on transforms)




Yes (with_panning, set_panning)

Channels/Grouping


No explicit concept (managed per-entity)


Yes (AudioChannel, DynamicAudioChannels for grouping and control)

Smooth Transitions (Tweens)


Not directly supported


Yes (fade_in, fade_out with AudioTween and easings)

Spatial Audio (Level of Detail)


Basic (pan based on transforms, SpatialScale settings)




Limited (volume/panning based on emitter/receiver positions, SpatialAudioPlugin)

Web Support (Formats & Issues)


Yes




Yes (ogg/flac/wav for WASM, mp3 problematic/unsupported); Chrome interaction, Firefox distortion

Asset Loading


AssetServer




AssetServer with format features

Custom Audio Sources/Synthesis


Yes (custom sources possible)




Yes (streaming generated audio)

The API design of bevy_kira_audio, particularly its use of a builder pattern for play commands and the introduction of explicit channels, signifies a shift towards a more declarative and composable audio control paradigm. In bevy_audio, control is primarily achieved by querying for an AudioSink component attached to a specific ECS entity, meaning audio playback is tightly coupled to that entity's lifecycle. If the entity is despawned, its audio control is lost. While this is straightforward for simple, one-off sounds, managing complex audio states or groups of sounds can become cumbersome.

bevy_kira_audio addresses this by providing a global Audio resource and a fluent, chained API for playing sounds, such as audio.play(...).looped().with_volume(...). This approach allows for a declarative style of configuring a sound at the moment of playback, leading to more concise and readable code for initial sound setup. Furthermore, the introduction of explicit

AudioChannels  offers a powerful mechanism for logically grouping sounds (e.g., "music channel," "SFX channel"). This enables collective control over these groups, such as pausing all music or muting all sound effects, a capability not directly supported by

bevy_audio's per-entity AudioSink model. The AudioInstance  provides a handle for individual control of a playing sound, decoupling its management from the entity that might have triggered it. The inclusion of

AudioTween for smooth transitions  further enhances dynamic audio effects, a feature absent in

bevy_audio. This architectural shift in bevy_kira_audio moves towards a more flexible, composable, and scalable audio system, aligning more closely with modern Bevy ECS patterns by providing dedicated resources and components for audio management that can interact with, but are not strictly bound to, general game entities. This enables more sophisticated audio behaviors and a cleaner separation of concerns, leading to more maintainable and robust audio code, particularly for games with complex soundscapes.

Pre-Migration Checklist & Preparation

Before initiating any code modifications, a systematic preparation phase is essential to ensure a smooth migration. This involves verifying version compatibility, implementing source control best practices, and thoroughly assessing existing audio usage patterns.

Version Compatibility

A critical first step involves ensuring that the current version of the Bevy engine in the project is compatible with the target version of bevy_kira_audio. The bevy_kira_audio documentation provides explicit compatibility tables, indicating which versions of the plugin work with specific Bevy engine versions. For instance, Bevy 0.16 is compatible with

bevy_kira_audio 0.23.0. If the existing Bevy project is on an older version (e.g., Bevy 0.14), and the developer aims to use the latest

bevy_kira_audio (which might require Bevy 0.16), a full Bevy engine upgrade may be necessary before the audio plugin migration can proceed.

This interdependency highlights a significant characteristic of the Bevy ecosystem: its rapid development cycle. While this brings new features and innovations, it also frequently introduces breaking changes in its API. Consequently, upgrading Bevy itself can be a "tedious" and "overwhelmingly long" process, often requiring extensive refactoring across various parts of the codebase. The mention of "easy upgrade path from Bevy 0.15 to Bevy 0.16" for other crates  further suggests that not all Bevy upgrades are straightforward. Therefore, the decision to migrate to

bevy_kira_audio is not an isolated task; it is intrinsically linked to the project's current Bevy version. If the existing Bevy version is outdated, the audio migration might expand into a complex, multi-stage upgrade process—first the Bevy engine, then bevy_kira_audio and potentially other plugins—significantly increasing the scope and risk of the undertaking. This emphasizes the critical importance of strategic planning and maintaining awareness of version compatibility across the entire dependency graph to avoid unforeseen complexities and extensive refactoring efforts.

Source Control Best Practices

Prior to making any changes to the codebase, it is imperative to create a new Git branch specifically for the migration effort. A descriptive branch name, such as upgrade-bevy-audio, is recommended. This practice is fundamental for robust software development, as it isolates the migration changes, allows for easy tracking of all modifications, and provides a clear point to revert to if any issues or unexpected regressions arise during or after the migration.

Identifying Existing bevy_audio Usage Patterns

A thorough review of the current codebase is necessary to identify all instances where bevy_audio is utilized. This includes:

    Locating where bevy::audio::AudioPlugin is added to the App builder in the main function or other plugin registration points.

    Identifying how audio assets are loaded, typically through asset_server.load("path/to/audio.ogg").

    Understanding how sounds are played, specifically looking for patterns such as commands.spawn(AudioBundle {... }) or commands.spawn(AudioPlayer::new(...)).

Analyzing how playback is controlled, which usually involves querying for AudioSink components and invoking their methods like play, pause, stop, set_speed, or set_volume.

Reviewing any custom or rudimentary spatial audio implementations that rely on bevy_audio's capabilities.

This initial assessment will provide a clear understanding of the scope of API changes required and help anticipate potential refactoring needs during the migration.

Step-by-Step Migration Guide

The migration from bevy_audio to bevy_kira_audio involves several distinct steps, starting from dependency configuration and progressing through plugin integration and API adaptation.

1. Dependency Management in Cargo.toml

The first and most critical step in the migration process involves modifying the project's Cargo.toml file to correctly configure dependencies.

Disabling default-features for bevy and excluding bevy_audio and vorbis

bevy_audio is enabled by default in Bevy's DefaultPlugins and directly conflicts with bevy_kira_audio. Similarly, Bevy's

vorbis feature must also be excluded. To resolve this, the

bevy dependency in Cargo.toml must be updated to explicitly disable default features and then selectively enable only the features required by the project, excluding bevy_audio and vorbis.

Consider the following example Cargo.toml modification for Bevy 0.16:
Ini, TOML

[dependencies]
bevy = { version = "0.16", default-features = false, features = }

The exact list of features will depend on the project's specific requirements. It is advisable to consult Bevy's Cargo.toml file for the target version to identify all default features and selectively enable those that are genuinely needed.

Adding bevy_kira_audio with desired format features

After configuring the bevy dependency, bevy_kira_audio must be added to Cargo.toml. It is important to enable the features corresponding to the audio file formats the game utilizes (e.g., mp3, wav, flac). The ogg feature is typically enabled by default for bevy_kira_audio.

An example Cargo.toml addition:
Ini, TOML

bevy_kira_audio = { version = "0.23", features = ["mp3", "wav"] }

It is crucial to ensure that the selected bevy_kira_audio version (e.g., 0.23) is compatible with the chosen Bevy version (e.g., 0.16), as indicated by the compatibility table in the bevy_kira_audio documentation.

Running cargo check to update Cargo.lock

After making these modifications to Cargo.toml, execute cargo check in the terminal. This command will resolve the updated dependencies, download any newly specified crates, and generate a new Cargo.lock file. This Cargo.lock file contains the precise versions of all project dependencies, ensuring consistency and correct configuration of the dependency tree before proceeding with code changes.

2. Plugin Integration

Once the Cargo.toml is correctly configured, the next step involves integrating bevy_kira_audio into the Bevy application.

Replacing bevy::audio::AudioPlugin with bevy_kira_audio::AudioPlugin

The primary integration point is the application's main function (typically in main.rs), where plugins are added to the App builder. The existing bevy::audio::AudioPlugin must be removed and bevy_kira_audio::AudioPlugin added in its place. Since bevy_kira_audio::prelude::* brings AudioPlugin into scope, this replacement is generally straightforward.

Before (using bevy_audio):
Rust

use bevy::prelude::*;
// Potentially: use bevy::audio::AudioPlugin; // Explicit import if not using DefaultPlugins
fn main() {
    App::new()
       .add_plugins(DefaultPlugins)
       .add_plugins(AudioPlugin) // This refers to bevy::audio::AudioPlugin
       .run();
}

After (using bevy_kira_audio):
Rust

use bevy::prelude::*;
use bevy_kira_audio::prelude::*; // This brings bevy_kira_audio::AudioPlugin into scope
fn main() {
    App::new()
       .add_plugins(DefaultPlugins)
       .add_plugins(AudioPlugin) // This now refers to bevy_kira_audio::AudioPlugin
       .run();
}

Resolving Audio struct name conflicts

A common issue encountered during this migration is a name conflict for the Audio type alias. Both bevy::prelude and bevy_kira_audio::prelude export a type alias named Audio. This can lead to ambiguity errors during compilation, as the compiler cannot determine which

Audio type is intended.

To resolve this, it is necessary to explicitly import Audio from bevy_kira_audio in any system or module where the Audio resource is used for playback.

Example of explicit import:
Rust

use bevy::prelude::*;
use bevy_kira_audio::Audio; // Explicitly imports bevy_kira_audio's Audio resource

// If other items from bevy_kira_audio's prelude are needed, both imports can be used:
// use bevy_kira_audio::prelude::*;
// use bevy_kira_audio::Audio; // This ensures clarity and resolves potential conflicts.

This Audio type name conflict is a subtle yet common issue in ecosystems that rely heavily on prelude modules, such as Bevy. preludes are designed to simplify common imports by automatically bringing frequently used items into scope with a single use statement. However, when multiple crates, like Bevy itself and bevy_kira_audio, export types with identical common names within their respective preludes, it creates ambiguity for the compiler. This forces developers to bypass the convenience of the prelude for that specific type and instead use a fully qualified path or an explicit, specific use statement. While a minor inconvenience, this situation represents a trade-off in API design: the ease of use provided by preludes can sometimes lead to naming collisions in a modular environment where many independent crates are integrated. This pattern suggests that developers should anticipate similar naming conflicts when integrating other third-party plugins into Bevy projects. Prioritizing explicit use statements or fully qualified paths for potentially ambiguous types is a good practice to maintain code clarity, prevent unexpected behavior, and ensure robust compilation, particularly as the Bevy ecosystem continues to grow and mature.

3. Asset Loading and Basic Playback

The process for loading audio assets remains largely consistent, while basic playback patterns will require adaptation to the new API.

Loading audio assets via AssetServer

Both bevy_audio and bevy_kira_audio utilize Bevy's standard AssetServer resource for loading audio files from the project's assets directory. This means the method for loading audio asset handles will generally remain unchanged.

Example:
Rust

use bevy::prelude::*;
//...
fn load_my_audio(asset_server: Res<AssetServer>) {
    let background_music_handle = asset_server.load("audio/background.ogg");
    let sound_effect_handle = asset_server.load("audio/explosion.wav");
    // These handles can then be stored in resources or components as required by the game logic.
}

Migrating simple audio.play() calls for one-off sounds

In bevy_audio, playing a one-off sound typically involves spawning a new entity with an AudioBundle or AudioPlayer component. The sound plays, and the entity might then be despawned or its component removed.

With bevy_kira_audio, the Audio resource (which is an alias for the default audio channel ) and its

play method are used. This method directly accepts an AssetPath or Handle<AudioSource> and immediately initiates playback.

Before (bevy_audio - simplified example):
Rust

use bevy::prelude::*;
// Assuming AudioPlayer and AudioBundle are in scope
fn play_sound_effect_old(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let sound_handle = asset_server.load("sounds/effect.ogg");
    commands.spawn(bevy::audio::AudioBundle {
        source: sound_handle,
        settings: bevy::audio::PlaybackSettings::ONCE, // Play once
    });
}

After (bevy_kira_audio):
Rust

use bevy::prelude::*;
use bevy_kira_audio::Audio; // Ensure this is bevy_kira_audio::Audio
fn play_sound_effect_new(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    audio.play(asset_server.load("sounds/effect.ogg"));
}

Implementing looped background audio

In bevy_audio, looped background music is achieved by setting PlaybackSettings::LOOP on an AudioPlayer component. The music continues to loop as long as the associated entity exists.

bevy_kira_audio provides a convenient .looped() method that can be chained directly onto the play command. It also offers more granular control with loop_from(position) and loop_until(position) for defining specific loop points within an audio file.

Before (bevy_audio - simplified example):
Rust

use bevy::prelude::*;
// Assuming AudioPlayer and AudioBundle are in scope
#[derive(Component)]
struct MusicBox; // Marker component for background music entity
fn start_background_music_old(
    asset_server: Res<AssetServer>,
    mut commands: Commands,
) {
    let music_handle = asset_server.load("music/background.ogg");
    commands.spawn((
        bevy::audio::AudioBundle {
            source: music_handle,
            settings: bevy::audio::PlaybackSettings::LOOP,
        },
        MusicBox, // Tag the entity for later control/despawn
    ));
}

After (bevy_kira_audio):
Rust

use bevy::prelude::*;
use bevy_kira_audio::Audio; // Ensure this is bevy_kira_audio::Audio
fn start_background_music_new(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
) {
    audio.play(asset_server.load("music/background.ogg")).looped();
    // For more advanced looping, e.g., skipping an intro:
    // audio.play(asset_server.load("music/background_with_intro.ogg")).loop_from(3.5);
}

4. Advanced Playback Control

The approach to advanced playback control undergoes a significant shift, moving from entity-bound AudioSink components to a more flexible system involving the Audio resource, AudioInstance handles, and AudioChannels.

The following table provides a direct mapping of common bevy_audio AudioSink functionalities to their bevy_kira_audio equivalents. This serves as a practical reference for refactoring code that manages audio playback.

Functionality


bevy_audio (AudioSink on AudioPlayer entity)


bevy_kira_audio (Audio resource, AudioInstance, AudioChannel)

Play/Resume


sink.play()




audio.play(...).paused(false) (on start) or instance.resume()

Pause


sink.pause()




audio.play(...).paused(true) (on start) or instance.pause()

Stop


sink.stop() (permanently stops, cannot restart)




instance.stop()

Mute


sink.mute()




instance.set_volume(Volume::ZERO) or channel.set_volume(Volume::ZERO)

Unmute


sink.unmute()




instance.set_volume(Volume::new(1.0)) or channel.set_volume(Volume::new(1.0))

Toggle Playback


sink.toggle_playback()




instance.toggle_playback() (requires AudioInstance handle)

Toggle Mute


sink.toggle_mute()




Not a direct method; manage volume

Get Playback Speed


sink.speed()




instance.playback_rate()

Set Playback Speed


sink.set_speed(rate)




audio.play(...).with_playback_rate(rate) (on start) or instance.set_playback_rate(rate)

Get Playback Status


sink.is_paused(), sink.is_muted()




instance.state() (returns PlaybackState enum)

Seek


sink.try_seek(position)




instance.seek_to(position)

Volume Control


sink.set_volume(volume)




audio.play(...).with_volume(volume) (on start) or instance.set_volume(volume) or channel.set_volume(volume)

Panning


Rudimentary spatial audio (via SpatialScale settings)




audio.play(...).with_panning(pan) (on start) or instance.set_panning(pan)

Looping


PlaybackSettings::LOOP on AudioPlayer entity




audio.play(...).looped(), loop_from(pos), loop_until(pos)

Smooth Transitions


Not directly supported


fade_in(), fade_out() with AudioTween and AudioEasing

Channels


No explicit concept, managed per-entity


AudioChannel, DynamicAudioChannels for grouping and control

The shift from AudioSink on an entity to a resource-based Audio with chained methods and explicit AudioInstance/AudioChannel control represents a move towards a more functional and composable API design in bevy_kira_audio. The bevy_audio model tightly couples audio playback and its control to the lifecycle of a specific ECS entity. This can be limiting; for example, if an entity is despawned, its AudioSink is gone, and any associated audio control is lost. Managing global or grouped audio requires custom logic to query and iterate over multiple AudioSink components.

bevy_kira_audio's audio.play() method returns a PlayAudioCommand that supports a fluent, builder-pattern API. This allows developers to declaratively define all initial settings—such as looping, volume, panning, and fades—in a single, readable line of code, promoting conciseness and reducing boilerplate. The introduction of explicit

AudioChannels  enables logical grouping of sounds (e.g., all music, all UI effects). This allows for collective control, such as pausing all music with a single command, without needing to manage individual entities.

AudioInstance  provides a handle for fine-grained control over a specific playing sound

after it has started, decoupling its control from the original entity that initiated playback. The inclusion of AudioTween  for smooth transitions (fade-in, fade-out) is a significant feature enhancement, moving beyond abrupt changes and enabling more professional audio effects. This design represents a more mature and flexible audio system, decoupling audio playback and control from direct entity ownership, offering a more powerful and scalable system. The builder pattern simplifies initial setup, while

AudioInstance and AudioChannel provide structured and efficient control over individual sounds or groups, respectively. This enables more sophisticated and dynamic audio behaviors that are difficult or impossible to achieve with bevy_audio's more basic AudioSink model, leading to cleaner and more maintainable audio code, especially for games with rich and interactive soundscapes.

Controlling volume, playback rate, and panning

These audio properties are typically configured using chained methods on the PlayAudioCommand when a sound is initiated, or by directly interacting with an AudioInstance or AudioChannel for ongoing control.

Example:
Rust

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;
use std::time::Duration;
fn configure_sound(asset_server: Res<AssetServer>, audio: Res<Audio>) {
    audio.play(asset_server.load("sounds/wind.ogg"))
       .with_volume(0.7) // Play at 70% volume
       .with_playback_rate(1.1) // Slightly faster playback
       .with_panning(-0.5) // Pan slightly to the left
       .fade_in(AudioTween::new(Duration::from_secs(1), AudioEasing::Linear)); // Fade in over 1 second
}

Utilizing AudioChannel for grouped sound control

bevy_kira_audio introduces the powerful concept of AudioChannels, which allows grouping multiple sounds together and applying controls (e.g., pause, stop, volume changes) to the entire group simultaneously. This is particularly useful for managing distinct sound categories such as background music, sound effects, or UI sounds. Custom channels can be defined by implementing the

AudioChannel trait and adding them to the App.

Example (conceptual, referencing custom_channel example in bevy_kira_audio repository ):

Rust

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

// Define a custom channel type
struct MusicChannel;
impl AudioChannel for MusicChannel {} // Marker trait

fn main() {
    App::new()
       .add_plugins((DefaultPlugins, AudioPlugin))
       .add_audio_channel::<MusicChannel>() // Add the custom channel
       .add_systems(Startup, start_music_on_channel)
       .add_systems(Update, control_music_channel)
       .run();
}

fn start_music_on_channel(asset_server: Res<AssetServer>, audio: Res<AudioChannel<MusicChannel>>) {
    audio.play(asset_server.load("music/game_theme.ogg")).looped();
}

fn control_music_channel(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    music_channel: Res<AudioChannel<MusicChannel>>,
) {
    if keyboard_input.just_pressed(KeyCode::M) {
        music_channel.toggle_playback(); // Pause/resume all sounds in MusicChannel
    }
}

Leveraging AudioInstance for individual sound control

For precise, real-time control over a single playing sound, bevy_kira_audio provides AudioInstance. When a sound is played, a

Handle<AudioInstance> can be obtained, which allows subsequent queries and modifications of that specific sound instance's properties.

Example (conceptual, referencing instance_control example in bevy_kira_audio repository ):

Rust

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

#
struct MySoundInstanceHandle(Handle<AudioInstance>);

fn spawn_and_get_instance_handle(
    asset_server: Res<AssetServer>,
    audio: Res<Audio>,
    mut commands: Commands,
) {
    let instance_handle = audio.play(asset_server.load("sounds/ambient.ogg")).looped().handle();
    commands.insert_resource(MySoundInstanceHandle(instance_handle));
}

fn control_specific_instance(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    my_instance_handle: Res<MySoundInstanceHandle>,
    mut audio_instances: ResMut<Assets<AudioInstance>>,
) {
    if keyboard_input.just_pressed(KeyCode::P) {
        if let Some(instance) = audio_instances.get_mut(&my_instance_handle.0) {
            instance.toggle_playback(); // Pause/resume this specific sound instance
        }
    }
}

Spatial Audio Considerations

Both bevy_audio and bevy_kira_audio offer forms of spatial audio, but with differing levels of sophistication.

bevy_audio includes a rudimentary spatial audio implementation that primarily pans sounds left or right in stereo based on the transforms of entities. Global spatial audio settings can be configured by overriding the

AudioPlugin settings.

bevy_kira_audio provides "limited spatial audio support". Currently, its capabilities are restricted to automatically changing the volume and panning of audio based on the positions of designated emitters and receivers. This functionality is enabled by adding the

SpatialAudioPlugin and attaching SpatialAudioEmitter and SpatialAudioReceiver components to entities. While

bevy_kira_audio's approach is more structured than bevy_audio's, it is noted that other third-party alternatives like bevy_oddio offer more advanced 3D spatial sound capabilities.

Conceptual Example for bevy_kira_audio Spatial Audio:
Rust

use bevy::prelude::*;
use bevy_kira_audio::prelude::*;

fn main() {
    App::new()
       .add_plugins((DefaultPlugins, AudioPlugin, SpatialAudioPlugin)) // Add SpatialAudioPlugin
       .add_systems(Startup, setup_spatial_audio)
       .run();
}

fn setup_spatial_audio(mut commands: Commands, asset_server: Res<AssetServer>, audio: Res<Audio>) {
    // Spawn an entity that emits sound
    commands.spawn((
        SpatialAudioEmitter, // Marker for an audio emitter
        Transform::from_xyz(5.0, 0.0, 0.0), // Position of the sound source
        // Play a sound that will be affected by spatial audio
        audio.play(asset_server.load("sounds/engine_hum.ogg")).looped().handle(),
    ));

    // Spawn an entity that receives sound (the listener)
    commands.spawn((
        SpatialAudioReceiver, // Marker for the audio listener
        Transform::from_xyz(0.0, 0.0, 0.0), // Position of the listener (e.g., camera)
    ));
}

Web Build Specifics

When migrating a Bevy game to use bevy_kira_audio for web builds (WASM), specific considerations regarding audio formats and browser behavior are necessary.

bevy_kira_audio generally supports web builds for .ogg, .flac, and .wav formats. However,

.mp3 support can be problematic or unsupported in web environments.

There are also known differences in how various browsers handle audio:

    Chrome: An interaction with the website (e.g., a button click) is typically required before the AudioContext is started and audio can play.

bevy_kira_audio documentation suggests that this issue can be resolved with a script in the index.html file.

Firefox: Audio might sound distorted. This issue could be related to overall performance or specific browser implementations.

Developers should thoroughly test audio functionality in target web browsers to ensure consistent behavior and quality.

Troubleshooting and Best Practices

Migrating an audio system can introduce various challenges. Adopting best practices can streamline the process and minimize issues.

Compilation Errors

    Dependency Conflicts: The most common compilation error will stem from not correctly disabling bevy_audio and vorbis features in the bevy dependency in Cargo.toml. Ensure default-features = false is set for bevy, and only necessary features are explicitly listed, excluding the conflicting ones.

Audio Type Ambiguity: As discussed, the Audio type alias exists in both bevy::prelude and bevy_kira_audio::prelude. This will result in compilation errors indicating ambiguity. The resolution involves explicitly importing bevy_kira_audio::Audio in any module where the Audio resource is used.

Runtime Issues

    No Sound: If no sound plays, verify that bevy_kira_audio's format features (e.g., mp3, wav) are correctly enabled in Cargo.toml for the audio files being used. For web builds, remember Chrome's requirement for user interaction before audio playback can begin.

Distorted Audio: On Firefox, distorted audio has been reported. This may require further investigation into browser-specific audio settings or potential performance bottlenecks in the game.

General Migration Tips

    Incremental Changes: Instead of attempting a full migration at once, apply changes incrementally. Migrate one audio playback pattern or one type of sound (e.g., background music first, then sound effects) to simplify debugging.

    Frequent Compilation and Testing: Compile and run the project frequently after each set of changes to catch errors early. This iterative approach helps pinpoint the source of any new issues.

    Leverage ast-grep for Semi-Automated Refactoring: For projects with extensive bevy_audio usage, tools like ast-grep can assist in semi-automating API migration.

ast-grep allows searching and replacing code patterns based on abstract syntax trees, which is more robust than simple text-based find-and-replace for complex API changes. This tool can significantly reduce manual effort for repetitive refactoring tasks, though manual verification of changes remains essential. The overall process involves maintaining a clean Git branch, updating dependencies, compiling, rewriting code, verifying, and formatting, repeating until the project compiles and tests pass.

Conclusion and Recommendations

The migration from Bevy's default bevy_audio plugin to bevy_kira_audio represents a significant upgrade in a game project's audio capabilities. bevy_kira_audio, powered by the Kira library, offers a richer set of features, including advanced playback controls, explicit audio channels for sound grouping, smooth transitions with tweens, and more structured spatial audio management. This transition moves the project from a basic, entity-bound audio control system to a more flexible, composable, and scalable architecture, aligning with modern Bevy ECS patterns and enabling more sophisticated audio experiences.

The process, while beneficial, demands a methodical approach. It is paramount to:

    Prioritize Dependency Management: Correctly configuring Cargo.toml by disabling Bevy's default-features and explicitly listing necessary features while excluding bevy_audio and vorbis is the most critical step for compatibility. Failure to do so will result in immediate conflicts.

    Verify Version Compatibility: Always consult the bevy_kira_audio documentation to ensure the chosen plugin version is compatible with the project's Bevy engine version. An outdated Bevy version might necessitate a full engine upgrade, expanding the scope of the migration considerably due to Bevy's rapid development and frequent breaking changes.

    Address API Paradigm Shifts: Understand that bevy_kira_audio shifts from AudioSink components on entities to a resource-based Audio system with chained methods, AudioInstance handles for individual control, and AudioChannels for grouped management. This new paradigm offers greater flexibility and control over soundscapes.

    Anticipate and Resolve Conflicts: Be prepared for common issues like the Audio type alias conflict, which requires explicit import statements. This highlights a broader need for vigilance when integrating multiple plugins in a modular ecosystem.

    Adopt Best Practices: Utilize source control (dedicated branches), perform incremental changes, and conduct frequent compilation and testing. For larger codebases, consider leveraging tools like ast-grep for semi-automated refactoring to streamline the process.

By meticulously following these steps and understanding the underlying architectural shifts, developers can successfully migrate their Bevy games to bevy_kira_audio, unlocking enhanced audio fidelity and more dynamic sound design possibilities.

----


Specific migration guide
---------------------------

I asked AI Gemini 2.5 Pro to read all Unhaunter's source code and the above migration guide and come up with a migration guide specific for Unhaunter:

---

### **Migration Plan for Unhaunter: `bevy_audio` to `bevy_kira_audio`**

#### **Executive Summary**

This plan outlines the specific steps to migrate your Bevy 0.16 project, Unhaunter, from the built-in `bevy_audio` to `bevy_kira_audio`. Your existing architecture, which uses a custom `SoundEvent` for sound effects, is a major advantage. Our primary focus will be replacing the system that currently handles these events (`ungear/src/systems.rs`) and refactoring the title music system (`unmenu/src/mainmenu.rs`) to use the new `bevy_kira_audio` paradigms.

The migration will involve:
1.  **Verifying `Cargo.toml`:** Confirming your dependency setup is correct.
2.  **Plugin Integration:** Adding the `bevy_kira_audio` plugins and creating dedicated audio channels for Music and Sound Effects in `unhaunter/src/app.rs`.
3.  **Event Handler Refactor:** Replacing the `sound_playback_system` with a new version that uses the `Audio` resource and `SpatialAudioPlugin` to handle your `SoundEvent`.
4.  **Music System Refactor:** Updating the `manage_title_song` system to use the new `MusicChannel` for robust control.
5.  **Cleanup:** Removing the now-obsolete `GearStuff` audio spawning logic.

---

### **Step 1: `Cargo.toml` Verification (You're 95% there!)**

Your `Cargo.toml` is already correctly configured to disable `bevy_audio` and include `bevy_kira_audio`. This is the most critical step, and you've done it perfectly.

For completeness, let's confirm the setup:

```toml
# Cargo.toml

# [workspace.dependencies] section
bevy = { version = "0.16", default-features = false, features = [
    "jpeg", "serialize", "animation", "async_executor", "bevy_asset",
    # "bevy_audio", # CORRECTLY DISABLED
    "bevy_color", "bevy_core_pipeline", "bevy_gilrs", "bevy_input_focus",
    "bevy_log", "bevy_picking", "bevy_render", "bevy_scene", "bevy_sprite",
    "bevy_sprite_picking_backend", "bevy_state", "bevy_text", "bevy_ui",
    "bevy_ui_picking_backend", "bevy_window", "bevy_winit", "custom_cursor",
    "default_font", "multi_threaded", "png", "std", "sysinfo_plugin",
    # "vorbis", # CORRECTLY DISABLED
    "webgl2", "x11",
], }

# [dependencies] section in root Cargo.toml
# Note: Since all your sounds are .ogg, the default features are sufficient.
# If you add other formats, you'll enable them here.
bevy_kira_audio = "0.23.0"
```

**Action:** No changes needed here unless you plan to add other audio formats like `.mp3` or `.wav`.

### **Step 2: Plugin and Audio Channel Setup (`unhaunter/src/app.rs`)**

We need to add the `bevy_kira_audio` plugins and define our audio channels. Using channels is a best practice that gives you easy global control over sound groups (e.g., "mute all SFX").

1.  **Define Audio Channels:** Create dedicated structs for your sound categories. A good place for this might be a new `unhaunter/src/audio.rs` file, or directly in `unhaunter/src/app.rs` for simplicity.

    ```rust
    // unhaunter/src/app.rs or a new audio module

    use bevy_kira_audio::prelude::*;

    // Define a channel for the main menu and in-game music
    pub struct MusicChannel;
    impl AudioChannel for MusicChannel {}

    // Define a channel for general sound effects
    pub struct SfxChannel;
    impl AudioChannel for SfxChannel {}
    ```

2.  **Update Your App Setup:** Modify `unhaunter/src/app.rs` to add the plugins.

    ```rust
    // unhaunter/src/app.rs

    // ... other imports
    use bevy_kira_audio::prelude::*; // Add this
    // Bring your channels into scope if they are in another file
    // use crate::audio::{MusicChannel, SfxChannel};

    pub fn app_run(cli_options: CliOptions) {
        let mut app = App::new();
        // ...
        // Your existing app.add_plugins(...)
        // ...

        // --- NEW AUDIO SETUP ---
        app.add_plugins(AudioPlugin) // This is bevy_kira_audio::AudioPlugin
            .add_plugins(SpatialAudioPlugin) // For positional audio
            .add_audio_channel::<MusicChannel>()
            .add_audio_channel::<SfxChannel>();
        // --- END NEW AUDIO SETUP ---

        // ... rest of your setup ...
        app.run();
    }
    ```

### **Step 3: Refactor the `SoundEvent` Handler (`ungear/src/systems.rs`)**

This is the most important code change. Your current `sound_playback_system` spawns entities with `AudioPlayer`. We will replace this with a new system that uses the `bevy_kira_audio` resource and handles spatial audio.

**File:** `ungear/src/systems.rs`

**Current Code (`sound_playback_system`):**
```rust
// THIS IS THE OLD CODE TO BE REPLACED
fn sound_playback_system(
    mut sound_events: EventReader<SoundEvent>,
    asset_server: Res<AssetServer>,
    gc: Res<GameConfig>,
    qp: Query<(Entity, &Position, &PlayerSprite)>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    // ... your existing implementation that uses commands.spawn(AudioPlayer) ...
}
```

**New Code (Replace the entire system with this):**
```rust
// THIS IS THE NEW CODE
use bevy_kira_audio::prelude::*;
use uncore::components::board::position::Position; // ensure this is in scope

fn sound_playback_system(
    mut sound_events: EventReader<SoundEvent>,
    // Use the SfxChannel for sound effects.
    // The main `Audio` resource is an alias for the default channel.
    // Being explicit with channels is better.
    sfx: Res<AudioChannel<SfxChannel>>,
    gc: Res<GameConfig>,
    qp: Query<(&Position, &PlayerSprite)>,
    mut commands: Commands,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    let Some((player_position, _)) =
        qp.iter().find(|( _, p)| p.id == gc.player_id) else { return; };

    for sound_event in sound_events.read() {
        if sound_event.sound_file.is_empty() {
            warn!("Attempted to play a sound with an empty file path. Ignoring.");
            continue;
        }

        let sound_handle = asset_server.load(sound_event.sound_file.clone());

        if let Some(position) = sound_event.position {
            // This is a spatial sound.
            // We spawn a temporary entity for the sound emitter.
            // Bevy_kira_audio will automatically despawn it when the sound finishes.
            let emitter = commands.spawn((
                SpatialAudioEmitter,
                // Make the emitter's transform global, not tied to a parent
                GlobalTransform::from_translation(position.to_screen_coord()),
            )).id();

            sfx.play(sound_handle)
               .with_volume(sound_event.volume as f64)
               .with_settings(PlaybackSettings {
                    // Tell the sound to follow the emitter entity.
                    emitter: Some(emitter),
                    // Set panning based on the listener's position.
                    panning: Panning::ListenerRelative,
                    ..default()
               });
        } else {
            // This is a non-spatial (UI) sound.
            sfx.play(sound_handle)
               .with_volume(sound_event.volume as f64);
        }
    }
}
```

**Action:**
1.  In `ungear/src/systems.rs`, replace your entire `sound_playback_system` with the new version above.
2.  Add `use bevy_kira_audio::prelude::*;` to the top of the file.
3.  Also in `ungear/src/systems.rs`, find your `app_setup` function and ensure the `SpatialListener` is added to the player entity. If it's already there from your previous attempt, that's great. If not, add it in `unmapload/src/entity_spawning.rs` inside the `spawn_player` function:

    ```rust
    // unmapload/src/entity_spawning.rs -> spawn_player function

    //...
    .insert(PlayerSprite::new(1, player_position).with_controls(**p.control_settings))
    // THIS IS THE IMPORTANT PART - If your old code had it, confirm it's still there.
    .insert(SpatialListener::new(p.audio_settings.sound_output.to_ear_offset()))
    //...
    ```

### **Step 4: Refactor the Music System (`unmenu/src/mainmenu.rs`)**

Your current music system manages an `AudioSink` on an entity. We will change this to use the `MusicChannel` we defined, which is a much cleaner and more robust way to handle background music.

**File:** `unmenu/src/mainmenu.rs`

**Current Code (`manage_title_song` and `despawn_sound`):**
```rust
// THIS IS THE OLD CODE TO BE REPLACED
pub fn manage_title_song(
    // ...
) {
    // ... logic that spawns an entity with AudioPlayer ...
}

pub fn despawn_sound(
    // ...
    mut qs: Query<(Entity, &mut AudioSink, &MenuSound)>,
    // ...
) {
    for (entity, mut sink, menusound) in &mut qs {
        // ... logic that manipulates sink.volume() and despawns ...
    }
}
```

**New Code (Replace both systems with this single, simpler one):**
```rust
// THIS IS THE NEW CODE
use bevy_kira_audio::prelude::*;
use crate::mainmenu::MusicChannel; // assuming you put the channels in mainmenu.rs or a shared module

#[derive(Component, Debug, Default)]
pub struct MenuSound {
    // Keep this to track if the song should be playing
    is_playing: bool
}

// A single system to manage the title song
pub fn manage_title_song(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    // Query for the music channel resource
    music_channel: Res<AudioChannel<MusicChannel>>,
    // We still need a way to know if we've started the song.
    // A simple query for our marker component works well.
    q_sound: Query<Entity, With<MenuSound>>,
    app_state: Res<State<AppState>>,
    audio_settings: Res<Persistent<AudioSettings>>,
) {
    let should_play_song = !matches!(app_state.get(), AppState::InGame);
    let is_already_playing = !q_sound.is_empty();

    let master_vol = audio_settings.volume_master.as_f32() as f64;
    let music_vol = audio_settings.volume_music.as_f32() as f64;
    music_channel.set_volume(master_vol * music_vol);

    if should_play_song && !is_already_playing {
        music_channel.play(asset_server.load("music/unhaunter_intro.ogg")).looped();
        // Spawn a marker entity so we know the music has been started.
        commands.spawn(MenuSound { is_playing: true });
    } else if !should_play_song && is_already_playing {
        music_channel.stop();
        // Despawn the marker entity.
        for entity in &q_sound {
            commands.entity(entity).despawn();
        }
    }
}

// In app_setup, replace both manage_title_song and despawn_sound with the new system
// pub fn app_setup(app: &mut App) {
//     app.add_systems(OnEnter(AppState::MainMenu), (setup, setup_ui))
//        .add_systems(OnExit(AppState::MainMenu), cleanup)
//        .add_systems(Update, menu_event)
//        // .add_systems(Update, despawn_sound) // REMOVE THIS
//        .add_systems(Update, manage_title_song); // KEEP/ADD THIS
// }
```

**Action:**
1.  In `unmenu/src/mainmenu.rs`, replace both `manage_title_song` and `despawn_sound` systems with the new `manage_title_song` system.
2.  Update the `MenuSound` component to the new, simpler version.
3.  In the `app_setup` function within that file, remove the call to `despawn_sound`.

### **Step 5: Clean up `GearStuff` (`ungear/src/gear_stuff.rs`)**

Your `GearStuff` SystemParam has `play_audio` and `play_audio_nopos` methods that spawn audio entities. Since all sound playback is now handled by the `sound_playback_system` via `SoundEvent`, these methods are now incorrect and can be simplified. They should *only* write the event.

**File:** `ungear/src/gear_stuff.rs`

**Current `play_audio` and `play_audio_nopos` methods:**
```rust
// OLD CODE
pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
    // ... lots of code spawning AudioPlayer ...
}
pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
    // ... lots of code spawning AudioPlayer ...
}
```

**New simplified `play_audio` and `play_audio_nopos` methods:**
```rust
// NEW CODE
pub fn play_audio(&mut self, sound_file: String, volume: f32, position: &Position) {
    if sound_file.is_empty() {
        warn!("Attempted to play a sound with an empty file path. Ignoring.");
        return;
    }
    self.sound_events.write(SoundEvent {
        sound_file,
        volume,
        position: Some(*position),
    });
}

pub fn play_audio_nopos(&mut self, sound_file: String, volume: f32) {
    if sound_file.is_empty() {
        warn!("Attempted to play a sound with an empty file path. Ignoring.");
        return;
    }
    self.sound_events.write(SoundEvent {
        sound_file,
        volume,
        position: None,
    });
}
```

**Action:** Replace the bodies of the two `play_audio` methods in `ungear/src/gear_stuff.rs`. You can also now remove `use bevy::audio::SpatialScale;` and other `bevy::audio` imports from that file.

### **Final Step: Compile and Fix Minor Issues**

After making these changes, run `cargo check` or `cargo run`. You will likely encounter one common compiler error:

*   **`Audio` is ambiguous:** As the generic guide predicted, both Bevy and `bevy_kira_audio` export an `Audio` type. The compiler will complain.
    *   **Fix:** In every file where you use the audio resource (like your new `sound_playback_system`), add an explicit import: `use bevy_kira_audio::Audio;`. If you are also using the prelude, that's fine, just add this line below it to resolve the ambiguity.


---


