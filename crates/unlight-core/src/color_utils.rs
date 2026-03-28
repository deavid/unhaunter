use bevy::prelude::*;

pub fn compute_color_exposure(
    rel_exposure: f32,
    dither: f32,
    gamma: f32,
    src_color: Color,
) -> Color {
    let exp = rel_exposure.powf(gamma.recip()) + dither;
    let src_srgba = src_color.to_srgba();
    let dst_color: Color = if exp < 1.0 {
        Color::Srgba(Srgba {
            red: src_srgba.red * exp,
            green: src_srgba.green * exp,
            blue: src_srgba.blue * exp,
            alpha: src_srgba.alpha,
        })
    } else {
        let rexp = exp.recip();
        Color::Srgba(Srgba {
            red: 1.0 - ((1.0 - src_srgba.red) * rexp),
            green: 1.0 - ((1.0 - src_srgba.green) * rexp),
            blue: 1.0 - ((1.0 - src_srgba.blue) * rexp),
            alpha: src_srgba.alpha,
        })
    };
    dst_color
}

pub fn lerp_color(start: Color, end: Color, t: f32) -> Color {
    let k = start.to_srgba().to_vec4();
    let l = end.to_srgba().to_vec4();
    let (sr, sg, sb, sa) = (k[0], k[1], k[2], k[3]);
    let (er, eg, eb, ea) = (l[0], l[1], l[2], l[3]);
    let r = sr + (er - sr) * t;
    let g = sg + (eg - sg) * t;
    let b = sb + (eb - sb) * t;
    let a = sa + (ea - sa) * t;
    Color::srgba(r, g, b, a)
}
