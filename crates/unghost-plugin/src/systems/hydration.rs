use bevy::prelude::*;
use bevy_replicon::prelude::Replicated;
use unbehavior_core::behavior::{Behavior, Util};
use unbehavior_core::components::{InteractableByGhost, Movable};
use unboard_core::components::physics::{FluidEmitter, ThermalEmitter};
use unboard_core::components::spawning::HostileSpawnPoint;
use unboard_core::entity::GameSprite;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::components::logic::ghost_sprite::GhostSprite;
use unghost_core::requests::{GhostBreachSpawnRequest, GhostSpawnRequest};
use unghost_core::tags::GhostTag;
use unlight_core::components::LightSensitive;
use unlight_core::spectral::SpectralInfluence;
use unmapload_core::hydration::HydrationStage;
use unmetrics_core::metrics::SendMetric;
use unreplicon_core::network_id::NetworkId;
use unreplicon_core::resources::AuthorityRole;
use unsoundfield_core::components::SoundFieldSource;
use unspatial_core::lerp_position::LerpPosition;
use unspatial_core::position::Position;

use crate::metrics;

fn hydration_ghost_logic_system(
    mut q: Query<(Entity, &Behavior), With<HydrationStage<3>>>,
    mut commands: Commands,
) {
    let measure = metrics::HYDRATION_GHOST_LOGIC.time_measure();
    for (entity, behavior) in q.iter_mut() {
        // Hostile (Ghost) Spawn Points
        if let Util::GhostSpawn = &behavior.p.util {
            commands.entity(entity).insert(HostileSpawnPoint);
        }

        // Add InteractableByGhost marker component for entities that ghosts can interact with
        let should_add_ghost_interaction =
            if behavior.p.is_door || behavior.p.is_switch || behavior.p.is_breaker {
                true
            } else {
                // For other classes, check properties
                let has_light = behavior.p.light.can_emit_light;
                let is_movable = behavior.p.object.movable;
                let is_throwable = behavior.p.object.throwable;
                let is_nudgeable = behavior.p.object.nudgeable;
                let haunt_movable = behavior.p.object.haunt_movable;

                has_light || is_movable || is_throwable || is_nudgeable || haunt_movable
            };

        if should_add_ghost_interaction {
            commands.entity(entity).insert(InteractableByGhost);
        }
    }
    measure.end_ms();
}

fn ghost_hydration_system(
    mut commands: Commands,
    q: Query<(Entity, &GhostSpawnRequest, &Position), Without<GhostTag>>,
) {
    for (entity, request, pos) in q.iter() {
        let mut ghost_sprite = GhostSprite::new(pos.to_board_position(), &request.ghost_types);
        ghost_sprite.breach_id = request.breach_entity;

        commands
            .entity(entity)
            .insert(ghost_sprite)
            .insert(GhostTag)
            .insert(NetworkId(0)) // Ghost is always 0 in MVP
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(Movable)
            .insert(LightSensitive {
                exposure_factor: 0.5,
                bias: 0.01,
            })
            .insert(SpectralInfluence::default().with_ultraviolet(1.0, 0.0))
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundFieldSource::default())
            .insert(Replicated)
            .insert(LerpPosition::new(*pos))
            .remove::<GhostSpawnRequest>();
    }
}

fn breach_hydration_system(
    mut commands: Commands,
    q: Query<(Entity, &Position), (With<GhostBreachSpawnRequest>, Without<GhostBreach>)>,
) {
    for (entity, pos) in q.iter() {
        commands
            .entity(entity)
            .insert(GhostBreach)
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.02,
            })
            .insert(SpectralInfluence::default().with_ultraviolet(1.0, 1.0))
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundFieldSource::default())
            .insert(Replicated)
            .remove::<GhostBreachSpawnRequest>();
    }
}

fn mark_breach_replicated(q: Query<Entity, Added<GhostBreach>>, mut commands: Commands) {
    for entity in q.iter() {
        commands.entity(entity).insert(Replicated);
    }
}

fn hydrate_ghost_local_field_components(
    mut commands: Commands,
    q: Query<(Entity, &Position), (With<GhostTag>, With<GhostSprite>, Without<GameSprite>)>,
) {
    for (entity, pos) in q.iter() {
        commands
            .entity(entity)
            .insert(GameSprite)
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(LightSensitive {
                exposure_factor: 0.5,
                bias: 0.01,
            })
            .insert(SpectralInfluence::default().with_ultraviolet(1.0, 0.0))
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundFieldSource::default())
            .insert(LerpPosition::new(*pos));
    }
}

fn hydrate_breach_local_field_components(
    mut commands: Commands,
    q: Query<(Entity, &Position), (With<GhostBreach>, Without<GameSprite>)>,
) {
    for (entity, pos) in q.iter() {
        commands
            .entity(entity)
            .insert(GameSprite)
            .insert(unspatial_core::boardposition::MapEntityFieldBPos(
                pos.to_board_position(),
            ))
            .insert(LightSensitive {
                exposure_factor: 1.1,
                bias: 0.02,
            })
            .insert(SpectralInfluence::default().with_ultraviolet(1.0, 1.0))
            .insert(ThermalEmitter {
                room_restricted: true,
                ..default()
            })
            .insert(FluidEmitter::default())
            .insert(SoundFieldSource::default());
    }
}

pub(crate) fn app_setup(app: &mut App) {
    app.add_systems(
        Update,
        (
            hydration_ghost_logic_system,
            breach_hydration_system,
            ghost_hydration_system,
        ),
    );
    app.add_systems(
        Update,
        mark_breach_replicated.run_if(resource_exists::<AuthorityRole>),
    );
    app.add_systems(
        Update,
        (
            hydrate_ghost_local_field_components,
            hydrate_breach_local_field_components,
        )
            .run_if(resource_exists::<unreplicon_core::resources::LocalPlayerRole>),
    );
}
