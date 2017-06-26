use std::sync::Arc;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use rayon::prelude::*;

use ::color::Color;
use ::color::blend::Blend;
use ::framebuffer::{Attachments, Framebuffer};
use ::primitive::Primitive;
use ::mesh::Mesh;
use ::geometry::{Dimensions, HasDimensions, Coordinate, ScreenVertex, FaceWinding};
use ::interpolate::Interpolate;
use ::pipeline::storage::SeparableScreenPrimitiveStorage;

use ::pipeline::PipelineObject;

use ::pipeline::types::{PipelineUniforms, ColorAttachment};

pub const DEFAULT_TILE_SIZE: Dimensions = Dimensions { width: 16, height: 16 };


/// Fragment shader stage.
///
/// The fragment shader is responsible for determining the color of pixels where the underlying geometry has been projected onto.
/// Usually this is individual triangles that are rasterized and shaded by the fragment shader, but it also supports point-cloud
/// and lines (pairs of vertices considered as endpoints for lines).
///
/// The fragment shader runs several tests before executing the given shader program, including a depth test.
/// If the depth of the geometry (from the camera), is farther away than geometry that has already been rendered,
/// the shader program isn't run at all, since it wouldn't be visible anyway. Additionally,
/// if the geometry is nearer than an existing fragment, the existing fragment is overwritten.
///
/// Uniforms passed from the vertex shader are interpolating inside the triangles using Interpolate interpolation,
/// which is why it must satisfy the [`Interpolate`](../uniform/trait.Interpolate.html) trait, which can be automatically implemented for many types using the
/// `declare_uniforms!` macro. See the documentation on that for more information on how to use it.
pub struct FragmentShader<'a, P: 'a, V, T, K, B> {
    pub ( in ::pipeline) pipeline: &'a mut P,
    pub ( in ::pipeline) mesh: Arc<Mesh<V>>,
    pub ( in ::pipeline) indexed_primitive: PhantomData<T>,
    pub ( in ::pipeline) indexed_vertices: Arc<Option<Vec<ScreenVertex<K>>>>,
    pub ( in ::pipeline) generated_primitives: Arc<SeparableScreenPrimitiveStorage<K>>,
    pub ( in ::pipeline) cull_faces: Option<FaceWinding>,
    pub ( in ::pipeline) blend: B,
    pub ( in ::pipeline) antialiased_lines: bool,
    pub ( in ::pipeline) tile_size: Dimensions,
}

/// Fragment returned by the fragment shader, which can either be a color
/// value for the pixel or a discard flag to skip that fragment altogether.
#[derive(Debug, Clone, Copy)]
pub enum Fragment<C> where C: Color {
    /// Discard the fragment altogether, as if it was never there.
    Discard,
    /// Desired color for the pixel
    Color(C)
}

impl<'a, P: 'a, V, T, K, B> Deref for FragmentShader<'a, P, V, T, K, B>
    where P: PipelineObject, B: Blend<ColorAttachment<P>> {
    type Target = B;
    fn deref(&self) -> &B { &self.blend }
}

impl<'a, P: 'a, V, T, K, B> DerefMut for FragmentShader<'a, P, V, T, K, B>
    where P: PipelineObject, B: Blend<ColorAttachment<P>> {
    fn deref_mut(&mut self) -> &mut B { &mut self.blend }
}

impl<'a, P: 'a, V, T, K, B> FragmentShader<'a, P, V, T, K, B> {
    /// Cull faces based on winding order. For more information on how and why this works,
    /// check out the documentation for the [`FaceWinding`](../geometry/winding/enum.FaceWinding.html) enum.
    pub fn cull_faces(&mut self, cull: Option<FaceWinding>) {
        self.cull_faces = cull;
    }

    /// Enables drawing antialiased lines for `Line` primitives
    /// primitives using Xiaolin Wu's algorithm,
    /// otherwise Bresenham's Algorithm is used.
    pub fn antialiased_lines(&mut self, enable: bool) {
        self.antialiased_lines = enable;
    }

    /// Duplicates all references to internal state to return a cloned fragment shader,
    /// which can be used to efficiently render the same geometry with different
    /// rasterization methods in quick succession.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> FragmentShader<'b, P, V, T, K, B> where 'a: 'b, B: Clone {
        FragmentShader {
            pipeline: self.pipeline,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
            indexed_vertices: self.indexed_vertices.clone(),
            generated_primitives: self.generated_primitives.clone(),
            cull_faces: self.cull_faces.clone(),
            blend: self.blend.clone(),
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }
}

impl<'a, P: 'a, V, T, K, O> FragmentShader<'a, P, V, T, K, O> where P: PipelineObject {
    #[must_use]
    pub fn with_blend<B>(self, blend: B) -> FragmentShader<'a, P, V, T, K, B>
        where B: Blend<ColorAttachment<P>> {
        FragmentShader {
            pipeline: self.pipeline,
            mesh: self.mesh,
            indexed_primitive: PhantomData,
            indexed_vertices: self.indexed_vertices,
            generated_primitives: self.generated_primitives,
            cull_faces: self.cull_faces,
            blend: blend,
            antialiased_lines: self.antialiased_lines,
            tile_size: self.tile_size,
        }
    }

    #[must_use]
    pub fn with_default_blend<B>(self) -> FragmentShader<'a, P, V, T, K, B>
        where B: Blend<ColorAttachment<P>> + Default {
        self.with_blend(B::default())
    }
}

impl<'a, P: 'a, V, T, K, B> FragmentShader<'a, P, V, T, K, B>
    where P: PipelineObject,
          V: Send + Sync,
          T: Primitive,
          K: Send + Sync + Interpolate,
          B: Blend<ColorAttachment<P>> {
    pub fn run<S>(self, fragment_shader: S)
        where S: Fn(&ScreenVertex<K>, &PipelineUniforms<P>) -> Fragment<ColorAttachment<P>> + Send + Sync {
        let FragmentShader {
            pipeline,
            mesh,
            indexed_vertices,
            generated_primitives,
            cull_faces,
            blend,
            antialiased_lines,
            tile_size,
            ..
        } = self;

        let dimensions = pipeline.framebuffer().dimensions();

        let tiles = {
            let mut tiles = Vec::new();

            let mut y = 0;

            while y < dimensions.height {
                let mut x = 0;

                let next_y = y + tile_size.height;

                while x < dimensions.width {
                    let next_x = x + tile_size.width;

                    tiles.push((
                        Coordinate::new(x, y),
                        Coordinate::new(next_x, next_y)
                    ));

                    x = next_x;
                }

                y = next_y;
            }

            tiles
        };

        tiles.into_par_iter().for_each(|tile| {
            println!("Tile {:?}", tile);
            //render
        });
    }
}