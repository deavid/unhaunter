use bevy::{prelude::*, render::camera::ScalingMode};
use bevy_persistent::Persistent;
use uncore::{
    components::{
        board::direction::Direction,
        game::{GCameraArena, GameSound, GameSprite},
        game_config::GameConfig,
        player_sprite::PlayerSprite,
    },
    controlkeys::ControlKeys,
    states::{AppState, GameState},
};
use unsettings::game::GameplaySettings;

pub fn setup(mut commands: Commands, qc: Query<Entity, With<GCameraArena>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn_recursive();
    }

    // 2D orthographic camera - Arena
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: 224.0,
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
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    mut game_next_state: ResMut<NextState<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<&mut Transform, With<GCameraArena>>,
    gc: Res<GameConfig>,
    pc: Query<(&PlayerSprite, &Transform, &Direction), Without<GCameraArena>>,
    time: Res<Time>,
    game_settings: Res<Persistent<GameplaySettings>>,
) {
    if *app_state.get() != AppState::InGame {
        return;
    }
    let in_game = *game_state.get() == GameState::None;
    if *game_state.get() == GameState::Pause {
        return;
    }
    let dt = time.delta_secs() * 60.0;
    if keyboard_input.just_pressed(KeyCode::Escape) && in_game {
        game_next_state.set(GameState::Pause);
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
        if in_game && game_settings.camera_controls.on() {
            let controls = match game_settings.character_controls {
                unsettings::game::CharacterControls::WASD => ControlKeys::ARROWS,
                unsettings::game::CharacterControls::Arrows => ControlKeys::WASD,
            };
            if keyboard_input.pressed(controls.right) {
                transform.translation.x += 2.0 * dt;
            }
            if keyboard_input.pressed(controls.left) {
                transform.translation.x -= 2.0 * dt;
            }
            if keyboard_input.pressed(controls.up) {
                transform.translation.y += 2.0 * dt;
            }
            if keyboard_input.pressed(controls.down) {
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
