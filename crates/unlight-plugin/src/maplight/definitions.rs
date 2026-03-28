use bevy::ecs::system::SystemParam;
use bevy::prelude::*;
use ndarray::Array3;
use unboard_core::resources::board_topology::{
    BoardCollisionField, BoardEntityField, BoardTopology,
};
use unfog_core::miasma::MiasmaGrid;
use unfog_core::resources::MiasmaConfig;
use unlight_core::types::light_type::LightType;
use unspatial_core::direction::Direction;
use unspatial_core::position::Position;

#[derive(Debug, Clone)]
pub(crate) struct FlashlightData {
    pub pos: Position,
    pub dir: Direction,
    pub power: f32,
    pub color: Color,
    pub light_type: LightType,
    pub vis_field: Array3<f32>,
    /// Precalculated unrotation axes
    pub unrot_axes: [Direction; 3],
    /// Precalculated beam focus factor
    pub beam_focus: f32,
    /// Precalculated unrotated light position offset
    pub lpos_unrot: Position,
    /// Precalculated power factor including focus
    pub power_f: f32,

    pub bounce_valid: bool,
    pub bounce_unrot_axes: [Direction; 3],
    pub bounce_lpos_unrot: Position,
    pub bounce_dir: Direction,
    pub bounce_power_f: f32,
}

impl FlashlightData {
    pub(crate) fn new(
        pos: Position,
        dir: Direction,
        power: f32,
        color: Color,
        light_type: LightType,
        board_dim: (usize, usize, usize),
    ) -> Self {
        let fldir = dir.with_max_dist(400.0);

        // Exact replication of unrotate_by_dir logic for precomputation
        let mut udir = Direction {
            dx: fldir.dx,
            dy: -fldir.dy,
            dz: -fldir.dz,
        };
        udir = udir.normalized();

        let unrot_axes = [
            Direction {
                dx: udir.dx,
                dy: udir.dy,
                dz: udir.dz,
            },
            Direction {
                dx: -udir.dy,
                dy: udir.dx,
                dz: udir.dz,
            },
            Direction {
                dx: -udir.dy,
                dy: udir.dz,
                dz: udir.dx,
            },
        ];

        let focus = (fldir.distance() + 0.1).max(6.0) / 20.0;
        let lpos = pos + fldir / 30.0;

        let lpos_unrot = Position {
            x: lpos.x * unrot_axes[0].dx + lpos.y * unrot_axes[1].dx + lpos.z * unrot_axes[2].dx,
            y: lpos.x * unrot_axes[0].dy + lpos.y * unrot_axes[1].dy + lpos.z * unrot_axes[2].dy,
            z: lpos.x * unrot_axes[0].dz + lpos.y * unrot_axes[1].dz + lpos.z * unrot_axes[2].dz,
            visual_priority: lpos.visual_priority,
        };

        Self {
            pos,
            dir: fldir,
            power,
            color,
            light_type,
            vis_field: Array3::from_elem(board_dim, -0.001_f32),
            unrot_axes,
            beam_focus: focus,
            lpos_unrot,
            power_f: power * (focus + 0.5).clamp(0.5, 8.0),
            bounce_valid: false,
            bounce_unrot_axes: unrot_axes,
            bounce_lpos_unrot: lpos_unrot,
            bounce_dir: fldir,
            bounce_power_f: 0.0,
        }
    }

    pub(crate) fn set_bounce(&mut self, wall_dist: f32, bounce_power_factor: f32) {
        if !(0.1..=100.0).contains(&wall_dist) {
            self.bounce_valid = false;
            return;
        }

        let bounce_pos = self.pos + self.dir.normalized() * (wall_dist * 2.0);

        let mut b_dir = Direction {
            dx: -self.dir.dx,
            dy: -self.dir.dy,
            dz: -self.dir.dz,
        };
        b_dir = b_dir.normalized();

        let b_fldir = Direction {
            dx: b_dir.dx * self.dir.distance(),
            dy: b_dir.dy * self.dir.distance(),
            dz: b_dir.dz * self.dir.distance(),
        };

        let mut b_udir = Direction {
            dx: b_fldir.dx,
            dy: -b_fldir.dy,
            dz: -b_fldir.dz,
        };
        b_udir = b_udir.normalized();

        let b_unrot_axes = [
            Direction {
                dx: b_udir.dx,
                dy: b_udir.dy,
                dz: b_udir.dz,
            },
            Direction {
                dx: -b_udir.dy,
                dy: b_udir.dx,
                dz: b_udir.dz,
            },
            Direction {
                dx: -b_udir.dy,
                dy: b_udir.dz,
                dz: b_udir.dx,
            },
        ];

        let b_lpos = bounce_pos + b_fldir / 30.0;

        let b_lpos_unrot = Position {
            x: b_lpos.x * b_unrot_axes[0].dx
                + b_lpos.y * b_unrot_axes[1].dx
                + b_lpos.z * b_unrot_axes[2].dx,
            y: b_lpos.x * b_unrot_axes[0].dy
                + b_lpos.y * b_unrot_axes[1].dy
                + b_lpos.z * b_unrot_axes[2].dy,
            z: b_lpos.x * b_unrot_axes[0].dz
                + b_lpos.y * b_unrot_axes[1].dz
                + b_lpos.z * b_unrot_axes[2].dz,
            visual_priority: b_lpos.visual_priority,
        };

        self.bounce_valid = true;
        self.bounce_dir = b_fldir;
        self.bounce_unrot_axes = b_unrot_axes;
        self.bounce_lpos_unrot = b_lpos_unrot;
        self.bounce_power_f = self.power_f * bounce_power_factor;
    }
}

#[derive(Resource, Default, Debug, Clone)]
pub(crate) struct ActiveFlashlights {
    pub list: Vec<FlashlightData>,
}

#[derive(SystemParam)]
pub(crate) struct GridResources<'w> {
    pub bf: Res<'w, BoardTopology>,
    pub bef: Res<'w, BoardEntityField>,
    pub bcf: Res<'w, BoardCollisionField>,
    pub miasma: If<Res<'w, MiasmaGrid>>,
    pub miasma_config: Res<'w, MiasmaConfig>,
}
