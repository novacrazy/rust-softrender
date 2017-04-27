extern crate nalgebra;
#[macro_use]
extern crate softrender;

use std::sync::Arc;

use nalgebra::{Point3, Vector4, Vector3};

use softrender::pixel::RGBAf32Pixel;
use softrender::mesh::{Mesh, Vertex};
use softrender::render::{FrameBuffer, Pipeline, ClipVertex, ScreenVertex};
use softrender::image_compat::ImageFrameBuffer;

fn get_cube() -> Arc<Mesh<()>> {
    let vertices = vec![Point3::new(0.583, 0.771, 0.014),
                        Point3::new(0.609, 0.115, 0.436),
                        Point3::new(0.327, 0.483, 0.844),
                        Point3::new(0.822, 0.569, 0.201),
                        Point3::new(0.435, 0.602, 0.223),
                        Point3::new(0.310, 0.747, 0.185),
                        Point3::new(0.597, 0.770, 0.761),
                        Point3::new(0.559, 0.436, 0.730),
                        Point3::new(0.359, 0.583, 0.152),
                        Point3::new(0.483, 0.596, 0.789),
                        Point3::new(0.559, 0.861, 0.639),
                        Point3::new(0.195, 0.548, 0.859),
                        Point3::new(0.014, 0.184, 0.576),
                        Point3::new(0.771, 0.328, 0.970),
                        Point3::new(0.406, 0.615, 0.116),
                        Point3::new(0.676, 0.977, 0.133),
                        Point3::new(0.971, 0.572, 0.833),
                        Point3::new(0.140, 0.616, 0.489),
                        Point3::new(0.997, 0.513, 0.064),
                        Point3::new(0.945, 0.719, 0.592),
                        Point3::new(0.543, 0.021, 0.978),
                        Point3::new(0.279, 0.317, 0.505),
                        Point3::new(0.167, 0.620, 0.077),
                        Point3::new(0.347, 0.857, 0.137),
                        Point3::new(0.055, 0.953, 0.042),
                        Point3::new(0.714, 0.505, 0.345),
                        Point3::new(0.783, 0.290, 0.734),
                        Point3::new(0.722, 0.645, 0.174),
                        Point3::new(0.302, 0.455, 0.848),
                        Point3::new(0.225, 0.587, 0.040),
                        Point3::new(0.517, 0.713, 0.338),
                        Point3::new(0.053, 0.959, 0.120),
                        Point3::new(0.393, 0.621, 0.362),
                        Point3::new(0.673, 0.211, 0.457),
                        Point3::new(0.820, 0.883, 0.371),
                        Point3::new(0.982, 0.099, 0.879)];

    let indices = (0..vertices.len()).map(|i| i as u32).collect();

    Arc::new(Mesh {
        vertices: vertices.into_iter().map(|vertex| Vertex { position: vertex, vertex_data: () }).collect(),
        indices: indices
    })
}

fn main() {
    let framebuffer = FrameBuffer::<RGBAf32Pixel>::new_with(1000, 1000, RGBAf32Pixel { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

    let cube = get_cube();

    let view = nalgebra::Isometry3::look_at_rh(
        &Point3::new(1.0, 1.0, 1.0),
        &Point3::origin(),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    let projection = nalgebra::Perspective3::new(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                 75.0f32.to_radians(), 0.001, 1000.0).to_homogeneous();

    let mut pipeline = Pipeline::new(framebuffer, ());

    {
        let vertex_shader = pipeline.render_mesh(cube.clone());

        declare_uniforms! {
            Uniforms {
                position: Vector4<f32>,
            }
        }

        let mut fragment_shader = vertex_shader.run(|vertex, _| {
            let world_position = vertex.position.to_homogeneous();

            let position = projection * view * world_position;

            ClipVertex::new(position, Uniforms {
                position: world_position,
            })
        });

        fragment_shader.set_blend_function(|a, b| {
            let sa = a.a;
            let da = 1.0 - sa;

            RGBAf32Pixel {
                r: a.r * sa + b.r * da,
                g: a.g * sa + b.g * da,
                b: a.b * sa + b.b * da,
                a: a.a * sa + b.a * da,
            }
        });

        fragment_shader.run(|screen_vertex, _| {
            let Uniforms { position } = screen_vertex.uniforms;

            // Determine the color of the pixel here
            RGBAf32Pixel { r: position.x, g: position.y, b: position.z, a: 1.0 }
        });
    }

    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("examples/basic.png").unwrap();
}