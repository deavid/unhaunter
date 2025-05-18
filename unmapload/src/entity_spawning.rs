//! # Entity Spawning Module
//!
//! This module is responsible for spawning various game entities like players, ghosts, and ambient sounds.
//! It handles the creation of these entities with all their required components and initial configuration.

use bevy::prelude::*;
use bevy::sprite::Anchor;
use ordered_float::OrderedFloat;
use rand::seq::SliceRandom;
use uncore::components::animation::{AnimationTimer, CharacterAnimation};
use uncore::components::board::direction::Direction;
use uncore::components::board::position::Position;
use uncore::components::focus_ring::FocusRing;
use uncore::components::game::GameSound;
use uncore::components::game::GameSprite;
use uncore::components::ghost_breach::GhostBreach;
use uncore::components::ghost_sprite::GhostSprite;
use uncore::components::player::Stamina;
use uncore::components::player_sprite::PlayerSprite;
use uncore::components::sprite_type::SpriteType;
use uncore::random_seed;
use uncore::resources::summary_data::SummaryData;
use uncore::types::game::SoundType;
use ungear::components::playergear::PlayerGear;
use ungearitems::from_gearkind::FromPlayerGearKind as _;

use crate::level_setup::LoadLevelSystemParam;

/// Spawns the player entity with all needed components.
///
/// This function handles:
/// - Selecting a spawn point from the available options
/// - Creating the player entity with appropriate appearance and components
/// - Setting up player controls and equipment
/// - Determining if the van should be open based on player position
///
/// # Arguments
/// * `p` - System parameters containing resources for player setup
/// * `commands` - Command buffer for entity creation
/// * `player_spawn_points` - List of potential spawn positions for the player
/// * `van_entry_points` - List of van entry positions (used to determine van open state)
///
/// # Returns
/// Boolean indicating if the van should be open (based on player proximity)
pub fn spawn_player(
    p: &LoadLevelSystemParam,
    commands: &mut Commands,
    player_spawn_points: &mut Vec<Position>,
    van_entry_points: &[Position],
) -> bool {
    // Shuffle spawn points for randomization
    player_spawn_points.shuffle(&mut random_seed::rng());
    if player_spawn_points.is_empty() {
        error!(
            "No player spawn points found!! - that will probably not display the map because the player will be out of bounds"
        );
        return false;
    }

    // Select and convert the spawn position
    let player_position = player_spawn_points.pop().unwrap();
    let player_scoord = player_position.to_screen_coord();

    // Calculate distance to nearest van entry point
    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

    // Spawn the player entity
    commands
        .spawn(Sprite {
            image: p.handles.images.character1.clone(),
            anchor: Anchor::Custom(p.handles.anchors.grid1x1x4),
            texture_atlas: Some(TextureAtlas {
                layout: p.handles.images.character1_atlas.clone(),
                ..Default::default()
            }),
            ..default()
        })
        .insert(
            Transform::from_xyz(player_scoord[0], player_scoord[1], player_scoord[2])
                .with_scale(Vec3::new(0.5, 0.5, 0.5)),
        )
        .insert(GameSprite)
        .insert(PlayerGear::from_playergearkind(
            p.difficulty.0.player_gear.clone(),
        ))
        .insert(PlayerSprite::new(1, player_position).with_controls(**p.control_settings))
        // Update the SpatialListener to use the ear offset from audio settings
        .insert(SpatialListener::new(
            -p.audio_settings.sound_output.to_ear_offset(),
        ))
        .insert(SpriteType::Player)
        .insert(player_position)
        .insert(Direction::new_right())
        .insert(AnimationTimer::from_range(
            Timer::from_seconds(0.20, TimerMode::Repeating),
            CharacterAnimation::from_dir(0.5, 0.5).to_vec(),
        ))
        .insert(Stamina::default());

    // Determine if the van should be open based on distance to van and difficulty setting
    dist_to_van < 8.0 && p.difficulty.0.van_auto_open
}

/// Spawns the ghost entity and its breach with all required components.
///
/// This function:
/// - Selects a ghost spawn point
/// - Determines the ghost type based on difficulty settings
/// - Creates both the ghost breach and ghost entity
/// - Updates the evidence list in the board data
/// - Sets up the SummaryData resource
///
/// # Arguments
/// * `p` - System parameters containing resources for ghost setup
/// * `commands` - Command buffer for entity creation
/// * `ghost_spawn_points` - List of potential spawn positions for the ghost
pub fn spawn_ghosts(
    p: &mut LoadLevelSystemParam,
    commands: &mut Commands,
    ghost_spawn_points: &mut [Position],
) {
    // Clear existing evidence records and get RNG
    p.bf.evidences.clear();
    let mut rng = random_seed::rng();

    // Select a ghost spawn point using the selection function
    let ghost_spawn = crate::selection::select_ghost_spawn_point(ghost_spawn_points, &mut rng)
        .unwrap_or_else(|| {
            error!(
                "No ghost spawn points found!! - that will probably break the gameplay as the ghost will spawn out of bounds"
            );
            // Fallback to a default position if no spawn points are available
            Position::new_i64(0, 0, 0)
        });

    // Determine ghost type based on difficulty settings
    let possible_ghost_types: Vec<_> = p.difficulty.0.ghost_set.as_vec();
    let ghost_sprite = GhostSprite::new(ghost_spawn.to_board_position(), &possible_ghost_types);
    let ghost_types = vec![ghost_sprite.class];

    // Collect ghost evidences in board data
    for evidence in ghost_sprite.class.evidences() {
        p.bf.evidences.insert(evidence);
    }

    // Store breach position in board data
    p.bf.breach_pos = ghost_spawn;

    // Update summary data resource with ghost information
    commands.insert_resource(SummaryData::new(ghost_types, p.difficulty.clone()));

    // Spawn the ghost breach entity
    let breach_id = commands
        .spawn(Sprite {
            image: p.asset_server.load("img/breach.png"),
            anchor: Anchor::Custom(p.handles.anchors.grid1x1x4),
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Transform::from_xyz(-1000.0, -1000.0, -1000.0))
        .insert(GameSprite)
        .insert(SpriteType::Breach)
        .insert(GhostBreach)
        .insert(ghost_spawn)
        .with_children(|parent| {
            parent
                .spawn(Sprite {
                    image: p.asset_server.load("img/focus_ring_vignette.png"),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                })
                .insert(
                    Transform::from_scale(Vec3::splat(0.5))
                        .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                )
                .insert(FocusRing::default());
        })
        .id();

    // Spawn the ghost entity
    commands
        .spawn(Sprite {
            image: p.asset_server.load("img/ghost.png"),
            anchor: Anchor::Custom(p.handles.anchors.grid1x1x4),
            color: Color::srgba(0.0, 0.0, 0.0, 0.0),
            ..default()
        })
        .insert(Transform::from_xyz(-1000.0, -1000.0, -1000.0))
        .insert(GameSprite)
        .insert(SpriteType::Ghost)
        .insert(ghost_sprite.with_breachid(breach_id))
        .insert(ghost_spawn)
        .with_children(|parent| {
            parent
                .spawn(Sprite {
                    image: p.asset_server.load("img/focus_ring_vignette.png"),
                    color: Color::srgba(1.0, 1.0, 1.0, 0.0),
                    ..default()
                })
                .insert(
                    Transform::from_scale(Vec3::splat(0.5))
                        .with_translation(Vec3::new(0.0, 0.0, 0.01)),
                )
                .insert(FocusRing::default());
        });
}

/// Spawns ambient sound entities for the game environment.
///
/// This function creates audio entities for various ambient sounds:
/// - Background house sounds
/// - Background street sounds
/// - Heartbeat sounds
/// - Insanity sounds
///
/// # Arguments
/// * `p` - System parameters containing asset server
/// * `commands` - Command buffer for entity creation
pub fn spawn_ambient_sounds(p: &LoadLevelSystemParam, commands: &mut Commands) {
    // Spawn background house sound
    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/background-noise-house-1.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
        })
        .insert(GameSound {
            class: SoundType::BackgroundHouse,
        });

    // Spawn background street sound
    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/ambient-clean.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
        })
        .insert(GameSound {
            class: SoundType::BackgroundStreet,
        });

    // Spawn heartbeat sound
    commands
        .spawn(AudioPlayer::new(
            p.asset_server.load("sounds/heartbeat-1.ogg"),
        ))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
        })
        .insert(GameSound {
            class: SoundType::HeartBeat,
        });

    // Spawn insane sound
    commands
        .spawn(AudioPlayer::new(p.asset_server.load("sounds/insane-1.ogg")))
        .insert(PlaybackSettings {
            mode: bevy::audio::PlaybackMode::Loop,
            volume: bevy::audio::Volume::new(0.00001),
            speed: 1.0,
            paused: false,
            spatial: false,
            spatial_scale: None,
        })
        .insert(GameSound {
            class: SoundType::Insane,
        });
}
