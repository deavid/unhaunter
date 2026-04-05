use bevy::prelude::*;
use rand::RngExt;
use unboard_core::components::mapcolor::MapColor;
use unboard_core::entity::GameSprite;
use uncommon_app_core::random_seed;
use unghost_core::events::GhostBreakerSparkRequest;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::position::Position;

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(Update, consume_breaker_spark_requests);
}

fn consume_breaker_spark_requests(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut ev_breaker_sparks: MessageReader<GhostBreakerSparkRequest>,
) {
    let mut rng = random_seed::rng();

    for event in ev_breaker_sparks.read() {
        for _ in 0..8 {
            commands
                .spawn(Sprite {
                    image: asset_server.load("img/particle_spark.png"),
                    color: Color::srgba(1.0, 0.9, 0.3, 0.9),
                    custom_size: Some(Vec2::new(0.1, 0.1)),
                    ..default()
                })
                .insert(Position {
                    x: event.position.x + rng.random_range(-0.1..0.1),
                    y: event.position.y + rng.random_range(-0.1..0.1),
                    z: event.position.z + rng.random_range(0.1..0.4),
                    visual_priority: event.position.visual_priority,
                })
                .insert(GameSprite)
                .insert(MapColor {
                    color: Color::srgba(1.0, 0.9, 0.3, 0.9),
                })
                .insert(SpriteLayer::default())
                .insert(unghost_core::components::presentation::interaction::InteractionParticle {
                    life: 0.8,
                    max_life: 0.8,
                    velocity: Vec3::new(
                        rng.random_range(-0.3..0.3),
                        rng.random_range(-0.3..0.3),
                        rng.random_range(0.2..0.6),
                    ),
                    particle_type:
                        unghost_core::components::presentation::interaction::InteractionParticleType::Spark,
                })
                .insert(Transform::from_scale(Vec3::splat(0.3)));
        }
    }
}
