extern crate nalgebra;
extern crate tobj;
extern crate image;
extern crate rayon;

#[macro_use]
extern crate softrender;

use std::sync::Arc;
use std::path::Path;

use nalgebra::{Point3, Vector4, Vector3, Matrix4};

use softrender::color::predefined::formats::RGBAf32Color;
use softrender::geometry::{Dimensions, HasDimensions, ClipVertex};
use softrender::primitive;
use softrender::mesh::{Mesh, Vertex};
use softrender::attachments::predefined::ColorDepthAttachments;
use softrender::framebuffer::{Framebuffer, RenderBuffer};
use softrender::pipeline::{Pipeline, PipelineObject};
use softrender::pipeline::stages::fragment::Fragment;

/// Defines data stored alongside vertex position
struct VertexData {
    normal: Vector3<f32>,
}

/// Loads a .obj mesh from the given path
fn load_model<P: AsRef<Path>>(p: P) -> Vec<Arc<Mesh<VertexData>>> {
    let (models, _): (Vec<tobj::Model>, _) = tobj::load_obj(p.as_ref().clone()).unwrap();

    models.into_iter().map(|model| {
        let ref mesh: tobj::Mesh = model.mesh;

        assert_eq!(mesh.positions.len(), mesh.normals.len());

        let positions = mesh.positions.chunks(3);
        let normals = mesh.normals.chunks(3);

        Arc::new(Mesh {
            vertices: positions.zip(normals).map(|(position, normal)| {
                Vertex {
                    position: Point3::new(position[0], position[1], position[2]),
                    vertex_data: VertexData {
                        normal: Vector3::new(normal[0], normal[1], normal[2]),
                    }
                }
            }).collect(),
            indices: mesh.indices.clone()
        })
    }).collect()
}

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
    #[derive(Debug, Clone, Copy)]
    pub struct Uniforms {
        pub position: Vector4<f32>,
        pub normal: Vector4<f32>,
    }
}


#[allow(non_snake_case)]
fn main() {
    let mut framebuffer = RenderBuffer::<ColorDepthAttachments<RGBAf32Color, f32>>::with_dimensions(Dimensions::new(2000, 2000));

    framebuffer.clear(RGBAf32Color::new(0.01, 0.01, 0.01, 1.0));

    // Create the model transform
    let model = nalgebra::Isometry3::new(Vector3::new(0.0, 0.0, 0.0),
                                         Vector3::new(0.0, 0.0, 0.0));

    // Define camera position
    let camera_position = Point3::new(1.0, 0.0, 2.0);

    // Create view matrix from camera position and look-at target
    let view = nalgebra::Isometry3::look_at_rh(
        &camera_position,
        &Point3::origin(),
        &Vector3::new(0.0, 1.0, 0.0)
    ).to_homogeneous();

    // Create a perspective projection matrix
    let projection = nalgebra::Perspective3::new(framebuffer.dimensions().width as f32 / framebuffer.dimensions().height as f32,
                                                 75.0f32.to_radians(), 0.001, 1000.0).to_homogeneous();

    // Create a new rendering pipeline and give it all the global uniforms.
    //
    // The global uniforms can be changed between renders by using pipeline.uniforms_mut()
    let mut pipeline = Pipeline::from_framebuffer(framebuffer, GlobalUniforms {
        camera: camera_position.to_homogeneous(),
        model: model.to_homogeneous(),
        model_inverse_transpose: model.inverse().to_homogeneous().transpose(),
        view: view,
        projection: projection
    });

    let meshes = load_model("examples/assets/suzanne.obj");

    println!("Rendering meshes");

    let light = Point3::new(5.0, 5.0, 5.0).to_homogeneous();
    let intensity = 4.0;

    // Convert sRGB color to linear for rendering
    let color = RGBAf32Color::new(0.1f32.powf(2.2), 0.5f32.powf(2.2), 0.1f32.powf(2.2), 1.0);

    // Iterate through available meshes
    for mesh in &meshes {
        let dimensions = pipeline.framebuffer().dimensions();

        // Begin the rendering of a given mesh, which returns the vertex shader object
        let vertex_shader = pipeline.render_mesh(primitive::Triangle, mesh.clone());

        let geometry_shader = vertex_shader.run(|vertex: &Vertex<VertexData>, global_uniforms: &GlobalUniforms| -> ClipVertex<Uniforms> {
            let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
            let VertexData { normal } = vertex.vertex_data;

            // Transform vertex position to world-space
            let world_position = model * vertex.position.to_homogeneous();

            // Transform normal to world-space
            let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();

            // Transform vertex position to clip-space (projection-space)
            let clip_position = projection * view * world_position;

            // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
            ClipVertex::new(clip_position, Uniforms {
                position: world_position,
                normal: normal,
            })
        });

        let fragment_shader = geometry_shader.clip_primitives().finish((dimensions.width as f32,
                                                                        dimensions.height as f32));

        // Render the vertices as triangles
        fragment_shader.run(|screen_vertex, global_uniforms| {
            // Get all the uniforms
            let GlobalUniforms { ref camera, .. } = *global_uniforms;
            let Uniforms { position, normal } = screen_vertex.uniforms;

            let shininess = 32.0;

            let view_dir = (camera - position).normalize();

            let light_dir = (light - position).normalize();
            let halfway_vector = (light_dir + view_dir).normalize();

            let NdotL = light_dir.dot(&normal).min(1.0).max(0.0);
            let NdotH = normal.dot(&halfway_vector).min(1.0).max(0.0);
            let VdotH = view_dir.dot(&halfway_vector).min(1.0).max(0.0);

            // Simple fresnel schlick approximation
            fn fresnel_schlick(cos_theta: f32, ior: f32) -> f32 {
                let f0 = ((1.0 - ior) / (1.0 + ior)).powi(2);

                f0 + (1.0 - f0) * (1.0 - cos_theta).powi(5)
            }

            // Fresnel is used to blend together specular and diffuse lighting
            let f = fresnel_schlick(VdotH, 1.45);

            let diffuse = NdotL * (1.0 - f);
            let specular = f * NdotH.powf(shininess * 2.0);

            // Return the color of the fragment, adjusting for gamma
            Fragment::Color(RGBAf32Color::new(
                (intensity * (specular + (diffuse * color.x))).powf(1.0 / 2.2),
                (intensity * (specular + (diffuse * color.y))).powf(1.0 / 2.2),
                (intensity * (specular + (diffuse * color.z))).powf(1.0 / 2.2),
                1.0
            ))
        });
    }

    /*
    println!("Saving to image");

    // copy the framebuffer into an image then save it to a file
    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("examples/suzanne.png").unwrap();
    */
}