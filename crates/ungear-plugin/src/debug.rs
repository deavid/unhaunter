use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_replicon::prelude::*;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::flashlight::Flashlight;
use unlight_core::components::LightEmitter;
use unplayer_core::components::{
    MainPlayer, PlayerDisconnected, PlayerInactive, PlayerSpectating, PlayerSprite,
};
use unreplicon_core::ownership::{LocallyOwned, Owner};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;
use unvitals_core::components::PlayerVitals;

#[derive(Resource, Default)]
struct GearDebugTimer(Stopwatch);

#[derive(Resource, Default)]
struct PlayerDebugTimer(Stopwatch);

pub(crate) fn app_setup(app: &mut App) {
    app.init_resource::<GearDebugTimer>();
    app.init_resource::<PlayerDebugTimer>();
    app.add_systems(Update, debug_gear_components);
    app.add_systems(Update, debug_player_components);
}

fn debug_gear_components(
    time: Res<Time>,
    mut timer: ResMut<GearDebugTimer>,
    q_gear: Query<
        (
            Entity,
            &GearKind,
            Option<&Flashlight>,
            Option<&LocallyOwned>,
            Option<&Owner>,
            Option<&Replicated>,
            Option<&bevy_replicon::prelude::Remote>,
            Option<&Position>,
            Option<&Direction>,
            Option<&LightEmitter>,
        ),
        With<GearMarker>,
    >,
) {
    timer.0.tick(time.delta());
    if timer.0.elapsed_secs() < 10.0 {
        return;
    }
    timer.0.reset();

    for (entity, kind, flashlight, local, owner, replicated, remote, pos, dir, light) in
        q_gear.iter()
    {
        let ownership = if local.is_some() {
            "LOCALLY_OWNED"
        } else {
            "REMOTE_AUTH"
        };
        let repl_status = if replicated.is_some() {
            "REPLICATED"
        } else {
            "LOCAL_ONLY"
        };
        let is_remote = if remote.is_some() {
            "REMOTE_COMP"
        } else {
            "LOCAL_COMP"
        };
        let owner_id = owner
            .map(|o| format!("{:?}", o.0))
            .unwrap_or_else(|| "None".to_string());

        let flashlight_data = flashlight
            .map(|f| format!("{:?}", f.status))
            .unwrap_or_else(|| "N/A".to_string());
        let pos_data = pos
            .map(|p| format!("({:.1}, {:.1}, {:.1})", p.x, p.y, p.z))
            .unwrap_or_else(|| "None".to_string());
        let dir_data = dir
            .map(|d| format!("({:.1}, {:.1}, {:.1})", d.dx, d.dy, d.dz))
            .unwrap_or_else(|| "None".to_string());
        let light_data = light
            .map(|l| format!("pow={:.1}, type={:?}", l.power, l.light_type))
            .unwrap_or_else(|| "None".to_string());

        trace!(
            "Entity[{:?}] Kind={:?} | {} | {} | {} | Owner={}\n  -> Flashlight={}  |  Pos={}  |  Dir={}  |  Light={}",
            entity,
            kind,
            ownership,
            repl_status,
            is_remote,
            owner_id,
            flashlight_data,
            pos_data,
            dir_data,
            light_data
        );
    }
}

type PlayerDebugQuery = (
    Entity,
    &'static PlayerSprite,
    Option<&'static Transform>,
    Option<&'static Position>,
    Option<&'static Direction>,
    Option<&'static LocallyOwned>,
    Option<&'static Owner>,
    Option<&'static Replicated>,
    Option<&'static bevy_replicon::prelude::Remote>,
    Has<MainPlayer>,
    Has<PlayerSpectating>,
    Has<PlayerDisconnected>,
    Has<PlayerInactive>,
    Has<PlayerGear>,
    Has<PlayerVitals>,
);

fn debug_player_components(
    time: Res<Time>,
    mut timer: ResMut<PlayerDebugTimer>,
    q_players: Query<PlayerDebugQuery, With<PlayerSprite>>,
) {
    timer.0.tick(time.delta());
    if timer.0.elapsed_secs() < 10.0 {
        return;
    }
    timer.0.reset();

    let total = q_players.iter().count();
    info!(
        "debug_player_components: {} PlayerSprite entities found",
        total
    );

    for item in q_players.iter() {
        let (
            entity,
            sprite,
            transform,
            pos,
            dir,
            local,
            owner,
            replicated,
            remote,
            is_main_player,
            is_spectating,
            is_disconnected,
            is_inactive,
            has_gear,
            has_vitals,
        ) = item;
        let ownership = if local.is_some() {
            "LOCALLY_OWNED"
        } else {
            "REMOTE_AUTH"
        };
        let repl_status = if replicated.is_some() {
            "REPLICATED"
        } else {
            "LOCAL_ONLY"
        };
        let is_remote = if remote.is_some() {
            "REMOTE_COMP"
        } else {
            "LOCAL_COMP"
        };
        let owner_id = owner
            .map(|o| format!("{:?}", o.0))
            .unwrap_or_else(|| "None".to_string());
        let pos_data = pos
            .map(|p| format!("({:.1}, {:.1}, {:.1})", p.x, p.y, p.z))
            .unwrap_or_else(|| "None".to_string());
        let transform_data = transform
            .map(|t| {
                format!(
                    "({:.1}, {:.1}, {:.1})",
                    t.translation.x, t.translation.y, t.translation.z
                )
            })
            .unwrap_or_else(|| "None".to_string());
        let dir_data = dir
            .map(|d| format!("({:.1}, {:.1}, {:.1})", d.dx, d.dy, d.dz))
            .unwrap_or_else(|| "None".to_string());

        let flags = [
            if is_main_player {
                Some("MAIN_PLAYER")
            } else {
                None
            },
            if is_spectating {
                Some("SPECTATING")
            } else {
                None
            },
            if is_disconnected {
                Some("DISCONNECTED")
            } else {
                None
            },
            if is_inactive { Some("INACTIVE") } else { None },
            if has_gear { Some("HAS_GEAR") } else { None },
            if has_vitals { Some("HAS_VITALS") } else { None },
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("+");
        let flags = if flags.is_empty() {
            "none".to_string()
        } else {
            flags
        };

        let vitals_data = "sanity=N/A | health=N/A".to_string();

        info!(
            "Player[{:?}] UUID={:?} NetId={:?} | {} | {} | {} | Owner={} | Flags={}\n  -> Pos={}  |  Transform={}  |  Dir={}  |  {}",
            entity,
            sprite.id,
            sprite.network_id,
            ownership,
            repl_status,
            is_remote,
            owner_id,
            flags,
            pos_data,
            transform_data,
            dir_data,
            vitals_data,
        );
    }
}
