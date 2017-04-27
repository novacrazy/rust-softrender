use std::sync::Arc;

use rayon::prelude::*;
use rayon::current_num_threads;

use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use ::render::geometry::{FaceWinding, ClipVertex, ScreenVertex};
use ::render::geometry::{winding_order_from_signed_area, triangle_signed_area};
use ::render::framebuffer::FrameBuffer;
use ::render::uniform::Barycentric;

pub struct Pipeline<U, P> where P: Pixel, U: Send + Sync {
    framebuffer: FrameBuffer<P>,
    uniforms: U,
}

pub struct VertexShader<'a, V, U, P: 'static> where V: Send + Sync,
                                                    U: Send + Sync + 'a,
                                                    P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
}

pub struct FragmentShader<'a, V, U, K, P: 'static> where V: Send + Sync,
                                                         U: Send + Sync + 'a,
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

impl<'a, V, U, P: 'static> VertexShader<'a, V, U, P> where V: Send + Sync,
                                                           U: Send + Sync + 'a,
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

impl<'a, V, U, K, P: 'static> FragmentShader<'a, V, U, K, P> where V: Send + Sync,
                                                                   U: Send + Sync + 'a,
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

    pub fn run<S>(mut self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> P + Send + Sync {
        println!("Running fragment shader");

        let bb = (self.framebuffer.width() as f32 - 1.0,
                  self.framebuffer.height() as f32 - 1.0);

        let triangles_per_thread = self.mesh.indices.len() / (3 * current_num_threads());

        let partial_framebuffers: Vec<FrameBuffer<P>> = self.mesh.indices.par_chunks(3).with_min_len(triangles_per_thread).filter_map(|triangle| {
            // if there are three points at all, go ahead
            if triangle.len() == 3 {
                //println!("Triangle {:?}", triangle);

                let ref a = self.screen_vertices[triangle[0] as usize];
                let ref b = self.screen_vertices[triangle[1] as usize];
                let ref c = self.screen_vertices[triangle[2] as usize];

                //TODO: Check if triangle is on screen at all.

                let area = triangle_signed_area(a.position.x, a.position.y,
                                                b.position.x, b.position.y,
                                                c.position.x, c.position.y);

                // If there is a winding order for culling,
                // compare it to the triangle winding order,
                // otherwise go ahead
                if let Some(winding) = self.cull_faces {
                    // Check if the winding order matches the desired order
                    if winding == winding_order_from_signed_area(area) {
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
        }).fold(|| {
            println!("New framebuffer clone");
            self.framebuffer.empty_clone()
        }, |mut framebuffer: FrameBuffer<P>, triangle| {
            let ref a = self.screen_vertices[triangle[0] as usize];
            let ref b = self.screen_vertices[triangle[1] as usize];
            let ref c = self.screen_vertices[triangle[2] as usize];

            let (x1, y1) = (a.position.x, a.position.y);
            let (x2, y2) = (b.position.x, b.position.y);
            let (x3, y3) = (c.position.x, c.position.y);

            let min_x = x1.min(x2).min(x3).max(0.0).min(bb.0);
            let max_x = x1.max(x2).max(x3).max(0.0).min(bb.0);

            let min_y = y1.min(y2).min(y3).max(0.0).min(bb.1);
            let max_y = y1.max(y2).max(y3).max(0.0).min(bb.1);

            let mut y = min_y;

            while y < max_y {
                let mut x = min_x;

                while x < max_x {
                    {
                        let x = x.floor();
                        let y = y.floor();

                        let (px, py) = (x as u32, y as u32);

                        let denom = (y2 - y3) * (x1 - x3) + (x3 - x2) * (y1 - y3);

                        let u = ((y2 - y3) * (x - x3) + (x3 - x2) * (y - y3)) / denom;
                        let v = ((y3 - y1) * (x - x3) + (x1 - x3) * (y - y3)) / denom;
                        let r = 1.0 - u - v;

                        if u >= 0.0 && v >= 0.0 && r >= 0.0 {
                            let position = a.position * u + b.position * v + c.position * r;

                            let (fc, fd) = unsafe { framebuffer.pixel_depth_mut(px, py) };

                            if position.z < *fd {
                                let c = fragment_shader(&ScreenVertex {
                                    position: position,
                                    uniforms: Barycentric::interpolate(u, &a.uniforms, v, &b.uniforms, r, &c.uniforms),
                                }, &self.uniforms);

                                *fc = (*self.blend_func)(c, *fc);
                                *fd = position.z;
                            }
                        }
                        //}
                    }

                    x += 1.0;
                }

                y += 1.0;
            }

            /*
            unsafe {
                if framebuffer.check_coordinate(x1 as u32, y1 as u32) {
                    *framebuffer.pixel_mut(x1 as u32, y1 as u32) = fragment_shader(&a, &self.uniforms);
                    *framebuffer.depth_mut(x1 as u32, y1 as u32) = z1;
                }
                if framebuffer.check_coordinate(x2 as u32, y2 as u32) {
                    *framebuffer.pixel_mut(x2 as u32, y2 as u32) = fragment_shader(&a, &self.uniforms);
                    *framebuffer.depth_mut(x2 as u32, y2 as u32) = z2;
                }
                if framebuffer.check_coordinate(x3 as u32, y3 as u32) {
                    *framebuffer.pixel_mut(x3 as u32, y3 as u32) = fragment_shader(&a, &self.uniforms);
                    *framebuffer.depth_mut(x3 as u32, y3 as u32) = z3;
                }
            }
            */

            //println!("Triangle {:?} with Area {}", ((x1, y1, z1), (x2, y2, z2), (x3, y3, z3)), area);

            framebuffer
        }).collect();

        for pf in partial_framebuffers.into_iter() {
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
}