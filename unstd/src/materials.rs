//! A shader and a material that uses it.
use bevy::{
    prelude::*,
    reflect::TypePath,
    render::{
        mesh::MeshVertexBufferLayoutRef,
        render_resource::{
            AsBindGroup, BlendComponent, BlendFactor, BlendOperation, BlendState,
            RenderPipelineDescriptor, ShaderRef, ShaderType, SpecializedMeshPipelineError,
        },
    },
    sprite::{AlphaMode2d, Material2d, Material2dKey},
};

#[derive(AsBindGroup, ShaderType, Debug, Clone)]
pub struct CustomMaterial1Data {
    pub color: LinearRgba,
    pub ambient_color: LinearRgba,
    pub gamma: f32,
    pub gtl: f32,
    pub gtr: f32,
    pub gbl: f32,
    pub gbr: f32,
    pub sheet_rows: u32,
    pub sheet_cols: u32,
    pub sheet_idx: u32,
    pub sprite_width: f32,
    pub sprite_height: f32,
    pub y_anchor: f32,
}

impl CustomMaterial1Data {
    pub fn delta(&self, other: &Self) -> f32 {
        let mut delta = 0.0;
        let color1 = self.color.to_f32_array();
        let color2 = other.color.to_f32_array();
        delta += (color1[0] - color2[0]).abs();
        delta += (color1[1] - color2[1]).abs();
        delta += (color1[2] - color2[2]).abs();
        let acolor1 = self.ambient_color.to_f32_array();
        let acolor2 = other.ambient_color.to_f32_array();
        delta += (acolor1[0] - acolor2[0]).abs();
        delta += (acolor1[1] - acolor2[1]).abs();
        delta += (acolor1[2] - acolor2[2]).abs();
        delta += (self.gamma - other.gamma).abs();
        delta += (self.gtl - other.gtl).abs();
        delta += (self.gtr - other.gtr).abs();
        delta += (self.gbl - other.gbl).abs();
        delta += (self.gbr - other.gbr).abs();
        delta += (self.sheet_rows as f32 - other.sheet_rows as f32).abs();
        delta += (self.sheet_cols as f32 - other.sheet_cols as f32).abs();
        delta += (self.sheet_idx as f32 - other.sheet_idx as f32).abs();
        delta *= color1[3] + color2[3] + 0.1;
        delta += (color1[3] - color2[3]).abs() * 15.0;
        delta
    }
}

impl Default for CustomMaterial1Data {
    fn default() -> Self {
        Self {
            color: Color::WHITE.into(),
            ambient_color: Color::BLACK.with_alpha(0.0).into(),
            gamma: 1.0,
            gtl: 2.0,
            gtr: 1.0,
            gbl: 0.1,
            gbr: 5.0,
            sheet_rows: 1,
            sheet_cols: 1,
            sheet_idx: 0,
            sprite_width: 10000.0,
            sprite_height: 10000.0,
            y_anchor: -0.25,
        }
    }
}

#[derive(AsBindGroup, TypePath, Debug, Clone, Component, Asset)]
pub struct CustomMaterial1 {
    // Uniform bindings must implement `ShaderType`, which will be used to convert the
    // value to its shader-compatible equivalent. Most core math types already
    // implement `ShaderType`.
    #[uniform(0)]
    pub data: CustomMaterial1Data,
    // Images can be bound as textures in shaders. If the Image's sampler is also
    // needed, just add the sampler attribute with a different binding index.
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

    /// Get the texture handle for this material
    pub fn texture(&self) -> &Handle<Image> {
        &self.color_texture
    }
}

impl Material2d for CustomMaterial1 {
    fn fragment_shader() -> ShaderRef {
        "shaders/custom_material1.wgsl".into()
    }

    fn alpha_mode(&self) -> AlphaMode2d {
        AlphaMode2d::Blend
    }

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BlendState::ALPHA_BLENDING);
                // target_state.blend = Some(BlendState::PREMULTIPLIED_ALPHA_BLENDING);
            }
        }
        Ok(())
    }
}

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
        if let Some(fragment) = &mut descriptor.fragment {
            if let Some(target_state) = &mut fragment.targets[0] {
                target_state.blend = Some(BLEND_ADD);
            }
        }
        Ok(())
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
