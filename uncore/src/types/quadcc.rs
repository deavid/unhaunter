use bevy::{asset::RenderAssetUsages, prelude::*};

/// A rectangle on the `XY` plane with custom center.
#[derive(Debug, Copy, Clone)]
pub struct QuadCC {
    /// Full width and height of the rectangle.
    pub size: Vec2,
    /// Horizontally-flip the texture coordinates of the resulting mesh.
    pub flip: bool,
    /// Center point of the quad
    pub center: Vec2,
}

impl Default for QuadCC {
    fn default() -> Self {
        QuadCC::new(Vec2::ONE, Vec2::default())
    }
}

impl QuadCC {
    pub fn new(size: Vec2, center: Vec2) -> Self {
        Self {
            size,
            flip: false,
            center,
        }
    }
}

impl From<QuadCC> for Mesh {
    fn from(quad: QuadCC) -> Self {
        let left_x = -quad.center.x;
        let right_x = quad.size.x - quad.center.x;
        let bottom_y = quad.center.y - quad.size.y;
        let top_y = quad.center.y;
        let (u_left, u_right) = if quad.flip { (1.0, 0.0) } else { (0.0, 1.0) };
        let vertices = [
            ([left_x, bottom_y, 0.0], [0.0, 0.0, 1.0], [u_left, 1.0]),
            ([left_x, top_y, 0.0], [0.0, 0.0, 1.0], [u_left, 0.0]),
            ([right_x, top_y, 0.0], [0.0, 0.0, 1.0], [u_right, 0.0]),
            ([right_x, bottom_y, 0.0], [0.0, 0.0, 1.0], [u_right, 1.0]),
        ];
        let indices = bevy::render::mesh::Indices::U32(vec![0, 2, 1, 0, 3, 2]);
        let positions: Vec<_> = vertices.iter().map(|(p, _, _)| *p).collect();
        let normals: Vec<_> = vertices.iter().map(|(_, n, _)| *n).collect();
        let uvs: Vec<_> = vertices.iter().map(|(_, _, uv)| *uv).collect();
        let mut mesh = Mesh::new(
            bevy::render::render_resource::PrimitiveTopology::TriangleList,
            RenderAssetUsages::all(),
        );
        mesh.insert_indices(indices);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}
