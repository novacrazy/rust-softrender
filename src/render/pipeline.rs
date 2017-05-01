use std::sync::Arc;

use rayon::prelude::*;
use rayon::current_num_threads;

use nalgebra::coordinates::XYZW;

use ::utils::clamp;
use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use ::render::geometry::{FaceWinding, ClipVertex, ScreenVertex};
use ::render::framebuffer::FrameBuffer;
use ::render::uniform::Barycentric;

pub struct Pipeline<U, P> where P: Pixel, U: Send + Sync {
    framebuffer: FrameBuffer<P>,
    uniforms: U,
}

pub struct VertexShader<'a, V, U: 'a, P: 'static> where V: Send + Sync,
                                                        U: Send + Sync,
                                                        P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
}

pub struct FragmentShader<'a, V, U: 'a, K, P: 'static> where V: Send + Sync,
                                                             U: Send + Sync,
                                                             K: Send + Sync + Barycentric,
                                                             P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
    screen_vertices: Vec<ScreenVertex<K>>,
    cull_faces: Option<FaceWinding>,
    blend_func: Box<Fn(P, P) -> P + Send + Sync>,
}

///////////////////////

impl<U, P> Pipeline<U, P> where U: Send + Sync,
                                P: Pixel {
    /// Create a new rendering pipeline instance
    pub fn new(framebuffer: FrameBuffer<P>, uniforms: U) -> Pipeline<U, P> {
        assert!(framebuffer.width() > 0, "Framebuffer must have a non-zero width");
        assert!(framebuffer.height() > 0, "Framebuffer must have a non-zero height");

        Pipeline {
            framebuffer: framebuffer,
            uniforms: uniforms,
        }
    }

    /// Start the shading pipeline for a given mesh
    pub fn render_mesh<V>(&mut self, mesh: Arc<Mesh<V>>) -> VertexShader<V, U, P> where V: Send + Sync {
        VertexShader {
            mesh: mesh,
            uniforms: &self.uniforms,
            framebuffer: &mut self.framebuffer,
        }
    }

    /// Returns a reference to the uniforms value
    pub fn uniforms(&self) -> &U { &self.uniforms }
    /// Returns a mutable reference to the uniforms value
    pub fn uniforms_mut(&mut self) -> &mut U { &mut self.uniforms }

    /// Returns a reference to the framebuffer
    pub fn framebuffer(&self) -> &FrameBuffer<P> { &self.framebuffer }
    /// Returns a mutable reference to the framebuffer
    pub fn framebuffer_mut(&mut self) -> &mut FrameBuffer<P> { &mut self.framebuffer }
}

impl<'a, V, U: 'a, P: 'static> VertexShader<'a, V, U, P> where V: Send + Sync,
                                                               U: Send + Sync,
                                                               P: Pixel {
    pub fn run<S, K>(self, vertex_shader: S) -> FragmentShader<'a, V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                     K: Send + Sync + Barycentric {
        let VertexShader {
            mesh,
            uniforms,
            framebuffer
        } = self;

        let viewport = framebuffer.viewport();

        let vertices_per_thread = mesh.indices.len() / current_num_threads();

        let screen_vertices = mesh.vertices.par_iter()
                                           .with_min_len(vertices_per_thread)
                                           .map(|vertex| {
                                               vertex_shader(vertex, &*uniforms)
                                                   .normalize(viewport)
                                           }).collect();

        FragmentShader {
            mesh: mesh,
            uniforms: uniforms,
            framebuffer: framebuffer,
            screen_vertices: screen_vertices,
            cull_faces: None,
            // Use empty "normal" blend by default
            blend_func: Box::new(|s, _| s),
        }
    }
}

/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<P> where P: Sized + Pixel {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(P)
}

/// Describes the style of lines to be drawn in wireframe rendering mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LineStyle {
    /// Thin, aliased lines drawn using Bresenham's algorithm
    Thin,
    /// Thin, antialiased line drawn using Xiaolin Wu's algorithm
    ThinAA,
}

impl Default for LineStyle {
    fn default() -> LineStyle { LineStyle::ThinAA }
}

impl<'a, V, U: 'a, K, P: 'static> FragmentShader<'a, V, U, K, P> where V: Send + Sync,
                                                                       U: Send + Sync,
                                                                       K: Send + Sync + Barycentric,
                                                                       P: Pixel {
    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/enum.FaceWinding.html) enum.
    #[inline(always)]
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    /// Sets the blend function for blending pixels together.
    ///
    /// The first parameter passed to the blend function is the output of the fragment shader, the source color.
    ///
    /// The second parameter passed to the blend function is the existing value in the framebuffer to blend over.
    ///
    /// You can use the tool [Here](http://www.andersriggelsen.dk/glblendfunc.php) to see how OpenGL does blending,
    /// and choose how you want to blend pixels.
    ///
    /// For a generic alpha-over blend function, check the Wikipedia article [Here](https://en.wikipedia.org/wiki/Alpha_compositing)
    /// for the *over* color function.
    #[inline(always)]
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Box::new(f);
    }

    /// Render the vertices as a point cloud. Shading is done per-vertex for a single pixel.
    pub fn points<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        // Pull all variables out of self so we can borrow them individually.
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            screen_vertices,
            blend_func,
            ..
        } = self;

        // template framebuffer for the render framebuffers, allowing the real framebuffer to be borrowed mutably later on.
        let empty_framebuffer = framebuffer.empty_clone();

        // Only allow as many new empty framebuffer clones as their are running threads, so one framebuffer per thread.
        // This has the benefit of running a large of number of triangles sequentially.
        let points_per_thread = mesh.indices.len() / current_num_threads();

        // Bounding box for points in framebuffer
        let bb = (framebuffer.width() as f32,
                  framebuffer.height() as f32);

        let partial_framebuffers = mesh.indices.par_iter().cloned().with_min_len(points_per_thread).fold(
            || { empty_framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, index| {
                let ref vertex = screen_vertices[index as usize];

                let XYZW { x, y, z, .. } = *vertex.position;

                // don't render points "behind" or outside of the camera view
                if 0.0 <= x && x < bb.0 && 0.0 <= y && y < bb.1 && z > 0.0 {
                    let px = x as u32;
                    let py = y as u32;

                    if framebuffer.check_coordinate(px, py) {
                        let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                        if z < *fd {
                            match fragment_shader(vertex, &uniforms) {
                                Fragment::Color(c) => {
                                    *fc = (*blend_func)(c, *fc);
                                    *fd = z;
                                }
                                Fragment::Discard => ()
                            };
                        }
                    }
                }

                framebuffer
            });

        // Merge incoming partial framebuffers in parallel
        partial_framebuffers.reduce_with(|mut a, b| {
            b.merge_into(&mut a, &blend_func);
            a
        }).map(|final_framebuffer| {
            // Merge final framebuffer into external framebuffer
            final_framebuffer.merge_into(framebuffer, &blend_func);
        });
    }

    /// Vertices 0 and 1 are considered a line. Vertices 2 and 3 are considered a line. And so on.
    ///
    /// If the user specifies a non-even number of vertices, then the extra vertex is ignored.
    ///
    /// Equivalent to `GL_LINES` primitive
    pub fn lines<S>(self, fragment_shader: S, style: LineStyle) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        // Pull all variables out of self so we can borrow them individually.
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            screen_vertices,
            blend_func,
            ..
        } = self;

        // Bounding box for the entire view space
        let bb = (framebuffer.width() - 1,
                  framebuffer.height() - 1);

        // template framebuffer for the render framebuffers, allowing the real framebuffer to be borrowed mutably later on.
        let empty_framebuffer = framebuffer.empty_clone();

        // Only allow as many new empty framebuffer clones as their are running threads, so one framebuffer per thread.
        // This has the benefit of running a large of number of triangles sequentially.
        let lines_per_thread = mesh.indices.len() / (2 * current_num_threads());

        let partial_framebuffers = mesh.indices.par_chunks(2).with_min_len(lines_per_thread).filter(|line| line.len() == 2).fold(
            || { empty_framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, line| {
                let ref a = screen_vertices[line[0] as usize];
                let ref b = screen_vertices[line[1] as usize];

                let (x1, y1) = (a.position.x, a.position.y);
                let (x2, y2) = (b.position.x, b.position.y);

                let d = (x1 - x2).hypot(y1 - y2);

                {
                    let plot_fragment = |x, y, alpha| {
                        if x >= 0 && y >= 0 {
                            let x = x as u32;
                            let y = y as u32;

                            if x <= bb.0 && y <= bb.1 {
                                let d1 = (x1 - x as f32).hypot(y1 - y as f32);

                                let t = d1 / d;

                                let position = a.position * (1.0 - t) + b.position * t;

                                // Don't render pixels "behind" the camera
                                if position.z > 0.0 {
                                    if framebuffer.check_coordinate(x, y) {
                                        let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(x, y) };

                                        if position.z < *fd {
                                            // run fragment shader
                                            let fragment = fragment_shader(&ScreenVertex {
                                                position: position,
                                                uniforms: Barycentric::interpolate((1.0 - t), &a.uniforms,
                                                                                   t, &b.uniforms,
                                                                                   0.0, &b.uniforms),
                                            }, &uniforms);

                                            match fragment {
                                                Fragment::Color(c) => {
                                                    // blend pixels together and set the new depth value
                                                    *fc = (*blend_func)(c.with_alpha(alpha as f32), *fc);
                                                    *fd = 0.0;
                                                }
                                                Fragment::Discard => ()
                                            };
                                        }
                                    }
                                }
                            }
                        }
                    };

                    match style {
                        LineStyle::Thin => {
                            ::render::line::draw_line_bresenham(x1 as i64, y1 as i64, x2 as i64, y2 as i64, plot_fragment);
                        }
                        LineStyle::ThinAA => {
                            ::render::line::draw_line_xiaolin_wu(x1 as f64, y1 as f64, x2 as f64, y2 as f64, plot_fragment);
                        }
                    }
                }

                framebuffer
            }
        );

        // Merge incoming partial framebuffers in parallel
        partial_framebuffers.reduce_with(|mut a, b| {
            b.merge_into(&mut a, &blend_func);
            a
        }).map(|final_framebuffer| {
            // Merge final framebuffer into external framebuffer
            final_framebuffer.merge_into(framebuffer, &blend_func);
        });
    }

    /// Render a wireframe for every triangle in the mesh. Shading it done along the lines using linear
    /// interpolation for uniforms in the connected vertices.
    pub fn wireframe<S>(self, fragment_shader: S, style: LineStyle) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        // Pull all variables out of self so we can borrow them individually.
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            screen_vertices,
            blend_func,
            ..
        } = self;

        // Bounding box for the entire view space
        let bb = (framebuffer.width() - 1,
                  framebuffer.height() - 1);

        // template framebuffer for the render framebuffers, allowing the real framebuffer to be borrowed mutably later on.
        let empty_framebuffer = framebuffer.empty_clone();

        // Only allow as many new empty framebuffer clones as their are running threads, so one framebuffer per thread.
        // This has the benefit of running a large of number of triangles sequentially.
        let triangles_per_thread = mesh.indices.len() / (3 * current_num_threads());

        let partial_framebuffers = mesh.indices.par_chunks(3).with_min_len(triangles_per_thread).filter(|triangle| triangle.len() == 3).fold(
            || { empty_framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, triangle| {
                let ref a = screen_vertices[triangle[0] as usize];
                let ref b = screen_vertices[triangle[1] as usize];
                let ref c = screen_vertices[triangle[2] as usize];

                for &(a, b) in &[(a, b), (b, c), (c, a)] {
                    let (x1, y1) = (a.position.x, a.position.y);
                    let (x2, y2) = (b.position.x, b.position.y);

                    let d = (x1 - x2).hypot(y1 - y2);

                    let plot_fragment = |x, y, alpha| {
                        if x >= 0 && y >= 0 {
                            let x = x as u32;
                            let y = y as u32;

                            if x <= bb.0 && y <= bb.1 {
                                let d1 = (x1 - x as f32).hypot(y1 - y as f32);

                                let t = d1 / d;

                                let position = a.position * (1.0 - t) + b.position * t;

                                // Don't render pixels "behind" the camera
                                if position.z > 0.0 {
                                    if framebuffer.check_coordinate(x, y) {
                                        let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(x, y) };

                                        if position.z < *fd {
                                            // run fragment shader
                                            let fragment = fragment_shader(&ScreenVertex {
                                                position: position,
                                                uniforms: Barycentric::interpolate((1.0 - t), &a.uniforms,
                                                                                   t, &b.uniforms,
                                                                                   0.0, &b.uniforms),
                                            }, &uniforms);

                                            match fragment {
                                                Fragment::Color(c) => {
                                                    // blend pixels together and set the new depth value
                                                    *fc = (*blend_func)(c.with_alpha(alpha as f32), *fc);
                                                    *fd = 0.0;
                                                }
                                                Fragment::Discard => ()
                                            };
                                        }
                                    }
                                }
                            }
                        }
                    };

                    match style {
                        LineStyle::Thin => {
                            ::render::line::draw_line_bresenham(x1 as i64, y1 as i64, x2 as i64, y2 as i64, plot_fragment);
                        }
                        LineStyle::ThinAA => {
                            ::render::line::draw_line_xiaolin_wu(x1 as f64, y1 as f64, x2 as f64, y2 as f64, plot_fragment);
                        }
                    }
                }

                framebuffer
            });

        // Merge incoming partial framebuffers in parallel
        partial_framebuffers.reduce_with(|mut a, b| {
            b.merge_into(&mut a, &blend_func);
            a
        }).map(|final_framebuffer| {
            // Merge final framebuffer into external framebuffer
            final_framebuffer.merge_into(framebuffer, &blend_func);
        });
    }

    /// Rasterize the given vertices as triangles.
    ///
    /// Equivalent to `GL_TRIANGLES`
    pub fn triangles<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> Fragment<P> + Send + Sync {
        // Pull all variables out of self so we can borrow them individually.
        let FragmentShader {
            mesh,
            uniforms,
            framebuffer,
            screen_vertices,
            cull_faces,
            blend_func
        } = self;

        // Bounding box for the entire view space
        let bb = (framebuffer.width() - 1,
                  framebuffer.height() - 1);

        // template framebuffer for the render framebuffers, allowing the real framebuffer to be borrowed mutably later on.
        let empty_framebuffer = framebuffer.empty_clone();

        // Only allow as many new empty framebuffer clones as their are running threads, so one framebuffer per thread.
        // This has the benefit of running a large of number of triangles sequentially.
        let triangles_per_thread = mesh.indices.len() / (3 * current_num_threads());

        let partial_framebuffers = mesh.indices.par_chunks(3).with_min_len(triangles_per_thread).filter(|triangle| {
            // if there are three points at all, go ahead
            if triangle.len() == 3 {
                //TODO: Check if triangle is on screen at all.

                // If there is a winding order for culling,
                // compare it to the triangle winding order,
                // otherwise go ahead
                if let Some(winding) = cull_faces {
                    let ref a = screen_vertices[triangle[0] as usize];
                    let ref b = screen_vertices[triangle[1] as usize];
                    let ref c = screen_vertices[triangle[2] as usize];

                    let (x1, y1) = (a.position.x, a.position.y);
                    let (x2, y2) = (b.position.x, b.position.y);
                    let (x3, y3) = (c.position.x, c.position.y);

                    let area2 = -x2 * y1 + 2.0 * x3 * y1 + x1 * y2 - x3 * y2 + 2.0 * x1 * y3 + x2 * y3;

                    // Check if the winding order matches the desired order
                    if winding == if area2.is_sign_negative() { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise } {
                        true
                    } else {
                        false
                    }
                } else {
                    true
                }
            } else {
                false
            }
        }).fold(|| { empty_framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, triangle| {
            let ref a = screen_vertices[triangle[0] as usize];
            let ref b = screen_vertices[triangle[1] as usize];
            let ref c = screen_vertices[triangle[2] as usize];

            let (x1, y1) = (a.position.x, a.position.y);
            let (x2, y2) = (b.position.x, b.position.y);
            let (x3, y3) = (c.position.x, c.position.y);

            // find x bounds for the bounding box
            let min_x: u32 = clamp(x1.min(x2).min(x3).floor() as u32, 0, bb.0);
            let max_x: u32 = clamp(x1.max(x2).max(x3).ceil() as u32, 0, bb.0);

            // find y bounds for the bounding box
            let min_y: u32 = clamp(y1.min(y2).min(y3).floor() as u32, 0, bb.1);
            let max_y: u32 = clamp(y1.max(y2).max(y3).ceil() as u32, 0, bb.1);

            let mut py = min_y;

            while py <= max_y {
                let mut px = min_x;

                while px <= max_x {
                    // Real screen position should be in the center of the pixel.
                    let (x, y) = (px as f32 + 0.5,
                                  py as f32 + 0.5);

                    // calculate determinant
                    let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

                    // calculate barycentric coordinates of the current point
                    let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                    let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                    let w = 1.0 - u - v;

                    // check if the point is inside the triangle at all
                    if u >= 0.0 && v >= 0.0 && w >= 0.0 {
                        // interpolate screen-space position
                        let position = a.position * u + b.position * v + c.position * w;

                        // don't render pixels "behind" the camera
                        if position.z > 0.0 {
                            let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                            // skip fragments that are behind over previous fragments
                            if position.z < *fd {
                                // run fragment shader
                                let fragment = fragment_shader(&ScreenVertex {
                                    position: position,
                                    // interpolate the uniforms
                                    uniforms: Barycentric::interpolate(u, &a.uniforms,
                                                                       v, &b.uniforms,
                                                                       w, &c.uniforms),
                                }, &uniforms);

                                match fragment {
                                    Fragment::Color(c) => {
                                        // blend pixels together and set the new depth value
                                        *fc = (*blend_func)(c, *fc);
                                        *fd = position.z;
                                    }
                                    Fragment::Discard => ()
                                };
                            }
                        }
                    }

                    px += 1;
                }

                py += 1;
            }

            framebuffer
        });

        // Merge incoming partial framebuffers in parallel
        partial_framebuffers.reduce_with(|mut a, b| {
            b.merge_into(&mut a, &blend_func);
            a
        }).map(|final_framebuffer| {
            // Merge final framebuffer into external framebuffer
            final_framebuffer.merge_into(framebuffer, &blend_func);
        });
    }
}