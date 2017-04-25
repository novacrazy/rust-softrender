use std::sync::Arc;

use nalgebra::{Point3, Vector2, Vector4};

use rayon::prelude::*;

use ::pixel::Pixel;
use ::mesh::{Mesh, Vertex};

use super::screen::FrameBuffer;
use super::uniform::BarycentricInterpolation;

pub struct Pipeline<U, P> where P: Pixel, U: Send + Sync {
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
}

impl<U, P> Pipeline<U, P> where U: Send + Sync, P: Pixel {}

pub struct VertexShader<V, U, P> where V: Send + Sync,
                                       U: Send + Sync,
                                       P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
}

#[derive(Debug, Clone)]
pub struct ClipVertex<K> where K: Send + Sync + BarycentricInterpolation {
    pub position: Vector4<f32>,
    pub uniforms: K,
}

impl<V, U, P> VertexShader<V, U, P> where V: Send + Sync,
                                          U: Send + Sync,
                                          P: Pixel {
    pub fn vertex_shader<S, K>(self, vertex_shader: S) -> FragmentShader<V, U, K, P> where S: Fn(&Vertex<V>, &U) -> ClipVertex<K> + Sync,
                                                                                           K: Send + Sync + BarycentricInterpolation {
        let clip_vertices = self.mesh.vertices.par_iter()
                                              .map(|vertex| vertex_shader(vertex, &*self.uniforms))
                                              .collect();

        FragmentShader {
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            clip_vertices: clip_vertices,
        }
    }
}

pub struct FragmentShader<V, U, K, P> where V: Send + Sync,
                                            U: Send + Sync,
                                            K: Send + Sync + BarycentricInterpolation,
                                            P: Pixel {
    mesh: Arc<Mesh<V>>,
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
    clip_vertices: Vec<ClipVertex<K>>,
}

impl<V, U, K, P> FragmentShader<V, U, K, P> where V: Send + Sync,
                                                  U: Send + Sync,
                                                  K: Send + Sync + BarycentricInterpolation,
                                                  P: Pixel {
    pub fn fragment_shader<S>(self, fragment_shader: S) where S: Fn() -> P {
        self.mesh.indices.par_chunks(3).map(|triangle| {
            if triangle.len() == 3 {
                let ref a = self.clip_vertices[triangle[0] as usize];
                let ref b = self.clip_vertices[triangle[1] as usize];
                let ref c = self.clip_vertices[triangle[2] as usize];
            }

            ()
        });
    }
}