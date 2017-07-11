use std::marker::PhantomData;
use std::sync::Arc;
use std::ops::Deref;

use rayon::prelude::*;

use ::pipeline::storage::SeparablePrimitiveStorage;
use ::pipeline::{PipelineObject, GeometryShader};
use ::primitive::Primitive;
use ::mesh::{Vertex, Mesh};
use ::interpolate::Interpolate;
use ::geometry::ClipVertex;

use ::pipeline::types::{PipelineUniforms, StencilValue};

/// Vertex shader stage.
///
/// The vertex shader is responsible for transforming all mesh vertices into a form which can be presented on screen (more or less),
/// which usually involved transforming object-space coordinates to world-space, then to camera-space, then finally to projection/clip-space,
/// at which point it and any uniforms are passed back and sent to the fragment shader.
///
/// For a full example of how this works, see the documentation on the `run` method below.
///
/// The vertex shader holds a reference to the pipeline framebuffer and global uniforms,
/// and for the given mesh given to it when created.
/// These cannot be modified while the vertex shader exists.

pub struct VertexShader<'a, P: 'a, V: Vertex, T> where P: PipelineObject {
    pub ( in ::pipeline) pipeline: &'a mut P,
    pub ( in ::pipeline) mesh: Arc<Mesh<V>>,
    pub ( in ::pipeline) indexed_primitive: PhantomData<T>,
    pub ( in ::pipeline) stencil_value: StencilValue<P>
}

impl<'a, P: 'a, V, T> VertexShader<'a, P, V, T> where P: PipelineObject,
                                                      V: Vertex,
                                                      T: Primitive {

    /// Duplicates all references to internal state to return a cloned vertex shader,
    /// though since the vertex shader itself has very little internal state at this point,
    /// it's not that useful.
    #[must_use]
    pub fn duplicate<'b>(&'b mut self) -> VertexShader<'b, P, V, T> where 'a: 'b {
        VertexShader {
            pipeline: self.pipeline,
            mesh: self.mesh.clone(),
            indexed_primitive: PhantomData,
            stencil_value: self.stencil_value,
        }
    }

    /// Executes the vertex shader on every vertex in the mesh,
    /// (hopefully) returning a `ClipVertex` with the transformed vertex in clip-space
    /// and any uniforms to be passed into the fragment shader.
    ///
    /// In case you don't want to research what clip-space is, it's basically the output of the projection transformation,
    /// so in your vertex shader you'd have something like:
    ///
    /// ```ignore
    /// let fragment_shader = vertex_shader.run(|vertex, global_uniforms| {
    ///     let GlobalUniforms { ref view, ref projection, ref model, ref model_inverse_transpose, .. } = *global_uniforms;
    ///     let VertexData { normal, uv } = vertex.vertex_data;
    ///
    ///     // Transform vertex position to world-space
    ///     let world_position = model * vertex.position.to_homogeneous();
    ///
    ///     // Transform normal to world-space
    ///     let normal = (model_inverse_transpose * normal.to_homogeneous()).normalize();
    ///
    ///     // Transform vertex position to clip-space (projection-space)
    ///     let clip_position = projection * view * world_position;
    ///
    ///     // Return the clip-space position and any uniforms to interpolate and pass into the fragment shader
    ///     ClipVertex::new(clip_position, Uniforms {
    ///         position: world_position,
    ///         normal: normal,
    ///         uv: uv,
    ///     })
    /// });
    /// ```
    ///
    /// where `GlobalUniforms`, `VertexData` and `Uniforms` are data structures defined by you.
    ///
    /// See the [`full_example`](https://github.com/novacrazy/rust-softrender/tree/master/full_example) project for this in action.
    #[must_use]
    pub fn run<S, K>(self, vertex_shader: S) -> GeometryShader<'a, P, V, T, K> where S: Fn(&V, &PipelineUniforms<P>) -> ClipVertex<V::Scalar, K> + Send + Sync,
                                                                                     K: Send + Sync + Interpolate {
        let VertexShader { pipeline, mesh, stencil_value, .. } = self;

        let indexed_vertices = {
            // borrow uniforms here so P isn't required to be Send/Sync
            let uniforms = pipeline.uniforms();

            mesh.vertices.par_iter().map(|vertex| {
                vertex_shader(vertex, uniforms)
            }).collect()
        };

        GeometryShader {
            pipeline,
            mesh,
            indexed_primitive: PhantomData,
            stencil_value,
            indexed_vertices: Some(indexed_vertices),
            generated_primitives: SeparablePrimitiveStorage::default(),
        }
    }
}