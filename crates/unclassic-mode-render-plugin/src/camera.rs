use bevy::{camera::ScalingMode, prelude::*};
use bevy_persistent::Persistent;
use unengine_core::GCameraArena;
use unpicking_core::picking::CustomSpritePickingCamera;
use unplayer_core::components::{MainPlayer, PlayerSprite};
use unsettings_core::controls::ControlKeys;
use unsettings_core::game::GameplaySettings;
use unspatial_core::direction::Direction;
use unspatial_core::perspective;
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

fn camera_follow_system(
    game_state: Res<State<GameState>>,
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut camera: Query<(&mut Transform, &mut Direction), With<GCameraArena>>,
    pc: Query<(&PlayerSprite, &Transform), (Without<GCameraArena>, With<MainPlayer>)>,
    time: Res<Time>,
    game_settings: Res<Persistent<GameplaySettings>>,
    control_settings: Res<Persistent<ControlKeys>>,
    mut warn_count: Local<u32>,
) {
    let in_game = *game_state.get() == GameState::Running;
    let Ok((player, p_transform)) = pc.single() else {
        *warn_count += 1;
        if *warn_count > 60 {
            warn!("Camera error - Player not found (or too many)");
            *warn_count = 0;
        }
        return;
    };
    let Ok((mut transform, mut cam_dir)) = camera.single_mut() else {
        *warn_count += 1;
        if *warn_count > 60 {
            warn!("Camera error - Camera not found (or too many)");
            *warn_count = 0;
        }
        return;
    };
    *warn_count = 0;

    let dt = time.delta_secs() * 60.0;

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

fn debug_tile_transforms(
    q_tiles: Query<
        (
            &MeshMaterial2d<unrender_std::materials::CustomMaterial1>,
            &Visibility,
        ),
        With<unrender_std::components::game::MapTileSprite>,
    >,
    q_stages: Query<
        (),
        Or<(
            With<untypes_core::hydration::HydrationStage<1>>,
            With<untypes_core::hydration::HydrationStage<2>>,
            With<untypes_core::hydration::HydrationStage<3>>,
            With<untypes_core::hydration::HydrationStage<4>>,
        )>,
    >,
    materials: Res<Assets<unrender_std::materials::CustomMaterial1>>,
    time: Res<Time>,
    mut timer: Local<f32>,
) {
    if *timer <= 0.0 {
        *timer = 5.0;
        let pending_count = q_stages.iter().count();
        let total_tiles = q_tiles.iter().count();

        // Collect alpha values for tiles that are not explicitly hidden (i.e. active/current-floor tiles).
        let alphas: Vec<f32> = q_tiles
            .iter()
            .filter(|(_, vis)| **vis != Visibility::Hidden)
            .filter_map(|(mat_handle, _)| materials.get(mat_handle))
            .map(|mat| mat.data.color.alpha)
            .collect();

        let active_count = alphas.len();

        if active_count == 0 {
            if total_tiles == 0 {
                info!("DEBUG tiles: no tiles found. Pending hydration: {pending_count}");
            } else {
                info!(
                    "DEBUG tiles: {total_tiles} total, 0 active (all hidden). Pending hydration: {pending_count}"
                );
            }
            return;
        }

        let mean = alphas.iter().copied().sum::<f32>() / active_count as f32;
        let variance = alphas.iter().map(|a| (a - mean).powi(2)).sum::<f32>() / active_count as f32;
        let std_dev = variance.sqrt();
        let min = alphas.iter().copied().fold(f32::INFINITY, f32::min);
        let max = alphas.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        info!(
            "DEBUG tiles: {active_count}/{total_tiles} active, alpha mean={mean:.3} ±{std_dev:.3} min={min:.3} max={max:.3}. Pending hydration: {pending_count}"
        );
    }
    *timer -= time.delta_secs();
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(OnEnter(AppState::InGame), setup);
    app.add_systems(
        Update,
        (
            camera_follow_system.run_if(in_state(AppState::InGame)),
            debug_tile_transforms.run_if(in_state(AppState::InGame)),
        ),
    );
}
