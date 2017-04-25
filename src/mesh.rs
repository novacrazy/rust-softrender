//! Generic mesh structure

use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use nalgebra::{Point3, Vector2, Vector3, Vector4};

/// A single vertex with a required position vector and any other vertex data
#[derive(Debug, Clone)]
pub struct Vertex<V> {
    pub position: Point3<f32>,
    pub vertex_data: V,
}

/// Mesh structure with indexed vertices.
#[derive(Clone)]
pub struct Mesh<V> {
    pub indices: Vec<u32>,
    pub vertices: Vec<Vertex<V>>,
}

impl<V> Debug for Mesh<V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Mesh {{ vertices: {} }}", self.vertices.len())
    }
}