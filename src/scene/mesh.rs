use std::sync::Arc;

use mesh::{Mesh, Vertex};

use super::Object;

pub struct MeshObject<V: Vertex> {
    mesh: Arc<Mesh<V>>,
}

impl<V: Vertex> Object for MeshObject<V> {}
