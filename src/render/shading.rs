use std::sync::Arc;

use nalgebra::{Point3, Vector2, Vector4};

use rayon::prelude::*;
use rayon::slice::ParallelSlice;

use ::mesh::{Mesh, Vertex};
use super::pixel::Pixel;
use super::screen::FrameBuffer;
use super::uniform::BarycentricInterpolation;

pub struct Pipeline<U, P> where P: Pixel, U: Send + Sync {
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
}

impl<U, P> Pipeline<U, P> where U: Send + Sync, P: Pixel {}

pub struct VertexShader<U, P> where U: Send + Sync, P: Pixel {
    mesh: Arc<Mesh>,
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
}

#[derive(Debug, Clone)]
pub struct ClipVertex<K> where K: Send + Sync + BarycentricInterpolation {
    pub position: Vector4<f32>,
    pub uniforms: K,
}

impl<U, P> VertexShader<U, P> where U: Send + Sync, P: Pixel {
    pub fn vertex_shader<V, K>(self, vertex_shader: V) -> FragmentShader<U, K, P> where V: Fn(&Vertex, &U) -> ClipVertex<K> + Sync,
                                                                                        K: Send + Sync + BarycentricInterpolation {
        let clip_vertices = {
            let vertices = self.mesh.vertices.clone();

            // Run the vertex shader in parallel using vector indices as the iterator
            (0..vertices.len())
                .into_par_iter()
                .map(|index| {
                    // We know the indices are valid, so get unchecked for extra performance in debug mode
                    vertex_shader(unsafe { vertices.get_unchecked(index) }, &*self.uniforms)
                }).collect()
        };

        FragmentShader {
            mesh: self.mesh,
            uniforms: self.uniforms,
            framebuffer: self.framebuffer,
            clip_vertices: clip_vertices,
        }
    }
}

pub struct FragmentShader<U, K, P> where U: Send + Sync,
                                         K: Send + Sync + BarycentricInterpolation,
                                         P: Pixel {
    mesh: Arc<Mesh>,
    uniforms: Arc<U>,
    framebuffer: Arc<FrameBuffer<P>>,
    clip_vertices: Vec<ClipVertex<K>>,
}

impl<U, K, P> FragmentShader<U, K, P> where U: Send + Sync,
                                            K: Send + Sync + BarycentricInterpolation,
                                            P: Pixel {
    pub fn fragment_shader<F>(self, fragment_shader: F) where F: Fn() -> P {
        self.mesh.indices.as_slice().par_chunks(3).map(|triangle| {
            if triangle.len() == 3 {
                let ref a = self.clip_vertices[triangle[0] as usize];
                let ref b = self.clip_vertices[triangle[1] as usize];
                let ref c = self.clip_vertices[triangle[2] as usize];
            }

            ()
        });
    }
}