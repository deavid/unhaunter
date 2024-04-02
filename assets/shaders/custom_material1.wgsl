#import bevy_pbr::forward_io::VertexOutput

struct CustomMaterial {
    color: vec4<f32>,
    ambient_color: vec4<f32>,
    gamma: f32,
    gtl: f32,
    gtr: f32,
    gbl: f32,
    gbr: f32,
    sheet_rows: u32,
    sheet_cols: u32,
    sheet_idx: u32,
    sprite_width: f32,
    sprite_height: f32,
};

@group(2) @binding(0)
var<uniform> material: CustomMaterial;
@group(2) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(2) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Full texture size computation:
    let tex_width = material.sprite_width * f32(material.sheet_cols);
    let tex_height = material.sprite_height * f32(material.sheet_rows);
    let tex_size = vec2<f32>(tex_width, tex_height);

    // Calculate sprite UVs considering the sprite sheet layout
    let row: u32 = material.sheet_idx / material.sheet_cols;
    let col: u32 = material.sheet_idx % material.sheet_cols;

    // Compute the size of each cell in the atlas (in UV space)
    let cell_width: f32 = 1.0 / f32(material.sheet_cols);
    let cell_height: f32 = 1.0 / f32(material.sheet_rows);

    // Compute the base UV coordinates for the sprite within the atlas
    let base_u: f32 = f32(col) * cell_width;
    let base_v: f32 = f32(row) * cell_height;

    let zero4 = vec4(0.0, 0.0, 0.0, 0.0);
    let one4 = vec4(1.0, 1.0, 1.0, 1.0);

    // Adding a margin to the sprite coordinates to prevent reading from neighboring sprite
    let margin = 0.5;
    let mx = margin / material.sprite_width ;
    let my = margin / material.sprite_height ;

    // Margin protects the sprites from reading the neighboring sprite
    let margin_uv = clamp(mesh.uv, vec2<f32>(0.0, my*2.0), vec2<f32>(1.0-mx, 1.0-my));

    // Correcting UV coordinates for the sprite
    var sprite_uv: vec2<f32> = vec2<f32>(
        base_u + margin_uv.x * cell_width,
        base_v + margin_uv.y * cell_height,
    );

    // -->> (Pixel perfect): This uses a neares neighbor that attempts to mitigate moiré effect by antialiasing sub-pixel movements.
    // Applying pixel-perfect sampling on the gamma corrected base color
    let uv = sprite_uv; // Using the corrected UV for sprite sheets
    let texel_per_px = abs(dpdx(mesh.uv.x) * material.sprite_width); // 0.1 at 10x zoom. Amount of texels that fit in one screen pixel.
    
    // We need to account that the pixels are centered 0.5 texels to a side, so we need to apply a correction
    let d_factor = 0.5;
    let d_corr = vec2<f32>(d_factor * sign(dpdx(mesh.uv.x)), d_factor * sign(dpdy(mesh.uv.y)));
    let src_pos = uv * tex_size - d_corr;
    let uv_frac = fract(src_pos);
    let uv_floor = (floor(src_pos) + d_corr) / tex_size;
    let softness = 3.0; // 2.0 -> leave 1px of gradient between pixels ; 4.0 -> 2px of gradient
    let uv_frac2 = clamp( (uv_frac - 0.5) / texel_per_px / softness + 0.5, vec2<f32>(0.0,0.0) , vec2<f32>(1.0,1.0));

    // Reading directly the texture mixes the color incorrectly because of differences in alpha
    let uv_comp = uv_floor + (uv_frac2) / tex_size;
    let color1: vec4<f32> = textureSample(base_color_texture, base_color_sampler, uv_comp);
    let color_s: vec4<f32> = textureSample(base_color_texture, base_color_sampler, uv);

    // Sample the four nearest texels for bilinear blending
    // This is for attempting a better alpha color mixing - when one part is transparent
    // and the other is not, it might darken the borders - this code should combat this.
    let texel_tl = textureSample(base_color_texture, base_color_sampler, uv_floor);
    let texel_tr = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(1.0 / tex_width, 0.0));
    let texel_bl = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(0.0, 1.0 / tex_height));
    let texel_br = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(1.0 / tex_width, 1.0 / tex_height));

    let texel_sum = texel_tl * texel_tl[3] + texel_tr  * texel_tr[3] + texel_bl  * texel_bl[3] + texel_br * texel_br[3];
    let total_a = texel_tl[3] + texel_tr[3] + texel_bl[3] + texel_br[3];
    let max_a1 = max(texel_tl[3], texel_tr[3]);
    let max_a2 = max(texel_bl[3], texel_br[3]);
    let max_a = max(max_a1, max_a2);
    var texel_avg = clamp(texel_sum / (total_a + 0.1), zero4, one4);
    let relight_factor = 1.6;
    texel_avg[0] *= relight_factor;
    texel_avg[1] *= relight_factor;
    texel_avg[2] *= relight_factor;
    texel_avg[3] = color_s[3];
    let avg_k = (1.0 - color1[3]) * texel_avg[3];

    var color: vec4<f32> = (color1 * (1.0 - avg_k) + texel_avg * avg_k);
    // <<--


    // Gamma correction based on location within the sprite for gradient effect
    let gamma_tl = material.gtl;
    let gamma_tr = material.gtr;
    let gamma_bl = material.gbl;
    let gamma_br = material.gbr;

    // Estimate coordinates of an isometric floor to mix the gamma color
    var z: f32 = 0.5;
    var dz: f32 = 2.0 * 35.0/128.0;
    var dp: vec2<f32> = vec2(35.0/128.0/z, 18.0/128.0*2.0/z);
    var dpx: vec2<f32> = vec2(-35.0/128.0/z, 18.0/128.0*2.0/z);

    // center of UV -> Anchors::calc(63, 95, 128, 128),
    var cnt: vec2<f32> = vec2(63.0/128.0, 95.0/128.0);
    var pttl: vec2<f32> = cnt - dp;
    var pttr: vec2<f32> = cnt - dpx;
    var ptbr: vec2<f32> = cnt + dp;
    var ptbl: vec2<f32> = cnt + dpx;

    var min_dst = 0.4 * 35.0/128.0;
    var uv1_y: f32 = (mesh.uv[1] - cnt[1]) * 2.0 + cnt[1];
    var uv_w: vec2<f32> = vec2(mesh.uv[0], uv1_y);

    var wtl: f32 = 2.0 / (max(min_dst, distance(uv_w, pttl)-dz));
    var wtr: f32 = 2.0 / (max(min_dst, distance(uv_w, pttr)-dz));
    var wbl: f32 = 2.0 / (max(min_dst, distance(uv_w, ptbl)-dz));
    var wbr: f32 = 2.0 / (max(min_dst, distance(uv_w, ptbr)-dz));
    var wct: f32 = 1.0 / (max(min_dst, distance(uv_w, cnt)));
    
    var wtt: f32 = (wct+wtl+wtr+wbl+wbr);

    var wpf: f32 = 3.0;

    wtl = pow(wtl / wtt, wpf);
    wtr = pow(wtr / wtt, wpf);
    wbl = pow(wbl / wtt, wpf);
    wbr = pow(wbr / wtt, wpf);
    wct = pow(wct / wtt, wpf);

    var gc: f32 = material.gamma;
    var gtl: f32 = material.gtl;
    var gtr: f32 = material.gtr;
    var gbl: f32 = material.gbl;
    var gbr: f32 = material.gbr;
    
    // Interpolate gamma values based on UV coordinates. Assume uv coordinates are normalized [0,1] within each sprite cell.
    var gamma: f32 = (gc * wct + gtl * wtl + gtr * wtr + gbl * wbl + gbr * wbr) / (wct+wtl+wtr+wbl+wbr);
    var wcf: f32 = 1.2; // <- softening effect. Higher increases the edge variability
    gamma = (gamma + gc / wcf) / (1.0 + wcf);

    // Black point:
    let black: f32 = 0.001 * gamma * gamma;
    let b4: vec4<f32> = vec4(black, black, black, 0.0);

    // Apply gamma correction
    let gamma4a: vec4<f32> = vec4<f32>(gamma, gamma, gamma, 1.0);
    let gamma4b: vec4<f32> = vec4<f32>(1.0 / gamma, 1.0 / gamma, 1.0 / gamma, 1.0);
    let gamma4c: vec4<f32> = vec4<f32>(1.0 + gamma, 1.0 + gamma, 1.0 + gamma, 1.0);
    let corrected_color_rgb = (pow(color + b4, gamma4b) * gamma4a + gamma4a * color) / (gamma4c);

    // Apply material color tint to the gamma-corrected color
    let final_color = corrected_color_rgb * material.color;

    // Apply material ambient color
    var ambient = material.ambient_color;
    ambient[3] = 0.0;

    let ambiented_color = clamp(final_color + ambient, zero4, one4);

    return ambiented_color;
}