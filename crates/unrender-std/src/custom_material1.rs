use bevy::{
    mesh::MeshVertexBufferLayoutRef,
    prelude::*,
    reflect::TypePath,
    render::render_resource::{
        AsBindGroup, BlendState, RenderPipelineDescriptor, ShaderType, SpecializedMeshPipelineError,
    },
    shader::ShaderRef,
    sprite_render::{AlphaMode2d, Material2d, Material2dKey},
};

#[derive(AsBindGroup, ShaderType, Debug, Clone)]
pub struct CustomMaterial1Data {
    pub color: LinearRgba,
    pub ctl: LinearRgba,
    pub ctr: LinearRgba,
    pub cbl: LinearRgba,
    pub cbr: LinearRgba,
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
    pub padding: f32,
    pub margin: f32,
    pub y_anchor: f32,
    pub upscale_factor: f32,
}

impl CustomMaterial1Data {
    pub fn delta(&self, other: &Self) -> f32 {
        let mut delta = 0.0;
        let color1 = self.color.to_f32_array();
        let color2 = other.color.to_f32_array();
        delta += (color1[0] - color2[0]).abs();
        delta += (color1[1] - color2[1]).abs();
        delta += (color1[2] - color2[2]).abs();
        let ctl1 = self.ctl.to_f32_array();
        let ctl2 = other.ctl.to_f32_array();
        delta += (ctl1[0] - ctl2[0]).abs();
        delta += (ctl1[1] - ctl2[1]).abs();
        delta += (ctl1[2] - ctl2[2]).abs();
        delta += (ctl1[3] - ctl2[3]).abs();
        let ctr1 = self.ctr.to_f32_array();
        let ctr2 = other.ctr.to_f32_array();
        delta += (ctr1[0] - ctr2[0]).abs();
        delta += (ctr1[1] - ctr2[1]).abs();
        delta += (ctr1[2] - ctr2[2]).abs();
        delta += (ctr1[3] - ctr2[3]).abs();
        let cbl1 = self.cbl.to_f32_array();
        let cbl2 = other.cbl.to_f32_array();
        delta += (cbl1[0] - cbl2[0]).abs();
        delta += (cbl1[1] - cbl2[1]).abs();
        delta += (cbl1[2] - cbl2[2]).abs();
        delta += (cbl1[3] - cbl2[3]).abs();
        let cbr1 = self.cbr.to_f32_array();
        let cbr2 = other.cbr.to_f32_array();
        delta += (cbr1[0] - cbr2[0]).abs();
        delta += (cbr1[1] - cbr2[1]).abs();
        delta += (cbr1[2] - cbr2[2]).abs();
        delta += (cbr1[3] - cbr2[3]).abs();
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
        delta += (self.upscale_factor - other.upscale_factor).abs();
        delta += (self.sprite_width - other.sprite_width).abs();
        delta += (self.sprite_height - other.sprite_height).abs();
        delta += (self.padding - other.padding).abs();
        delta += (self.margin - other.margin).abs();
        delta += (self.y_anchor - other.y_anchor).abs();
        delta *= color1[3] + color2[3] + 0.1;
        delta += (color1[3] - color2[3]).abs() * 15.0;
        delta
    }
}

impl Default for CustomMaterial1Data {
    fn default() -> Self {
        Self {
            color: Color::NONE.into(),
            ctl: Color::NONE.into(),
            ctr: Color::NONE.into(),
            cbl: Color::NONE.into(),
            cbr: Color::NONE.into(),
            ambient_color: Color::BLACK.with_alpha(0.0).into(),
            gamma: 1.0,
            gtl: 1.0,
            gtr: 1.0,
            gbl: 1.0,
            gbr: 1.0,
            sheet_rows: 1,
            sheet_cols: 1,
            sheet_idx: 0,
            sprite_width: 10000.0,
            sprite_height: 10000.0,
            padding: 0.0,
            margin: 0.0,
            y_anchor: -0.25,
            upscale_factor: 1.0,
        }
    }
}

#[derive(AsBindGroup, TypePath, Debug, Clone, Component, Asset)]
pub struct CustomMaterial1 {
    #[uniform(0)]
    pub data: CustomMaterial1Data,
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

    fn specialize(
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayoutRef,
        _key: Material2dKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        if let Some(fragment) = &mut descriptor.fragment
            && let Some(target_state) = &mut fragment.targets[0]
        {
            target_state.blend = Some(BlendState::ALPHA_BLENDING);
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
