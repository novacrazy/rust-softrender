extern crate nalgebra;
extern crate image;
extern crate sdl2;
extern crate softrender;
extern crate full_example;

use std::sync::Arc;

use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels::{Color, PixelFormatEnum};
use sdl2::rect::{Point, Rect};

use softrender::render::{FrameBuffer, Pipeline, FaceWinding};
use softrender::mesh::Mesh;
use softrender::image_compat::ImageFrameBuffer;

use full_example::color::{self, Color as RenderColor};
use full_example::uniforms::GlobalUniforms;
use full_example::{generate_global_uniforms, model_matrix};
use full_example::mesh::MeshData;

fn render_frame(mut pipeline: &mut Pipeline<GlobalUniforms, RenderColor>, meshes: &Vec<MeshData>) {
    pipeline.framebuffer_mut().clear();

    // Iterator through all the meshes
    for mesh in meshes {
        // Initialize the pipeline for mesh, and return the vertex_shader object
        let vertex_shader = pipeline.render_mesh(mesh.mesh.clone());

        // Run the vertex shader, which returns the fragment shader
        let mut fragment_shader = vertex_shader.run(full_example::shaders::vertex_shader);

        // Set our pixel blend function
        fragment_shader.set_blend_function(|a, b| color::blend(a, b));

        fragment_shader.cull_faces(Some(FaceWinding::Clockwise));

        fragment_shader.triangles(full_example::shaders::fragment_shader);
    }
}

fn main() {
    let size = (800, 600);

    // Create the image framebuffer with a near-black background
    let framebuffer = FrameBuffer::new_with(size.0, size.1, RenderColor { r: 0.01, g: 0.01, b: 0.01, a: 1.0 });

    // Create any global uniforms you wish the shaders to have access to.
    let global_uniforms = generate_global_uniforms(framebuffer.width() as f32 / framebuffer.height() as f32,
                                                   75.0f32.to_radians(), 2.0,
                                                   0.0,
                                                   65.0f32.to_radians());

    // Create the graphics pipeline from the spawned framebuffer
    let mut pipeline = Pipeline::new(framebuffer, global_uniforms);

    // Load any meshes
    let meshes = full_example::mesh::load_model("../full_example/assets/models/suzanne_highres.obj");

    /////////////////

    let sdl_context = sdl2::init().unwrap();

    let video_subsystem = sdl_context.video().unwrap();

    let window = video_subsystem
        .window("realtime_example", size.0, size.1)
        .position_centered()
        .build()
        .unwrap();

    let mut canvas = window.into_canvas().software().build().unwrap();

    let creator = canvas.texture_creator();

    let mut texture = creator.create_texture_streaming(PixelFormatEnum::RGBA8888, size.0, size.1).unwrap();

    let mut angle: f32 = 0.0;

    'mainloop: loop {
        for event in sdl_context.event_pump().unwrap().poll_iter() {
            match event {
                Event::Quit { .. } |
                Event::KeyDown { keycode: Some(Keycode::Escape), .. } => break 'mainloop,
                _ => {}
            }
        }

        angle = (angle + 3.0) % 360.0;

        {
            let (model, mit) = model_matrix(angle.to_radians());

            pipeline.uniforms_mut().model = model;
            pipeline.uniforms_mut().model_inverse_transpose = mit;

            render_frame(&mut pipeline, &meshes);

            texture.with_lock(None, |buffer: &mut [u8], _: usize| {
                let color = pipeline.framebuffer().color_buffer();

                assert_eq!(color.len() * 4, buffer.len());

                for i in 0..color.len() {
                    let c = color[i];

                    let j = i * 4;

                    buffer[j + 0] = (c.a * 255.0) as u8;
                    buffer[j + 1] = (c.b * 255.0) as u8;
                    buffer[j + 2] = (c.g * 255.0) as u8;
                    buffer[j + 3] = (c.r * 255.0) as u8;
                }
            }).unwrap();
        }

        canvas.copy(&texture, None, None).expect("Render failed");

        canvas.present();
    }
}