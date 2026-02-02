use bevy::{camera::ScalingMode, prelude::*};
use bevy_persistent::Persistent;
use unboard_core::resources::board_topology::BoardTopology;
use unpicking_core::picking::CustomSpritePickingCamera;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unplayer_core::resources::game_config::GameConfig;
use unrender_std::components::game::{GameSound, GameSprite};
use unsettings_core::controls::ControlKeys;
use unsettings_core::game::GameplaySettings;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
use unspatial_core::position::Position;
use untags_core::game::GCameraArena;
use untypes_core::states::{AppState, GameState};

fn setup(mut commands: Commands, qc: Query<Entity, With<GCameraArena>>) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // 2D orthographic camera - Arena
    let mut projection = OrthographicProjection::default_2d();
    projection.scaling_mode = ScalingMode::FixedVertical {
        viewport_height: 224.0,
    };
    commands
        .spawn(Camera2d)
        .insert(Projection::Orthographic(projection))
        .insert(GCameraArena)
        .insert(Direction::zero())
        .insert(CustomSpritePickingCamera);
}

fn cleanup(
    mut commands: Commands,
    qc: Query<Entity, With<GCameraArena>>,
    qgs: Query<Entity, With<GameSprite>>,
    qs: Query<Entity, With<GameSound>>,
) {
    // Despawn old camera if exists
    for cam in qc.iter() {
        commands.entity(cam).despawn();
    }

    // Despawn game sprites if not used
    for gs in qgs.iter() {
        commands.entity(gs).despawn();
    }

    // Despawn game sound
    for gs in qs.iter() {
        commands.entity(gs).despawn();
    }
}

fn keyboard(
    app_state: Res<State<AppState>>,
    game_state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut Direction), With<GCameraArena>>,
    pc: Query<(&PlayerSprite, &Transform, &Direction), (With<MainPlayer>, Without<GCameraArena>)>,
    time: Res<Time>,
    game_settings: Res<Persistent<GameplaySettings>>,
    control_settings: Res<Persistent<ControlKeys>>,
) {
    if *app_state.get() != AppState::InGame {
        return;
    }
    let in_game = *game_state.get() == GameState::None;
    if *game_state.get() == GameState::Pause {
        return;
    }
    let dt = time.delta_secs() * 60.0;
    for (mut transform, mut cam_dir) in camera.iter_mut() {
        for (player, p_transform, _p_dir) in pc.iter() {
            // Camera movement
            let mut ref_point = p_transform.translation;
            // Move the reference point a bit up since we have the UI on the bottom, so the player is better centered on the remaining available space.
            ref_point.y -= 10.0;
            // let sc_dir = p_dir.to_screen_coord();
            let sc_dir = perspective::direction_to_screen_coord(player.movement);
            const CAMERA_AHEAD_FACTOR: f32 = 0.11 / 1.8;
            ref_point.y += 20.0 + sc_dir.y * CAMERA_AHEAD_FACTOR;
            ref_point.x += sc_dir.x * CAMERA_AHEAD_FACTOR;
            let dist = (transform.translation.distance(ref_point) - 1.0).max(0.00001);
            let mut delta = ref_point - transform.translation;
            delta.z = 0.0;
            const RED: f32 = 120.0 * 2.0;
            const MEAN_DIST: f32 = 120.0 / 15.0;
            const MAX_DIST: f32 = 1000.0;
            let strength = ((dist.min(MAX_DIST) / MEAN_DIST).powf(1.4) * MEAN_DIST) / RED
                * (delta.dot(cam_dir.to_vec3()).clamp(0.2, 1.0));
            let vector = delta.normalize() * strength;
            let f_strength: f32 = 0.05;
            cam_dir.dx = (cam_dir.dx + vector.x * f_strength * dt) / (1.0 + f_strength * dt);
            cam_dir.dy = (cam_dir.dy + vector.y * f_strength * dt) / (1.0 + f_strength * dt);
            cam_dir.dz = (cam_dir.dz + vector.z * f_strength * dt) / (1.0 + f_strength * dt);

            transform.translation += cam_dir.to_vec3() * dt;
        }
        if in_game && game_settings.camera_controls.on() {
            if keyboard_input.pressed(control_settings.camera_right) {
                transform.translation.x += 2.0 * dt;
            }
            if keyboard_input.pressed(control_settings.camera_left) {
                transform.translation.x -= 2.0 * dt;
            }
            if keyboard_input.pressed(control_settings.camera_up) {
                transform.translation.y += 2.0 * dt;
            }
            if keyboard_input.pressed(control_settings.camera_down) {
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

/// System to handle temporary floor switching using Y and H keys
///
/// Y key: Move up one floor
/// H key: Move down one floor
///
/// This is a temporary debugging system for testing multi-floor maps
fn keyboard_floor_switch(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut player_query: Query<(&PlayerSprite, &mut Position), With<MainPlayer>>,
    board_topology: Res<BoardTopology>,
) {
    const DEBUG_FLOORS: bool = false;
    // Only act when Y or H key is just pressed
    let go_up = keyboard_input.just_pressed(KeyCode::KeyY);
    let go_down = keyboard_input.just_pressed(KeyCode::KeyH);

    if !DEBUG_FLOORS {
        return;
    }
    if !go_up && !go_down {
        return;
    }

    debug!(
        "Floor switch: Trying to switch floors (up: {}, down: {})",
        go_up, go_down
    );

    // Find the player entity matching the active player ID
    let mut player_position = None;
    for (player, pos) in player_query.iter_mut() {
        player_position = Some(pos);
        break;
    }

    // If we didn't find the player or there's no floor mapping data, exit
    let Some(mut player_pos) = player_position else {
        warn!("Floor switch: Player not found");
        return;
    };

    if board_topology.floor_z_map.is_empty() || board_topology.z_floor_map.is_empty() {
        warn!("Floor switch: No floor mapping data available");
        return;
    }

    // Get the current z position (should be an integer value)
    let current_z = player_pos.z.round() as usize;

    // Calculate the target floor z based on the key pressed
    let target_z = if go_up {
        // Find the next higher floor if it exists
        if current_z + 1 < board_topology.map_size.2 {
            current_z + 1
        } else {
            debug!("Floor switch: Already at the highest floor ({})", current_z);
            current_z
        }
    } else {
        // Find the next lower floor if it exists
        if current_z > 0 {
            current_z - 1
        } else {
            debug!("Floor switch: Already at the lowest floor ({})", current_z);
            current_z
        }
    };

    // Update the player's z position if we're changing floors
    if current_z != target_z {
        // Convert from usize to f32 for Position.z
        player_pos.z = target_z as f32;

        // Log the floor change for debugging
        if let Some(tiled_floor) = board_topology.z_floor_map.get(&target_z) {
            debug!(
                "Floor switch: Moving to z={} (Tiled floor number: {})",
                target_z, tiled_floor
            );
        } else {
            debug!(
                "Floor switch: Moving to z={} (unknown Tiled floor)",
                target_z
            );
        }
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup);
    app.add_systems(OnExit(AppState::InGame), cleanup);
    app.add_systems(
        Update,
        (keyboard, keyboard_floor_switch).run_if(in_state(AppState::InGame)),
    );
}
