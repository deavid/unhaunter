use bevy::prelude::*;

use crate::materials::CustomMaterial1;

#[derive(Debug, Default, States, Copy, Clone, Eq, PartialEq, Hash)]
pub enum State {
    #[default]
    MainMenu,
    InGame,
    Editor,
}

#[derive(Debug, Clone)]
pub struct LondrinaFontAssets {
    pub w100_thin: Handle<Font>,
    pub w300_light: Handle<Font>,
    pub w400_regular: Handle<Font>,
    pub w900_black: Handle<Font>,
}

#[derive(Debug, Clone)]
pub struct FontAssets {
    pub londrina: LondrinaFontAssets,
}

#[derive(Debug, Clone)]
pub struct ImageAssets {
    pub title: Handle<Image>,
    pub tile1: Handle<Image>,
    pub wall_left: Handle<Image>,
    pub wall_right: Handle<Image>,
    pub minwall_left: Handle<Image>,
    pub minwall_right: Handle<Image>,
    pub frame_left: Handle<Image>,
    pub frame_right: Handle<Image>,
    pub grid1x1: Handle<Image>,
    pub grid1x1x4: Handle<Image>,
    pub pillar: Handle<Image>,
    pub ceiling_light: Handle<Image>,
    pub character_position: Handle<Image>,
    pub old_character: Handle<TextureAtlas>,
    pub character1: Handle<TextureAtlas>,
}

#[derive(Debug, Clone)]
pub struct Anchors {
    pub base: Vec2,
    pub grid1x1: Vec2,
    pub grid1x1x4: Vec2,
    pub character: Vec2,
}

impl Anchors {
    /// Computes the anchors for the given sprite in pixels
    pub fn calc(pos_x: i32, pos_y: i32, size_x: i32, size_y: i32) -> Vec2 {
        Anchors::calc_f32(pos_x as f32, pos_y as f32, size_x as f32, size_y as f32)
    }

    /// Computes the anchors for the given sprite in pixels, f32 variant
    pub fn calc_f32(pos_x: f32, pos_y: f32, size_x: f32, size_y: f32) -> Vec2 {
        let x = pos_x / size_x - 0.5;
        let y = 0.5 - pos_y / size_y;
        Vec2::new(x, y)
    }
}

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

        let mut mesh = Mesh::new(bevy::render::render_resource::PrimitiveTopology::TriangleList);
        mesh.set_indices(Some(indices));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh
    }
}

#[derive(Debug, Clone)]
pub struct Meshes {
    pub quad128: Handle<Mesh>,
}

#[derive(Debug, Clone)]
pub struct Materials {
    pub custom1: Handle<CustomMaterial1>,
}

#[derive(Debug, Clone, Resource)]
pub struct GameAssets {
    pub images: ImageAssets,
    pub fonts: FontAssets,
    pub anchors: Anchors,
    pub meshes: Meshes,
}

pub fn load_assets(
    mut commands: Commands,
    server: Res<AssetServer>,
    mut meshes: ResMut<Assets<Mesh>>,
    // mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut texture_atlases: ResMut<Assets<TextureAtlas>>,
) {
    // Anchors::calc(63, 95, 128, 128),
    let quad = QuadCC::new(Vec2::new(128.0, 128.0), Vec2::new(63.0, 95.0));
    let base_quad = Mesh::from(quad);

    commands.insert_resource(GameAssets {
        images: ImageAssets {
            title: server.load("img/title.png"),
            tile1: server.load("img/base-tiles/base/floor.png"),
            wall_left: server.load("img/base-tiles/base/wall-left.png"),
            wall_right: server.load("img/base-tiles/base/wall-right.png"),
            minwall_left: server.load("img/base-tiles/base/minwall-left.png"),
            minwall_right: server.load("img/base-tiles/base/minwall-right.png"),
            frame_left: server.load("img/base-tiles/base/frame-left.png"),
            frame_right: server.load("img/base-tiles/base/frame-right.png"),
            grid1x1: server.load("img/grid1x1.png"),
            grid1x1x4: server.load("img/grid1x1x4.png"),
            pillar: server.load("img/base-tiles/base/block.png"),
            ceiling_light: server.load("img/light_x4.png"),
            character_position: server.load("img/character_position.png"),
            old_character: texture_atlases.add(TextureAtlas::from_grid(
                server.load("img/character.png"),
                Vec2::new(26.0, 48.0),
                9,
                1,
                Some(Vec2::new(0.0, 0.0)),
                Some(Vec2::new(0.0, 0.0)),
            )),
            character1: texture_atlases.add(TextureAtlas::from_grid(
                server.load("img/characters-model1-demo.png"),
                Vec2::new(32.0, 32.0),
                16,
                4,
                Some(Vec2::new(0.0, 0.0)),
                Some(Vec2::new(0.0, 0.0)),
            )),
        },
        fonts: FontAssets {
            londrina: LondrinaFontAssets {
                w100_thin: server.load("fonts/londrina_solid/LondrinaSolid-Thin.ttf"),
                w300_light: server.load("fonts/londrina_solid/LondrinaSolid-Light.ttf"),
                w400_regular: server.load("fonts/londrina_solid/LondrinaSolid-Regular.ttf"),
                w900_black: server.load("fonts/londrina_solid/LondrinaSolid-Black.ttf"),
            },
        },
        anchors: Anchors {
            base: Anchors::calc(63, 95, 128, 128),
            grid1x1: Anchors::calc(18, 31, 36, 44),
            grid1x1x4: Anchors::calc(18, 85, 36, 98),
            character: Anchors::calc(13, 43, 26, 48),
        },
        meshes: Meshes {
            quad128: meshes.add(base_quad),
        },
        // materials: Materials {
        //     custom1: ,
        // },
    });
}
