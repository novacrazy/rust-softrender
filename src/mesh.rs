use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use nalgebra::{Point3, Vector2, Vector3, Vector4};

#[derive(Debug, Clone)]
pub struct Vertex<V> {
    pub position: Point3<f32>,
    pub vertex_data: V,
}

#[derive(Clone)]
pub struct Mesh<V> {
    pub indices: Arc<Vec<u32>>,
    pub vertices: Arc<Vec<Vertex<V>>>,
}

impl<V> Debug for Mesh<V> {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Mesh {{ vertices: {} }}", self.vertices.len())
    }
}