extern crate nalgebra;
extern crate tobj;
extern crate image;
extern crate rayon;
#[macro_use]
extern crate softrender;

use nalgebra::{Vector3, Point3, Perspective3, Isometry3};

use softrender::render::{FrameBuffer, Pipeline};
use softrender::image_compat::ImageFrameBuffer;

pub mod color;
pub mod mesh;
pub mod texture;
pub mod light;
pub mod uniforms;
pub mod shaders;

use self::color::Color;
use self::light::Light;
use self::uniforms::GlobalUniforms;

fn generate_global_uniforms(aspect_ratio: f32) -> GlobalUniforms {
    // Create the model transform
    let model = nalgebra::Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                         Vector3::new(0.0, 0.0, 0.0));

    // Define camera position
    let camera_position = Point3::new(1.0, 2.2, 2.0);

    // Create view matrix from camera position and look-at target
    let view = nalgebra::Isometry3::look_at_rh(
        &camera_position,
        &Point3::new(0.0, 0.25, 0.0),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    // Create a perspective projection matrix
    let projection = nalgebra::Perspective3::new(aspect_ratio, 65.0f32.to_radians(), 0.1, 1000.0).to_homogeneous();

    // Convenience variable to tweak distance of lights from object
    let light_scale = 1.0;

    GlobalUniforms {
        camera: camera_position.to_homogeneous(),
        model: model.to_homogeneous(),
        model_inverse_transpose: model.inverse().to_homogeneous().transpose(),
        view: view,
        projection: projection,
        lights: vec![
            Light::new_white(Point3::new(light_scale * -1.0, light_scale * 1.0, light_scale * -1.0), 6.0),
            Light::new(Point3::new(light_scale * 1.0, light_scale * 1.0, light_scale * 1.0), 6.0, Color {
                r: 0.6, g: 0.6, b: 1.0, a: 1.0,
            }),
            Light::new(Point3::new(light_scale * 0.0, light_scale * 3.0, light_scale * -1.0), 6.0, Color {
                r: 1.0, g: 0.3, b: 0.3, a: 1.0,
            }),
            Light::new(Point3::new(light_scale * -2.0, light_scale * -1.0, light_scale * 1.0), 6.0, Color {
                r: 0.7, g: 1.0, b: 0.7, a: 1.0,
            }),
        ]
    }
}

fn main() {
    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::<Color>::new_with(1920, 1080, Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    let global_uniforms = generate_global_uniforms(framebuffer.width() as f32 / framebuffer.height() as f32);

    let mut pipeline = Pipeline::new(framebuffer, global_uniforms);

    let meshes = self::mesh::load_model("assets/models/suzanne_highres.obj");

    for mesh in &meshes {
        let vertex_shader = pipeline.render_mesh(mesh.mesh.clone());

        let fragment_shader = vertex_shader.run(self::shaders::vertex_shader);

        fragment_shader.triangles(self::shaders::fragment_shader);
    }

    println!("Saving to image");

    // copy the framebuffer into an image then save it to a file
    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("example.png").unwrap();
}
