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
        println!("Running vertex shader");

        let screen_vertices = self.mesh.vertices.par_iter().map(|vertex| {
            vertex_shader(vertex, &*self.uniforms)
                .normalize(self.framebuffer.viewport())
        }).collect();

        FragmentShader {
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            screen_vertices: screen_vertices,
            cull_faces: None,
            blend_func: Box::new(|s, _| s),
        }
    }
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
    /// and choose how you want to blend functions.
    ///
    /// For a generic alpha-over blend function, check the Wikipedia article [Here](https://en.wikipedia.org/wiki/Alpha_compositing)
    /// for the *over* color function.
    #[inline(always)]
    pub fn set_blend_function<F>(&mut self, f: F) where F: Fn(P, P) -> P + Send + Sync + 'static {
        self.blend_func = Box::new(f);
    }

    fn merge_framebuffers(&mut self, sources: Vec<FrameBuffer<P>>) {
        for pf in sources.into_iter() {
            let (pcolor, pdepth) = pf.buffers();
            let (mut fcolor, mut fdepth) = self.framebuffer.buffers_mut();

            let fiter = fcolor.iter_mut().zip(fdepth.iter_mut());
            let piter = pcolor.iter().zip(pdepth.iter());

            for ((pc, pd), (fc, fd)) in piter.zip(fiter) {
                if *pd < *fd {
                    *fd = *pd;
                    *fc = (*self.blend_func)(*pc, *fc);
                }
            }
        }
    }

    pub fn points<S>(mut self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> P + Send + Sync {
        let points_per_thread = self.mesh.indices.len() / current_num_threads();

        let bb = (self.framebuffer.width() as f32,
                  self.framebuffer.height() as f32);

        let partial_framebuffers = self.mesh.indices.par_iter().cloned().with_min_len(points_per_thread).fold(
            || { self.framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, index| {
                let ref vertex = self.screen_vertices[index as usize];

                let XYZW { x, y, z, .. } = *vertex.position;

                // don't render pixels "behind" or outside of the camera view
                if 0.0 <= x && x < bb.0 && 0.0 <= y && y < bb.1 && z > 0.0 {
                    let px = x as u32;
                    let py = y as u32;

                    if framebuffer.check_coordinate(px, py) {
                        let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                        if z < *fd {
                            let c = fragment_shader(vertex, &self.uniforms);

                            *fc = (*self.blend_func)(c, *fc);
                            *fd = z;
                        }
                    }
                }

                framebuffer
            }).collect();

        self.merge_framebuffers(partial_framebuffers)
    }

    pub fn triangles<S>(mut self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> P + Send + Sync {
        let bb = (self.framebuffer.width() as f32 - 1.0,
                  self.framebuffer.height() as f32 - 1.0);

        let triangles_per_thread = self.mesh.indices.len() / (3 * current_num_threads());

        let partial_framebuffers = self.mesh.indices.par_chunks(3).with_min_len(triangles_per_thread).filter_map(|triangle| {
            // if there are three points at all, go ahead
            if triangle.len() == 3 {
                //TODO: Check if triangle is on screen at all.

                // If there is a winding order for culling,
                // compare it to the triangle winding order,
                // otherwise go ahead
                if let Some(winding) = self.cull_faces {
                    let ref a = self.screen_vertices[triangle[0] as usize];
                    let ref b = self.screen_vertices[triangle[1] as usize];
                    let ref c = self.screen_vertices[triangle[2] as usize];

                    let (x1, y1) = (a.position.x, a.position.y);
                    let (x2, y2) = (b.position.x, b.position.y);
                    let (x3, y3) = (c.position.x, c.position.y);

                    let area2 = -x2 * y1 + 2.0 * x3 * y1 + x1 * y2 - x3 * y2 + 2.0 * x1 * y3 + x2 * y3;

                    // Check if the winding order matches the desired order
                    if winding == if area2.is_sign_negative() { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise } {
                        Some(triangle)
                    } else {
                        None
                    }
                } else {
                    Some(triangle)
                }
            } else {
                None
            }
        }).fold(|| { self.framebuffer.empty_clone() }, |mut framebuffer: FrameBuffer<P>, triangle| {
            let ref a = self.screen_vertices[triangle[0] as usize];
            let ref b = self.screen_vertices[triangle[1] as usize];
            let ref c = self.screen_vertices[triangle[2] as usize];

            let (x1, y1) = (a.position.x, a.position.y);
            let (x2, y2) = (b.position.x, b.position.y);
            let (x3, y3) = (c.position.x, c.position.y);

            // find x bounds for the bounding box
            let min_x = clamp(x1.min(x2).min(x3).floor(), 0.0, bb.0);
            let max_x = clamp(x1.max(x2).max(x3).ceil(), 0.0, bb.0);

            // find y bounds for the bounding box
            let min_y = clamp(y1.min(y2).min(y3).floor(), 0.0, bb.1);
            let max_y = clamp(y1.max(y2).max(y3).ceil(), 0.0, bb.1);

            let mut y = min_y;

            while y <= max_y {
                let mut x = min_x;

                while x <= max_x {
                    let (px, py) = (x as u32, y as u32);

                    // calculate determinant
                    let det = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

                    // calculate barycentric coordinates of the current point
                    let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / det;
                    let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / det;
                    let r = 1.0 - u - v;

                    // check if the point is inside the triangle at all
                    if u >= 0.0 && v >= 0.0 && r >= 0.0 {
                        // interpolate screen-space position
                        let position = a.position * u + b.position * v + c.position * r;

                        // don't render pixels "behind" the camera
                        if position.z > 0.0 {
                            let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                            // skip fragments that are behind over previous fragments
                            if position.z < *fd {
                                // run fragment shader
                                let c = fragment_shader(&ScreenVertex {
                                    position: position,
                                    // interpolate the uniforms
                                    uniforms: Barycentric::interpolate(u, &a.uniforms,
                                                                       v, &b.uniforms,
                                                                       r, &c.uniforms),
                                }, &self.uniforms);

                                // blend pixels together and set the new depth value
                                *fc = (*self.blend_func)(c, *fc);
                                *fd = position.z;
                            }
                        }
                    }

                    x += 1.0;
                }

                y += 1.0;
            }

            framebuffer
        }).collect();

        self.merge_framebuffers(partial_framebuffers)
    }
}