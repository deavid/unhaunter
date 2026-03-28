//! A shader and a material that uses it.

use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
        RenderPipelineDescriptor, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey},
};

// -- additive material example --
#[derive(AsBindGroup, TypePath, Debug, Clone, Asset)]
pub struct CustomMaterial2 {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the
    // value to its shader-compatible equivalent. Most core math types already
    // implement `ShaderType`.
    #[uniform(0)]
    color: LinearRgba,
    // Images can be bound as textures in shaders. If the Image's sampler is also
    // needed, just add the sampler attribute with a different binding index.
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
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment
            && let Some(target_state) = &mut fragment.targets[0]
        {
            target_state.blend = Some(BLEND_ADD);
        }

        if let Some(depth_stencil) = &mut descriptor.depth_stencil {
            depth_stencil.depth_write_enabled = false;
        }

        Ok(())
    }
    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }
}

#[derive(AsBindGroup, Asset, TypePath, Debug, Clone)]
pub struct UIPanelMaterial {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the
    // value to its shader-compatible equivalent. Most core math types already
    // implement `ShaderType`.
    #[uniform(0)]
    pub color: LinearRgba,
}

// All functions on `UiMaterial` have default impls. You only need to implement
// the functions that are relevant for your material.
impl UiMaterial for UIPanelMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/uipanel_material.wgsl".into()
    }
}
