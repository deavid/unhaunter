#import bevy_sprite::mesh2d_vertex_output::VertexOutput

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
    padding: f32,
    margin: f32,
    y_anchor: f32,
    upscale_factor: f32,
};

@group(#{MATERIAL_BIND_GROUP}) @binding(0)
var<uniform> material: CustomMaterial;
@group(#{MATERIAL_BIND_GROUP}) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(#{MATERIAL_BIND_GROUP}) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // Full texture size computation:
    let tex_width = material.margin * 2.0 + material.sprite_width * f32(material.sheet_cols) + material.padding * f32(material.sheet_cols - 1u);
    let tex_height = material.margin * 2.0 + material.sprite_height * f32(material.sheet_rows) + material.padding * f32(material.sheet_rows - 1u);
    let tex_size = vec2<f32>(tex_width, tex_height);

    // Calculate sprite UVs considering the sprite sheet layout
    let row: u32 = material.sheet_idx / material.sheet_cols;
    let col: u32 = material.sheet_idx % material.sheet_cols;

    // Compute the start position of the sprite in pixel coordinates
    let pixel_u: f32 = material.margin + f32(col) * (material.sprite_width + material.padding);
    let pixel_v: f32 = material.margin + f32(row) * (material.sprite_height + material.padding);

    // Compute the size of each cell in the atlas (in UV space)
    let cell_width: f32 = material.sprite_width / tex_width;
    let cell_height: f32 = material.sprite_height / tex_height;

    // Compute the base UV coordinates for the sprite within the atlas
    let base_u: f32 = pixel_u / tex_width;
    let base_v: f32 = pixel_v / tex_height;

    let zero4 = vec4(0.0, 0.0, 0.0, 0.0);
    let one4 = vec4(1.0, 1.0, 1.0, 1.0);

    // Adding a margin to the sprite coordinates to prevent reading from neighboring sprite
    let margin = 0.5;
    let mx = margin / material.sprite_width;
    let my = margin / material.sprite_height;

    // Margin protects the sprites from reading the neighboring sprite
    let margin_uv = clamp(mesh.uv, vec2<f32>(mx, my), vec2<f32>(1.0 - mx, 1.0 - my));

    // Correcting UV coordinates for the sprite
    var sprite_uv: vec2<f32> = vec2<f32>(
        base_u + margin_uv.x * cell_width,
        base_v + margin_uv.y * cell_height,
    );

    var color: vec4<f32>;

    if (material.upscale_factor >= 2.0) {
        // For upscaled textures, we use simple bilinear filtering
        color = textureSample(base_color_texture, base_color_sampler, sprite_uv);
    } else {
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

        // Manual alpha-weighted bilinear sampling to prevent "black bleed" from 0-alpha texels.
        // We sample the four nearest texels at their centers.
        let texel_tl = textureSample(base_color_texture, base_color_sampler, uv_floor);
        let texel_tr = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(1.0 / tex_width, 0.0));
        let texel_bl = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(0.0, 1.0 / tex_height));
        let texel_br = textureSample(base_color_texture, base_color_sampler, uv_floor + vec2<f32>(1.0 / tex_width, 1.0 / tex_height));

        // Extract alphas
        let a_tl = texel_tl[3];
        let a_tr = texel_tr[3];
        let a_bl = texel_bl[3];
        let a_br = texel_br[3];

        // Bilinear weights from the sharpened fractional part
        let w_x = uv_frac2.x;
        let w_y = uv_frac2.y;
        let w_tl = (1.0 - w_x) * (1.0 - w_y);
        let w_tr = w_x * (1.0 - w_y);
        let w_bl = (1.0 - w_x) * w_y;
        let w_br = w_x * w_y;

        // Compute the accurately interpolated alpha
        let final_alpha = a_tl * w_tl + a_tr * w_tr + a_bl * w_bl + a_br * w_br;

        // Compute alpha-weighted color sums
        let weighted_rgb = (texel_tl.rgb * a_tl * w_tl + texel_tr.rgb * a_tr * w_tr + texel_bl.rgb * a_bl * w_bl + texel_br.rgb * a_br * w_br);
        let total_a = a_tl + a_tr + a_bl + a_br;
        let neighbor_avg_rgb = (texel_tl.rgb * a_tl + texel_tr.rgb * a_tr + texel_bl.rgb * a_bl + texel_br.rgb * a_br) / max(total_a, 0.001);

        // Calculate the base color by "un-premultiplying" the interpolated result.
        // This ensures that even at 10% alpha, the color intensity is preserved and not pulled towards black.
        var final_rgb = weighted_rgb / max(final_alpha, 0.001);

        // Apply a "color bleed" (blur) effect that becomes stronger as the pixel becomes more transparent.
        // This spreads the neighborhood's average color into the antialiased edges.
        let blur_k = clamp(1.0 - final_alpha, 0.0, 1.0);
        final_rgb = mix(final_rgb, neighbor_avg_rgb, blur_k * blur_k);

        color = vec4<f32>(final_rgb, final_alpha);
        // <<--
    }


    // Gamma correction based on location within the sprite for gradient effect
    let gamma_tl = material.gtl;
    let gamma_tr = material.gtr;
    let gamma_bl = material.gbl;
    let gamma_br = material.gbr;

    // Estimate coordinates of an isometric floor to mix the gamma color
    var z: f32 = 0.5;

    // Fixed floor diamond ratios (derived from a 128x128 pixel reference)
    // 35px half-width, 18px half-height
    let ref_size = 128.0;
    let floor_half_w = 35.0 / ref_size;
    let floor_half_h = 18.0 / ref_size;

    var dz: f32 = 2.0 * floor_half_w;
    var dp: vec2<f32> = vec2(floor_half_w / z, floor_half_h * 2.0 / z);
    var dpx: vec2<f32> = vec2(-floor_half_w / z, floor_half_h * 2.0 / z);

    // center of UV derived from material anchor.
    // Anchors::calc(63, 95, 128, 128) results in ~ (0.5, 0.742)
    // 0.5 - (-0.25) = 0.75
    var cnt: vec2<f32> = vec2(0.5, 0.5 - material.y_anchor);
    // --- Improved Linear Gradient Blend ---
    // floor_half_w/h already defined above

    let u_rel = mesh.uv.x - cnt.x;
    let v_rel = mesh.uv.y - cnt.y;

    // Logical axes mapped from UV space
    // Lx and Ly will be in range [-0.5, 0.5] inside the floor diamond
    let Lx = 0.5 * (u_rel / floor_half_w - v_rel / floor_half_h);
    let Ly = 0.5 * (u_rel / floor_half_w + v_rel / floor_half_h);

    // Locked vertices in logical space:
    // Right: (0.5, 0.5)
    // Left: (-0.5, -0.5)
    // Top: (0.5, -0.5)
    // Bottom: (-0.5, 0.5)
    let d2_c = max(0.0001, Lx*Lx + Ly*Ly); // Center
    let d2_tr = max(0.0001, (Lx-0.5)*(Lx-0.5) + (Ly-0.5)*(Ly-0.5)); // Right
    let d2_tl = max(0.0001, (Lx-0.5)*(Lx-0.5) + (Ly+0.5)*(Ly+0.5)); // Top
    let d2_br = max(0.0001, (Lx+0.5)*(Lx+0.5) + (Ly-0.5)*(Ly-0.5)); // Bottom
    let d2_bl = max(0.0001, (Lx+0.5)*(Lx+0.5) + (Ly+0.5)*(Ly+0.5)); // Left

    // --- Improved Smooth Bilinear + Center Bump ---
    // Calculate weights for standard bilinear interpolation on the 4 corners.
    // Lxc and Lyc are in range [-0.5, 0.5] inside the floor diamond.
    let Lxc = clamp(Lx, -0.5, 0.5);
    let Lyc = clamp(Ly, -0.5, 0.5);

    // Standard bilinear weights for a square grid:
    // Top (gtl): Lx=0.5, Ly=-0.5
    // Right (gtr): Lx=0.5, Ly=0.5
    // Left (gbl): Lx=-0.5, Ly=-0.5
    // Bottom (gbr): Lx=-0.5, Ly=0.5
    let w_tl = (0.5 + Lxc) * (0.5 - Lyc);
    let w_tr = (0.5 + Lxc) * (0.5 + Lyc);
    let w_bl = (0.5 - Lxc) * (0.5 - Lyc);
    let w_br = (0.5 - Lxc) * (0.5 + Lyc);

    let gamma_corners = material.gtl * w_tl + material.gtr * w_tr + material.gbl * w_bl + material.gbr * w_br;

    // Center bump function: 1.0 at (0,0), exactly 0.0 at all four edges.
    // Pow(L, 2.0) makes the center nuance more localized and reduces over-shading.
    let w_center = pow(clamp((1.0 - 2.0 * abs(Lxc)) * (1.0 - 2.0 * abs(Lyc)), 0.0, 1.0), 2.0);

    var gamma: f32 = mix(gamma_corners, material.gamma, w_center);

    // Dithering to hide banding in smooth gradients
    let noise = fract(sin(dot(mesh.uv, vec2<f32>(12.9898, 78.233))) * 43758.5453);
    gamma += (noise - 0.5) / 255.0;

    // --- End of Blend ---

    // Black point:
    let black: f32 = 0.001 * gamma * gamma;
    let b4: vec4<f32> = vec4(black, black, black, 0.0);

    // Apply gamma correction
    let gamma4a: vec4<f32> = vec4<f32>(gamma, gamma, gamma, 1.0);
    let gamma4b: vec4<f32> = vec4<f32>(1.0 / gamma, 1.0 / gamma, 1.0 / gamma, 1.0);
    let gamma4c: vec4<f32> = vec4<f32>(1.0 + gamma, 1.0 + gamma, 1.0 + gamma, 2.0);
    let corrected_color_rgb = (pow(color + b4, gamma4b) * gamma4a + gamma4a * color) / (gamma4c);

    // Apply material color tint to the gamma-corrected color
    let final_color = corrected_color_rgb * material.color;

    // Apply material ambient color
    var ambient = material.ambient_color;
    ambient[3] = 0.0;

    let ambiented_color = clamp(final_color + ambient, zero4, one4);

    return ambiented_color;
}