use std::fmt::{Debug, Formatter, Result as FmtResult};
use std::sync::Arc;

use nalgebra::{Point3, Vector2, Vector3, Vector4};

#[derive(Debug, Clone)]
pub struct Vertex {
    pub position: Point3<f32>,
    pub normal: Vector3<f32>,
    pub uv: Vector2<f32>,
}

#[derive(Clone)]
pub struct Mesh {
    pub indices: Arc<Vec<u32>>,
    pub vertices: Arc<Vec<Vertex>>,
}

impl Debug for Mesh {
    fn fmt(&self, f: &mut Formatter) -> FmtResult {
        write!(f, "Mesh {{ vertices: {} }}", self.vertices.len())
    }
}