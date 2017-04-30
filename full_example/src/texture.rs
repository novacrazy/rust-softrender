use image::RgbaImage;

use ::color::{Color, decode_gamma, SRGB_GAMMA};

pub struct Texture(RgbaImage);

pub struct Material {
    ambient_texture: Option<Texture>,
    diffuse_texture: Option<Texture>,
    specular_texture: Option<Texture>,
    normal_texture: Option<Texture>,
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

impl Texture {
    pub fn new(image: RgbaImage) -> Texture {
        Texture(image)
    }

    pub fn sample(&self, u: f32, v: f32, method: SamplingMethod, edge: EdgeBehavior) -> Color {
        let (u, v) = edge.edge(u, v);

        let c = match method {
            SamplingMethod::Nearest => {
                let x = (u * (self.0.width() - 1) as f32).round() as u32;
                let y = (v * (self.0.height() - 1) as f32).round() as u32;

                self.0.get_pixel(x, y)
            }
            _ => unimplemented!()
        };

        decode_gamma(Color {
            r: (c.data[0] as f32) / 255.0,
            g: (c.data[1] as f32) / 255.0,
            b: (c.data[2] as f32) / 255.0,
            a: (c.data[3] as f32) / 255.0,
        }, SRGB_GAMMA)
    }
}
