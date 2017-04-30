use softrender::pixel::RGBAf32Pixel;

pub type Color = RGBAf32Pixel;

pub fn blend(a: Color, b: Color) -> Color {
    let sa = a.a;
    let da = 1.0 - sa;

    Color {
        r: a.r * sa + b.r * da,
        g: a.g * sa + b.g * da,
        b: a.b * sa + b.b * da,
        a: a.a * sa + b.a * da,
    }
}

#[inline(always)]
fn aces_filmic_tonemap_component(x: f32) -> f32 {
    const A: f32 = 2.51;
    const B: f32 = 0.03;
    const C: f32 = 2.43;
    const D: f32 = 0.59;
    const E: f32 = 0.14;

    (x * (A * x + B)) / (x * (C * x + D) + E)
}

pub fn aces_filmic_tonemap(color: Color) -> Color {
    Color {
        r: aces_filmic_tonemap_component(color.r),
        g: aces_filmic_tonemap_component(color.g),
        b: aces_filmic_tonemap_component(color.b),
        a: color.a
    }
}

pub const SRGB_GAMMA: f32 = 2.2;

pub fn encode_gamma(color: Color, gamma: f32) -> Color {
    Color {
        r: color.r.powf(1.0 / gamma),
        g: color.g.powf(1.0 / gamma),
        b: color.b.powf(1.0 / gamma),
        a: color.a
    }
}

pub fn decode_gamma(color: Color, gamma: f32) -> Color {
    Color {
        r: color.r.powf(gamma),
        g: color.g.powf(gamma),
        b: color.b.powf(gamma),
        a: color.a
    }
}