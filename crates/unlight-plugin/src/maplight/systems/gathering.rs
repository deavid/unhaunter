use crate::maplight::definitions::{ActiveFlashlights, FlashlightData, GridResources};
use crate::maplight::visibility::compute_visibility;
use crate::metrics::PLAYER_VISIBILITY;
use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::resources::board_topology::BoardCollisionField;
use unboard_core::resources::roomdb::RoomTopology;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::{EquipmentPosition, Hand};
use uninteraction_core::interaction::Toggleable;
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::types::LightType;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unrender_core::resources::visibility_data::VisibilityData;
use unrender_std::components::light::LightEmitter;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

pub(crate) fn player_visibility_system(
    mut q_vf: Query<(&Position, &mut VisibilityData), With<MainPlayer>>,
    bcf: Res<BoardCollisionField>,
    mut room_topology: ResMut<RoomTopology>,
) {
    if bcf.0.dim().0 == 0 || bcf.0.dim().1 == 0 || bcf.0.dim().2 == 0 {
        return;
    }
    let measure = PLAYER_VISIBILITY.clone().time_measure();

    for (pos, mut vf) in q_vf.iter_mut() {
        if vf.visibility_field.dim() != bcf.0.dim() {
            vf.visibility_field = Array3::from_elem(bcf.0.dim(), -0.001_f32);
        } else {
            vf.visibility_field.fill(-0.001_f32);
        }
        // Calculate visibility
        compute_visibility(
            &mut vf.visibility_field,
            &bcf.0,
            pos,
            Some(&mut room_topology),
            false,
        );
    }
    measure.end_ms();
}

pub(crate) fn gather_flashlights_system(
    q_deployed: Query<(&Position, &DeployedGear, &LightEmitter, &Toggleable)>,
    qp: Query<(&Position, &Direction, &PlayerGear)>,
    q_spectator: Query<&Position, (With<MainPlayer>, With<PlayerSpectating>)>,
    q_flashlight: Query<(&LightEmitter, &Toggleable)>,
    grids: GridResources,
    mut active_flashlights: ResMut<ActiveFlashlights>,
) {
    let bf = &grids.bf;
    let bcf = &grids.bcf;
    let board_dim = bcf.0.dim();
    if bf.map_size.0 == 0 {
        return;
    }

    let mut flashlights = vec![];
    const FLASHLIGHT_POWER_FACTOR: f32 = 0.1;

    // Deployed gear
    for (pos, deployed_gear, fl, toggle) in q_deployed.iter() {
        if !toggle.is_on {
            continue;
        }
        let power = fl.power;
        let color = fl.color;
        let light_type = fl.light_type;

        if power > 0.0 {
            flashlights.push(FlashlightData::new(
                *pos,
                deployed_gear.direction,
                power * FLASHLIGHT_POWER_FACTOR,
                color,
                light_type,
                board_dim,
            ));
        }
    }

    for (pos, direction, gear) in qp.iter() {
        let mut player_flashlight: Vec<(f32, Color, EquipmentPosition, LightType)> = vec![];

        let mut check_gear = |entity: Entity, p: EquipmentPosition| {
            if let Ok((fl, toggle)) = q_flashlight.get(entity)
                && toggle.is_on
            {
                player_flashlight.push((
                    fl.power * FLASHLIGHT_POWER_FACTOR,
                    fl.color,
                    p,
                    fl.light_type,
                ));
            }
        };

        if let Some(e) = gear.left_hand {
            check_gear(e, EquipmentPosition::Hand(Hand::Left));
        }
        if let Some(e) = gear.right_hand {
            check_gear(e, EquipmentPosition::Hand(Hand::Right));
        }
        for e in &gear.inventory {
            check_gear(*e, EquipmentPosition::Stowed);
        }

        for (power, color, p, light_type) in player_flashlight {
            if power > 0.0 {
                let mut fldir = *direction;
                if p == EquipmentPosition::Stowed {
                    fldir = Direction {
                        dx: fldir.dx / 1000.0,
                        dy: fldir.dy / 1000.0,
                        dz: fldir.dz / 1000.0,
                    };
                }
                flashlights.push(FlashlightData::new(
                    *pos, fldir, power, color, light_type, board_dim,
                ));
            }
        }
    }

    if let Ok(pos) = q_spectator.single() {
        // Fictional flashlight for spectator that always points down
        // Using a slightly tilted direction to avoid degenerate matrix in FlashlightData
        let dir = Direction {
            dx: 0.001,
            dy: 0.0001,
            dz: -0.0001,
        };
        flashlights.push(FlashlightData::new(
            *pos,
            dir,
            32.0,
            Color::srgb(1.0, 0.04, 0.001),
            LightType::Red,
            board_dim,
        ));
    }

    for flash in flashlights.iter_mut() {
        compute_visibility(&mut flash.vis_field, &bcf.0, &flash.pos, None, false);
    }

    active_flashlights.list = flashlights;
}

pub(crate) fn update_exposure_system(
    qp: Query<(&Position, Has<MainPlayer>)>,
    q_vf: Query<&VisibilityData, With<MainPlayer>>,
    active_flashlights: Res<ActiveFlashlights>,
    mut lg: If<ResMut<LightGrid>>,
    time: Res<Time>,
) {
    let Ok(vf) = q_vf.single() else {
        return;
    };
    if vf.visibility_field.is_empty() {
        return;
    }

    let Some((pos, _)) = qp.iter().find(|x| x.1) else {
        return;
    };

    let board_dim = lg.light_field.dim();
    let mut cursor_exp: f32 = 0.0;
    let mut exp_count: f32 = 0.0001;

    // Weight highlights more when calculating exposure (Power Average)
    const HIGHLIGHT_PRIORITY_POWER: f32 = 1.0;

    let cursor_pos = pos.to_board_position();
    for npos in cursor_pos.iter_xy_neighbors(10, board_dim) {
        let lf = &lg.light_field[npos.ndidx()];
        let vis = vf.visibility_field[npos.ndidx()].max(0.00001);

        // Power average to prioritize highlights in the field of view.
        cursor_exp += lf.lux.powf(HIGHLIGHT_PRIORITY_POWER) * vis;
        exp_count += vis;
    }

    cursor_exp = (cursor_exp / exp_count).powf(HIGHLIGHT_PRIORITY_POWER.recip());

    let fl_total_power: f32 = active_flashlights
        .list
        .iter()
        .map(|x| {
            let mut power = x.power;
            power *= match x.light_type {
                LightType::Visible => 1.0,
                LightType::Red => 0.0,
                LightType::InfraRedNV => 2.5,
                LightType::UltraViolet => 0.5,
            };
            power / (pos.distance2(&x.pos) + 1.0)
        })
        .sum();
    cursor_exp += fl_total_power.sqrt();

    // FIR Filter with Hann Window (240 frames)
    lg.exposure.add_sample(cursor_exp);
    lg.exposure.update(time.delta_secs());
}
