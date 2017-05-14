extern crate softrender;
extern crate full_example;

use softrender::render::{FrameBuffer, Pipeline, GenericBlend, Primitive, PrimitiveMut, PrimitiveRef};
use softrender::image_compat::ImageFrameBuffer;

use full_example::color::{self, Color};
use full_example::generate_global_uniforms;
use full_example::uniforms::GlobalUniforms;

fn example1() {
    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::<Color>::new_with(1920, 1080, Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    // Create any global uniforms you wish the shaders to have access to.
    let global_uniforms = generate_global_uniforms(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                   75.0f32.to_radians(), 2.0,
                                                   45.0f32.to_radians(),
                                                   65.0f32.to_radians());

    // Create the graphics pipeline from the spawned framebuffer
    let mut pipeline = Pipeline::new(framebuffer, global_uniforms);

    // Load any meshes
    let meshes = full_example::mesh::load_model("assets/models/suzanne_highres.obj");

    // Iterate through all the meshes
    for mesh in &meshes {
        // Initialize the pipeline for mesh, and return the vertex_shader object
        let vertex_shader = pipeline.render_mesh(Primitive::Triangle, mesh.mesh.clone());

        // Run the vertex shader and skip the geometry shader
        let mut fragment_shader = vertex_shader.run_to_fragment(full_example::shaders::vertex_shader)
                                               .with_default_blend::<GenericBlend<_>>();

        // Set our pixel blend function
        fragment_shader.set_blend_function(|a, b| color::blend(a, b));

        fragment_shader.run(full_example::shaders::fragment_shader);
    }

    println!("Saving example 1 to image");

    // copy the framebuffer into an image then save it to a file
    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("example.png").unwrap();
}

fn example2() {
    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::<Color>::new_with(1920, 1080, Color { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    // Create any global uniforms you wish the shaders to have access to.
    let global_uniforms = generate_global_uniforms(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                   75.0f32.to_radians(), 2.0,
                                                   45.0f32.to_radians(),
                                                   65.0f32.to_radians());

    // Create the graphics pipeline from the spawned framebuffer
    let mut pipeline = Pipeline::new(framebuffer, global_uniforms);

    // Load any meshes
    let meshes = full_example::mesh::load_model("assets/models/suzanne_highres.obj");

    // Iterate through all the meshes
    for mesh in &meshes {
        // Initialize the pipeline for mesh, and return the vertex_shader object
        let vertex_shader = pipeline.render_mesh(Primitive::Triangle, mesh.mesh.clone());

        // Run the vertex shader, which returns the geometry shader
        let mut geometry_shader = vertex_shader.run(full_example::shaders::vertex_shader);

        {
            let mut fragment_shader = geometry_shader.duplicate().finish().with_default_blend::<GenericBlend<_>>();

            // Set our pixel blend function
            fragment_shader.set_blend_function(|a, b| color::blend(a, b));

            fragment_shader.run(full_example::shaders::fragment_shader);
        }

        {
            let geometry_shader = geometry_shader.replace(full_example::shaders::geometry_shader_visualize_face_normals);

            let mut fragment_shader = geometry_shader.finish().with_default_blend::<GenericBlend<_>>();

            // Set our pixel blend function
            fragment_shader.set_blend_function(|a, b| color::blend(a, b));

            fragment_shader.run(full_example::shaders::fragment_shader_green);
        }
    }

    println!("Saving example 2 to image");

    // copy the framebuffer into an image then save it to a file
    let image = pipeline.framebuffer().copy_to_image().unwrap();

    image.save("example2.png").unwrap();
}

fn main() {
    example1();
    example2();
}
