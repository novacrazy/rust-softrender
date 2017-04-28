extern crate nalgebra;
extern crate tobj;
#[macro_use]
extern crate softrender;

use std::sync::Arc;
use std::path::Path;

use nalgebra::{Point3, Vector4, Vector3, Matrix4};

use softrender::pixel::RGBAf32Pixel;
use softrender::mesh::{Mesh, Vertex};
use softrender::render::{FaceWinding, FrameBuffer, Pipeline, ClipVertex, ScreenVertex};
use softrender::image_compat::ImageFrameBuffer;

struct VertexData {
    normal: Vector3<f32>,
}

fn load_mesh<P: AsRef<Path>>(p: P) -> Arc<Mesh<VertexData>> {
    let (models, _): (Vec<tobj::Model>, _) = tobj::load_obj(p.as_ref().clone()).unwrap();

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

fn get_meshes() -> Vec<Arc<Mesh<VertexData>>> {
    vec![
        load_mesh("examples/suzanne_highres.obj"),
        load_mesh("examples/plane.obj")
    ]
}

#[allow(non_snake_case)]
fn main() {
    let framebuffer = FrameBuffer::<RGBAf32Pixel>::new_with(2000, 2000, RGBAf32Pixel { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    struct GlobalUniforms {
        camera: Vector4<f32>,
        model: Matrix4<f32>,
        model_inverse_transpose: Matrix4<f32>,
        view: Matrix4<f32>,
        projection: Matrix4<f32>,
    }

    declare_uniforms! {
        Uniforms {
            position: Vector4<f32>,
            normal: Vector4<f32>,
        }
    }

    let model = nalgebra::Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                         Vector3::new(0.0, 0.0, 0.0));

    let camera_position = Point3::new(2.0, 2.0, 2.0);

    let view = nalgebra::Isometry3::look_at_rh(
        &camera_position,
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    let projection = nalgebra::Perspective3::new(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                 75.0f32.to_radians(), 0.001, 1000.0).to_homogeneous();

    let mut pipeline = Pipeline::new(framebuffer, GlobalUniforms {
        camera: camera_position.to_homogeneous(),
        model: model.to_homogeneous(),
        model_inverse_transpose: model.inverse().to_homogeneous().transpose(),
        view: view,
        projection: projection
    });

    for mesh in &get_meshes() {
        let vertex_shader = pipeline.render_mesh(mesh.clone());

        let mut fragment_shader = vertex_shader.run(|vertex, global_uniforms| {
            let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
            let VertexData { normal } = vertex.vertex_data;

            let world_position = vertex.position.to_homogeneous();
            let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();

            let clip_position = projection * view * world_position;

            ClipVertex::new(clip_position, Uniforms {
                position: world_position,
                normal: normal,
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

        let light_scale = 1.0;

        let lights = [
            Point3::new(light_scale * 1.0, light_scale * 1.0, light_scale * 1.0),
            Point3::new(light_scale * -1.0, light_scale * 1.0, light_scale * -1.0),
            Point3::new(light_scale * 0.0, light_scale * 3.0, light_scale * -1.0),
            Point3::new(light_scale * -2.0, light_scale * -1.0, light_scale * 1.0),
        ];

        let diffuse_color = RGBAf32Pixel { r: 0.1, g: 0.1, b: 0.6, a: 1.0 };

        fragment_shader.triangles(|screen_vertex, global_uniforms| {
            let GlobalUniforms { ref camera, .. } = *global_uniforms;
            let frag_coord = screen_vertex.position;
            let Uniforms { position, normal } = screen_vertex.uniforms;

            let view_dir = (camera - position).normalize();
            let NdotV = normal.dot(&view_dir).min(1.0).max(0.0);

            let mut specular = 0.0;
            let mut diffuse = 0.0;

            for light in &lights {
                let light = light.to_homogeneous();

                let light_dir = (light - position).normalize();
                let halfway_vector = (light_dir + view_dir).normalize();

                let NdotL = light_dir.dot(&normal).min(1.0).max(0.0);
                let NdotH = normal.dot(&halfway_vector).min(1.0).max(0.0);
                let VdotH = view_dir.dot(&halfway_vector).min(1.0).max(0.0);

                fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
                    let f0 = ((1.0 - ior) / (1.0 + ior)).powi(2);

                    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
                }

                let f = fresnel_schlick(VdotH, 1.45);

                diffuse += NdotL * (1.0 - f);
                specular += f * NdotH.powf(24.0);
            }

            RGBAf32Pixel {
                r: specular * 10.0 + (diffuse * diffuse_color.r),
                g: specular * 10.0 + (diffuse * diffuse_color.g),
                b: specular * 10.0 + (diffuse * diffuse_color.b),
                a: diffuse_color.a
            }
        });
    }

    println!("Adjusting gamma");

    for pixel in pipeline.framebuffer_mut().color_buffer_mut().iter_mut() {
        pixel.r = pixel.r.powf(1.0 / 2.2);
        pixel.g = pixel.g.powf(1.0 / 2.2);
        pixel.b = pixel.b.powf(1.0 / 2.2);
    }

    println!("Saving to image");

    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("examples/basic.png").unwrap();
}