pub mod evidence;
pub mod level;
pub mod roomchanged;
pub mod ui;

use uncore::components::game::{GCameraArena, GameSound, GameSprite};

use crate::player::{self, PlayerSprite};
use crate::{uncore_board, uncore_root};

use bevy::prelude::*;
use bevy::render::camera::ScalingMode;

pub use uncore::components::game_config::GameConfig;
pub use uncore::components::sprite_type::SpriteType;

pub fn setup(mut commands: Commands, qc: Query<Entity, With<GCameraArena>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }

    // 2D orthographic camera - Arena
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: 200.0,
    };
    commands
        .spawn(Camera2d)
        .insert(projection)
        .insert(GCameraArena);
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
    app_state: Res<State<uncore_root::AppState>>,
    game_state: Res<State<uncore_root::GameState>>,
    mut game_next_state: ResMut<NextState<uncore_root::GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform, &uncore_board::Direction), Without<GCameraArena>>,
    time: Res<Time>,
) {
    if *app_state.get() != uncore_root::AppState::InGame {
        return;
    }
    let in_game = *game_state.get() == uncore_root::GameState::None;
    if *game_state.get() == uncore_root::GameState::Pause {
        return;
    }
    let dt = time.delta_secs() * 60.0;
    if keyboard_input.just_pressed(KeyCode::Escape) && in_game {
        game_next_state.set(uncore_root::GameState::Pause);
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
        const DEBUG_CAMERA: bool = true;
        if in_game && DEBUG_CAMERA {
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
        .add_systems(OnEnter(uncore_root::AppState::InGame), setup)
        .add_systems(OnExit(uncore_root::AppState::InGame), cleanup)
        .add_systems(
            Update,
            keyboard.before(player::systems::keyboard::keyboard_player),
        );
    level::app_setup(app);
    ui::app_setup(app);
    evidence::app_setup(app);
    roomchanged::app_setup(app);
}
