use std::sync::Arc;
use std::path::Path;

use nalgebra::{Vector2, Point3, Vector3};

use tobj;

use softrender::mesh::{Mesh, Vertex};

/// Defines data stored alongside vertex position
pub struct VertexData {
    pub normal: Vector3<f32>,
    pub uv: Vector2<f32>,
}

pub struct MeshData {
    pub mesh: Arc<Mesh<VertexData>>,
    pub material: Option<tobj::Material>,
}

/// Loads a .obj mesh from the given path
pub fn load_model<P: AsRef<Path>>(p: P) -> Vec<MeshData> {
    let (models, materials): (Vec<tobj::Model>, Vec<tobj::Material>) = tobj::load_obj(p.as_ref().clone()).unwrap();

    models.into_iter().map(|model| {
        let ref mesh: tobj::Mesh = model.mesh;

        assert_eq!(mesh.positions.len(), mesh.normals.len());
        assert_eq!(mesh.positions.len(), mesh.texcoords.len() / 2 * 3);

        let positions = mesh.positions.chunks(3);
        let normals = mesh.normals.chunks(3);
        let uvs = mesh.texcoords.chunks(2);

        MeshData {
            mesh: Arc::new(Mesh {
                vertices: positions.zip(normals).zip(uvs).map(|((position, normal), uv)| {
                    Vertex {
                        position: Point3::new(position[0], position[1], position[2]),
                        vertex_data: VertexData {
                            normal: Vector3::new(normal[0], normal[1], normal[2]),
                            uv: Vector2::new(uv[0], uv[1]),
                        }
                    }
                }).collect(),
                indices: mesh.indices.clone()
            }),
            material: mesh.material_id.map(|id| materials[id].clone())
        }
    }).collect()
}