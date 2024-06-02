pub mod evidence;
pub mod level;
pub mod ui;

use crate::player::{self, PlayerSprite};
use crate::{board, root};
use bevy::render::view::RenderLayers;
use bevy::{prelude::*, render::camera::ScalingMode};

#[derive(Component)]
pub struct GCameraArena;

#[derive(Component, Debug)]
pub struct GameSprite;

#[derive(Component, Debug, Default)]
pub struct MapUpdate {
    pub last_update: f32,
}

#[derive(Component, Debug)]
pub struct GameSound {
    pub class: SoundType,
}

#[derive(Debug, PartialEq, Eq)]
pub enum SoundType {
    BackgroundHouse,
    BackgroundStreet,
    HeartBeat,
    Insane,
}

/// Resource to know basic stuff of the game.
#[derive(Debug, Resource)]
pub struct GameConfig {
    /// Which player should the camera and lighting follow
    pub player_id: usize,
}

impl Default for GameConfig {
    fn default() -> Self {
        Self { player_id: 1 }
    }
}

#[derive(Component, Debug, Clone, PartialEq, Eq, Default)]
pub enum SpriteType {
    Ghost,
    Breach,
    Player,
    #[default]
    Other,
}

pub fn setup(mut commands: Commands, qc: Query<Entity, With<GCameraArena>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // 2D orthographic camera - Arena
    let mut cam = Camera2dBundle::default();
    cam.projection.scaling_mode = ScalingMode::FixedVertical(200.0);
    commands
        .spawn(cam)
        .insert(GCameraArena)
        .insert(RenderLayers::from_layers(&[0, 1]));
}

pub fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<GCameraArena>>,
    qgs: Query<Entity, With<GameSprite>>,
    qs: Query<Entity, With<GameSound>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }
    // Despawn game sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn_recursive();
    }
    // Despawn game sound
    for gs in qs.iter() {
        commands.entity(gs).despawn_recursive();
    }
}

#[allow(clippy::too_many_arguments)]
pub fn keyboard(
    app_state: Res<State<root::State>>,
    game_state: Res<State<root::GameState>>,
    mut game_next_state: ResMut<NextState<root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform, &board::Direction), Without<GCameraArena>>,
    time: Res<Time>,
) {
    if *app_state.get() != root::State::InGame {
        return;
    }
    let in_game = *game_state.get() == root::GameState::None;
    if *game_state.get() == root::GameState::Pause {
        return;
    }
    let dt = time.delta_seconds() * 60.0;
    if keyboard_input.just_pressed(KeyCode::Escape) && in_game {
        game_next_state.set(root::GameState::Pause);
    }
    for mut transform in camera.iter_mut() {
        for (player, p_transform, p_dir) in pc.iter() {
            if player.id != gc.player_id {
                continue;
            }
            // Camera movement
            let mut ref_point = p_transform.translation;
            let sc_dir = p_dir.to_screen_coord();
            const CAMERA_AHEAD_FACTOR: f32 = 0.11;
            ref_point.y += 20.0 + sc_dir.y * CAMERA_AHEAD_FACTOR;
            ref_point.x += sc_dir.x * CAMERA_AHEAD_FACTOR;
            ref_point.z = transform.translation.z;
            let dist = (transform.translation.distance(ref_point) - 1.0).max(0.00001);
            let mut delta = ref_point - transform.translation;
            delta.z = 0.0;
            const RED: f32 = 120.0;
            const MEAN_DIST: f32 = 120.0;
            let vector = delta.normalize() * ((dist / MEAN_DIST).powf(2.2) * MEAN_DIST);
            transform.translation += vector / RED * dt;
        }
        if in_game {
            if keyboard_input.pressed(KeyCode::ArrowRight) {
                transform.translation.x += 2.0 * dt;
            }
            if keyboard_input.pressed(KeyCode::ArrowLeft) {
                transform.translation.x -= 2.0 * dt;
            }
            if keyboard_input.pressed(KeyCode::ArrowUp) {
                transform.translation.y += 2.0 * dt;
            }
            if keyboard_input.pressed(KeyCode::ArrowDown) {
                transform.translation.y -= 2.0 * dt;
            }
            if keyboard_input.pressed(KeyCode::NumpadAdd) {
                transform.scale.x /= 1.02_f32.powf(dt);
                transform.scale.y /= 1.02_f32.powf(dt);
            }
            if keyboard_input.pressed(KeyCode::NumpadSubtract) {
                transform.scale.x *= 1.02_f32.powf(dt);
                transform.scale.y *= 1.02_f32.powf(dt);
            }
        }
    }
}

pub fn app_setup(app: &mut App) {
    app.init_resource::<GameConfig>()
        .add_systems(OnEnter(root::State::InGame), setup)
        .add_systems(OnExit(root::State::InGame), cleanup)
        .add_systems(Update, keyboard.before(player::keyboard_player));

    level::app_setup(app);
    ui::app_setup(app);
    evidence::app_setup(app);
}
