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

/// Defines data stored alongside vertex position
struct VertexData {
    normal: Vector3<f32>,
}

/// Loads a .obj mesh from the given path
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

/// Gets example meshes
fn get_meshes() -> Vec<Arc<Mesh<VertexData>>> {
    vec![
        load_mesh("examples/suzanne_highres.obj"),
        load_mesh("examples/plane.obj")
    ]
}

// GLSL habit die hard
#[allow(non_snake_case)]
fn main() {
    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::<RGBAf32Pixel>::new_with(2000, 2000, RGBAf32Pixel { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    // Define global uniforms
    struct GlobalUniforms {
        camera: Vector4<f32>,
        model: Matrix4<f32>,
        // the inverse transpose of the model matrix is
        // multiplied by the normal vector to get the correct value
        model_inverse_transpose: Matrix4<f32>,
        view: Matrix4<f32>,
        projection: Matrix4<f32>,
    }

    // Define shader uniforms that can be interpolated on the triangle.
    // The declare_uniforms! macro helps implement the Barycentric trait on the resulting structure
    declare_uniforms! {
        Uniforms {
            position: Vector4<f32>,
            normal: Vector4<f32>,
        }
    }

    // Create the model transform
    let model = nalgebra::Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                         Vector3::new(0.0, 0.0, 0.0));

    // Define camera position
    let camera_position = Point3::new(2.0, 2.0, 2.0);

    // Create view matrix from camera position and look-at target
    let view = nalgebra::Isometry3::look_at_rh(
        &camera_position,
        &Point3::new(0.0, 0.0, 0.0),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    // Create a perspective projection matrix
    let projection = nalgebra::Perspective3::new(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                 75.0f32.to_radians(), 0.001, 1000.0).to_homogeneous();

    // Create a new rendering pipeline and give it all the global uniforms.
    //
    // The global uniforms can be changed between renders by using pipeline.uniforms_mut()
    let mut pipeline = Pipeline::new(framebuffer, GlobalUniforms {
        camera: camera_position.to_homogeneous(),
        model: model.to_homogeneous(),
        model_inverse_transpose: model.inverse().to_homogeneous().transpose(),
        view: view,
        projection: projection
    });

    // Iterate through available meshes
    for mesh in &get_meshes() {
        // Begin the rendering of a given mesh, which returns the vertex shader object
        let vertex_shader = pipeline.render_mesh(mesh.clone());

        // Run the vertex shader and perform all the required transforms, returning a clip-space coordinate and the declared uniforms.
        //
        // After the vertex shader is done, it continues on and returns the fragment shader.
        let mut fragment_shader = vertex_shader.run(|vertex, global_uniforms| {
            let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
            let VertexData { normal } = vertex.vertex_data;

            let world_position = model * vertex.position.to_homogeneous();

            let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();

            let clip_position = projection * view * world_position;

            // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
            ClipVertex::new(clip_position, Uniforms {
                position: world_position,
                normal: normal,
            })
        });

        // Set the blend function to
        //
        // source * sourceAlpha + destination * (1 - sourceAlpha)
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

        // Convenience variable to tweak distance of lights from object
        let light_scale = 1.0;

        // Define a few lights to evaluate in the fragment shader
        let lights = [
            Point3::new(light_scale * 1.0, light_scale * 1.0, light_scale * 1.0),
            Point3::new(light_scale * -1.0, light_scale * 1.0, light_scale * -1.0),
            Point3::new(light_scale * 0.0, light_scale * 3.0, light_scale * -1.0),
            Point3::new(light_scale * -2.0, light_scale * -1.0, light_scale * 1.0),
        ];

        // Define the diffuse color of the objects
        let diffuse_color = RGBAf32Pixel { r: 0.1, g: 0.1, b: 0.6, a: 1.0 };

        // Render the vertices as triangles
        fragment_shader.triangles(|screen_vertex, global_uniforms| {
            // Get all the uniforms
            let GlobalUniforms { ref camera, .. } = *global_uniforms;
            let Uniforms { position, normal } = screen_vertex.uniforms;

            // Equivalent to gl_FragCoord
            let frag_coord = screen_vertex.position;

            let view_dir = (camera - position).normalize();

            let mut specular = 0.0;
            let mut diffuse = 0.0;

            // Accumulate lighting
            for light in &lights {
                let light = light.to_homogeneous();

                let light_dir = (light - position).normalize();
                let halfway_vector = (light_dir + view_dir).normalize();

                let NdotL = light_dir.dot(&normal).min(1.0).max(0.0);
                let NdotH = normal.dot(&halfway_vector).min(1.0).max(0.0);
                let VdotH = view_dir.dot(&halfway_vector).min(1.0).max(0.0);

                // Simple fresnel schlick fresnel approximation
                fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
                    let f0 = ((1.0 - ior) / (1.0 + ior)).powi(2);

                    f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
                }

                // Fresnel is used to blend together specular and diffuse lighting
                let f = fresnel_schlick(VdotH, 1.45);

                diffuse += NdotL * (1.0 - f);
                specular += f * NdotH.powf(24.0);
            }

            // Return the color of the fragment
            RGBAf32Pixel {
                r: specular * 10.0 + (diffuse * diffuse_color.r),
                g: specular * 10.0 + (diffuse * diffuse_color.g),
                b: specular * 10.0 + (diffuse * diffuse_color.b),
                a: diffuse_color.a
            }
        });
    }

    println!("Adjusting gamma");

    // Convert linear color into sRGB color space
    for pixel in pipeline.framebuffer_mut().color_buffer_mut().iter_mut() {
        pixel.r = pixel.r.powf(1.0 / 2.2);
        pixel.g = pixel.g.powf(1.0 / 2.2);
        pixel.b = pixel.b.powf(1.0 / 2.2);
    }

    println!("Saving to image");

    // copy the framebuffer into an image then save it to a file
    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("examples/basic.png").unwrap();
}