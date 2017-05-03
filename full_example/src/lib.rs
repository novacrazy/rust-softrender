extern crate nalgebra;
extern crate tobj;
extern crate image;
extern crate rayon;

#[macro_use]
extern crate softrender;

pub mod color;
pub mod mesh;
pub mod texture;
pub mod light;
pub mod uniforms;
pub mod shaders;

use nalgebra::{Vector3, Point3, Perspective3, Isometry3, Matrix4};

use self::color::Color;
use self::light::Light;
use self::uniforms::GlobalUniforms;

pub fn model_matrix(rotation: f32) -> (Matrix4<f32>, Matrix4<f32>) {
    let transform = Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                   Vector3::new(0.0, rotation, 0.0));

    (transform.to_homogeneous(),
     transform.inverse().to_homogeneous().transpose())
}

pub fn generate_global_uniforms(aspect_ratio: f32,
                                camera_rotation: f32,
                                camera_distance: f32,
                                object_rotation: f32,
                                fov: f32) -> GlobalUniforms {
    // Create the model transform
    let (model, mit) = model_matrix(object_rotation);

    // Define camera position
    let camera_position = Point3::new(camera_rotation.cos() * camera_distance,
                                      camera_distance,
                                      camera_rotation.sin() * camera_distance);

    // Create view matrix from camera position and look-at target
    let view = Isometry3::look_at_rh(
        &camera_position,
        &Point3::new(0.0, 0.25, 0.0),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    // Create a perspective projection matrix
    let projection = Perspective3::new(aspect_ratio, fov, 0.1, 1000.0).to_homogeneous();

    // Convenience variable to tweak distance of lights from object
    let light_scale = 1.0;

    GlobalUniforms {
        camera: camera_position.to_homogeneous(),
        model: model,
        model_inverse_transpose: mit,
        view: view,
        projection: projection,
        // Create a few lights with varying colors, intensities and locations
        lights: vec![
            Light::new_white(Point3::new(light_scale * -1.0, light_scale * 1.0, light_scale * -1.0), 9.0),
            Light::new(Point3::new(light_scale * 1.0, light_scale * 1.0, light_scale * 1.0), 9.0, Color { r: 0.6, g: 0.6, b: 1.0, a: 1.0 }),
            Light::new(Point3::new(light_scale * 0.0, light_scale * 3.0, light_scale * -1.0), 25.0, Color { r: 1.0, g: 0.3, b: 0.3, a: 1.0 }),
            Light::new(Point3::new(light_scale * -2.0, light_scale * -1.0, light_scale * 1.0), 25.0, Color { r: 0.7, g: 1.0, b: 0.7, a: 1.0 }),
        ]
    }
}
