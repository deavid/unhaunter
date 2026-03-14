use bevy::prelude::*;
use bevy::time::Stopwatch;
use bevy_replicon::prelude::*;
use ungear_core::resources::spawner::GearMarker;
use ungear_core::types::gear::kind::GearKind;
use ungearitems_core::components::flashlight::Flashlight;
use unrender_std::components::light::LightEmitter;
use unreplicon_core::ownership::{LocallyOwned, Owner};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

#[derive(Resource, Default)]
struct DebugTimer(Stopwatch);

pub(super) fn app_setup(app: &mut App) {
    app.init_resource::<DebugTimer>();
    app.add_systems(Update, debug_gear_components);
}

fn debug_gear_components(
    time: Res<Time>,
    mut timer: ResMut<DebugTimer>,
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

        info!(
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
