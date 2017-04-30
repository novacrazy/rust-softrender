use std::sync::Arc;
use std::path::Path;

use nalgebra::{Vector2, Vector3};

use softrender::mesh::{Mesh, Vertex};

/// Defines data stored alongside vertex position
pub struct VertexData {
    pub normal: Vector3<f32>,
    //pub uv: Vector2<f32>,
}

pub struct MeshData {
    //pub mesh: Arc<Mesh<VertexData>>,
}

pub fn load_mesh<P>(path: P) -> MeshData where P: AsRef<Path> {

    MeshData {}
}