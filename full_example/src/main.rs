extern crate nalgebra;
extern crate wavefront_obj;
extern crate image;
extern crate rayon;
#[macro_use]
extern crate softrender;

use nalgebra::{Vector3, Point3, Perspective3, Isometry3};

use softrender::render::{FrameBuffer};

pub mod color;
pub mod mesh;
pub mod texture;
pub mod uniforms;
pub mod shaders;

use self::color::Color;

fn main() {
    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::<Color>::new_with(2000, 2000, Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    // Create the model transform
    let model = nalgebra::Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                         Vector3::new(0.0, 0.0, 0.0));

    // Define camera position
    let camera_position = Point3::new(1000.0, 800.0, 50.0);

    // Create view matrix from camera position and look-at target
    let view = nalgebra::Isometry3::look_at_rh(
        &camera_position,
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    // Create a perspective projection matrix
    let projection = nalgebra::Perspective3::new(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                 75.0f32.to_radians(), 0.1, 1000.0).to_homogeneous();

}
