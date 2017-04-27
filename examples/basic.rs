extern crate nalgebra;
extern crate tobj;
#[macro_use]
extern crate softrender;

use std::sync::Arc;
use std::path::Path;

use nalgebra::{Point3, Vector4, Vector3};

use softrender::pixel::RGBAf32Pixel;
use softrender::mesh::{Mesh, Vertex};
use softrender::render::{FrameBuffer, Pipeline, ClipVertex, ScreenVertex};
use softrender::image_compat::ImageFrameBuffer;

struct VertexData {
    normal: Vector3<f32>,
}

fn get_mesh() -> Arc<Mesh<VertexData>> {
    let (models, _): (Vec<tobj::Model>, _) = tobj::load_obj(Path::new("examples/suzanne.obj")).unwrap();

    let ref mesh: tobj::Mesh = models[0].mesh;

    assert_eq!(mesh.positions.len(), mesh.normals.len());

    Arc::new(Mesh {
        vertices: mesh.positions.chunks(3).zip(mesh.normals.chunks(3)).map(|(position, normal)| {
            Vertex {
                position: Point3::new(position[0], position[1], position[2]),
                vertex_data: VertexData {
                    normal: Vector3::new(normal[0], normal[1], normal[2]),
                }
            }
        }).collect(),
        indices: mesh.indices.clone()
    })
}

fn main() {
    let framebuffer = FrameBuffer::<RGBAf32Pixel>::new_with(1000, 1000, RGBAf32Pixel { r: 1.0, g: 1.0, b: 1.0, a: 1.0 });

    let mesh = get_mesh();

    let view = nalgebra::Isometry3::look_at_rh(
        &Point3::new(1.0, 0.0, 3.0),
        &Point3::origin(),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    let projection = nalgebra::Perspective3::new(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                 75.0f32.to_radians(), 0.001, 1000.0).to_homogeneous();

    let mut pipeline = Pipeline::new(framebuffer, ());

    {
        let vertex_shader = pipeline.render_mesh(mesh.clone());

        declare_uniforms! {
            Uniforms {
                position: Vector4<f32>,
                normal: Vector4<f32>,
            }
        }

        let mut fragment_shader = vertex_shader.run(|vertex, _| {
            let VertexData { normal } = vertex.vertex_data;

            let world_position = vertex.position.to_homogeneous();

            let clip_position = projection * view * world_position;

            ClipVertex::new(clip_position, Uniforms {
                position: world_position,
                normal: normal.to_homogeneous(),
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

        fragment_shader.triangles(|screen_vertex, _| {
            let Uniforms { position, normal } = screen_vertex.uniforms;

            let light = Point3::new(1.0, 1.0, 1.0).to_homogeneous();

            let light_dir = (light - position).normalize();

            let cos_theta = light_dir.dot(&normal);

            // Determine the color of the pixel here
            RGBAf32Pixel { r: 0.0, g: cos_theta, b: 0.0, a: 1.0 }
        });
    }

    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("examples/basic.png").unwrap();
}