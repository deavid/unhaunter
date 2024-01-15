#import bevy_pbr::forward_io VertexOutput

struct CustomMaterial {
    color: vec4<f32>,
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

@group(1) @binding(0)
var<uniform> material: CustomMaterial;
@group(1) @binding(1)
var base_color_texture: texture_2d<f32>;
@group(1) @binding(2)
var base_color_sampler: sampler;

@fragment
fn fragment(
    mesh: VertexOutput,
) -> @location(0) vec4<f32> {
    // separation between tiles is 35x18, sprite is 128x128
    var z: f32 = 0.5;
    var dz: f32 = 2.0 * 35.0/128.0;
    var dp: vec2<f32> = vec2(35.0/128.0/z, 18.0/128.0*2.0/z);
    var dpx: vec2<f32> = vec2(-35.0/128.0/z, 18.0/128.0*2.0/z);
    // center of UV -> Anchors::calc(63, 95, 128, 128),
    var cnt: vec2<f32> = vec2(63.0/128.0, 95.0/128.0);
    // UV seems to go 0..1 and it is X,Y
    // TODO: Ideally the Y distance should count as doulbe to account for isometric.

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
    
    var g: f32 = (gc * wct + gtl * wtl + gtr * wtr + gbl * wbl + gbr * wbr) / (wct+wtl+wtr+wbl+wbr);

    var wcf: f32 = 1.2; // <- softening effect. Higher increases the edge variability
    g = (g + gc / wcf)/(1.0+wcf);

    var e: f32 = 1.0/g;
    var e4: vec4<f32> = vec4(e,e,e,1.0);
    var g4: vec4<f32> = vec4(g,g,g,1.0);
    var black: f32 = 0.0001 * g * g;
    var b4: vec4<f32> = vec4(black, black, black, 0.0);


    /// remap mesh.uv for spritesheets, cutting the spritesheet to get the sprite_idx
    var sheet_idx = material.sheet_idx;  
    var sheet_rows = material.sheet_rows;  
    var sheet_cols = material.sheet_cols;

    var row = f32(sheet_idx / sheet_cols);
    var col = f32(sheet_idx % sheet_cols);

    var cell_width = 1.0 / f32(sheet_cols); 
    var cell_height = 1.0 / f32(sheet_rows);

    var cell_min_x = col * cell_width;
    // var cell_max_x = cell_min_x + cell_width;

    var cell_min_y = row * cell_height;  
    // var cell_max_y = cell_min_y + cell_height;
    var d = 0.5;
    var muvx = (round(mesh.uv.x * material.sprite_width) + d) / material.sprite_width;
    var muvy = (round(mesh.uv.y * material.sprite_height) + d) / material.sprite_height;
    var duv = vec2<f32>(mesh.uv.x+ d / material.sprite_width - muvx, mesh.uv.y+ d / material.sprite_height - muvy);
    var ddx = pow(abs(dpdx(mesh.uv.x)) * material.sprite_width, 3.0);
    var duv2 = max(length(duv) - 0.002/ material.sprite_width / ddx, 0.0) * normalize(duv);
    muvx += duv2.x / 1.0;
    muvy += duv2.y / 1.0;
    
    
    var uv_x = muvx * cell_width + cell_min_x;
    var uv_y = muvy * cell_height + cell_min_y;

    var sprite_uv = vec2(uv_x, uv_y);
    //

    var c: vec4<f32> = textureSample(base_color_texture, base_color_sampler, sprite_uv);

    return (pow(c + b4, e4) * g + g4 * c) / (1.0 + g) * material.color;
}