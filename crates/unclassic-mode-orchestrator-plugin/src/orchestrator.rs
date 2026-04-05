use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use ordered_float::OrderedFloat;
use rand::prelude::IndexedRandom;
use unbehavior_core::components::Movable;
use unboard_core::components::spawning::{HostileSpawnPoint, PlayerSpawnPoint, VanEntryPoint};
use unboard_core::resources::board_topology::BoardTopology;
use unboard_core::resources::roomdb::RoomTopology;
use uncommon_app_core::random_seed;
use undifficulty_core::current_difficulty::CurrentDifficulty;
use undifficulty_core::difficulty_settings::DifficultySettings;
use unghost_core::components::logic::ghost_breach::GhostBreach;
use unghost_core::difficulty_ext::DifficultyGhostExt;
use unghost_core::requests::{GhostBreachSpawnRequest, GhostSpawnRequest};
use unghost_core::resources::haunt_state::HauntState;
use unmapload_core::events::loadlevel::MapEntitiesReadyEvent;
use unmission_core::events::LevelReadyEvent;
use unmission_core::summary::SummaryData;
use unplayer_core::components::PlayerSprite;
use unspatial_core::position::Position;

#[derive(SystemParam)]
pub(crate) struct OrchestratorParam<'w> {
    pub authority_role: Option<Res<'w, unreplicon_core::resources::AuthorityRole>>,
    pub haunt_state: ResMut<'w, HauntState>,
    pub difficulty: Res<'w, CurrentDifficulty>,
    pub board_topology: Res<'w, BoardTopology>,
    pub room_topology: Res<'w, RoomTopology>,
}

pub(crate) fn classic_mode_orchestrator(
    p: OrchestratorParam,
    mut commands: Commands,
    mut ev_level_ready: MessageWriter<LevelReadyEvent>,
    mut ev_entities_ready: MessageReader<MapEntitiesReadyEvent>,
    q_ghost_breach: Query<&Position, With<GhostBreach>>,
    q_player_sprite: Query<&Position, With<PlayerSprite>>,
    q_position: Query<&Position>,
    q_player_spawns: Query<&Position, With<PlayerSpawnPoint>>,
    q_ghost_spawns: Query<&Position, With<HostileSpawnPoint>>,
    q_van_entry: Query<&Position, With<VanEntryPoint>>,
    q_movable: Query<Entity, With<Movable>>,
) {
    let Some(_) = ev_entities_ready.read().next() else {
        return;
    };

    let player_spawn_points: Vec<Position> = q_player_spawns.iter().copied().collect();
    let ghost_spawn_points: Vec<Position> = q_ghost_spawns.iter().copied().collect();
    let van_entry_points: Vec<Position> = q_van_entry.iter().copied().collect();
    let movable_objects: Vec<Entity> = q_movable.iter().collect();

    if player_spawn_points.is_empty() {
        error!("No player spawn points found!!");
        return;
    }

    // --- Determine Player/Van Position ---
    let mut rng = random_seed::rng();
    let player_position = player_spawn_points.choose(&mut rng).copied().unwrap();

    let dist_to_van = van_entry_points
        .iter()
        .map(|v| OrderedFloat(v.distance(&player_position)))
        .min()
        .unwrap_or(OrderedFloat(1000.0))
        .into_inner();

    let open_van = dist_to_van < 8.0 && p.difficulty.0.van_auto_open();

    // Join clients do not spawn the ghost locally; they receive the replicated entity
    // from the server and set up its visuals via hydrate_ghosts_system.
    if p.authority_role.is_some() {
        // --- Spawn Ghost ---
        {
            let ghost_spawn = ghost_spawn_points
                .choose(&mut rng)
                .copied()
                .unwrap_or(Position::new_i64(0, 0, 0));

            let possible_ghost_types: Vec<_> = p.difficulty.0.ghost_set().as_vec();
            let ghost_type_names = possible_ghost_types
                .iter()
                .map(|ghost_type| ghost_type.name())
                .collect::<Vec<_>>()
                .join(", ");

            info!(
                "MULTIPLAYER_GHOST_POOL_AUTHORITY: difficulty={:?} ghost_count={} ghost_types=[{}]",
                p.difficulty.0,
                possible_ghost_types.len(),
                ghost_type_names
            );
            if possible_ghost_types.is_empty() {
                warn!(
                    "MULTIPLAYER_GHOST_POOL_AUTHORITY: authoritative difficulty {:?} produced an empty ghost pool during mission setup",
                    p.difficulty.0
                );
            }

            commands.insert_resource(SummaryData::new(
                possible_ghost_types.clone(),
                *p.difficulty,
            ));

            let breach_id = commands.spawn((ghost_spawn, GhostBreachSpawnRequest)).id();

            commands.spawn((
                ghost_spawn,
                GhostSpawnRequest {
                    ghost_types: possible_ghost_types,
                    breach_entity: Some(breach_id),
                },
                p.haunt_state.ghost_dynamics,
            ));

            crate::influence_system::assign_ghost_influence(
                &mut commands,
                &movable_objects,
                &q_ghost_breach,
                &q_player_sprite,
                &q_position,
                &p.room_topology,
                &p.board_topology,
                &p.haunt_state,
            );
        }
    } // end if !Join

    ev_level_ready.write(LevelReadyEvent { open_van });
}
