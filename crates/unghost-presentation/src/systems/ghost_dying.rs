use bevy::prelude::*;
use rand::prelude::*;
use unboard_core::components::mapcolor::MapColor;
use uncommon_app_core::random_seed;
use uncommon_states_core::UIContextState;
use unghost_core::components::logic::ghost_death::GhostDeathSignal;
use unghost_core::components::presentation::ghost_dying::GhostDying;
use unreplicon_core::resources::LocalPlayerRole;
use unspatial_core::position::Position;
use unboard_core::entity::GameSprite;
use bevy::sprite::Sprite;
use unrender_std::components::sprite_layer::SpriteLayer;
use unspatial_core::perspective::to_screen_coord;

#[derive(Component)]
struct GhostDeathSmokeParticle {
    timer: Timer,
    dx: f32,
    dy: f32,
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            start_ghost_dying_visuals_system,
            tick_ghost_dying_visuals_system,
            ghost_dying_visuals_system,
            tick_ghost_death_smoke_system,
        )
            .run_if(in_state(UIContextState::InGame))
            .run_if(resource_exists::<LocalPlayerRole>),
    );
}

fn start_ghost_dying_visuals_system(
    mut commands: Commands,
    query: Query<(Entity, &GhostDeathSignal), (Added<GhostDeathSignal>, Without<GhostDying>)>,
) {
    for (entity, death_signal) in query.iter() {
        commands
            .entity(entity)
            .insert(GhostDying::new(death_signal.duration_secs));
    }
}

fn tick_ghost_dying_visuals_system(
    mut commands: Commands,
    time: Res<Time>,
    asset_server: Res<AssetServer>,
    mut query: Query<(&mut GhostDying, Option<&Position>)>,
) {
    let mut rng = random_seed::rng();
    for (mut dying, pos_opt) in query.iter_mut() {
        dying.timer.tick(time.delta());

        let rem_f = dying.timer.remaining_secs() / dying.timer.duration().as_secs_f32();
        if dying.timer.remaining_secs() > 0.0 {
            let roll_chance = ((1.0 - rem_f) / 3.0) as f64;
            if let Some(pos) = pos_opt.filter(|_| rng.random_bool(roll_chance)) {
                commands
                    .spawn((
                        Sprite {
                            image: asset_server.load("img/smoke.png"),
                            color: Color::NONE,
                            ..default()
                        },
                        *pos,
                        GameSprite,
                        MapColor {
                            color: Color::srgba(1.0, 1.0, 1.0, 0.20),
                        },
                        SpriteLayer::default(),
                    ))
                    .insert(Transform::from_translation(to_screen_coord(*pos))
                        .with_scale(Vec3::new(0.2, 0.2, 0.2)))
                    .insert(GhostDeathSmokeParticle {
                        timer: Timer::from_seconds(5.0, TimerMode::Once),
                        dx: rng.random_range(-0.9..0.9),
                        dy: rng.random_range(-0.9..0.9),
                    });
            }
        }
    }
}

fn tick_ghost_death_smoke_system(
    mut commands: Commands,
    time: Res<Time>,
    mut smoke_particles: Query<(
        Entity,
        &mut Position,
        &mut Transform,
        &mut GhostDeathSmokeParticle,
        &mut MapColor,
    )>,
) {
    let dt = time.delta_secs();
    for (entity, mut position, mut transform, mut smoke_particle, mut map_color) in
        smoke_particles.iter_mut()
    {
        smoke_particle.timer.tick(time.delta());
        if smoke_particle.timer.just_finished() {
            commands.entity(entity).despawn();
            continue;
        }

        let elap = smoke_particle.timer.elapsed_secs();
        let rem = smoke_particle.timer.remaining_secs();
        let a = ((elap * 3.0)
            .clamp(0.0, 1.0)
            .min((rem / 2.0 - 0.01).clamp(0.0, 1.0)))
        .powf(2.0);
        map_color.color.set_alpha(a * 0.4);

        position.z += 0.3 * dt / (1.0 + elap.powi(2));
        position.x += smoke_particle.dx * dt;
        position.y += smoke_particle.dy * dt;
        transform.scale.x += 0.1 * dt;
        transform.scale.y += 0.1 * dt;
    }
}

fn ghost_dying_visuals_system(mut query: Query<(&GhostDying, &mut MapColor)>) {
    for (dying, mut map_color) in query.iter_mut() {
        let rem_f = dying.timer.remaining_secs() / dying.timer.duration().as_secs_f32();
        map_color.color.set_alpha(rem_f.powi(2));
    }
}
