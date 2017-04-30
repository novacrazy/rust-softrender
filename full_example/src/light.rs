use nalgebra::Point3;

use ::color::Color;

pub struct Light {
    pub color: Color,
    pub position: Point3<f32>,
    pub intensity: f32,
}

impl Light {
    pub fn new_white(position: Point3<f32>, intensity: f32) -> Light {
        Light {
            color: Color { r: 1.0, g: 1.0, b: 1.0, a: 1.0 },
            position,
            intensity
        }
    }

    pub fn new(position: Point3<f32>, intensity: f32, color: Color) -> Light {
        Light { position, intensity, color }
    }
}
