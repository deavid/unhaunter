#import bevy_ui::ui_vertex_output::UiVertexOutput

struct CustomMaterial {
    color: vec4<f32>,
}

@group(1) @binding(0)
var<uniform> material: CustomMaterial;

@fragment
fn fragment(in: UiVertexOutput) -> @location(0) vec4<f32> {

  var dx = dpdx(in.uv.x);
  var dy = dpdy(in.uv.y);

  var cornerSize = vec2<f32>(dx * 15.0, dy * 45.0);

  // Calculateuv coordinate within quad
  var uv = in.uv;
  var pos = in.position;

  // Check if in top right corner
  var isCorner = uv.x > 1.0 - cornerSize.x && uv.y < cornerSize.y;

  var col = material.color;

  var col_mix = col;
  col_mix[3] = 0.0;

  if(isCorner) {
    // Slant corner by lerping color
    var mixAmount = (uv.y / cornerSize.y) + (1.0 - uv.x) / cornerSize.x;
    mixAmount = (mixAmount - 1.0) * 5.0 + 1.0;

    return mix(col_mix, col, clamp(mixAmount, 0.0, 1.0)); 
  } else {
    return col;

  }

}
