//! Shader Execution Pipeline

use std::sync::Arc;
use std::ops::Deref;

use nalgebra::{Point3, Vector2, Vector4};
use nalgebra::core::coordinates::XYZW;

use rayon::prelude::*;

use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use super::framebuffer::FrameBuffer;
use super::uniform::BarycentricInterpolation;

pub struct Pipeline<U, P> where P: Pixel, U: Send + Sync {
    framebuffer: FrameBuffer<P>,
    uniforms: U,
}

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

pub struct VertexShader<'a, V, U, P: 'static> where V: Send + Sync,
                                                    U: Send + Sync + 'a,
                                                    P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
}

pub struct ClipVertex<K> where K: Send + Sync + BarycentricInterpolation {
    pub position: Vector4<f32>,
    pub uniforms: K,
}

impl<K> ClipVertex<K> where K: Send + Sync + BarycentricInterpolation {
    #[inline(always)]
    pub fn new(position: Vector4<f32>, uniforms: K) -> ClipVertex<K> {
        ClipVertex { position: position, uniforms: uniforms }
    }

    pub fn normalize(self, viewport: (f32, f32)) -> ScreenVertex<K> {
        ScreenVertex {
            position: {
                let XYZW { x, y, z, w } = *self.position;

                Vector4::new(
                    (x / w + 1.0) * (viewport.0 / 2.0),
                    // Vertical is flipped
                    (1.0 - y / w) * (viewport.1 / 2.0),
                    -z,
                    1.0
                )
            },
            uniforms: self.uniforms,
        }
    }
}

pub struct ScreenVertex<K> where K: Send + Sync + BarycentricInterpolation {
    pub position: Vector4<f32>,
    pub uniforms: K,
}

impl<'a, V, U, P: 'static> VertexShader<'a, V, U, P> where V: Send + Sync,
                                                           U: Send + Sync + 'a,
                                                           P: Pixel {
    pub fn run<S, K>(self, vertex_shader: S) -> FragmentShader<'a, V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                     K: Send + Sync + BarycentricInterpolation {
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
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceWinding {
    Clockwise,
    CounterClockwise
}

pub struct FragmentShader<'a, V, U, K, P: 'static> where V: Send + Sync,
                                                         U: Send + Sync + 'a,
                                                         K: Send + Sync + BarycentricInterpolation,
                                                         P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
    screen_vertices: Vec<ScreenVertex<K>>,
    cull_faces: Option<FaceWinding>
}

#[inline]
fn triangle_signed_area(x1: f32, y1: f32, x2: f32, y2: f32, x3: f32, y3: f32) -> f32 {
    0.5 * (-x2 * y1 + 2.0 * x3 * y1 + x1 * y2 - x3 * y2 + 2.0 * x1 * y3 + x2 * y3)
}

#[inline(always)]
fn winding_order(area: f32) -> FaceWinding {
    if area < 0.0 { FaceWinding::Clockwise } else { FaceWinding::CounterClockwise }
}

impl<'a, V, U, K, P: 'static> FragmentShader<'a, V, U, K, P> where V: Send + Sync,
                                                                   U: Send + Sync + 'a,
                                                                   K: Send + Sync + BarycentricInterpolation,
                                                                   P: Pixel {
    pub fn run<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>, &U) -> P {
        let bb = (self.framebuffer.width() as f32, self.framebuffer.height() as f32);

        println!("Running fragment shader");

        self.mesh.indices.par_chunks(3).filter_map(|triangle| -> Option<(_, f32)> {
            // if there are three points at all, go ahead
            if triangle.len() == 3 {
                let ref a = self.screen_vertices[triangle[0] as usize];
                let ref b = self.screen_vertices[triangle[1] as usize];
                let ref c = self.screen_vertices[triangle[2] as usize];

                let (x1, y1) = (a.position.x, a.position.y);
                let (x2, y2) = (b.position.x, b.position.y);
                let (x3, y3) = (c.position.x, c.position.y);

                let area = triangle_signed_area(x1, y1, x2, y2, x3, y3);

                //TODO: Check if triangle is on screen at all.

                // If there is a winding order for culling,
                // compare it to the triangle winding order,
                // otherwise go ahead
                if let Some(winding) = self.cull_faces {
                    // Check if the winding order matches the desired order
                    if winding == winding_order(area) {
                        Some((triangle, area))
                    } else {
                        None
                    }
                } else {
                    Some((triangle, area))
                }
            } else {
                None
            }
        }).map(|(triangle, area)| {
            let ref a = self.screen_vertices[triangle[0] as usize];
            let ref b = self.screen_vertices[triangle[1] as usize];
            let ref c = self.screen_vertices[triangle[2] as usize];

            let (x1, y1) = (a.position.x, a.position.y);
            let (x2, y2) = (b.position.x, b.position.y);
            let (x3, y3) = (c.position.x, c.position.y);

            let min_x = x1.min(x2).min(x3).max(0.0).min(bb.0);
            let max_x = x1.max(x2).max(x3).min(bb.0).max(0.0);

            let min_y = y1.min(y2).min(y3).max(0.0).min(bb.1);
            let max_y = y1.max(y2).max(y3).min(bb.1).max(0.0);

            let mut x = min_x;

            while x < max_x {
                let mut y = min_y;

                while y < max_y {
                    //TODO
                    y += 1.0;
                }

                x += 1.0;
            }

            println!("Triangle {:?} with Area {}", ((x1, y1), (x2, y2), (x3, y3)), area)
        }).count();
    }
}