use image::{RgbaImage, Rgba};

use ::color::{Color, decode_gamma, SRGB_GAMMA};

pub struct Texture(RgbaImage);

pub struct Material {
   pub ambient_texture: Option<Texture>,
   pub diffuse_texture: Option<Texture>,
   pub specular_texture: Option<Texture>,
   pub normal_texture: Option<Texture>,
}

pub enum SamplingMethod {
    Nearest,
    Bilinear
}

pub enum EdgeBehavior {
    Clamp,
    Wrap
}

impl EdgeBehavior {
    pub fn edge(self, u: f32, v: f32) -> (f32, f32) {
        match self {
            EdgeBehavior::Clamp => (u.min(1.0).max(0.0), v.min(1.0).max(0.0)),
            EdgeBehavior::Wrap => (u.fract(), v.fract())
        }
    }
}

fn from_image_texel(texel: &Rgba<u8>) -> Color {
    Color {
        r: (texel.data[0] as f32) / 255.0,
        g: (texel.data[1] as f32) / 255.0,
        b: (texel.data[2] as f32) / 255.0,
        a: (texel.data[3] as f32) / 255.0,
    }
}

impl Texture {
    pub fn new(image: RgbaImage) -> Texture {
        Texture(image)
    }

    pub fn sample(&self, u: f32, v: f32, method: SamplingMethod, edge: EdgeBehavior) -> Color {
        let (u, v) = edge.edge(u, v);

        let texel = match method {
            SamplingMethod::Nearest => {
                let x = (u * (self.0.width() - 1) as f32).round() as u32;
                let y = (v * (self.0.height() - 1) as f32).round() as u32;

                from_image_texel(self.0.get_pixel(x, y))
            }
            SamplingMethod::Bilinear => {
                let u = (u * (self.0.width() - 1) as f32) + 0.5;
                let v = (v * (self.0.height() - 1) as f32) + 0.5;
                let x = u.floor() as u32;
                let y = v.floor() as u32;

                let u_ratio = u - x as f32;
                let v_ratio = v - y as f32;

                let u_opposite = 1.0 - u_ratio;
                let v_opposite = 1.0 - v_ratio;

                let xy = from_image_texel(self.0.get_pixel(x, y));
                let x1y = from_image_texel(self.0.get_pixel(x + 1, y));
                let xy1 = from_image_texel(self.0.get_pixel(x, y + 1));
                let x1y1 = from_image_texel(self.0.get_pixel(x + 1, y + 1));

                Color {
                    r: (xy.r * u_opposite + x1y.r * u_ratio) * v_opposite + (xy1.r * u_opposite + x1y1.r * u_ratio) * v_ratio,
                    g: (xy.g * u_opposite + x1y.g * u_ratio) * v_opposite + (xy1.g * u_opposite + x1y1.g * u_ratio) * v_ratio,
                    b: (xy.b * u_opposite + x1y.b * u_ratio) * v_opposite + (xy1.b * u_opposite + x1y1.b * u_ratio) * v_ratio,
                    a: (xy.a * u_opposite + x1y.a * u_ratio) * v_opposite + (xy1.a * u_opposite + x1y1.a * u_ratio) * v_ratio,
                }
            }
        };

        decode_gamma(texel, SRGB_GAMMA)
    }
}
