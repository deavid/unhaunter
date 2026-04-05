use crate::maplight::definitions::GridResources;
use crate::maplight::visibility::compute_visibility;
use crate::metrics::PLAYER_VISIBILITY;
use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::resources::board_topology::BoardCollisionField;
use unboard_core::resources::roomdb::RoomTopology;
use unboard_core::resources::visibility_data::VisibilityData;
use ungear_core::components::deployedgear::DeployedGear;
use ungear_core::components::playergear::PlayerGear;
use ungear_core::types::gear::equipment::{EquipmentPosition, Hand};
use uninteraction_core::interaction::Toggleable;
use unlight_core::components::{FlashlightBounceState, LightEmitter};
use unlight_core::flashlight::{ActiveFlashlights, FlashlightData};
use unlight_core::resources::light_grid::LightGrid;
use unlight_core::types::light_type::LightType;
use unmetrics_core::metrics::SendMetric;
use unplayer_core::components::{MainPlayer, PlayerSpectating};
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

pub(crate) fn player_visibility_system(
    mut q_vf: Query<(&Position, &Direction, &mut VisibilityData), With<MainPlayer>>,
    bcf: Res<BoardCollisionField>,
    mut room_topology: ResMut<RoomTopology>,
    lg: Option<Res<LightGrid>>,
) {
    if bcf.0.dim().0 == 0 || bcf.0.dim().1 == 0 || bcf.0.dim().2 == 0 {
        return;
    }
    let measure = PLAYER_VISIBILITY.clone().time_measure();

    let exposure = lg.map(|x| x.exposure.current).unwrap_or(1.0);

    for (pos, dir, mut vf) in q_vf.iter_mut() {
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
            Some(dir),
            Some(exposure),
            false,
        );
    }
    measure.end_ms();
}

pub(crate) fn gather_flashlights_system(
    mut commands: Commands,
    mut q_deployed: Query<(
        Entity,
        &Position,
        &DeployedGear,
        &LightEmitter,
        &Toggleable,
        Option<&mut FlashlightBounceState>,
    )>,
    qp: Query<(&Position, &Direction, &PlayerGear)>,
    q_spectator: Query<&Position, (With<MainPlayer>, With<PlayerSpectating>)>,
    mut q_flashlight: Query<
        (
            Entity,
            &LightEmitter,
            &Toggleable,
            Option<&mut FlashlightBounceState>,
        ),
        Without<DeployedGear>,
    >,
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

    let raymarch_bounce = |entity: Entity,
                           pos: Position,
                           dir: Direction,
                           power: f32,
                           color: Color,
                           light_type: LightType,
                           bounce_state: Option<Mut<FlashlightBounceState>>,
                           commands: &mut Commands|
     -> FlashlightData {
        let mut flash = FlashlightData::new(pos, dir, power, color, light_type, board_dim);
        compute_visibility(
            &mut flash.vis_field,
            &bcf.0,
            &flash.pos,
            None,
            None,
            None,
            false,
        );

        // Simple raymarch down the barrel of the flashlight beam
        let fdir = flash.dir.normalized();
        let mut raw_wall_dist = 60.0; // max plausible range without wall

        let mut t = 0.0;
        let step = 0.5;
        while t < 60.0 {
            let sample_pos = flash.pos + fdir * t;
            let sample_bpos = sample_pos.to_board_position();
            if let Some(idx) = sample_bpos.ndidx_checked(bf.map_size) {
                let collision = &bcf.0[idx];
                if !collision.see_through {
                    raw_wall_dist = t;
                    break;
                }
            } else {
                raw_wall_dist = t;
                break;
            }
            t += step;
        }

        let smoothed_dist = if let Some(mut state) = bounce_state {
            let prev_dist = state.smoothed_dist;
            let smooth = 0.15;
            let new_dist = prev_dist * (1.0 - smooth) + raw_wall_dist * smooth;
            state.smoothed_dist = new_dist;
            new_dist
        } else {
            commands.entity(entity).insert(FlashlightBounceState {
                smoothed_dist: raw_wall_dist,
            });
            raw_wall_dist
        };

        // E.g. Albedo 0.1 means bounce reflects 10% of light
        flash.set_bounce(smoothed_dist, 0.10);
        flash
    };

    // Deployed gear
    for (entity, pos, deployed_gear, fl, toggle, bounce_state) in q_deployed.iter_mut() {
        if !toggle.is_on {
            continue;
        }
        let power = fl.power;
        let color = fl.color;
        let light_type = fl.light_type;

        if power > 0.0 {
            flashlights.push(raymarch_bounce(
                entity,
                *pos,
                deployed_gear.direction,
                power * FLASHLIGHT_POWER_FACTOR,
                color,
                light_type,
                bounce_state,
                &mut commands,
            ));
        }
    }

    for (pos, direction, gear) in qp.iter() {
        let mut check_gear = |entity: Entity, p: EquipmentPosition| {
            if let Ok((_, fl, toggle, bounce_state)) = q_flashlight.get_mut(entity)
                && toggle.is_on
            {
                let mut fldir = *direction;
                if p == EquipmentPosition::Stowed {
                    fldir = Direction {
                        dx: fldir.dx / 1000.0,
                        dy: fldir.dy / 1000.0,
                        dz: fldir.dz / 1000.0,
                    };
                }
                flashlights.push(raymarch_bounce(
                    entity,
                    *pos,
                    fldir,
                    fl.power * FLASHLIGHT_POWER_FACTOR,
                    fl.color,
                    fl.light_type,
                    bounce_state,
                    &mut commands,
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
    }

    if let Ok(pos) = q_spectator.single() {
        // Fictional flashlight for spectator that always points down
        // Using a slightly tilted direction to avoid degenerate matrix in FlashlightData
        let dir = Direction {
            dx: 0.001,
            dy: 0.0001,
            dz: -0.0001,
        };
        let mut flash = FlashlightData::new(
            *pos,
            dir,
            32.0,
            Color::srgb(1.0, 0.04, 0.001),
            LightType::Red,
            board_dim,
        );
        compute_visibility(
            &mut flash.vis_field,
            &bcf.0,
            &flash.pos,
            None,
            None,
            None,
            false,
        );
        flashlights.push(flash);
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
    const HIGHLIGHT_PRIORITY_POWER: f32 = 0.5;
    const CORRECTION_FACTOR: f32 = 0.7;

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
    lg.exposure.add_sample(cursor_exp * CORRECTION_FACTOR);
    lg.exposure.update(time.delta_secs());
}
