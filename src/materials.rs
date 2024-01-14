//! A shader and a material that uses it.

use bevy::{
    prelude::*,
    reflect::{TypePath, TypeUuid},
    render::{
        mesh::MeshVertexBufferLayout,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError,
        },
    },
    sprite::{Material2d, Material2dKey, Material2dPlugin, MaterialMesh2dBundle},
};

#[allow(dead_code)]
fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins,
            Material2dPlugin::<CustomMaterial1>::default(),
            Material2dPlugin::<CustomMaterial2>::default(),
        ))
        .add_systems(Startup, setup)
        .run();
}

// Spawn an entity using `CustomMaterial`.
fn setup(
    mut commands: Commands,
    mut materials1: ResMut<Assets<CustomMaterial1>>,
    mut meshes: ResMut<Assets<Mesh>>,
    asset_server: Res<AssetServer>,
) {
    let size = Vec2::new(128.0, 128.0);
    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(size))).into(),
        // transform: Transform::from_translation(position.extend(10.0)),
        material: materials1.add(CustomMaterial1 {
            data: CustomMaterial1Data {
                color: Color::RED,
                gamma: 10.0,
                gbl: 5.0,
                gtl: 5.0,
                gbr: 0.1,
                gtr: 1.0,
                ..Default::default()
            },
            color_texture: asset_server.load("img/light_x4.png"),
        }),
        transform: Transform::from_xyz(-25.0, 0.0, 0.0),
        ..Default::default()
    });

    commands.spawn(MaterialMesh2dBundle {
        mesh: meshes.add(Mesh::from(shape::Quad::new(size))).into(),
        // transform: Transform::from_translation(position.extend(10.0)),
        material: materials1.add(CustomMaterial1 {
            data: CustomMaterial1Data {
                color: Color::RED,
                gamma: 5.0,
                gbl: 5.0,
                gtl: 5.0,
                gbr: 0.1,
                gtr: 1.0,
                ..Default::default()
            },
            color_texture: asset_server.load("img/light_x4.png"),
        }),
        transform: Transform::from_xyz(25.0, 0.0, 0.01),
        ..Default::default()
    });

    commands.spawn(Camera2dBundle {
        ..Default::default()
    });
}
#[derive(AsBindGroup, ShaderType, Debug, Clone)]
pub struct CustomMaterial1Data {
    pub color: Color,
    pub gamma: f32,
    pub gtl: f32,
    pub gtr: f32,
    pub gbl: f32,
    pub gbr: f32,
    pub sheet_rows: u32,
    pub sheet_cols: u32,
    pub sheet_idx: u32,
}

impl Default for CustomMaterial1Data {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            gamma: 1.0,
            gtl: 2.0,
            gtr: 1.0,
            gbl: 0.1,
            gbr: 5.0,
            sheet_rows: 1,
            sheet_cols: 1,
            sheet_idx: 0,
        }
    }
}

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone, Component, Asset)]
#[uuid = "f690fdae-d598-45ab-8225-97e2a3f056e0"]
pub struct CustomMaterial1 {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the value to
    // its shader-compatible equivalent. Most core math types already implement `ShaderType`.
    #[uniform(0)]
    pub data: CustomMaterial1Data,
    // Images can be bound as textures in shaders. If the Image's sampler is also needed, just
    // add the sampler attribute with a different binding index.
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

impl CustomMaterial1 {
    pub fn from_texture(img_handle: Handle<Image>) -> Self {
        Self {
            color_texture: img_handle,
            data: default(),
        }
    }
}

impl Material2d for CustomMaterial1 {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material1.wgsl".into()
    }
}

// -- additive material example --

#[derive(AsBindGroup, TypeUuid, TypePath, Debug, Clone, Asset)]
#[uuid = "cdf95663-792e-484b-a806-b688a5c6ee54"]
pub struct CustomMaterial2 {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the value to
    // its shader-compatible equivalent. Most core math types already implement `ShaderType`.
    #[uniform(0)]
    color: Color,
    // Images can be bound as textures in shaders. If the Image's sampler is also needed, just
    // add the sampler attribute with a different binding index.
    #[texture(1)]
    #[sampler(2)]
    color_texture: Handle<Image>,
}

const BLEND_ADD: BlendState = BlendState {
    color: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },

    alpha: BlendComponent {
        src_factor: BlendFactor::SrcAlpha,
        dst_factor: BlendFactor::One,
        operation: BlendOperation::Add,
    },
};

impl Material2d for CustomMaterial2 {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material2.wgsl".into()
    }
    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BLEND_ADD);
            }
        }

        Ok(())
    }
}
