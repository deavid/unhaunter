mod tiledmap;

use bevy::{prelude::*, window::WindowResolution};
use tiledmap::SpriteEnum;

fn main() {
    App::new()
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Unhaunter".to_string(),
                        resolution: WindowResolution::new(1500.0, 800.0),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .init_resource::<tiledmap::MapTileSetDb>()
        .add_systems(Startup, setup)
        //        .add_systems(Update, sprite_movement)
        .add_systems(Update, camera_movement)
        .run();
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    texture_atlases: ResMut<Assets<TextureAtlas>>,
    tilesetdb: ResMut<tiledmap::MapTileSetDb>,
) {
    commands.spawn(Camera2dBundle::default());

    let sprites = tiledmap::bevy_load_map(
        "assets/maps/map_house1_3x.tmx",
        asset_server,
        texture_atlases,
        tilesetdb,
    );

    for bundle in sprites {
        match bundle {
            SpriteEnum::One(b) => commands.spawn(b),
            SpriteEnum::Sheet(b) => commands.spawn(b),
        };
    }
}

fn camera_movement(
    time: Res<Time>,
    mut camera_position: Query<(&mut Camera2d, &mut Transform)>,
    keyboard_input: Res<Input<KeyCode>>,
) {
    // const RADIUS: f32 = 300.0;
    // let phase = time.elapsed_seconds() / 10.0;

    let delta = time.delta_seconds() * 200.0;
    let mov = 2.0;
    let zoom = 1.0;
    for (_cam, mut transform) in camera_position.iter_mut() {
        if keyboard_input.pressed(KeyCode::D) {
            transform.translation.x += delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::A) {
            transform.translation.x -= delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::W) {
            transform.translation.y += delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::S) {
            transform.translation.y -= delta * transform.scale.x * mov;
        }
        if keyboard_input.pressed(KeyCode::Plus) {
            transform.scale /= f32::powf(1.003, delta * zoom);
        }
        if keyboard_input.pressed(KeyCode::Minus) {
            transform.scale *= f32::powf(1.003, delta * zoom);
        }
        if keyboard_input.pressed(KeyCode::Key1) {
            let z: f32 = 1.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key2) {
            let z: f32 = 1.0 / 2.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key3) {
            let z: f32 = 1.0 / 4.0;
            transform.scale = Vec3::new(z, z, z);
        }
        if keyboard_input.pressed(KeyCode::Key4) {
            let z: f32 = 1.0 / 8.0;
            transform.scale = Vec3::new(z, z, z);
        }
        // transform.translation.x = phase.cos() * RADIUS;
        // transform.translation.y = phase.sin() * RADIUS;
        // transform.scale = Vec3::new(0.5, 0.5, 0.5);
    }
}
