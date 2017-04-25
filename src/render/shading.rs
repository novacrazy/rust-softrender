//! Shader Execution Pipeline

use std::sync::Arc;

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
    pub fn start_mesh<V>(&mut self, mesh: Arc<Mesh<V>>) -> VertexShader<V, U, P> where V: Send + Sync {
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
    pub fn vertex_shader<S, K>(self, vertex_shader: S) -> FragmentShader<'a, V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                               K: Send + Sync + BarycentricInterpolation {
        let screen_vertices = self.mesh.vertices.par_iter()
                                                .map(|vertex| {
                                                    vertex_shader(vertex, &*self.uniforms)
                                                        .normalize(self.framebuffer.viewport())
                                                })
                                                .collect();

        FragmentShader {
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            screen_vertices: screen_vertices,
        }
    }
}

pub struct FragmentShader<'a, V, U, K, P: 'static> where V: Send + Sync,
                                                         U: Send + Sync + 'a,
                                                         K: Send + Sync + BarycentricInterpolation,
                                                         P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: &'a U,
    framebuffer: &'a mut FrameBuffer<P>,
    screen_vertices: Vec<ScreenVertex<K>>,
}

impl<'a, V, U, K, P: 'static> FragmentShader<'a, V, U, K, P> where V: Send + Sync,
                                                                   U: Send + Sync + 'a,
                                                                   K: Send + Sync + BarycentricInterpolation,
                                                                   P: Pixel {
    pub fn fragment_shader<S>(self, fragment_shader: S) where S: Fn(&ScreenVertex<K>) -> P {
        self.mesh.indices.par_chunks(3).map(|triangle| {
            if triangle.len() == 3 {
                let ref a = self.screen_vertices[triangle[0] as usize];
                let ref b = self.screen_vertices[triangle[1] as usize];
                let ref c = self.screen_vertices[triangle[2] as usize];
            }

            ()
        });
    }
}