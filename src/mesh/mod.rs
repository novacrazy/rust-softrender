use std::fmt::{self, Debug};

pub mod vertex;

use error::{RenderError, RenderResult};

pub use self::vertex::Vertex;

#[derive(Debug, Clone, Copy, Hash, PartialEq)]
pub enum Primitive {
    Triangle,
    TriangleStrip,
    TriangleFan,
    Point,
    Line,
    LineStrip,
    LineLoop,
}

#[derive(Clone)]
pub struct Mesh<V: Vertex> {
    pub indices: Option<Vec<usize>>,
    pub vertices: Vec<V>,
    pub primitive: Primitive,
}

impl<V: Vertex> Mesh<V> {
    pub fn index(&mut self) -> RenderResult<()> {
        if self.indices.is_some() {
            Err(RenderError::IndicesAlreadyExist)
        } else {
            match self.primitive {
                Primitive::Triangle => {
                    let len = self.vertices.len();

                    if len % 3 == 0 {
                        self.indices = Some((0..len).collect());
                    } else {
                        return Err(RenderError::InvalidVertexCount(len, self.primitive));
                    }
                }
                _ => unimplemented!(),
            }

            Ok(())
        }
    }
}

impl<V: Vertex> Debug for Mesh<V> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Mesh ({:?}) {{ vertices: {} }}", self.primitive, self.vertices.len())
    }
}
